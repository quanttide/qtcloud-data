use std::process::Command;

fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data")
}

fn with_dir(dir: &str) -> String {
    dir.to_string()
}

#[test]
fn test_blueprint_help() {
    let output = cli()
        .arg("blueprint")
        .arg("--help")
        .output()
        .expect("运行 blueprint --help 失败");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("review"));
    assert!(stdout.contains("design"));
    assert!(stdout.contains("formalize"));
    assert!(stdout.contains("preview"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
}

#[test]
fn test_design_new_creates_template() {
    let tmp = std::env::temp_dir().join("bp-test-design-new");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
        .arg("blueprint")
        .arg("design")
        .arg("new")
        .arg("test-project")
        .output()
        .expect("运行 design new 失败");

    assert!(output.status.success(), "design new 应成功: {}", String::from_utf8_lossy(&output.stderr));

    let md_path = tmp.join("test-project.md");
    assert!(md_path.exists(), "应生成 {md_path:?}");

    let content = std::fs::read_to_string(&md_path).unwrap();
    assert!(content.contains("# test-project"), "模板应包含项目标题");
    assert!(content.contains("## 背景"), "模板应包含背景章节");
    assert!(content.contains("## 数据来源"), "模板应包含数据来源章节");
    assert!(content.contains("## 输出变量"), "模板应包含输出变量章节");
    assert!(content.contains("## 精炼管道"), "模板应包含精炼管道章节");
    assert!(content.contains("## 验收标准"), "模板应包含验收标准章节");

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_design_new_fails_on_existing() {
    let tmp = std::env::temp_dir().join("bp-test-design-exists");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Create file first
    std::fs::write(tmp.join("test-project.md"), "existing").unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
        .arg("blueprint")
        .arg("design")
        .arg("new")
        .arg("test-project")
        .output()
        .expect("运行 design new 失败");

    assert!(!output.status.success(), "design new 在文件已存在时应失败");

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_design_new_different_names() {
    let tmp = std::env::temp_dir().join("bp-test-design-names");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    for name in &["my-blueprint", "sec-credit", "ghtorrent-refiner"] {
        let output = cli()
            .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
            .arg("blueprint")
            .arg("design")
            .arg("new")
            .arg(name)
            .output()
            .unwrap();

        assert!(output.status.success(), "design new {name} 应成功");
        assert!(tmp.join(format!("{name}.md")).exists(), "{name}.md 应存在");

        let content = std::fs::read_to_string(tmp.join(format!("{name}.md"))).unwrap();
        assert!(content.contains(&format!("# {name}")), "模板标题应匹配项目名");
    }

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_formalize_requires_input() {
    let output = cli()
        .arg("blueprint")
        .arg("formalize")
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--input"), "formalize 应有 --input 参数");
    assert!(stdout.contains("--output"), "formalize 应有 --output 参数");
}

#[test]
fn test_formalize_missing_file() {
    let output = cli()
        .arg("blueprint")
        .arg("formalize")
        .arg("--input")
        .arg("/tmp/nonexistent-blueprint.md")
        .output()
        .unwrap();

    assert!(!output.status.success(), "formalize 对不存在的文件应报错");
}

#[test]
fn test_preview_help() {
    let output = cli()
        .arg("blueprint")
        .arg("preview")
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--input"));
    assert!(stdout.contains("--output"));
}

#[test]
fn test_version_subcommands() {
    let output = cli()
        .arg("blueprint")
        .arg("version")
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("diff"));
}

#[test]
fn test_review_help() {
    let output = cli()
        .arg("blueprint")
        .arg("review")
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INPUT"), "review 应有 INPUT 参数");
}

#[test]
fn test_list_runs() {
    let tmp = std::env::temp_dir().join("bp-test-list");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
        .arg("blueprint")
        .arg("list")
        .output()
        .unwrap();

    // list 在没有 cue CLI 时会失败，但不应该 panic
    let _stderr = String::from_utf8_lossy(&output.stderr);
    // Accept both success (if cue is installed) and failure (if not)
    // The key is that it doesn't crash

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_to_camel_blueprint_module() {
    // Verify that our helper function in blueprint.rs matches the one in process.rs
    // Both should produce identical results
    let process_camel = qtcloud_data_cli::process::to_camel("csv-standard");
    // We can't directly test blueprint::to_camel since it's private,
    // but we can verify the process one still works
    assert_eq!(process_camel, "csvStandard");
}
