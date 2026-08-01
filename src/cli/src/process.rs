use clap::Args;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::catalog::{self, RegisterVolume};
use crate::store;

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

pub fn run(args: &ProcessArgs) {
    let pipeline = if let Some(bp) = &args.blueprint {
        resolve_blueprint_pipeline(bp)
    } else {
        args.pipeline.clone().unwrap_or_else(|| {
            std::env::var("PIPELINE").unwrap_or_else(|_| "csv-standard".to_string())
        })
    };

    let qtdata = std::env::var("QTDATA_CLI").unwrap_or_else(|_| "qtcloud-data".to_string());
    let started_at = store::now_utc();
    let job_id = new_job_id(&args.customer_id);
    let work_dir = work_dir();
    let customer_dir = work_dir.join(&args.customer_id);
    std::fs::create_dir_all(&customer_dir).expect("创建工作目录失败");

    let raw_path = customer_dir.join("raw.csv");
    let expected_output_path = customer_dir.join("final.csv");
    let link_path = customer_dir.join("share-link.txt");
    let log_path = store::catalog_dir()
        .join("jobs")
        .join(format!("{job_id}.log"));

    let paths = ProcessJobPaths {
        raw: path_string(&raw_path),
        expected_output: path_string(&expected_output_path),
        link: path_string(&link_path),
        log: path_string(&log_path),
    };

    let mut log_lines = vec![
        format!("started_at={started_at}"),
        format!("customer_id={}", args.customer_id),
        format!("source_url={}", redact_source(&args.source_url)),
        format!("pipeline={pipeline}"),
        format!("raw_path={}", paths.raw),
        format!("output_path={}", paths.expected_output),
        format!("link_path={}", paths.link),
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
    println!("  Pipeline: {}", pipeline);
    println!("══════════════════════════════════════════════");
    println!();

    // Step 1: Receive
    println!("▶ Step 1: 接收数据");
    let status = Command::new(&qtdata)
        .args([
            "transfer",
            "receive",
            &args.source_url,
            "--output",
            &paths.raw,
        ])
        .status();
    let status = match status {
        Ok(status) => status,
        Err(err) => {
            log_lines.push(format!("receive failed to start: {err}"));
            save_final_job_record(
                FinalJobRecord {
                    args,
                    job_id: &job_id,
                    pipeline: &pipeline,
                    raw_path: &paths.raw,
                    output_path: &paths.expected_output,
                    link_path: &paths.link,
                    log_path: &paths.log,
                    started_at: &started_at,
                    status: "failed",
                },
                &log_lines,
            );
            eprintln!("执行 receive 失败: {err}");
            std::process::exit(1);
        }
    };
    if !status.success() {
        log_lines.push("receive failed".to_string());
        save_final_job_record(
            FinalJobRecord {
                args,
                job_id: &job_id,
                pipeline: &pipeline,
                raw_path: &paths.raw,
                output_path: &paths.expected_output,
                link_path: &paths.link,
                log_path: &paths.log,
                started_at: &started_at,
                status: "failed",
            },
            &log_lines,
        );
        eprintln!("接收失败");
        std::process::exit(1);
    }
    log_lines.push("receive completed".to_string());
    println!("✓ 已接收");
    println!();

    // Step 2: Pipeline
    println!("▶ Step 2: 执行 Pipeline");
    let customer_dir = path_string(&customer_dir);
    let result_path = match run_pipeline(&paths.raw, &customer_dir, &pipeline) {
        Ok(path) => path,
        Err(err) => {
            log_lines.push(format!("pipeline failed: {err}"));
            save_final_job_record(
                FinalJobRecord {
                    args,
                    job_id: &job_id,
                    pipeline: &pipeline,
                    raw_path: &paths.raw,
                    output_path: &paths.expected_output,
                    link_path: &paths.link,
                    log_path: &paths.log,
                    started_at: &started_at,
                    status: "failed",
                },
                &log_lines,
            );
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    log_lines.push(format!("pipeline completed output={result_path}"));
    println!("✓ Pipeline 完成");
    println!();

    // Step 3: Send
    println!("▶ Step 3: 交付结果");
    let status = Command::new(&qtdata)
        .args(["transfer", "send", &result_path, "--output", &paths.link])
        .status();
    let status = match status {
        Ok(status) => status,
        Err(err) => {
            log_lines.push(format!("send failed to start: {err}"));
            save_final_job_record(
                FinalJobRecord {
                    args,
                    job_id: &job_id,
                    pipeline: &pipeline,
                    raw_path: &paths.raw,
                    output_path: &result_path,
                    link_path: &paths.link,
                    log_path: &paths.log,
                    started_at: &started_at,
                    status: "failed",
                },
                &log_lines,
            );
            eprintln!("执行 send 失败: {err}");
            std::process::exit(1);
        }
    };
    if !status.success() {
        log_lines.push("send failed".to_string());
        save_final_job_record(
            FinalJobRecord {
                args,
                job_id: &job_id,
                pipeline: &pipeline,
                raw_path: &paths.raw,
                output_path: &result_path,
                link_path: &paths.link,
                log_path: &paths.log,
                started_at: &started_at,
                status: "failed",
            },
            &log_lines,
        );
        eprintln!("交付失败");
        std::process::exit(1);
    }
    let link = std::fs::read_to_string(&paths.link).unwrap_or_default();
    log_lines.push("send completed".to_string());
    register_process_output(&job_id, &result_path);
    log_lines.push(format!("catalog registered source=process:{job_id}"));
    save_final_job_record(
        FinalJobRecord {
            args,
            job_id: &job_id,
            pipeline: &pipeline,
            raw_path: &paths.raw,
            output_path: &result_path,
            link_path: &paths.link,
            log_path: &paths.log,
            started_at: &started_at,
            status: "delivered",
        },
        &log_lines,
    );
    println!("✓ 结果已交付: {link}");
    println!();
    println!("────────────────────────────────────────────");
    println!("✓ 完成: {}", args.customer_id);
    println!("  原始数据: {}", paths.raw);
    println!("  最终结果: {result_path}");
}

fn register_process_output(job_id: &str, result_path: &str) {
    let volume_name = format!("{job_id}-final");
    let source = format!("process:{job_id}");

    if let Err(err) = catalog::register_volume(RegisterVolume {
        path: result_path,
        name: Some(&volume_name),
        provider: Some("process"),
        source: Some(&source),
        status: "delivered",
    }) {
        eprintln!("登记 process 产物到 catalog 失败: {err}");
    }
}

fn resolve_blueprint_pipeline(name: &str) -> String {
    let dir =
        std::env::var("BLUEPRINT_DIR").unwrap_or_else(|_| ".quanttide/data/blueprint".to_string());
    let key = to_camel(name);
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
        .expect("执行 cue 失败，请先安装 cue (v0.16+)");
    if !output.status.success() {
        eprintln!("找不到 Blueprint: {name}");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    let pipe: String = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        eprintln!("解析 Blueprint pipeline 失败: {err}");
        std::process::exit(1);
    });
    if pipe.trim().is_empty() {
        eprintln!("Blueprint {name} 中未定义 pipeline");
        std::process::exit(1);
    }
    pipe
}

/// 从 cue 导出的 JSON 顶层收集各定义的 `name` 字段（结构化解析，替代文本 grep）。
pub fn collect_defined_names(value: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    if let serde_json::Value::Object(map) = value {
        for nested in map.values() {
            if let Some(name) = nested.get("name").and_then(|n| n.as_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

pub fn to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

struct FinalJobRecord<'a> {
    args: &'a ProcessArgs,
    job_id: &'a str,
    pipeline: &'a str,
    raw_path: &'a str,
    output_path: &'a str,
    link_path: &'a str,
    log_path: &'a str,
    started_at: &'a str,
    status: &'a str,
}

struct ProcessJobPaths {
    raw: String,
    expected_output: String,
    link: String,
    log: String,
}

fn save_final_job_record(input: FinalJobRecord<'_>, log_lines: &[String]) {
    let status = input.status;
    let log_path = input.log_path;
    let record_input = ProcessJobRecordInput {
        id: input.job_id.to_string(),
        customer_id: input.args.customer_id.clone(),
        source_url: input.args.source_url.clone(),
        blueprint: input.args.blueprint.clone(),
        pipeline: input.pipeline.to_string(),
        raw_path: input.raw_path.to_string(),
        output_path: input.output_path.to_string(),
        link_path: input.link_path.to_string(),
        log_path: input.log_path.to_string(),
        started_at: input.started_at.to_string(),
        finished_at: store::now_utc(),
    };

    let record = if status == "delivered" {
        ProcessJobRecord::delivered(record_input)
    } else {
        ProcessJobRecord::failed(record_input)
    };

    if let Err(err) = write_job_log(Path::new(log_path), log_lines) {
        eprintln!("写入 process 日志失败: {err}");
    }
    if let Err(err) = save_job_record(&record) {
        eprintln!("写入 process job 记录失败: {err}");
    }
}

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
    save_job_record_in(&store::catalog_dir(), record)
}

fn save_job_record_in(catalog_dir: &Path, record: &ProcessJobRecord) -> io::Result<()> {
    let mut registry = store::Registry::open(&catalog_dir.join("jobs.json"))?;
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
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn temp_catalog_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
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
