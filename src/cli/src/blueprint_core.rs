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
        // Without env var set, returns default
        let dir = blueprint_dir();
        assert_eq!(dir, ".quanttide/data/blueprint");
    }
}
