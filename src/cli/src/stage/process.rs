//! 编排命令：`StepExecutor`（receive → pipeline → send）+ job 记录。

use clap::Args;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::catalog::{self, RegisterVolume};
use crate::error::CliError;
use crate::registry;
use crate::stage::transfer;
use crate::util;

#[derive(Args)]
pub struct ProcessArgs {
    /// 客户 ID
    pub customer_id: String,
    /// 数据来源 URL
    pub source_url: String,
    /// 使用 blueprint（自动解析关联的 pipeline）
    #[arg(long)]
    pub blueprint: Option<String>,
    /// 直接指定 pipeline
    #[arg(long)]
    pub pipeline: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
// ── 数据模型（ProcessJobRecord / ProcessArgs） ──
pub struct ProcessJobRecord {
    pub id: String,
    pub customer_id: String,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blueprint: Option<String>,
    pub pipeline: String,
    pub raw_path: String,
    pub output_path: String,
    pub link_path: String,
    pub log_path: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
}

pub struct ProcessJobRecordInput {
    pub id: String,
    pub customer_id: String,
    pub source_url: String,
    pub blueprint: Option<String>,
    pub pipeline: String,
    pub raw_path: String,
    pub output_path: String,
    pub link_path: String,
    pub log_path: String,
    pub started_at: String,
    pub finished_at: String,
}

impl ProcessJobRecord {
    pub fn delivered(input: ProcessJobRecordInput) -> Self {
        Self::with_status(input, "delivered")
    }

    fn failed(input: ProcessJobRecordInput) -> Self {
        Self::with_status(input, "failed")
    }

    fn with_status(input: ProcessJobRecordInput, status: &str) -> Self {
        Self {
            id: input.id,
            customer_id: input.customer_id,
            source_url: redact_source(&input.source_url),
            blueprint: input.blueprint,
            pipeline: input.pipeline,
            raw_path: input.raw_path,
            output_path: input.output_path,
            link_path: input.link_path,
            log_path: input.log_path,
            status: status.to_string(),
            started_at: input.started_at,
            finished_at: input.finished_at,
        }
    }
}

// ── 命令与 StepExecutor（receive → pipeline → send） ──
/// 编排命令入口：按 blueprint 执行 receive → pipeline → send。
pub fn run(args: &ProcessArgs) -> Result<(), CliError> {
    let pipeline = resolve_pipeline(args)?;
    let started_at = util::now_utc();
    let job_id = new_job_id(&args.customer_id);
    let work_dir = work_dir();
    let customer_dir = work_dir.join(&args.customer_id);
    std::fs::create_dir_all(&customer_dir)
        .map_err(|e| CliError::new(format!("创建工作目录失败: {e}")))?;

    let raw_path = customer_dir.join("raw.csv");
    let expected_output_path = customer_dir.join("final.csv");
    let link_path = customer_dir.join("share-link.txt");
    let log_path = util::catalog_dir()
        .join("jobs")
        .join(format!("{job_id}.log"));

    let mut log_lines = vec![
        format!("started_at={started_at}"),
        format!("customer_id={}", args.customer_id),
        format!("source_url={}", redact_source(&args.source_url)),
        format!("pipeline={pipeline}"),
        format!("raw_path={}", path_string(&raw_path)),
        format!("output_path={}", path_string(&expected_output_path)),
        format!("link_path={}", path_string(&link_path)),
    ];
    if let Some(bp) = &args.blueprint {
        log_lines.push(format!("blueprint={bp}"));
    }

    println!("══════════════════════════════════════════════");
    println!("  客户: {}", args.customer_id);
    println!("  来源: {}", redact_source(&args.source_url));
    if let Some(bp) = &args.blueprint {
        println!("  Blueprint: {}", bp);
    }
    println!("  Pipeline: {pipeline}");
    println!("══════════════════════════════════════════════");
    println!();

    let mut executor = StepExecutor {
        args,
        job_id,
        pipeline,
        customer_dir,
        raw_path,
        expected_output_path,
        link_path,
        log_path,
        started_at,
        log_lines,
    };
    executor.run()
}

// ── pipeline 解析 ──
fn resolve_pipeline(args: &ProcessArgs) -> Result<String, CliError> {
    if let Some(bp) = &args.blueprint {
        resolve_blueprint_pipeline(bp)
    } else {
        Ok(args.pipeline.clone().unwrap_or_else(|| {
            std::env::var("PIPELINE").unwrap_or_else(|_| "csv-standard".to_string())
        }))
    }
}

/// Receive → Pipeline → Send 状态机：统一失败处理（记录 failed job + 日志后返回 Err）。
struct StepExecutor<'a> {
    args: &'a ProcessArgs,
    job_id: String,
    pipeline: String,
    customer_dir: PathBuf,
    raw_path: PathBuf,
    expected_output_path: PathBuf,
    link_path: PathBuf,
    log_path: PathBuf,
    started_at: String,
    log_lines: Vec<String>,
}

impl StepExecutor<'_> {
    fn run(&mut self) -> Result<(), CliError> {
        self.receive()?;
        let result_path = path_string(&self.pipeline()?);
        let link = self.send(&result_path)?;
        self.finish_delivered(&result_path, &link)
    }

    fn receive(&mut self) -> Result<(), CliError> {
        println!("▶ Step 1: 接收数据");
        transfer::receive(&self.args.source_url, &self.raw_path, "dropbox")
            .map_err(|err| self.fail(format!("receive failed: {err}")))?;
        self.log_lines.push("receive completed".to_string());
        println!("✓ 已接收");
        println!();
        Ok(())
    }

    fn pipeline(&mut self) -> Result<PathBuf, CliError> {
        println!("▶ Step 2: 执行 Pipeline");
        let result_path = run_pipeline(
            &path_string(&self.raw_path),
            &path_string(&self.customer_dir),
            &self.pipeline,
        )
        .map_err(|err| self.fail(format!("pipeline failed: {err}")))?;
        self.log_lines
            .push(format!("pipeline completed output={result_path}"));
        println!("✓ Pipeline 完成");
        println!();
        Ok(PathBuf::from(result_path))
    }

    fn send(&mut self, result_path: &str) -> Result<String, CliError> {
        println!("▶ Step 3: 交付结果");
        let link = transfer::send(result_path, None, Some(&self.link_path), "dropbox")
            .map_err(|err| self.fail(format!("send failed: {err}")))?;
        self.log_lines.push("send completed".to_string());
        Ok(link)
    }

    fn finish_delivered(&mut self, result_path: &str, link: &str) -> Result<(), CliError> {
        register_process_output(&self.job_id, result_path);
        self.log_lines
            .push(format!("catalog registered source=process:{}", self.job_id));
        let record = ProcessJobRecord::delivered(self.record_input(result_path));
        self.persist_record(&record);

        println!("✓ 结果已交付: {link}");
        println!();
        println!("────────────────────────────────────────────");
        println!("✓ 完成: {}", self.args.customer_id);
        println!("  原始数据: {}", self.raw_path.to_string_lossy());
        println!("  最终结果: {result_path}");
        Ok(())
    }

    /// 失败统一出口：记录日志、落 failed job 记录，返回 CliError。
    fn fail(&mut self, log_line: String) -> CliError {
        let output_path = path_string(&self.expected_output_path);
        self.log_lines.push(log_line.clone());
        let record = ProcessJobRecord::failed(self.record_input(&output_path));
        self.persist_record(&record);
        CliError::new(log_line)
    }

    fn record_input(&self, output_path: &str) -> ProcessJobRecordInput {
        ProcessJobRecordInput {
            id: self.job_id.clone(),
            customer_id: self.args.customer_id.clone(),
            source_url: self.args.source_url.clone(),
            blueprint: self.args.blueprint.clone(),
            pipeline: self.pipeline.clone(),
            raw_path: path_string(&self.raw_path),
            output_path: output_path.to_string(),
            link_path: path_string(&self.link_path),
            log_path: path_string(&self.log_path),
            started_at: self.started_at.clone(),
            finished_at: util::now_utc(),
        }
    }

    fn persist_record(&self, record: &ProcessJobRecord) {
        if let Err(err) = write_job_log(&self.log_path, &self.log_lines) {
            eprintln!("写入 process 日志失败: {err}");
        }
        if let Err(err) = save_job_record(record) {
            eprintln!("写入 process job 记录失败: {err}");
        }
    }
}

// ── job 记录与日志 ──
fn register_process_output(job_id: &str, result_path: &str) {
    let volume_name = format!("{job_id}-final");
    let source = format!("process:{job_id}");

    if let Err(err) = catalog::register_volume(RegisterVolume {
        path: result_path,
        name: Some(&volume_name),
        provider: Some("process"),
        source: Some(&source),
        status: catalog::VolumeStatus::Delivered,
    }) {
        eprintln!("登记 process 产物到 catalog 失败: {err}");
    }
}

fn resolve_blueprint_pipeline(name: &str) -> Result<String, CliError> {
    let dir =
        std::env::var("BLUEPRINT_DIR").unwrap_or_else(|_| ".quanttide/data/blueprint".to_string());
    let key = crate::util::to_camel(name);
    let output = Command::new("cue")
        .args([
            "export",
            "--out",
            "json",
            "--expression",
            &format!("{key}.pipeline"),
            &dir,
        ])
        .output()
        .map_err(|err| CliError::new(format!("执行 cue 失败，请先安装 cue (v0.16+): {err}")))?;
    if !output.status.success() {
        return Err(CliError::new(format!(
            "找不到 Blueprint: {name}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let pipe: String = serde_json::from_slice(&output.stdout)
        .map_err(|err| CliError::new(format!("解析 Blueprint pipeline 失败: {err}")))?;
    if pipe.trim().is_empty() {
        return Err(CliError::new(format!("Blueprint {name} 中未定义 pipeline")));
    }
    Ok(pipe)
}

// ── 脱敏工具 ──
/// 脱敏来源 URL：移除 query 与 fragment（不含 token）。
pub fn redact_source(source: &str) -> String {
    let without_fragment = source.split('#').next().unwrap_or(source);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);

    if let Some((scheme, rest)) = without_query.split_once("://")
        && let Some((_, host_and_path)) = rest.rsplit_once('@')
    {
        return format!("{scheme}://{host_and_path}");
    }

    without_query.to_string()
}

fn save_job_record(record: &ProcessJobRecord) -> io::Result<()> {
    save_job_record_in(&util::catalog_dir(), record)
}

fn save_job_record_in(catalog_dir: &Path, record: &ProcessJobRecord) -> io::Result<()> {
    let mut registry = registry::Registry::open(&catalog_dir.join("jobs.json"))?;
    registry.insert(record.id.clone(), record.clone())
}

fn write_job_log(path: &Path, lines: &[String]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", lines.join("\n")))
}

fn work_dir() -> PathBuf {
    std::env::var("WORKDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("qtcloud-data"))
}

fn new_job_id(customer_id: &str) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{millis}", sanitize_id(customer_id))
}

fn sanitize_id(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "job".to_string()
    } else {
        sanitized.to_string()
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn run_pipeline(input: &str, work_dir: &str, pipeline_spec: &str) -> Result<String, String> {
    let mut prev = input.to_string();
    let steps: Vec<&str> = pipeline_spec.split(',').collect();

    for (i, step) in steps.iter().enumerate() {
        let step_name = std::path::Path::new(step)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(step);
        let step_output = if i == steps.len() - 1 {
            format!("{work_dir}/final.csv")
        } else {
            format!("{work_dir}/step_{i}_{step_name}.csv")
        };

        println!("  ▶ Step {}/{}: {step_name}", i + 1, steps.len());

        let status = if step.ends_with(".py") {
            Command::new("python3")
                .arg(step)
                .arg(&prev)
                .arg(&step_output)
                .status()
        } else if step.ends_with(".sh") {
            Command::new("bash")
                .arg(step)
                .arg(&prev)
                .arg(&step_output)
                .status()
        } else {
            Command::new(step).arg(&prev).arg(&step_output).status()
        }
        .map_err(|err| format!("执行 pipeline 步骤 {step_name} 失败: {err}"))?;

        if !status.success() {
            return Err(format!("Pipeline 步骤 {step_name} 失败"));
        }
        prev = step_output;
    }
    Ok(prev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::write_script;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn temp_catalog_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
    }

    fn script_path(dir: &Path, stem: &str) -> PathBuf {
        let ext = if cfg!(windows) { "cmd" } else { "sh" };
        dir.join(format!("{stem}.{ext}"))
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

    #[test]
    fn redact_source_removes_url_query_and_fragment() {
        let source = "https://example.com/data.csv?access_token=secret#download";

        assert_eq!(redact_source(source), "https://example.com/data.csv");
    }

    #[test]
    fn build_delivered_job_record_captures_handoff_fields() {
        let record = ProcessJobRecord::delivered(ProcessJobRecordInput {
            id: "job-1".to_string(),
            customer_id: "ABC-001".to_string(),
            source_url: "https://example.com/raw.csv?token=secret".to_string(),
            blueprint: Some("csv-standardization".to_string()),
            pipeline: "normalize.py,enrich.py".to_string(),
            raw_path: "work/ABC-001/raw.csv".to_string(),
            output_path: "work/ABC-001/final.csv".to_string(),
            link_path: "work/ABC-001/share-link.txt".to_string(),
            log_path: ".quanttide/data/catalog/jobs/job-1.log".to_string(),
            started_at: "2026-07-30 10:00:00".to_string(),
            finished_at: "2026-07-30 10:01:00".to_string(),
        });

        assert_eq!(record.id, "job-1");
        assert_eq!(record.customer_id, "ABC-001");
        assert_eq!(record.source_url, "https://example.com/raw.csv");
        assert_eq!(record.blueprint.as_deref(), Some("csv-standardization"));
        assert_eq!(record.pipeline, "normalize.py,enrich.py");
        assert_eq!(record.raw_path, "work/ABC-001/raw.csv");
        assert_eq!(record.output_path, "work/ABC-001/final.csv");
        assert_eq!(record.link_path, "work/ABC-001/share-link.txt");
        assert_eq!(record.log_path, ".quanttide/data/catalog/jobs/job-1.log");
        assert_eq!(record.status, "delivered");
    }

    #[test]
    fn build_failed_job_record_captures_failed_status() {
        let record = ProcessJobRecord::failed(ProcessJobRecordInput {
            id: "job-2".to_string(),
            customer_id: "ABC-002".to_string(),
            source_url: "sftp://user:secret@example.com/raw.csv".to_string(),
            blueprint: None,
            pipeline: "normalize.py".to_string(),
            raw_path: "work/ABC-002/raw.csv".to_string(),
            output_path: "work/ABC-002/final.csv".to_string(),
            link_path: "work/ABC-002/share-link.txt".to_string(),
            log_path: ".quanttide/data/catalog/jobs/job-2.log".to_string(),
            started_at: "2026-07-30 10:00:00".to_string(),
            finished_at: "2026-07-30 10:01:00".to_string(),
        });

        assert_eq!(record.status, "failed");
        assert_eq!(record.source_url, "sftp://example.com/raw.csv");
    }

    #[test]
    fn step_executor_delivers_and_writes_delivered_record() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_catalog_dir("qtcloud-executor-delivered");
        std::fs::create_dir_all(&root).unwrap();

        let fake_qtdata = script_path(&root, "fake-qtdata");
        let pipeline = script_path(&root, "copy-pipeline");
        write_script(&fake_qtdata, fake_qtdata_script());
        write_script(&pipeline, copy_pipeline_script());

        let catalog_dir = root.join("catalog");
        let work_dir = root.join("work");
        unsafe {
            std::env::set_var("QTDATA_CLI", &fake_qtdata);
            std::env::set_var("CATALOG_DIR", &catalog_dir);
            std::env::set_var("WORKDIR", &work_dir);
        }

        let args = ProcessArgs {
            customer_id: "EXEC-001".to_string(),
            source_url: "https://example.com/raw.csv?token=secret".to_string(),
            blueprint: None,
            pipeline: Some(pipeline.to_string_lossy().to_string()),
        };
        let result = run(&args);

        unsafe {
            std::env::remove_var("QTDATA_CLI");
            std::env::remove_var("CATALOG_DIR");
            std::env::remove_var("WORKDIR");
        }

        assert!(result.is_ok(), "run 应成功: {:?}", result);
        let content = std::fs::read_to_string(catalog_dir.join("jobs.json")).unwrap();
        let jobs: BTreeMap<String, ProcessJobRecord> = serde_json::from_str(&content).unwrap();
        let record = jobs.values().next().unwrap();
        assert_eq!(record.customer_id, "EXEC-001");
        assert_eq!(record.status, "delivered");
        assert_eq!(record.source_url, "https://example.com/raw.csv");
        assert!(work_dir.join("EXEC-001").join("final.csv").is_file());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn step_executor_propagates_pipeline_failure_and_writes_failed_record() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_catalog_dir("qtcloud-executor-failed");
        std::fs::create_dir_all(&root).unwrap();

        let fake_qtdata = script_path(&root, "fake-qtdata");
        let pipeline = script_path(&root, "failing-pipeline");
        write_script(&fake_qtdata, fake_qtdata_script());
        write_script(&pipeline, failing_pipeline_script());

        let catalog_dir = root.join("catalog");
        let work_dir = root.join("work");
        unsafe {
            std::env::set_var("QTDATA_CLI", &fake_qtdata);
            std::env::set_var("CATALOG_DIR", &catalog_dir);
            std::env::set_var("WORKDIR", &work_dir);
        }

        let args = ProcessArgs {
            customer_id: "EXEC-002".to_string(),
            source_url: "https://example.com/raw.csv".to_string(),
            blueprint: None,
            pipeline: Some(pipeline.to_string_lossy().to_string()),
        };
        let result = run(&args);

        unsafe {
            std::env::remove_var("QTDATA_CLI");
            std::env::remove_var("CATALOG_DIR");
            std::env::remove_var("WORKDIR");
        }

        assert!(result.is_err(), "pipeline 失败应返回 Err");
        let content = std::fs::read_to_string(catalog_dir.join("jobs.json")).unwrap();
        let jobs: BTreeMap<String, ProcessJobRecord> = serde_json::from_str(&content).unwrap();
        let record = jobs.values().next().unwrap();
        assert_eq!(record.customer_id, "EXEC-002");
        assert_eq!(record.status, "failed");
        assert!(record.output_path.ends_with("final.csv"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn save_job_record_creates_jobs_registry() {
        let catalog_dir = temp_catalog_dir("qtcloud-process-jobs");
        let record = ProcessJobRecord {
            id: "job-1".to_string(),
            customer_id: "ABC-001".to_string(),
            source_url: "https://example.com/raw.csv".to_string(),
            blueprint: None,
            pipeline: "normalize.py".to_string(),
            raw_path: "work/ABC-001/raw.csv".to_string(),
            output_path: "work/ABC-001/final.csv".to_string(),
            link_path: "work/ABC-001/share-link.txt".to_string(),
            log_path: "jobs/job-1.log".to_string(),
            status: "delivered".to_string(),
            started_at: "2026-07-30 10:00:00".to_string(),
            finished_at: "2026-07-30 10:01:00".to_string(),
        };

        save_job_record_in(&catalog_dir, &record).unwrap();

        let registry_path = catalog_dir.join("jobs.json");
        let content = std::fs::read_to_string(&registry_path).unwrap();
        let jobs: BTreeMap<String, ProcessJobRecord> = serde_json::from_str(&content).unwrap();
        assert_eq!(
            jobs.get("job-1").unwrap().output_path,
            "work/ABC-001/final.csv"
        );

        std::fs::remove_dir_all(&catalog_dir).ok();
    }
}
