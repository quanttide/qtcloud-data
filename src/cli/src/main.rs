//! 二进制入口：CLI 参数解析与命令分发（run_command）。

use clap::{Parser, Subcommand};
use qtcloud_data_cli::error::CliError;
use qtcloud_data_cli::stage::{clarify, design, implement, process, transfer};
use qtcloud_data_cli::{catalog, doctor, pipeline, review, spec};

#[derive(Parser)]
#[command(name = "qtcloud-data", about = "量潮数据云 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 从客户上下文澄清需求 → 生成 DRD（数据需求文档）
    Clarify(clarify::ClarifyArgs),
    /// 设计 Specification（Contract + Blueprint）← 从 DRD
    Design(design::DesignArgs),
    /// 审计 Specification 完整性和一致性
    Review(review::ReviewArgs),
    /// Specification YAML 契约工具
    Spec(spec::SpecArgs),
    /// 规格版本管理（已废弃：v0.3 移除，改用 `spec version`）
    Version(spec::version::SpecVersionArgs),
    /// 检查本机 DataOps 环境
    Doctor(doctor::DoctorArgs),
    /// 蓝图管理（list / show）
    Blueprint(spec::blueprint::BlueprintArgs),
    /// 契约查看
    Contract(spec::contract::ContractArgs),
    /// 管道管理
    Pipeline(pipeline::PipelineArgs),
    /// 数据目录
    Catalog(catalog::CatalogArgs),
    /// 从 Specification 生成代码实现
    Implement(implement::ImplementArgs),
    /// 编排流程（receive → pipeline → send）
    Process(process::ProcessArgs),
    /// 数据传输（send / receive）
    Transfer(transfer::TransferArgs),
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run_command(&cli.command) {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}

/// 命令分发：返回 `Result<(), CliError>` 的命令由顶层统一格式化；
/// 其余命令保持内部 exit 处理（逐步迁移中）。
/// LLM 命令通过 Handler 注入：生产路径构造 `LLM::default()`，测试替换为 fake。
fn run_command(command: &Commands) -> Result<(), CliError> {
    match command {
        Commands::Clarify(args) => {
            clarify::ClarifyHandler::new(quanttide_agent::LLM::default()).run(args)
        }
        Commands::Design(args) => {
            design::DesignHandler::new(quanttide_agent::LLM::default()).run(args)
        }
        Commands::Review(args) => {
            review::ReviewHandler::new(quanttide_agent::LLM::default()).run(args)
        }
        Commands::Spec(args) => spec::run(args),
        Commands::Version(args) => spec::version::run(args),
        Commands::Doctor(args) => doctor::run(args),
        Commands::Blueprint(args) => spec::blueprint::run(args),
        Commands::Contract(args) => spec::contract::run(args),
        Commands::Pipeline(args) => pipeline::run(args),
        Commands::Catalog(args) => catalog::run(args),
        Commands::Implement(args) => {
            implement::ImplementHandler::new(quanttide_agent::LLM::default()).run(args)
        }
        Commands::Process(args) => process::run(args),
        Commands::Transfer(args) => transfer::run(args),
    }
}
