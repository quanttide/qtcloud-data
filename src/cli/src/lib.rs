//! qtcloud-data-cli 库入口：模块注册与测试共享支持。

use clap::Subcommand;

pub mod doctor;
pub mod error;
pub mod implementation;
pub mod output;
pub mod registry;
pub mod review;
pub mod runtime;
pub mod spec;
pub mod stage;
pub mod storage;
pub mod util;

/// 命令输出模式；JSON 模式逐个命令迁移，文本模式保持现有行为。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputMode {
    #[default]
    Text,
    Json,
}

/// CLI 子命令集合，供二进制入口解析，也供库级分发测试复用。
#[derive(Subcommand)]
pub enum Commands {
    /// 从客户上下文澄清需求 → 生成 DRD（数据需求文档）
    Clarify(stage::clarify::ClarifyArgs),
    /// 设计 Specification（Contract + Blueprint）← 从 DRD
    Design(stage::design::DesignArgs),
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
    Pipeline(implementation::pipeline::PipelineArgs),
    /// 数据目录
    Catalog(implementation::catalog::CatalogArgs),
    /// 从 Specification 生成代码实现
    Implement(stage::implement::ImplementArgs),
    /// 编排流程（receive → pipeline → send）
    Process(stage::process::ProcessArgs),
    /// 数据传输（send / receive）
    Transfer(stage::transfer::TransferArgs),
}

/// 使用生产环境默认 LLM 执行一个 CLI 子命令。
pub fn run_command(command: &Commands) -> Result<(), error::CliError> {
    run_command_with_mode(command, OutputMode::Text)
}

/// 使用调用方注入的 LLM 执行一个 CLI 子命令，便于测试和集成。
pub fn run_command_with_llm(
    command: &Commands,
    llm: quanttide_agent::LLM,
) -> Result<(), error::CliError> {
    dispatch_command(command, Some(llm), OutputMode::Text)
}

/// 使用指定输出模式执行一个 CLI 子命令。
pub fn run_command_with_mode(command: &Commands, mode: OutputMode) -> Result<(), error::CliError> {
    dispatch_command(command, None, mode)
}

fn dispatch_command(
    command: &Commands,
    injected_llm: Option<quanttide_agent::LLM>,
    mode: OutputMode,
) -> Result<(), error::CliError> {
    match command {
        Commands::Clarify(args) => {
            stage::clarify::ClarifyHandler::new(injected_llm.unwrap_or_default()).run(args)
        }
        Commands::Design(args) => {
            stage::design::DesignHandler::new(injected_llm.unwrap_or_default()).run(args)
        }
        Commands::Review(args) => {
            review::ReviewHandler::new(injected_llm.unwrap_or_default()).run(args)
        }
        Commands::Spec(args) => spec::run_with_mode(args, mode),
        Commands::Version(args) => spec::version::run(args),
        Commands::Doctor(args) => doctor::run(args),
        Commands::Blueprint(args) => spec::blueprint::run_with_mode(args, mode),
        Commands::Contract(args) => spec::contract::run_with_mode(args, mode),
        Commands::Pipeline(args) => implementation::pipeline::run_with_mode(args, mode),
        Commands::Catalog(args) => implementation::catalog::run_with_mode(args, mode),
        Commands::Implement(args) => {
            stage::implement::ImplementHandler::new(injected_llm.unwrap_or_default()).run(args)
        }
        Commands::Process(args) => stage::process::run(args),
        Commands::Transfer(args) => stage::transfer::run(args),
    }
}

/// 测试共享的全局环境变量锁：各模块测试直接 `std::env::set_var` 时统一互斥，
/// 避免并行执行互相污染进程级环境变量。
#[cfg(test)]
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// LLM 命令测试支持：构造返回预设响应的假 LLM（复用 quanttide-agent 的
/// `HttpClient` 抽象，不发起真实网络请求）。
#[cfg(test)]
pub mod test_support {
    use quanttide_agent::llm::{HttpClient, LLMError};

    pub struct FakeHttpClient {
        pub response: serde_json::Value,
    }

    impl HttpClient for FakeHttpClient {
        fn post_json(
            &self,
            _url: &str,
            _auth: &str,
            _body: &serde_json::Value,
        ) -> Result<serde_json::Value, LLMError> {
            Ok(self.response.clone())
        }
    }

    /// 构造一个 `complete()` 返回给定 OpenAI 格式 JSON 的假 LLM。
    pub fn fake_llm(content: &str) -> quanttide_agent::LLM {
        let response = serde_json::json!({
            "choices": [{
                "message": { "content": content },
                "finish_reason": "stop"
            }]
        });
        quanttide_agent::LLM::with_client(
            "test-model",
            "http://fake",
            "test-key",
            Box::new(FakeHttpClient { response }),
        )
    }

    /// 临时目录：删除残留后创建（名字前缀 + pid，避免并行冲突）。
    /// RAII 临时目录：`Drop` 时自动清理（Rust 版 fixture teardown，panic 也兜底）。
    pub struct TempDir(std::path::PathBuf);

    impl std::ops::Deref for TempDir {
        type Target = std::path::Path;

        fn deref(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl AsRef<std::path::Path> for TempDir {
        fn as_ref(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl AsRef<std::ffi::OsStr> for TempDir {
        fn as_ref(&self) -> &std::ffi::OsStr {
            self.0.as_os_str()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 创建临时目录（名字前缀 + pid），返回 RAII 句柄，作用域结束时自动删除。
    pub fn temp_dir(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    /// 写可执行脚本（unix 设 0o755），父目录自动创建。
    pub fn write_script(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{SpecAction, SpecArgs};
    use crate::stage::clarify::{ClarifyAction, ClarifyArgs};

    #[test]
    fn command_dispatch_accepts_injected_llm() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = test_support::temp_dir("qtcloud-command-dispatch");
        let chat = root.join("chat.md");
        let drd_dir = root.join("drd");
        std::fs::write(&chat, "客户需要清洗订单数据\n").unwrap();

        unsafe {
            std::env::set_var("DRD_DIR", &drd_dir);
        }

        let command = Commands::Clarify(ClarifyArgs {
            action: ClarifyAction::FromChat {
                input: chat.to_string_lossy().into_owned(),
            },
        });
        run_command_with_llm(&command, test_support::fake_llm("# 订单数据 DRD\n")).unwrap();

        unsafe {
            std::env::remove_var("DRD_DIR");
        }

        assert_eq!(
            std::fs::read_to_string(drd_dir.join("chat.md")).unwrap(),
            "# 订单数据 DRD\n"
        );
    }

    #[test]
    fn command_dispatch_uses_default_path_for_non_llm_command() {
        let root = test_support::temp_dir("qtcloud-command-dispatch-default");
        let missing = root.join("missing.yaml");
        let command = Commands::Spec(SpecArgs {
            action: SpecAction::Validate {
                input: missing.to_string_lossy().into_owned(),
            },
        });

        let error = run_command(&command).unwrap_err();
        assert!(error.to_string().contains("无法读取 YAML"));
    }
}
