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

/// (v0.2.0) Formalize prompt — same logic, renamed under design namespace.
pub fn design_formalize_prompt(md: &str) -> String {
    format!(
        r#"你是一个 CUE 语言专家。请将以下 Blueprint Markdown 文档形式化为 CUE 格式。

要求：
1. 输出 package blueprints，然后是 #Blueprint 实例
2. 使用 struct 定义 contract（input/output 的 schema/format/rules）
3. 使用 struct 列表定义 pipeline steps（name/from/to/desc/depends）
4. 保留原始文档中的所有业务信息，不要遗漏字段
5. 只输出 CUE 代码，不要解释

Blueprint Markdown 文档:
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

/// Resolve a user input to a .cue file path.
pub fn resolve_cue_path(input: &str, dir: &str) -> Option<PathBuf> {
    let p = Path::new(input);
    if p.exists() {
        Some(p.to_path_buf())
    } else {
        let with_ext = Path::new(dir).join(format!("{input}.cue"));
        if with_ext.exists() {
            Some(with_ext)
        } else {
            None
        }
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
        assert!(prompt.contains("package blueprints"));
        assert!(prompt.contains("#Blueprint"));
    }

    #[test]
    fn test_formalize_prompt_includes_cue_instructions() {
        let prompt = formalize_prompt("hello");
        assert!(prompt.contains("CUE 语言专家"));
        assert!(prompt.contains("只输出 CUE 代码"));
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

/// Build the design-contract prompt: DRD → Contract (.cue + .md).
pub fn design_contract_prompt(drd: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请根据以下数据需求文档（DRD），生成数据契约（Contract）。

Contract 定义数据的输入输出结构约束，包含：

1. **输入契约**（客户需要提供什么数据）：
   - 字段名、数据类型、业务含义、约束条件（必填/格式/枚举值）

2. **输出契约**（我们将交付什么数据）：
   - 字段名、数据类型、业务含义、质量承诺

输出格式：先输出 .cue 格式的结构化定义（package spec），再输出 .md 格式的人类可读版本。两者之间用 `---` 分隔。

DRD:
{drd}"#
    )
}

/// Build the design-blueprint prompt: DRD → Blueprint (.cue + .md + .html).
pub fn design_blueprint_prompt(drd: &str) -> String {
    format!(
        r#"你是一个数据工程规格设计师。请根据以下数据需求文档（DRD），生成处理蓝图（Blueprint）。

Blueprint 定义数据处理的工作流步骤，包含：

1. **步骤名称**：简洁描述这一步骤做什么
2. **输入（from）**：数据从哪里来
3. **输出（to）**：数据到哪里去
4. **处理逻辑**：用业务语言描述具体做什么操作
5. **依赖（depends）**：依赖前面哪几个步骤

输出格式：先输出 .cue 格式的结构化定义（package spec），再输出 .md 格式的人类可读版本。两者之间用 `---` 分隔。

DRD:
{drd}"#
    )
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
        assert!(prompt.contains("package spec"));
        assert!(prompt.contains("输入契约"));
        assert!(prompt.contains("输出契约"));
        assert!(prompt.contains("用户画像"));
    }

    #[test]
    fn test_design_blueprint_prompt() {
        let prompt = design_blueprint_prompt("清洗订单数据");
        assert!(prompt.contains("package spec"));
        assert!(prompt.contains("from"));
        assert!(prompt.contains("to"));
        assert!(prompt.contains("depends"));
        assert!(prompt.contains("清洗订单"));
    }
}
