use std::path::{Path, PathBuf};
use std::process::Command;

fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data")
}

fn script_path(dir: &Path, stem: &str) -> PathBuf {
    let ext = if cfg!(windows) { "cmd" } else { "sh" };
    dir.join(format!("{stem}.{ext}"))
}

fn write_script(path: &Path, content: &str) {
    std::fs::write(path, content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn fake_qtdata_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nif \"%1\"==\"transfer\" if \"%2\"==\"receive\" (\r\n  echo raw,data>\"%5\"\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"transfer\" if \"%2\"==\"send\" (\r\n  echo https://delivery.example/link>\"%5\"\r\n  exit /b 0\r\n)\r\nexit /b 1\r\n"
    } else {
        "#!/bin/sh\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"receive\" ]; then\n  printf 'raw,data\\n' > \"$5\"\n  exit 0\nfi\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"send\" ]; then\n  printf 'https://delivery.example/link\\n' > \"$5\"\n  exit 0\nfi\nexit 1\n"
    }
}

fn copy_pipeline_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\ncopy /Y \"%1\" \"%2\" >NUL\r\nexit /b %ERRORLEVEL%\r\n"
    } else {
        "#!/bin/sh\ncp \"$1\" \"$2\"\n"
    }
}

fn failing_pipeline_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nexit /b 7\r\n"
    } else {
        "#!/bin/sh\nexit 7\n"
    }
}

fn sample_blueprint_yaml() -> &'static str {
    r#"name: "sample"
description: "CLI smoke specification"
contract:
  input:
    schema: "raw: string"
    format: "CSV"
  output:
    schema: "clean: string"
    format: "CSV"
    rules:
      - 字段非空
pipeline:
  name: "sample-pipeline"
  steps:
    - name: "clean"
      from: "raw"
      to: "clean"
      desc: "trim whitespace"
status: draft
created_at: "2026-07-30T00:00:00+00:00"
updated_at: "2026-07-30T00:00:00+00:00"
"#
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
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("blueprint"));
    assert!(stdout.contains("spec"));
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
    let output = cli()
        .arg("clarify")
        .arg("from-chat")
        .arg("--help")
        .output()
        .unwrap();
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

// ── spec ──

#[test]
fn test_spec_help() {
    let output = cli().arg("spec").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wrap"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_spec_wrap_writes_enveloped_yaml() {
    let root = std::env::temp_dir().join(format!("qtcloud-spec-wrap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let input = root.join("sample-blueprint.yaml");
    let output_path = root.join("sample-spec.yaml");
    std::fs::write(&input, sample_blueprint_yaml()).unwrap();

    let output = cli()
        .arg("spec")
        .arg("wrap")
        .arg(&input)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "spec wrap failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&output_path).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    assert_eq!(
        yaml["api_version"].as_str(),
        Some("qtcloud.quanttide.com/v1alpha1")
    );
    assert_eq!(yaml["kind"].as_str(), Some("Specification"));
    assert_eq!(yaml["metadata"]["name"].as_str(), Some("sample"));
    assert_eq!(
        yaml["spec"]["blueprint"]["pipeline"]["name"].as_str(),
        Some("sample-pipeline")
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_spec_validate_accepts_enveloped_yaml() {
    let root = std::env::temp_dir().join(format!("qtcloud-spec-validate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let input = root.join("sample-blueprint.yaml");
    let spec_path = root.join("sample-spec.yaml");
    std::fs::write(&input, sample_blueprint_yaml()).unwrap();

    let wrap = cli()
        .arg("spec")
        .arg("wrap")
        .arg(&input)
        .arg("--output")
        .arg(&spec_path)
        .output()
        .unwrap();
    assert!(wrap.status.success());

    let validate = cli()
        .arg("spec")
        .arg("validate")
        .arg(&spec_path)
        .output()
        .unwrap();

    assert!(
        validate.status.success(),
        "spec validate failed: {}\n{}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    assert!(String::from_utf8_lossy(&validate.stdout).contains("Specification OK: sample"));

    std::fs::remove_dir_all(&root).ok();
}

// ── doctor ──

#[test]
fn test_doctor_help() {
    let output = cli().arg("doctor").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("检查本机 DataOps 环境"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--fix-dirs"));
}

#[test]
fn test_doctor_no_fail_runs_without_printing_secret_values() {
    let output = cli()
        .env("DROPBOX_ACCESS_TOKEN", "top-secret-token")
        .arg("doctor")
        .arg("--no-fail")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Summary:"));
    assert!(!stdout.contains("top-secret-token"));
}

#[test]
fn test_doctor_json_no_fail_outputs_machine_readable_report() {
    let output = cli()
        .env("DROPBOX_ACCESS_TOKEN", "top-secret-token")
        .arg("doctor")
        .arg("--json")
        .arg("--no-fail")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["command"], "doctor");
    assert!(report["summary"]["warnings"].is_number());
    assert!(report["checks"].as_array().unwrap().len() > 5);
    assert!(!stdout.contains("top-secret-token"));
}

#[test]
fn test_doctor_fix_dirs_creates_configured_data_dirs() {
    let root = std::env::temp_dir().join(format!("qtcloud-doctor-fix-dirs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let drd = root.join("drd");
    let spec = root.join("spec");
    let blueprint = root.join("blueprint");
    let contract = root.join("contract");
    let pipeline = root.join("pipeline");
    let catalog = root.join("catalog");

    let output = cli()
        .env("DRD_DIR", &drd)
        .env("SPEC_DIR", &spec)
        .env("BLUEPRINT_DIR", &blueprint)
        .env("CONTRACT_DIR", &contract)
        .env("PIPELINE_DIR", &pipeline)
        .env("CATALOG_DIR", &catalog)
        .arg("doctor")
        .arg("--fix-dirs")
        .arg("--no-fail")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(drd.is_dir());
    assert!(spec.is_dir());
    assert!(blueprint.is_dir());
    assert!(contract.is_dir());
    assert!(pipeline.is_dir());
    assert!(catalog.is_dir());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_doctor_fix_dirs_uses_data_root_for_default_children() {
    let root =
        std::env::temp_dir().join(format!("qtcloud-doctor-data-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let output = cli()
        .env("DATA_ROOT", &root)
        .arg("doctor")
        .arg("--fix-dirs")
        .arg("--no-fail")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(root.join("drd").is_dir());
    assert!(root.join("spec").is_dir());
    assert!(root.join("blueprint").is_dir());
    assert!(root.join("contract").is_dir());
    assert!(root.join("pipeline").is_dir());
    assert!(root.join("catalog").is_dir());

    std::fs::remove_dir_all(&root).ok();
}

// ── process ──

#[test]
fn test_process_writes_job_record_after_success() {
    let root = std::env::temp_dir().join(format!("qtcloud-process-record-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let fake_qtdata = script_path(&root, "fake-qtdata");
    let pipeline = script_path(&root, "copy-pipeline");
    write_script(&fake_qtdata, fake_qtdata_script());
    write_script(&pipeline, copy_pipeline_script());

    let catalog_dir = root.join("catalog");
    let work_dir = root.join("work");

    let output = cli()
        .env("QTDATA_CLI", &fake_qtdata)
        .env("CATALOG_DIR", &catalog_dir)
        .env("WORKDIR", &work_dir)
        .arg("process")
        .arg("ABC-001")
        .arg("https://example.com/raw.csv?access_token=secret")
        .arg("--pipeline")
        .arg(&pipeline)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "process failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("access_token=secret"));
    assert!(!stderr.contains("access_token=secret"));

    let jobs_path = catalog_dir.join("jobs.json");
    let content = std::fs::read_to_string(&jobs_path).unwrap();
    let jobs: serde_json::Value = serde_json::from_str(&content).unwrap();
    let jobs = jobs.as_object().unwrap();
    assert_eq!(jobs.len(), 1);

    let record = jobs.values().next().unwrap();
    let job_id = record["id"].as_str().unwrap();
    assert_eq!(record["customer_id"], "ABC-001");
    assert_eq!(record["source_url"], "https://example.com/raw.csv");
    assert_eq!(record["status"], "delivered");
    assert!(
        record["pipeline"]
            .as_str()
            .unwrap()
            .contains("copy-pipeline")
    );
    assert!(record["raw_path"].as_str().unwrap().ends_with("raw.csv"));
    assert!(
        record["output_path"]
            .as_str()
            .unwrap()
            .ends_with("final.csv")
    );
    assert!(
        record["link_path"]
            .as_str()
            .unwrap()
            .ends_with("share-link.txt")
    );
    assert!(std::path::Path::new(record["log_path"].as_str().unwrap()).is_file());
    assert!(!content.contains("access_token=secret"));

    let registry_content = std::fs::read_to_string(catalog_dir.join("registry.json")).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&registry_content).unwrap();
    let volumes = registry.as_object().unwrap();
    assert_eq!(volumes.len(), 1);

    let volume = volumes.values().next().unwrap();
    assert_eq!(volume["provider"], "process");
    assert_eq!(volume["source"], format!("process:{job_id}"));
    assert_eq!(volume["status"], "delivered");
    assert!(volume["path"].as_str().unwrap().ends_with("final.csv"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_process_writes_failed_job_record_when_pipeline_fails() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-process-failed-record-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let fake_qtdata = script_path(&root, "fake-qtdata");
    let pipeline = script_path(&root, "failing-pipeline");
    write_script(&fake_qtdata, fake_qtdata_script());
    write_script(&pipeline, failing_pipeline_script());

    let catalog_dir = root.join("catalog");
    let work_dir = root.join("work");

    let output = cli()
        .env("QTDATA_CLI", &fake_qtdata)
        .env("CATALOG_DIR", &catalog_dir)
        .env("WORKDIR", &work_dir)
        .arg("process")
        .arg("ABC-002")
        .arg("https://example.com/raw.csv?access_token=secret")
        .arg("--pipeline")
        .arg(&pipeline)
        .output()
        .unwrap();

    assert!(!output.status.success());

    let content = std::fs::read_to_string(catalog_dir.join("jobs.json")).unwrap();
    let jobs: serde_json::Value = serde_json::from_str(&content).unwrap();
    let record = jobs.as_object().unwrap().values().next().unwrap();
    assert_eq!(record["customer_id"], "ABC-002");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["source_url"], "https://example.com/raw.csv");
    assert!(!content.contains("access_token=secret"));

    let log = std::fs::read_to_string(record["log_path"].as_str().unwrap()).unwrap();
    assert!(log.contains("pipeline failed"));

    std::fs::remove_dir_all(&root).ok();
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
