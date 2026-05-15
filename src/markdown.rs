use comrak::{Arena, Options, parse_document, nodes::*};

/// Parse `markdown` (CommonMark + GFM extensions) and emit Typst markup.
pub(crate) fn to_typst(markdown: &str) -> String {
    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.strikethrough = true;
    opts.extension.tasklist = true;
    opts.extension.autolink = true;
    opts.extension.math_dollars = true;
    opts.render.r#unsafe = false;

    let root = parse_document(&arena, markdown, &opts);
    let mut out = String::new();
    render_block(root, &mut out);
    out
}

fn render_block<'a>(node: &'a AstNode<'a>, out: &mut String) {
    use NodeValue::*;
    match &node.data.borrow().value {
        Document => {
            for child in node.children() {
                render_block(child, out);
            }
        }
        Heading(h) => {
            out.push('\n');
            for _ in 0..h.level {
                out.push('=');
            }
            out.push(' ');
            for child in node.children() {
                render_inline(child, out);
            }
            out.push('\n');
        }
        Paragraph => {
            out.push('\n');
            for child in node.children() {
                render_inline(child, out);
            }
            out.push('\n');
        }
        BlockQuote => {
            out.push_str("\n#quote(block: true)[\n");
            for child in node.children() {
                render_block(child, out);
            }
            out.push_str("]\n");
        }
        List(list) => {
            out.push('\n');
            render_list(node, list, out);
        }
        Item(_) => {
            // handled by render_list
        }
        CodeBlock(block) => {
            out.push_str("\n```");
            let lang = block.info.trim();
            if !lang.is_empty() {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(&block.literal);
            if !block.literal.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }
        ThematicBreak => {
            out.push_str("\n#line(length: 100%)\n");
        }
        Table(_) => {
            render_table(node, out);
        }
        HtmlBlock(_) => {
            // skip raw HTML
        }
        _ => {
            for child in node.children() {
                render_block(child, out);
            }
        }
    }
}

fn render_list<'a>(node: &'a AstNode<'a>, list: &NodeList, out: &mut String) {
    let ordered = list.list_type == ListType::Ordered;
    for item in node.children() {
        if ordered {
            out.push_str("+ ");
        } else {
            out.push_str("- ");
        }
        let children: Vec<_> = item.children().collect();
        for (i, child) in children.iter().enumerate() {
            match &child.data.borrow().value {
                NodeValue::Paragraph => {
                    for inline in child.children() {
                        render_inline(inline, out);
                    }
                    if i + 1 < children.len() {
                        out.push('\n');
                    }
                }
                NodeValue::List(inner_list) => {
                    out.push('\n');
                    let mut nested = String::new();
                    render_list(child, inner_list, &mut nested);
                    for line in nested.lines() {
                        out.push_str("  ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                _ => render_block(child, out),
            }
        }
        out.push('\n');
    }
}

fn render_table<'a>(node: &'a AstNode<'a>, out: &mut String) {
    let rows: Vec<_> = node.children().collect();
    if rows.is_empty() {
        return;
    }
    let col_count = rows[0].children().count();
    if col_count == 0 {
        return;
    }

    out.push_str("\n#table(\n");
    out.push_str(&format!("  columns: {},\n", col_count));
    out.push_str("  table.hline(),\n");

    for (row_idx, row) in rows.iter().enumerate() {
        for cell in row.children() {
            out.push_str("  [");
            for child in cell.children() {
                render_inline(child, out);
            }
            out.push_str("],\n");
        }
        if row_idx == 0 {
            out.push_str("  table.hline(),\n");
        }
    }

    out.push_str("  table.hline(),\n");
    out.push_str(")\n");
}

fn render_inline<'a>(node: &'a AstNode<'a>, out: &mut String) {
    use NodeValue::*;
    match &node.data.borrow().value {
        Text(t) => out.push_str(&escape(t)),
        SoftBreak => out.push(' '),
        LineBreak => out.push_str("\\ \n"),
        Strong => {
            out.push('*');
            for child in node.children() { render_inline(child, out); }
            out.push('*');
        }
        Emph => {
            out.push('_');
            for child in node.children() { render_inline(child, out); }
            out.push('_');
        }
        Strikethrough => {
            out.push_str("#strike[");
            for child in node.children() { render_inline(child, out); }
            out.push(']');
        }
        Code(c) => {
            out.push('`');
            out.push_str(&c.literal.replace('`', "\\`"));
            out.push('`');
        }
        Math(m) => {
            if m.display_math {
                out.push_str("$ ");
                out.push_str(&m.literal);
                out.push_str(" $");
            } else {
                out.push('$');
                out.push_str(&m.literal);
                out.push('$');
            }
        }
        Link(link) => {
            out.push_str("#link(\"");
            out.push_str(&escape_url(&link.url));
            out.push_str("\")[");
            for child in node.children() { render_inline(child, out); }
            out.push(']');
        }
        Image(img) => {
            out.push_str("\n#figure(image(\"");
            out.push_str(&escape_url(&img.url));
            out.push_str("\", fit: \"contain\"))\n");
        }
        HtmlInline(_) => {
            // skip
        }
        // In comrak 0.52, TaskItem holds NodeTaskItem { symbol: Option<char>, .. }
        // symbol is Some(_) when checked, None when unchecked.
        TaskItem(task) => {
            if task.symbol.is_some() {
                out.push_str("#box(stroke: 0.5pt, inset: 2pt)[✓] ");
            } else {
                out.push_str("#box(stroke: 0.5pt, inset: 2pt)[ ] ");
            }
            for child in node.children() { render_inline(child, out); }
        }
        _ => {
            for child in node.children() { render_inline(child, out); }
        }
    }
}

/// Escape characters that have special meaning in Typst content.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '*' | '_' | '[' | ']' | '#' | '@' | '$' | '<' | '`' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn escape_url(url: &str) -> String {
    url.replace('"', "%22")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_levels() {
        let out = to_typst("# H1\n## H2\n### H3\n");
        assert!(out.contains("= H1"));
        assert!(out.contains("== H2"));
        assert!(out.contains("=== H3"));
    }

    #[test]
    fn bold_and_italic() {
        let out = to_typst("**bold** and _italic_");
        assert!(out.contains("*bold*"));
        assert!(out.contains("_italic_"));
    }

    #[test]
    fn code_block_preserves_lang() {
        let out = to_typst("```rust\nfn main() {}\n```");
        assert!(out.contains("```rust"));
        assert!(out.contains("fn main() {}"));
    }

    #[test]
    fn inline_code() {
        let out = to_typst("use `foo` here");
        assert!(out.contains("`foo`"));
    }

    #[test]
    fn inline_math_passthrough() {
        let out = to_typst("The angle $alpha$ is small.");
        assert!(out.contains("$alpha$"), "got: {out}");
    }

    #[test]
    fn display_math_passthrough() {
        let out = to_typst("$$E = mc^2$$");
        assert!(out.contains("$ E = mc^2 $"), "got: {out}");
    }

    #[test]
    fn literal_dollar_still_escaped() {
        let out = to_typst("It costs $5.");
        assert!(out.contains("\\$5"), "got: {out}");
    }

    #[test]
    fn table_produces_typst_table() {
        let out = to_typst("| A | B |\n|---|---|\n| 1 | 2 |\n");
        assert!(out.contains("#table("));
        assert!(out.contains("columns: 2"));
        assert!(out.contains("[A]"));
        assert!(out.contains("[1]"));
    }
}
