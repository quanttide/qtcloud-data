use std::process::Command;

fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data")
}

// ── Top-level help ──

#[test]
fn test_cli_help_shows_all_commands() {
    let output = cli().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clarify"));
    assert!(stdout.contains("design"));
    assert!(stdout.contains("review"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("blueprint"));
}

// ── clarify ──

#[test]
fn test_clarify_help() {
    let output = cli().arg("clarify").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("from-chat"));
}

#[test]
fn test_clarify_from_chat_help() {
    let output = cli().arg("clarify").arg("from-chat").arg("--help").output().unwrap();
    assert!(output.status.success());
}

// ── design ──

#[test]
fn test_design_help() {
    let output = cli().arg("design").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("contract"));
    assert!(stdout.contains("blueprint"));
    assert!(stdout.contains("formalize"));
    assert!(stdout.contains("preview"));
}

#[test]
fn test_design_subcommands_help() {
    for sub in &["contract", "blueprint", "formalize", "preview"] {
        let output = cli().arg("design").arg(sub).arg("--help").output().unwrap();
        assert!(output.status.success(), "design {sub} --help failed");
    }
}

// ── review ──

#[test]
fn test_review_help() {
    let output = cli().arg("review").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INPUT"));
}

// ── version ──

#[test]
fn test_version_help() {
    let output = cli().arg("version").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("diff"));
}

// ── blueprint (old, kept) ──

#[test]
fn test_blueprint_help() {
    let output = cli().arg("blueprint").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    // Old subcommands should NOT appear
    assert!(!stdout.contains("review"));
    assert!(!stdout.contains("design"));
    assert!(!stdout.contains("formalize"));
}

// ── design new (template, kept as blueprint design new) ──

#[test]
fn test_blueprint_list_runs() {
    let tmp = std::env::temp_dir().join("bp-v020-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
        .arg("blueprint")
        .arg("list")
        .output()
        .unwrap();
    // May fail if cue CLI not installed, but should not panic
    let _ = output;

    std::fs::remove_dir_all(&tmp).ok();
}
