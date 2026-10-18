use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::ops::Range;

struct Heading {
    level: HeadingLevel,
    range: Range<usize>,
    title: String,
}

pub(crate) fn extract(source: &str, section: &str) -> Result<String, String> {
    // Container depth excludes headings in blockquotes/lists from the index.
    let mut depth = 0;
    let mut headings = Vec::new();
    let mut current: Option<Heading> = None;
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) if depth == 0 => {
                current = Some(Heading {
                    level,
                    range,
                    title: String::new(),
                });
                depth += 1;
            }
            Event::Start(_) => depth += 1,
            Event::End(end) => {
                depth -= 1;
                if matches!(end, TagEnd::Heading(_))
                    && depth == 0
                    && let Some(heading) = current.take()
                {
                    headings.push(heading);
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = &mut current {
                    heading.title.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = &mut current {
                    heading.title.push(' ');
                }
            }
            _ => {}
        }
    }
    let mut matches = headings
        .iter()
        .enumerate()
        .filter(|(_, h)| h.level == HeadingLevel::H2 && h.title.trim() == section);
    let Some((index, heading)) = matches.next() else {
        let available: Vec<_> = headings
            .iter()
            .filter(|h| h.level == HeadingLevel::H2)
            .map(|h| h.title.trim())
            .collect();
        return Err(format!(
            "h2 section `{section}` not found; available sections: {}",
            if available.is_empty() {
                "(none)".to_owned()
            } else {
                available.join(", ")
            }
        ));
    };
    if matches.next().is_some() {
        return Err(format!(
            "h2 section `{section}` is ambiguous (duplicate headings)"
        ));
    }
    let start = heading.range.end;
    let end = headings[index + 1..]
        .iter()
        .find(|h| h.level <= HeadingLevel::H2)
        .map_or(source.len(), |h| h.range.start);
    let mut result = String::new();
    let mut cursor = start;
    for h in &headings[index + 1..] {
        if h.range.start >= end {
            break;
        }
        // Promote actual top-level ATX headings; preserve all other raw markup.
        let raw = &source[h.range.clone()];
        let indent = raw.bytes().take_while(|&b| b == b' ').count();
        let hashes = raw[indent..].bytes().take_while(|&b| b == b'#').count();
        if hashes == h.level as usize && hashes >= 3 {
            let marker = h.range.start + indent;
            result.push_str(&source[cursor..marker]);
            result.push_str(&"#".repeat(hashes - 2));
            cursor = marker + hashes;
        }
    }
    result.push_str(&source[cursor..end]);
    Ok(result.trim_matches(['\n', '\r']).to_owned())
}

#[cfg(test)]
mod tests {
    use super::extract;

    #[test]
    fn selects_body_and_promotes_subheadings() {
        let md = "# Fruit\n\n## banana\n\nA banana.\n\n### Examples\n\n```rust\nlet x = 1;\n```\n\n#### Details\ntext\n##### More\nmore\n###### End\nend\n\n## orange\nAn orange.\n";
        assert_eq!(
            extract(md, "banana").unwrap(),
            "A banana.\n\n# Examples\n\n```rust\nlet x = 1;\n```\n\n## Details\ntext\n### More\nmore\n#### End\nend"
        );
    }

    #[test]
    fn stops_at_h1() {
        assert_eq!(
            extract("## a\nbody\n# Chapter\nother\n## b\nb", "a").unwrap(),
            "body"
        );
    }

    #[test]
    fn ignores_fake_headings_in_code_blocks() {
        for fence in ["```", "~~~~"] {
            let body = format!(
                "{fence}markdown\n## fake\n### keep\n{fence}\n\n    ## indented\n\n### Real\nreal"
            );
            let md = format!("## a\n{body}\n## b\nend");
            assert_eq!(
                extract(&md, "a").unwrap(),
                body.replace("### Real", "# Real")
            );
            assert!(extract(&md, "fake").is_err());
        }
    }

    #[test]
    fn ignores_container_and_html_headings() {
        let body =
            "> ## quote\n> ### nested\n\n- item\n\n  ## list\n\n<div>\n## html\n</div>\n\nafter";
        assert_eq!(
            extract(&format!("## a\n{body}\n## b\nend"), "a").unwrap(),
            body
        );
    }

    #[test]
    fn accepts_setext_and_formatted_titles() {
        let md = "Fruit\n=====\n\n`banana::peeling`\n-----------------\n\nBody\n\n## **orange** ###\nother";
        assert_eq!(extract(md, "banana::peeling").unwrap(), "Body");
        assert_eq!(extract(md, "orange").unwrap(), "other");
    }

    #[test]
    fn preserves_unicode_markdown_and_crlf() {
        // Japanese text exercises Unicode headings and bodies.
        let md = "## 日本語\r\n\r\n本文 [link](https://example.com)\r\n\r\n  ### 詳細 ###\r\n\r\n| a | b |\r\n| - | - |\r\n| 1 | 2 |\r\n";
        assert_eq!(
            extract(md, "日本語").unwrap(),
            "本文 [link](https://example.com)\r\n\r\n  # 詳細 ###\r\n\r\n| a | b |\r\n| - | - |\r\n| 1 | 2 |"
        );
    }

    #[test]
    fn supports_empty_and_last_section() {
        assert_eq!(extract("## empty\n\n## last\nlast", "empty").unwrap(), "");
        assert_eq!(
            extract("## empty\n\n## last\nlast", "last").unwrap(),
            "last"
        );
    }

    #[test]
    fn reports_missing_and_duplicate_sections() {
        assert!(
            extract("## a\nbody", "missing")
                .unwrap_err()
                .contains("available sections: a")
        );
        assert!(extract("# a\nbody", "a").unwrap_err().contains("(none)"));
        assert!(
            extract("## a\nfirst\n## a\nsecond", "a")
                .unwrap_err()
                .contains("duplicate")
        );
    }

    #[test]
    fn matches_exact_title_instead_of_slug() {
        assert_eq!(
            extract("## Banana Peel\nbody", "Banana Peel").unwrap(),
            "body"
        );
        assert!(extract("## Banana Peel\nbody", "banana-peel").is_err());
    }
}
