//! Blueprint pure functions — separated from I/O for testability.
//!
//! All functions here take data and return data. No file I/O, no subprocess, no LLM calls.

use std::path::{Path, PathBuf};

/// Get the blueprint directory from env or default.
pub fn blueprint_dir() -> String {
    std::env::var("BLUEPRINT_DIR").unwrap_or_else(|_| ".quanttide/data/blueprint".to_string())
}

/// Generate a .md template for a new Blueprint.
pub fn design_template(name: &str) -> String {
    format!(
        r#"# {name}

## 背景



## 数据来源

| 来源 | 说明 |
|------|------|
| | |

## 输出变量

| 变量 | 描述 |
|------|------|
| | |

## 精炼管道

| 阶段 | 步骤 |
|------|------|
| | |

## 验收标准

- [ ]
"#
    )
}

/// Build the review prompt for LLM.
pub fn review_prompt(
    name: &str,
    status: &str,
    step_count: usize,
    input_schema: &str,
    output_schema: &str,
    issues: &str,
) -> String {
    format!(
        r#"你是数据工程 Blueprint 审计专家。请审查以下 Blueprint 并输出结构化问题清单。

Blueprint:
- 名称: {name}
- 状态: {status}
- 管道步骤数: {step_count}
- 输入 schema: {input_schema}
- 输出 schema: {output_schema}

结构校验问题:
{issues_section}

请按以下格式输出问题清单：
1. 【严重】阻断性问题（缺失必填字段、契约不完整）
2. 【警告】可能导致交付偏差的问题（口径不明确、步骤描述过于简略）
3. 【建议】可以优化的地方（命名规范、文档完整性）

每个问题标注：严重程度、位置（字段/步骤名）、具体问题、建议修复。"#,
        name = name,
        status = status,
        step_count = step_count,
        input_schema = input_schema,
        output_schema = output_schema,
        issues_section = if issues.is_empty() { "无" } else { issues },
    )
}

/// Build the formalize prompt for LLM.
pub fn formalize_prompt(md: &str) -> String {
    design_formalize_prompt(md)
}

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
            let e = response[s..].find("```").map(|i| s + i).unwrap_or(response.len());
            let code = response[s..e].trim();
            if code.contains("package") || code.contains("#Blueprint") {
                return code.to_string();
            }
        }
    }
    response.to_string()
}

/// Render a Blueprint to HTML.
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

/// Resolve a user input to a .cue or .yaml file path.
pub fn resolve_cue_path(input: &str, dir: &str) -> Option<PathBuf> {
    let p = Path::new(input);
    if p.exists() {
        Some(p.to_path_buf())
    } else {
        for ext in &["yaml", "cue"] {
            let with_ext = Path::new(dir).join(format!("{input}.{ext}"));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }
        None
    }
}

/// Resolve a user input to a .md file path.
pub fn resolve_md_path(name: &str, dir: &str) -> PathBuf {
    let p = Path::new(name);
    if p.exists() {
        p.to_path_buf()
    } else {
        let with_ext = Path::new(dir).join(format!("{name}.md"));
        if with_ext.exists() {
            with_ext
        } else {
            Path::new(dir).join(format!("{name}.md"))
        }
    }
}

/// Convert kebab-case to camelCase.
pub fn to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for c in s.chars() {
        if c == '-' || c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── design_template ──

    #[test]
    fn test_design_template_contains_sections() {
        let tpl = design_template("my-project");
        assert!(tpl.contains("# my-project"));
        assert!(tpl.contains("## 背景"));
        assert!(tpl.contains("## 数据来源"));
        assert!(tpl.contains("## 输出变量"));
        assert!(tpl.contains("## 精炼管道"));
        assert!(tpl.contains("## 验收标准"));
    }

    #[test]
    fn test_design_template_non_empty_for_any_name() {
        for name in &["a", "sec-credit", "test-blueprint-v2"] {
            let tpl = design_template(name);
            assert!(!tpl.is_empty());
            assert!(tpl.contains(name));
        }
    }

    // ── review_prompt ──

    #[test]
    fn test_review_prompt_contains_key_info() {
        let prompt = review_prompt("test-bp", "draft", 5, "input-schema", "output-schema", "");
        assert!(prompt.contains("test-bp"));
        assert!(prompt.contains("draft"));
        assert!(prompt.contains("管道步骤数: 5"));
        assert!(prompt.contains("input-schema"));
        assert!(prompt.contains("output-schema"));
        assert!(prompt.contains("【严重】"));
        assert!(prompt.contains("【警告】"));
        assert!(prompt.contains("【建议】"));
    }

    #[test]
    fn test_review_prompt_with_issues() {
        let prompt = review_prompt("bp", "submitted", 0, "in", "out", "step1: missing desc");
        assert!(prompt.contains("step1: missing desc"));
    }

    #[test]
    fn test_review_prompt_empty_issues_shows_none() {
        let prompt = review_prompt("bp", "draft", 0, "in", "out", "");
        assert!(prompt.contains("无"));
    }

    // ── formalize_prompt ──

    #[test]
    fn test_formalize_prompt_contains_md() {
        let prompt = formalize_prompt("# Test Blueprint\n\nSome content");
        assert!(prompt.contains("# Test Blueprint"));
        assert!(prompt.contains("Some content"));
        assert!(prompt.contains("YAML"));
    }

    #[test]
    fn test_formalize_prompt_includes_yaml_instructions() {
        let prompt = formalize_prompt("hello");
        assert!(prompt.contains("YAML"));
        assert!(prompt.contains("只输出 YAML"));
    }

    // ── extract_cue ──

    #[test]
    fn test_extract_cue_from_markdown_block() {
        let response = "Here is the CUE:\n```cue\npackage blueprints\nx: {name: \"test\"}\n```\nDone.";
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

    // ── render_html ──

    #[test]
    fn test_render_html_contains_name() {
        let html = render_html("test", None, "draft", "2026-01-01", "2026-01-01", "in", "out", &[]);
        assert!(html.contains("<title>test — Blueprint</title>"));
        assert!(html.contains("<h1>test</h1>"));
    }

    #[test]
    fn test_render_html_with_steps() {
        let steps = [("s1", "a", "b", "do it"), ("s2", "b", "c", "then this")];
        let html = render_html("bp", Some("desc"), "confirmed", "t1", "t2", "in", "out", &steps);
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

    // ── resolve_cue_path ──

    #[test]
    fn test_resolve_cue_path_nonexistent() {
        let result = resolve_cue_path("nonexistent-blueprint-12345", "/tmp");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_cue_path_by_name() {
        let tmp = std::env::temp_dir().join("bp-test-resolve");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("my-bp.cue"), "package blueprints\n{name: \"test\"}").unwrap();

        let result = resolve_cue_path("my-bp", tmp.to_str().unwrap());
        assert!(result.is_some());

        std::fs::remove_dir_all(&tmp).ok();
    }

    // ── resolve_md_path ──

    #[test]
    fn test_resolve_md_path_new_file() {
        let tmp = std::env::temp_dir().join("bp-test-md");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let path = resolve_md_path("new-project", tmp.to_str().unwrap());
        assert_eq!(path.extension().unwrap(), "md");

        std::fs::remove_dir_all(&tmp).ok();
    }

    // ── to_camel ──

    #[test]
    fn test_to_camel_basic() {
        assert_eq!(to_camel("csv-standard"), "csvStandard");
        assert_eq!(to_camel("simple"), "simple");
        assert_eq!(to_camel("abc-def-ghi"), "abcDefGhi");
        assert_eq!(to_camel(""), "");
    }

    #[test]
    fn test_to_camel_with_underscores() {
        assert_eq!(to_camel("my_variable_name"), "myVariableName");
    }

    #[test]
    fn test_to_camel_single_word() {
        assert_eq!(to_camel("name"), "name");
    }

    // ── blueprint_dir ──

    #[test]
    fn test_blueprint_dir_default() {
        let dir = blueprint_dir();
        assert_eq!(dir, ".quanttide/data/blueprint");
    }
}

// ── v0.2.0: 新框架目录 ──

/// Get the DRD directory (Data Requirements Document).
pub fn drd_dir() -> String {
    std::env::var("DRD_DIR").unwrap_or_else(|_| ".quanttide/data/drd".to_string())
}

/// Get the Specification directory.
pub fn spec_dir() -> String {
    std::env::var("SPEC_DIR").unwrap_or_else(|_| ".quanttide/data/spec".to_string())
}

// ── v0.2.0: clarify prompt ──

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

// ── v0.2.0: design prompts ──

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
pub fn contract_tables_to_yaml(md_tables: &str) -> (String, String) {
    let input_fields = parse_md_table(md_tables, "输入契约");
    let output_fields = parse_md_table(md_tables, "输出契约");

    if !input_fields.is_empty() && input_fields == output_fields {
        eprintln!("错误: 输入契约和输出契约解析到相同字段。LLM 可能跳过了 section 标题，请在 prompt 中要求 LLM 输出 ## 输入契约 和 ## 输出契约 标题。");
        std::process::exit(1);
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
    (yaml, md)
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
        if in_section && line.starts_with('|') && !line.contains("---") && !line.contains("字段名") {
            let cells: Vec<String> = line.split('|')
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
            if line.starts_with('|') && !line.contains("---") && !line.contains("字段名") {
                let cells: Vec<String> = line.split('|')
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
    if fields.is_empty() { return "待定义".into(); }
    let items: Vec<String> = fields.iter()
        .filter_map(|f| f.first().map(|name| format!("{name}: {}", f.get(1).unwrap_or(&"string".into()))))
        .collect();
    format!("{{\n    {}\n  }}", items.join(",\n    "))
}

fn render_contract_md(input: &[Vec<String>], output: &[Vec<String>]) -> String {
    let mut md = String::from("## 输入契约\n\n| 字段名 | 类型 | 业务含义 | 约束条件 |\n|--------|------|----------|----------|\n");
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

/// Build the design-blueprint prompt: DRD → Blueprint (Markdown table).
/// CLI 代码解析 Markdown 表格，生成 .cue + .md + .html 文件。LLM 不直接写 CUE。
pub fn design_blueprint_prompt(drd: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请根据以下数据需求文档（DRD），生成处理蓝图（Blueprint）的工作流步骤。

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

    let mut steps_cue = String::new();
    for row in &steps {
        let name = row.first().map(|s| s.as_str()).unwrap_or("unnamed");
        let from = row.get(1).map(|s| s.as_str()).unwrap_or("");
        let to = row.get(2).map(|s| s.as_str()).unwrap_or("");
        let desc = row.get(3).map(|s| s.as_str()).unwrap_or("");
        let deps = row.get(4).map(|s| s.as_str()).unwrap_or("");

        let deps_str = if deps.is_empty() || deps == "-" {
            String::new()
        } else {
            let dep_list: Vec<String> = deps.split(',').map(|d| format!("\"{}\"", d.trim())).collect();
            format!("\n            depends: [{}]", dep_list.join(", "))
        };

        let deps_yaml = if deps.is_empty() || deps == "-" {
            String::new()
        } else {
            let dep_list: String = deps.split(',').map(|d| format!("\n          - {}", d.trim())).collect();
            format!("\n        depends:{}", dep_list)
        };

        steps_cue.push_str(&format!(
            r#"      - name: "{name}"
        from: "{from}"
        to: "{to}"
        desc: "{desc}"{deps}
"#,
            name = name,
            from = from,
            to = to,
            desc = desc,
            deps = deps_yaml,
        ));
    }

    let yaml = format!(
        r#"name: "{name}"
description: "从 DRD 自动生成的 Blueprint"
pipeline:
  name: "{name}-pipeline"
  steps:
{steps}status: draft
created_at: "2026-07-24T00:00:00+00:00"
updated_at: "2026-07-24T00:00:00+00:00"
"#,
        name = project_name,
        steps = steps_cue,
    );

    let md = render_blueprint_md(project_name, &steps);
    (yaml, md)
}

fn render_blueprint_md(name: &str, steps: &[Vec<String>]) -> String {
    let mut md = format!("# {name}\n\n## 处理步骤\n\n| 步骤名 | 输入 | 输出 | 处理逻辑 | 依赖 |\n|--------|------|------|----------|------|\n");
    for row in steps {
        let cells: Vec<&str> = row.iter().map(|c| c.as_str()).collect();
        let padded: Vec<String> = (0..5).map(|i| cells.get(i).unwrap_or(&"").to_string()).collect();
        md.push_str(&format!("| {} |\n", padded.join(" | ")));
    }
    md
}

#[cfg(test)]
mod tests_v020 {
    use super::*;

    #[test]
    fn test_drd_dir_default() {
        assert_eq!(drd_dir(), ".quanttide/data/drd");
    }

    #[test]
    fn test_spec_dir_default() {
        assert_eq!(spec_dir(), ".quanttide/data/spec");
    }

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

    #[test]
    fn test_design_contract_prompt() {
        let prompt = design_contract_prompt("客户需要用户画像数据");
        assert!(prompt.contains("输入契约"));
        assert!(prompt.contains("输出契约"));
        assert!(prompt.contains("Markdown 表格"));
        assert!(prompt.contains("用户画像"));
    }

    #[test]
    fn test_design_blueprint_prompt() {
        let prompt = design_blueprint_prompt("清洗订单数据");
        assert!(prompt.contains("处理步骤"));
        assert!(prompt.contains("from"));
        assert!(prompt.contains("desc"));
        assert!(prompt.contains("Markdown 表格"));
        assert!(prompt.contains("清洗订单"));
    }
}
