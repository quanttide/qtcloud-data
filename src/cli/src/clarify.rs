use clap::{Args, Subcommand};
use std::path::Path;

use crate::error::CliError;
use crate::util;

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

    pub fn run(&self, args: &ClarifyArgs) -> Result<(), CliError> {
        match &args.action {
            ClarifyAction::FromChat { input } => self.cmd_from_chat(input),
        }
    }

    fn cmd_from_chat(&self, input: &str) -> Result<(), CliError> {
        let chat_path = Path::new(input);
        let chat_content = std::fs::read_to_string(chat_path)
            .map_err(|e| CliError::new(format!("无法读取文件 {}: {e}", input)))?;

        let prompt = clarify_prompt(&chat_content);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        println!("正在分析聊天记录，生成 DRD ...");
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let drd_content = resp.content;
                let drd_dir = util::drd_dir();
                std::fs::create_dir_all(&drd_dir)
                    .map_err(|e| CliError::new(format!("无法创建目录 {drd_dir}: {e}")))?;

                // Use the chat filename stem as DRD name
                let stem = chat_path.file_stem().unwrap_or_default().to_string_lossy();
                let output_path = Path::new(&drd_dir).join(format!("{stem}.md"));
                std::fs::write(&output_path, &drd_content)
                    .map_err(|e| CliError::new(format!("写入 DRD 失败: {e}")))?;
                println!("已生成: {}", output_path.display());
                Ok(())
            }
            Err(e) => Err(CliError::new(format!("LLM 调用失败: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarify_prompt_contains_sections() {
        let prompt = clarify_prompt("客户是做电商的，需要清洗订单数据");
        assert!(prompt.contains("业务背景"));
        assert!(prompt.contains("数据来源"));
        assert!(prompt.contains("期望产出"));
        assert!(prompt.contains("约束与要求"));
        assert!(prompt.contains("待确认事项"));
        assert!(prompt.contains("客户是做电商的"));
    }

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
        handler
            .run(&ClarifyArgs {
                action: ClarifyAction::FromChat {
                    input: chat.to_string_lossy().to_string(),
                },
            })
            .unwrap();
        unsafe {
            std::env::remove_var("DRD_DIR");
        }

        let drd = std::fs::read_to_string(drd_dir.join("chat.md")).unwrap();
        assert!(drd.contains("# DRD：GitHub 活跃度"));

        std::fs::remove_dir_all(&root).ok();
    }
}

// ── 自 blueprint_core 回迁 ──

/// Build the clarify prompt for LLM: convert chat logs into a DRD.
pub fn clarify_prompt(chat: &str) -> String {
    format!(
        r#"你是一个数据工程需求分析师。请从以下客户聊天记录中，提取并生成一份数据需求文档（DRD）。

DRD 是面向客户沟通用的，用业务语言撰写。包含以下章节：

# <项目名称>

## 1. 业务背景
- 客户是谁，做什么业务
- 当前面临什么问题

## 2. 数据来源
- 客户能提供什么数据（格式、大致规模、更新频率）
- 是否有样例数据

## 3. 期望产出
- 客户希望最终拿到什么（报表？数据集？API？）
- 对产出格式有什么偏好

## 4. 约束与要求
- 时间要求
- 安全/合规要求
- 其他特殊要求

## 5. 待确认事项
- 哪些信息客户还没说清楚，需要后续确认

聊天记录:
{chat}"#
    )
}
