//! 二进制入口：CLI 参数解析，命令分发由库层 `run_command` 负责。

use clap::Parser;
use qtcloud_data_cli::{Commands, OutputMode, run_command_with_mode};

#[derive(Parser)]
#[command(name = "qtcloud-data", about = "量潮数据云 CLI")]
struct Cli {
    /// 以机器可读 JSON 输出命令错误
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run_command_with_mode(
        &cli.command,
        if cli.json {
            OutputMode::Json
        } else {
            OutputMode::Text
        },
    ) {
        if cli.json {
            println!(
                "{}",
                serde_json::json!({
                    "error": err.to_json_value(),
                })
            );
        } else {
            eprintln!("错误: {err}");
        }
        std::process::exit(1);
    }
}
