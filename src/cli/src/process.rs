use clap::Args;
use std::process::Command;

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

pub fn run(args: &ProcessArgs) {
    let pipeline = if let Some(bp) = &args.blueprint {
        resolve_blueprint_pipeline(bp)
    } else {
        args.pipeline.clone().unwrap_or_else(|| {
            std::env::var("PIPELINE").unwrap_or_else(|_| "csv-standard".to_string())
        })
    };

    let qtdata = std::env::var("QTDATA_CLI").unwrap_or_else(|_| "qtcloud-data".to_string());
    let work_dir = std::env::temp_dir().join("qtcloud-data");
    let work_dir = work_dir.to_string_lossy().to_string();
    let customer_dir = format!("{}/{}", work_dir, args.customer_id);
    std::fs::create_dir_all(&customer_dir).expect("创建工作目录失败");

    println!("══════════════════════════════════════════════");
    println!("  客户: {}", args.customer_id);
    println!("  来源: {}", args.source_url);
    if args.blueprint.is_some() {
        println!("  Blueprint: {}", args.blueprint.as_ref().unwrap());
    }
    println!("  Pipeline: {}", pipeline);
    println!("══════════════════════════════════════════════");
    println!();

    // Step 1: Receive
    println!("▶ Step 1: 接收数据");
    let raw_file = format!("{customer_dir}/raw.csv");
    let status = Command::new(&qtdata)
        .args([
            "transfer",
            "receive",
            &args.source_url,
            "--output",
            &raw_file,
        ])
        .status()
        .expect("执行 receive 失败");
    if !status.success() {
        eprintln!("接收失败");
        std::process::exit(1);
    }
    println!("✓ 已接收");
    println!();

    // Step 2: Pipeline
    println!("▶ Step 2: 执行 Pipeline");
    let result_file = run_pipeline(&raw_file, &customer_dir, &pipeline);
    println!("✓ Pipeline 完成");
    println!();

    // Step 3: Send
    println!("▶ Step 3: 交付结果");
    let link_file = format!("{customer_dir}/share-link.txt");
    let status = Command::new(&qtdata)
        .args(["transfer", "send", &result_file, "--output", &link_file])
        .status()
        .expect("执行 send 失败");
    if !status.success() {
        eprintln!("交付失败");
        std::process::exit(1);
    }
    let link = std::fs::read_to_string(&link_file).unwrap_or_default();
    println!("✓ 结果已交付: {link}");
    println!();
    println!("────────────────────────────────────────────");
    println!("✓ 完成: {}", args.customer_id);
    println!("  原始数据: {raw_file}");
    println!("  最终结果: {result_file}");
}

fn resolve_blueprint_pipeline(name: &str) -> String {
    let dir =
        std::env::var("BLUEPRINT_DIR").unwrap_or_else(|_| ".quanttide/data/blueprint".to_string());
    let key = to_camel(name);
    let output = Command::new("cue")
        .args([
            "export",
            "--out",
            "yaml",
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
    let pipe = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_matches('"')
        .to_string();
    if pipe.is_empty() {
        eprintln!("Blueprint {name} 中未定义 pipeline");
        std::process::exit(1);
    }
    pipe
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

fn run_pipeline(input: &str, work_dir: &str, pipeline_spec: &str) -> String {
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
        .expect("执行 pipeline 步骤失败");

        if !status.success() {
            eprintln!("Pipeline 步骤 {step_name} 失败");
            std::process::exit(1);
        }
        prev = step_output;
    }
    prev
}
