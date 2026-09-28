//! Turning a note into something shareable.

/// Render a plain-text document as a small, standalone HTML page.
///
/// The text sits verbatim inside a `<pre>`, so the line breaks the writer typed
/// survive, and it is escaped so markup in the document cannot break out of the
/// page. Deliberately dependency-free: an HTML file is shareable and printable
/// without pulling a rendering engine into a writing tool.
pub fn html(title: &str, body: &str) -> String {
    const TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>TITLE</title>
<style>
body { max-width: 40rem; margin: 4rem auto; padding: 0 1.5rem;
       font: 1.05rem/1.7 Georgia, "Times New Roman", serif;
       color: #1a1a1a; background: #faf9f6; }
pre { white-space: pre-wrap; font: inherit; }
</style>
</head>
<body>
<pre>BODY</pre>
</body>
</html>
"#;

    TEMPLATE
        .replace("TITLE", &escape(title))
        .replace("BODY", &escape(body))
}

/// Escape the characters that would otherwise be read as markup.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_in_the_document_is_escaped() {
        let page = html("a & b", "<script>alert(1)</script> & more");
        assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; more"));
        assert!(page.contains("<title>a &amp; b</title>"));
        assert!(!page.contains("<script>"));
    }

    #[test]
    fn the_document_keeps_its_line_breaks() {
        let page = html("note", "one\ntwo\n");
        assert!(page.contains("<pre>one\ntwo\n</pre>"));
    }

    #[test]
    fn the_page_stands_alone() {
        let page = html("note", "text");
        assert!(page.starts_with("<!DOCTYPE html>"));
        assert!(page.trim_end().ends_with("</html>"));
        assert!(page.contains("<meta charset=\"utf-8\">"));
    }
}
