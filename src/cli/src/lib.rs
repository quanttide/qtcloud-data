//! qtcloud-data-cli 库入口：模块注册与测试共享支持。

pub mod catalog;
pub mod doctor;
pub mod error;
pub mod pipeline;
pub mod registry;
pub mod review;
pub mod spec;
pub mod stage;
pub mod storage;
pub mod util;
pub mod version;

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
