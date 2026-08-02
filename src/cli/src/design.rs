use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use crate::error::CliError;
use crate::spec;
use crate::util;

#[derive(Args)]
pub struct DesignArgs {
    #[command(subcommand)]
    pub action: DesignAction,
}

#[derive(Subcommand)]
pub enum DesignAction {
    /// 从 DRD 生成数据契约（Contract: .yaml + .md）
    Contract {
        /// DRD .md 文件路径
        input: String,
    },
    /// 从 DRD 生成处理蓝图（Blueprint: .yaml + .md + .html）
    Blueprint {
        /// DRD .md 文件路径
        input: String,
    },
    /// 将 Markdown 形式化为 YAML 结构化定义
    Formalize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 从 YAML 生成可视化 HTML 页面
    Preview {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub struct DesignHandler {
    llm: quanttide_agent::LLM,
}

impl DesignHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self {
        Self { llm }
    }

    pub fn run(&self, args: &DesignArgs) -> Result<(), CliError> {
        match &args.action {
            DesignAction::Contract { input } => self.cmd_contract(input),
            DesignAction::Blueprint { input } => self.cmd_blueprint(input),
            DesignAction::Formalize { input, output } => self.cmd_formalize(input, output),
            DesignAction::Preview { input, output } => self.cmd_preview(input, output),
        }
    }

    // ── Contract: LLM outputs Markdown table, code generates YAML ──

    fn cmd_contract(&self, input: &str) -> Result<(), CliError> {
        let drd = read_drd(input)?;
        let prompt = design_contract_prompt(&drd);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        let stem = Path::new(input)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("正在从 DRD 生成 Contract: {stem} ...");

        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let (yaml_content, md_content) = contract_tables_to_yaml(&resp.content)?;
                write_spec_files(&stem, "contract", &yaml_content, &md_content)?;
                Ok(())
            }
            Err(e) => Err(CliError::new(format!("LLM 调用失败: {e}"))),
        }
    }

    // ── Blueprint: LLM outputs Markdown table, code generates YAML ──

    fn cmd_blueprint(&self, input: &str) -> Result<(), CliError> {
        let drd = read_drd(input)?;
        let prompt = design_blueprint_prompt(&drd);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        let stem = Path::new(input)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("正在从 DRD 生成 Blueprint: {stem} ...");

        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let (yaml_content, md_content) = blueprint_table_to_yaml(&resp.content, &stem);
                write_spec_files(&stem, "blueprint", &yaml_content, &md_content)?;

                // Generate HTML preview from YAML
                let bp: quanttide_data::Blueprint = serde_yaml::from_str(&yaml_content)
                    .map_err(|e| CliError::new(format!("解析 YAML 失败: {e}")))?;
                let step_refs: Vec<(&str, &str, &str, &str)> = bp
                    .pipeline
                    .steps
                    .iter()
                    .map(|s| {
                        (
                            s.name.as_str(),
                            s.from.as_str(),
                            s.to.as_str(),
                            s.desc.as_str(),
                        )
                    })
                    .collect();
                let html = render_html(
                    &bp.name,
                    bp.description.as_deref(),
                    bp.status.as_str(),
                    &bp.created_at,
                    &bp.updated_at,
                    "",
                    "",
                    &step_refs,
                );
                let spec_dir = util::spec_dir();
                let html_path = Path::new(&spec_dir).join(format!("{stem}-blueprint.html"));
                std::fs::write(&html_path, &html)
                    .map_err(|e| CliError::new(format!("写入 .html 失败: {e}")))?;
                println!("已生成: {}", html_path.display());
                Ok(())
            }
            Err(e) => Err(CliError::new(format!("LLM 调用失败: {e}"))),
        }
    }

    // ── Formalize ──

    fn cmd_formalize(&self, input: &str, output: &Option<String>) -> Result<(), CliError> {
        let md_path = Path::new(input);
        let md_content = std::fs::read_to_string(md_path)
            .map_err(|e| CliError::new(format!("无法读取 .md 文件: {e}")))?;

        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = md_path.file_stem().unwrap_or_default();
                Path::new(&util::spec_dir())
                    .join(stem)
                    .with_extension("yaml")
            }
        };

        let prompt = design_formalize_prompt(&md_content);
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        println!("正在形式化 {} ...", md_path.display());
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let yaml_code = extract_cue(&resp.content);
                std::fs::write(&output_path, &yaml_code)
                    .map_err(|e| CliError::new(format!("写入 .yaml 失败: {e}")))?;
                println!("已生成: {}", output_path.display());
                Ok(())
            }
            Err(e) => Err(CliError::new(format!("LLM 调用失败: {e}"))),
        }
    }

    // ── Preview ──

    fn cmd_preview(&self, input: &str, output: &Option<String>) -> Result<(), CliError> {
        let yaml_path = Path::new(input);
        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = yaml_path.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{stem}.html"))
            }
        };

        let yaml_content = std::fs::read_to_string(yaml_path)
            .map_err(|e| CliError::new(format!("无法读取 .yaml: {e}")))?;

        let bp = spec::load_blueprint_from_yaml(&yaml_content)?;

        let step_refs: Vec<(&str, &str, &str, &str)> = bp
            .pipeline
            .steps
            .iter()
            .map(|s| {
                (
                    s.name.as_str(),
                    s.from.as_str(),
                    s.to.as_str(),
                    s.desc.as_str(),
                )
            })
            .collect();
        let html = render_html(
            &bp.name,
            bp.description.as_deref(),
            bp.status.as_str(),
            &bp.created_at,
            &bp.updated_at,
            "",
            "",
            &step_refs,
        );
        std::fs::write(&output_path, &html)
            .map_err(|e| CliError::new(format!("写入 .html 失败: {e}")))?;
        println!("已生成: {}", output_path.display());
        Ok(())
    }
}

// ── Helpers ──

fn read_drd(input: &str) -> Result<String, CliError> {
    std::fs::read_to_string(input)
        .map_err(|e| CliError::new(format!("无法读取 DRD 文件 {input}: {e}")))
}

fn write_spec_files(stem: &str, kind: &str, yaml: &str, md: &str) -> Result<(), CliError> {
    let spec_dir = util::spec_dir();
    std::fs::create_dir_all(&spec_dir)
        .map_err(|e| CliError::new(format!("无法创建目录 {spec_dir}: {e}")))?;
    let yaml_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.yaml"));
    let md_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.md"));
    std::fs::write(&yaml_path, yaml).map_err(|e| CliError::new(format!("写入 .yaml 失败: {e}")))?;
    std::fs::write(&md_path, md).map_err(|e| CliError::new(format!("写入 .md 失败: {e}")))?;
    println!("已生成: {}", yaml_path.display());
    println!("已生成: {}", md_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    // ── contract 生成 ──
    fn test_formalize_prompt_contains_md() {
        let prompt = design_formalize_prompt("# Test Blueprint\n\nSome content");
        assert!(prompt.contains("# Test Blueprint"));
        assert!(prompt.contains("Some content"));
        assert!(prompt.contains("YAML"));
    }

    #[test]
    fn test_formalize_prompt_includes_yaml_instructions() {
        let prompt = design_formalize_prompt("hello");
        assert!(prompt.contains("YAML"));
        assert!(prompt.contains("只输出 YAML"));
    }

    #[test]

    // ── extract_cue ──
    fn test_extract_cue_from_markdown_block() {
        let response =
            "Here is the CUE:\n```cue\npackage blueprints\nx: {name: \"test\"}\n```\nDone.";
        let cue = extract_cue(response);
        assert!(cue.contains("package blueprints"));
        assert!(!cue.contains("```"));
    }

    #[test]
    fn test_extract_cue_from_plain_block() {
        let response = "```\npackage blueprints\n{name: \"x\"}\n```";
        let cue = extract_cue(response);
        assert!(cue.contains("package blueprints"));
    }

    #[test]
    fn test_extract_cue_fallback() {
        let response = "package blueprints\nx: {name: \"test\"}";
        let cue = extract_cue(response);
        assert_eq!(cue, response);
    }

    #[test]

    // ── render_html ──
    fn test_render_html_contains_name() {
        let html = render_html(
            "test",
            None,
            "draft",
            "2026-01-01",
            "2026-01-01",
            "in",
            "out",
            &[],
        );
        assert!(html.contains("<title>test — Blueprint</title>"));
        assert!(html.contains("<h1>test</h1>"));
    }

    #[test]
    fn test_render_html_with_steps() {
        let steps = [("s1", "a", "b", "do it"), ("s2", "b", "c", "then this")];
        let html = render_html(
            "bp",
            Some("desc"),
            "confirmed",
            "t1",
            "t2",
            "in",
            "out",
            &steps,
        );
        assert!(html.contains("s1"));
        assert!(html.contains("s2"));
        assert!(html.contains("do it"));
        assert!(html.contains("then this"));
        assert!(html.contains(">2<")); // step count
    }

    #[test]
    fn test_render_html_empty_steps() {
        let html = render_html("bp", None, "draft", "t1", "t2", "in", "out", &[]);
        assert!(html.contains("(0 步)") || html.contains(">0<") || html.contains("0 步"));
    }

    #[test]
    fn test_design_contract_prompt() {
        let prompt = design_contract_prompt("客户需要用户画像数据");
        assert!(prompt.contains("输入契约"));
        assert!(prompt.contains("输出契约"));
        assert!(prompt.contains("Markdown 表格"));
        assert!(prompt.contains("用户画像"));
    }

    #[test]

    // ── blueprint 生成 ──
    fn test_design_blueprint_prompt() {
        let prompt = design_blueprint_prompt("清洗订单数据");
        assert!(prompt.contains("处理步骤"));
        assert!(prompt.contains("状态机"));
        assert!(prompt.contains("start_at"));
        assert!(prompt.contains("states"));
        assert!(prompt.contains("from"));
        assert!(prompt.contains("desc"));
        assert!(prompt.contains("Markdown 表格"));
        assert!(prompt.contains("清洗订单"));
    }

    #[test]
    fn test_blueprint_table_to_yaml_includes_state_machine() {
        let table = r#"
## 处理步骤

| 步骤名 | 输入(from) | 输出(to) | 处理逻辑(desc) | 依赖(depends) |
|--------|-----------|----------|---------------|--------------|
| 数据加载与校验 | 原始 CSV 文件 | 校验后的数据 | 检查必填字段 | - |
| 字段标准化 | 校验后的数据 | 标准化数据 | 统一日期格式 | 数据加载与校验 |
"#;

        let (yaml, _md) = blueprint_table_to_yaml(table, "customer-chat");

        assert!(yaml.contains("start_at: \"数据加载与校验\""));
        assert!(yaml.contains("states:"));
        assert!(yaml.contains("type: task"));
        assert!(yaml.contains("resource: builtin:copy"));
        assert!(yaml.contains("next: \"字段标准化\""));
        assert!(yaml.contains("end: true"));

        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(
            parsed["pipeline"]["states"]["数据加载与校验"]["resource"].as_str(),
            Some("builtin:copy")
        );
        assert_eq!(
            parsed["pipeline"]["states"]["字段标准化"]["end"].as_bool(),
            Some(true)
        );
    }

    use crate::ENV_LOCK;
    use crate::test_support::fake_llm;
    use crate::test_support::temp_dir;

    const CONTRACT_TABLES: &str = "## 输入契约\n\n| 字段名 | 类型 | 说明 |\n|--------|------|------|\n| user_id | string | 用户 ID |\n\n## 输出契约\n\n| 字段名 | 类型 | 说明 |\n|--------|------|------|\n| repo | string | 仓库名 |\n| stars | int | 星数 |\n";

    #[test]
    fn design_contract_writes_spec_files_from_llm_tables() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-design-contract");
        let drd = root.join("abc.md");
        std::fs::write(&drd, "# DRD ABC\n").unwrap();
        let spec_dir = root.join("spec");

        unsafe {
            std::env::set_var("SPEC_DIR", &spec_dir);
        }
        let handler = DesignHandler::new(fake_llm(CONTRACT_TABLES));
        handler
            .run(&DesignArgs {
                action: DesignAction::Contract {
                    input: drd.to_string_lossy().to_string(),
                },
            })
            .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        let yaml = std::fs::read_to_string(spec_dir.join("abc-contract.yaml")).unwrap();
        assert!(yaml.contains("contract:"), "{yaml}");
        assert!(yaml.contains("user_id"), "{yaml}");
        let md = std::fs::read_to_string(spec_dir.join("abc-contract.md")).unwrap();
        assert!(md.contains("输入契约"), "{md}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn design_formalize_writes_yaml_from_cue_block() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-design-formalize");
        let md = root.join("note.md");
        std::fs::write(&md, "# 需求\n").unwrap();
        let output = root.join("out.yaml");

        let handler = DesignHandler::new(fake_llm("```cue\nname: \"demo\"\nversion: 1\n```\n"));
        handler
            .run(&DesignArgs {
                action: DesignAction::Formalize {
                    input: md.to_string_lossy().to_string(),
                    output: Some(output.to_string_lossy().to_string()),
                },
            })
            .unwrap();

        let yaml = std::fs::read_to_string(&output).unwrap();
        assert!(yaml.contains("demo"), "{yaml}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn design_preview_renders_html_from_blueprint_yaml() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-design-preview");
        let yaml_in = root.join("bp.yaml");
        std::fs::write(
            &yaml_in,
            "name: demo\nstatus: draft\ndescription: 示例\ncreated_at: \"2026-01-01\"\nupdated_at: \"2026-01-01\"\ncontract:\n  input:\n    schema: a\n    format: CSV\n  output:\n    schema: b\n    format: CSV\npipeline:\n  name: demo-pipeline\n  steps:\n    - name: step1\n      from: \"[]\"\n      to: \"[]\"\n      desc: 第一步\n",
        )
        .unwrap();
        let output = root.join("out.html");

        let handler = DesignHandler::new(fake_llm("unused"));
        handler
            .run(&DesignArgs {
                action: DesignAction::Preview {
                    input: yaml_in.to_string_lossy().to_string(),
                    output: Some(output.to_string_lossy().to_string()),
                },
            })
            .unwrap();

        let html = std::fs::read_to_string(&output).unwrap();
        assert!(html.contains("demo"), "{html}");

        std::fs::remove_dir_all(&root).ok();
    }
}

// ── prompt 与解析（纯函数，按子领域分组）──

// ── contract 生成 ──

/// Build the design-contract prompt: DRD → Contract (Markdown tables).
/// CLI 代码解析 Markdown 表格，生成 .cue + .md 文件。LLM 不直接写 CUE。
pub fn design_contract_prompt(drd: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请根据以下数据需求文档（DRD），生成数据契约（Contract）。

输出以下两个 Markdown 表格（只输出表格，不要输出任何 CUE 代码或解释文字）：

## 输入契约

| 字段名 | 类型 | 业务含义 | 约束条件 |
|--------|------|----------|----------|
| user_id | string | 用户唯一标识 | 必填，不可为空 |
| user_name | string | 用户姓名 | 必填 |
| created_at | date | 注册日期 | 格式 YYYY-MM-DD |

## 输出契约

| 字段名 | 类型 | 业务含义 | 质量承诺 |
|--------|------|----------|----------|
| standard_user_id | string | 标准化用户ID | 去重，非空 |
| clean_user_name | string | 清洗后姓名 | 去除首尾空格 |
| age_group | string | 年龄段 | 枚举值校验 |

示例行只用于展示格式，你需要根据 DRD 生成实际的字段替换它们。

DRD:
{drd}"#
    )
}

/// Parse contract Markdown tables into a Contract struct, return CUE + MD.
pub fn contract_tables_to_yaml(
    md_tables: &str,
) -> Result<(String, String), crate::error::CliError> {
    let input_fields = parse_md_table(md_tables, "输入契约");
    let output_fields = parse_md_table(md_tables, "输出契约");

    if !input_fields.is_empty() && input_fields == output_fields {
        return Err(crate::error::CliError::new(
            "错误: 输入契约和输出契约解析到相同字段。LLM 可能跳过了 section 标题，请在 prompt 中要求 LLM 输出 ## 输入契约 和 ## 输出契约 标题。",
        ));
    }

    let input_schema = fields_to_schema_desc(&input_fields);
    let output_schema = fields_to_schema_desc(&output_fields);

    let yaml = format!(
        r#"contract:
  input:
    schema: "{}"
    format: CSV
  output:
    schema: "{}"
    format: CSV
    rules:
      - 数据完整性校验
      - 字段类型校验
"#,
        input_schema.replace('"', "'"),
        output_schema.replace('"', "'"),
    );

    let md = render_contract_md(&input_fields, &output_fields);
    Ok((yaml, md))
}

fn parse_md_table(text: &str, section: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut in_section = false;
    let mut found_section = false;
    for line in text.lines() {
        if line.contains(section) {
            in_section = true;
            found_section = true;
            continue;
        }
        if in_section
            && line.starts_with('|')
            && !line.contains("---")
            && !line.contains("字段名")
            && !line.contains("步骤名")
        {
            let cells: Vec<String> = line
                .split('|')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        if in_section && line.starts_with("##") {
            in_section = false;
        }
    }
    // Fallback: if section header not found, parse all | lines as table data
    if !found_section {
        for line in text.lines() {
            if line.starts_with('|')
                && !line.contains("---")
                && !line.contains("字段名")
                && !line.contains("步骤名")
            {
                let cells: Vec<String> = line
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }
        }
    }
    rows
}

fn fields_to_schema_desc(fields: &[Vec<String>]) -> String {
    if fields.is_empty() {
        return "待定义".into();
    }
    let items: Vec<String> = fields
        .iter()
        .filter_map(|f| {
            f.first()
                .map(|name| format!("{name}: {}", f.get(1).unwrap_or(&"string".into())))
        })
        .collect();
    format!("{{\n    {}\n  }}", items.join(",\n    "))
}

fn render_contract_md(input: &[Vec<String>], output: &[Vec<String>]) -> String {
    let mut md = String::from(
        "## 输入契约\n\n| 字段名 | 类型 | 业务含义 | 约束条件 |\n|--------|------|----------|----------|\n",
    );
    for row in input {
        let cells: Vec<&str> = row.iter().map(|c| c.as_str()).collect();
        if cells.len() >= 4 {
            md.push_str(&format!("| {} |\n", cells[..4].join(" | ")));
        }
    }
    md.push_str("\n## 输出契约\n\n| 字段名 | 类型 | 业务含义 | 质量承诺 |\n|--------|------|----------|----------|\n");
    for row in output {
        let cells: Vec<&str> = row.iter().map(|c| c.as_str()).collect();
        if cells.len() >= 4 {
            md.push_str(&format!("| {} |\n", cells[..4].join(" | ")));
        }
    }
    md
}

// ── blueprint 生成 ──

/// Build the design-blueprint prompt: DRD → Blueprint (Markdown table).
/// CLI 代码解析 Markdown 表格，生成 .cue + .md + .html 文件。LLM 不直接写 CUE。
pub fn design_blueprint_prompt(drd: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请根据以下数据需求文档（DRD），生成处理蓝图（Blueprint）的工作流步骤。

Blueprint 是设计与实现之间的中间规格，参考 AWS Step Functions 的状态机思路：先描述工作流结构，再由 implement/execute 转换为代码或执行计划。当前 CLI 会根据表格生成 YAML，其中包含兼容旧实现的 steps，以及更明确的 start_at/states 状态机结构。

输出一个 Markdown 表格（只输出表格，不要输出任何 CUE 代码或解释文字）：

## 处理步骤

| 步骤名 | 输入(from) | 输出(to) | 处理逻辑(desc) | 依赖(depends) |
|--------|-----------|----------|---------------|--------------|
| 数据加载与校验 | 原始 CSV 文件 | 校验后的数据 | 读取输入文件，检查必填字段非空、日期格式正确，不合规行记录到异常日志 | - |
| 字段标准化 | 校验后的数据 | 标准化数据 | 将性别缩写统一为全称，日期转为标准格式，数值字段去除单位符号 | 数据加载与校验 |
| 去重与输出 | 标准化数据 | 最终交付数据 | 按 user_id 去重保留最新记录，生成符合输出契约的 CSV 文件 | 字段标准化 |

示例行只用于展示格式，你需要根据 DRD 生成实际的步骤替换它们。依赖(depends) 填写前置步骤名，无依赖填 -，多个用逗号分隔。

DRD:
{drd}"#
    )
}

/// Parse blueprint Markdown table into CUE + MD.
pub fn blueprint_table_to_yaml(md_table: &str, project_name: &str) -> (String, String) {
    let steps = parse_md_table(md_table, "处理步骤");
    let workflow_rows: Vec<Vec<String>> = steps
        .iter()
        .filter(|row| {
            let name = row.first().map(|s| s.as_str()).unwrap_or("unnamed");
            let name_lower = name.trim().to_lowercase();
            !matches!(
                name_lower.as_str(),
                "步骤名" | "步骤" | "step name" | "step" | "unnamed"
            )
        })
        .cloned()
        .collect();

    let mut steps_cue = String::new();
    for row in &workflow_rows {
        let name = row.first().map(|s| s.as_str()).unwrap_or("unnamed");
        let from = row.get(1).map(|s| s.as_str()).unwrap_or("");
        let to = row.get(2).map(|s| s.as_str()).unwrap_or("");
        let desc = row.get(3).map(|s| s.as_str()).unwrap_or("");
        let deps = row.get(4).map(|s| s.as_str()).unwrap_or("");

        let deps_yaml = if deps.is_empty() || deps == "-" {
            String::new()
        } else {
            let dep_list: String = deps
                .split(',')
                .map(|d| format!("\n          - {}", d.trim()))
                .collect();
            format!("\n        depends:{}", dep_list)
        };

        steps_cue.push_str(&format!(
            r#"      - name: "{name}"
        from: "{from}"
        to: "{to}"
        desc: "{desc}"
        resource: builtin:copy{deps}
"#,
            name = name,
            from = from,
            to = to,
            desc = desc,
            deps = deps_yaml,
        ));
    }

    let start_at = workflow_rows
        .first()
        .and_then(|row| row.first())
        .map(|s| s.as_str())
        .unwrap_or("");
    let mut states_yaml = String::new();
    for (i, row) in workflow_rows.iter().enumerate() {
        let name = row.first().map(|s| s.as_str()).unwrap_or("unnamed");
        let from = row.get(1).map(|s| s.as_str()).unwrap_or("");
        let to = row.get(2).map(|s| s.as_str()).unwrap_or("");
        let desc = row.get(3).map(|s| s.as_str()).unwrap_or("");
        let deps = row.get(4).map(|s| s.as_str()).unwrap_or("");
        let deps_yaml = if deps.is_empty() || deps == "-" {
            String::new()
        } else {
            let dep_items: String = deps
                .split(',')
                .map(|d| format!("\n        - {}", d.trim()))
                .collect();
            format!("\n      depends:{dep_items}")
        };
        let transition = workflow_rows
            .get(i + 1)
            .and_then(|next| next.first())
            .map(|next_name| format!("next: \"{next_name}\""))
            .unwrap_or_else(|| "end: true".to_string());

        states_yaml.push_str(&format!(
            r#"    "{name}":
      type: task
      from: "{from}"
      to: "{to}"
      desc: "{desc}"
      resource: builtin:copy
      {transition}{depends}
"#,
            name = name,
            from = from,
            to = to,
            desc = desc,
            transition = transition,
            depends = deps_yaml,
        ));
    }

    let yaml = format!(
        r#"name: "{name}"
description: "从 DRD 自动生成的 Blueprint"
contract:
  input:
    schema: ""
    format: ""
  output:
    schema: ""
    format: ""
pipeline:
  name: "{name}-pipeline"
  start_at: "{start_at}"
  states:
{states}
  steps:
{steps}status: draft
created_at: "2026-07-24T00:00:00+00:00"
updated_at: "2026-07-24T00:00:00+00:00"
"#,
        name = project_name,
        start_at = start_at,
        states = states_yaml,
        steps = steps_cue,
    );

    let md = render_blueprint_md(project_name, &steps);
    (yaml, md)
}

fn render_blueprint_md(name: &str, steps: &[Vec<String>]) -> String {
    let mut md = format!(
        "# {name}\n\n## 处理步骤\n\n| 步骤名 | 输入 | 输出 | 处理逻辑 | 依赖 |\n|--------|------|------|----------|------|\n"
    );
    for row in steps {
        let cells: Vec<&str> = row.iter().map(|c| c.as_str()).collect();
        let padded: Vec<String> = (0..5)
            .map(|i| cells.get(i).unwrap_or(&"").to_string())
            .collect();
        md.push_str(&format!("| {} |\n", padded.join(" | ")));
    }
    md
}

// ── formalize ──

/// (v0.1.0-beta.1) Formalize prompt — converts Markdown to YAML.
pub fn design_formalize_prompt(md: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请将以下 Blueprint Markdown 文档转化为 YAML 格式。

输出格式:
name: "项目名称"
description: "业务描述"
pipeline:
  name: "管道名称"
  steps:
    - name: "步骤1"
      from: "输入"
      to: "输出"
      desc: "业务逻辑描述"

只输出 YAML，不要解释。

文档:
{md}"#
    )
}

/// Extract CUE code from LLM response (handles markdown code blocks).
pub fn extract_cue(response: &str) -> String {
    for marker in &["```cue", "```CUE", "```"] {
        if let Some(start) = response.find(marker) {
            let s = start + marker.len();
            let e = response[s..]
                .find("```")
                .map(|i| s + i)
                .unwrap_or(response.len());
            let code = response[s..e].trim();
            if code.contains("package") || code.contains("#Blueprint") {
                return code.to_string();
            }
        }
    }
    response.to_string()
}

// ── 渲染 ──

/// Render a Blueprint to HTML.
#[allow(clippy::too_many_arguments)]
pub fn render_html(
    name: &str,
    description: Option<&str>,
    status: &str,
    created_at: &str,
    updated_at: &str,
    input_schema: &str,
    output_schema: &str,
    steps: &[(&str, &str, &str, &str)],
) -> String {
    let mut steps_html = String::new();
    for (i, (name, from, to, desc)) in steps.iter().enumerate() {
        steps_html.push_str(&format!(
            r#"<tr><td>{i}</td><td>{name}</td><td>{from}</td><td>{to}</td><td>{desc}</td></tr>"#,
            i = i + 1,
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="UTF-8"><title>{name} — Blueprint</title>
<style>body{{font-family:sans-serif;max-width:960px;margin:0 auto;padding:2rem}}h1{{color:#2563eb}}table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:.5rem;border-bottom:1px solid #ddd}}th{{color:#6c757d}}</style></head>
<body><h1>{name}</h1><p>{desc}</p><p>状态: {status} | 创建: {created} | 更新: {updated}</p>
<h2>契约</h2><h3>输入</h3><pre>{input_schema}</pre><h3>输出</h3><pre>{output_schema}</pre>
<h2>管道 ({step_count} 步)</h2><table><tr><th>#</th><th>名称</th><th>From</th><th>To</th><th>描述</th></tr>{steps}</table>
</body></html>"#,
        name = name,
        desc = description.unwrap_or(""),
        status = status,
        created = created_at,
        updated = updated_at,
        input_schema = input_schema,
        output_schema = output_schema,
        step_count = steps.len(),
        steps = steps_html,
    )
}
