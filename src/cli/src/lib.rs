pub mod blueprint;
pub mod blueprint_core;
pub mod catalog;
pub mod clarify;
pub mod contract;
pub mod design;
pub mod doctor;
pub mod error;
pub mod implement;
pub mod pipeline;
pub mod process;
pub mod providers;
pub mod review;
pub mod spec;
pub mod store;
pub mod transfer;
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
}
