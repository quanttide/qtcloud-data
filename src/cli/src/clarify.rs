use clap::{Args, Subcommand};
use std::path::Path;

use crate::blueprint_core;

#[derive(Args)]
pub struct ClarifyArgs {
    #[command(subcommand)]
    pub action: ClarifyAction,
}

#[derive(Subcommand)]
pub enum ClarifyAction {
    /// 从聊天记录/上下文生成数据需求文档（DRD）
    FromChat {
        /// 输入的聊天记录文件路径（.txt 或 .md）
        input: String,
    },
}

pub struct ClarifyHandler {
    llm: quanttide_agent::LLM,
}

impl ClarifyHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self {
        Self { llm }
    }

    pub fn run(&self, args: &ClarifyArgs) {
        match &args.action {
            ClarifyAction::FromChat { input } => self.cmd_from_chat(input),
        }
    }

    fn cmd_from_chat(&self, input: &str) {
        let chat_path = Path::new(input);
        let chat_content = std::fs::read_to_string(chat_path).unwrap_or_else(|e| {
            eprintln!("无法读取文件 {}: {e}", input);
            std::process::exit(1);
        });

        let prompt = blueprint_core::clarify_prompt(&chat_content);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        println!("正在分析聊天记录，生成 DRD ...");
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let drd_content = resp.content;
                let drd_dir = blueprint_core::drd_dir();
                std::fs::create_dir_all(&drd_dir).unwrap_or_else(|e| {
                    eprintln!("无法创建目录 {drd_dir}: {e}");
                    std::process::exit(1);
                });

                // Use the chat filename stem as DRD name
                let stem = chat_path.file_stem().unwrap_or_default().to_string_lossy();
                let output_path = Path::new(&drd_dir).join(format!("{stem}.md"));
                std::fs::write(&output_path, &drd_content).unwrap_or_else(|e| {
                    eprintln!("写入 DRD 失败: {e}");
                    std::process::exit(1);
                });
                println!("已生成: {}", output_path.display());
            }
            Err(e) => {
                eprintln!("LLM 调用失败: {e}");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::fake_llm;

    #[test]
    fn clarify_writes_drd_from_llm_response() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = std::env::temp_dir().join(format!("qtcloud-clarify-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        let chat = root.join("chat.log");
        std::fs::write(&chat, "user: 我要统计 GitHub 活跃度\n").unwrap();
        let drd_dir = root.join("drd");

        unsafe {
            std::env::set_var("DRD_DIR", &drd_dir);
        }
        let handler = ClarifyHandler::new(fake_llm("# DRD：GitHub 活跃度\n## 需求\n..."));
        handler.run(&ClarifyArgs {
            action: ClarifyAction::FromChat {
                input: chat.to_string_lossy().to_string(),
            },
        });
        unsafe {
            std::env::remove_var("DRD_DIR");
        }

        let drd = std::fs::read_to_string(drd_dir.join("chat.md")).unwrap();
        assert!(drd.contains("# DRD：GitHub 活跃度"));

        std::fs::remove_dir_all(&root).ok();
    }
}
