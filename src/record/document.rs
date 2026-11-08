//! Lossless record document envelope.

use crate::error::AppError;
use crate::record::frontmatter;
use crate::record::frontmatter::FrontMatter;

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub heading: String,
    pub body: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub source: String,
    pub front_matter: Option<FrontMatter>,
    pub frontmatter: Option<FrontMatter>,
    pub heading: Option<String>,
    pub preamble: Vec<String>,
    pub sections: Vec<Section>,
}

impl Document {
    pub fn parse(content: &str) -> Result<Self, AppError> {
        parse(content)
    }

    pub fn render(&self) -> String {
        render(self)
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", render(self))
    }
}

pub fn parse(content: &str) -> Result<Document, AppError> {
    let (fm, body_start) = match frontmatter::parse_block(content)? {
        Some((f, bs)) => (Some(f), bs),
        None => (None, 0),
    };
    let body = &content[body_start..];
    let mut lines: Vec<&str> = body.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let heading = if lines.is_empty() {
        None
    } else {
        Some(lines[0].to_string())
    };
    let mut sections: Vec<Section> = Vec::new();
    let mut preamble: Vec<String> = Vec::new();
    let mut i = 1usize;
    while i < lines.len() {
        if let Some(rest) = lines[i].strip_prefix("## ") {
            let heading_line = format!("## {}", rest);
            let mut body_lines: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("## ") {
                body_lines.push(lines[i].to_string());
                i += 1;
            }
            sections.push(Section {
                heading: heading_line,
                body: body_lines,
            });
        } else if sections.is_empty() {
            preamble.push(lines[i].to_string());
            i += 1;
        } else if let Some(last) = sections.last_mut() {
            last.body.push(lines[i].to_string());
            i += 1;
        } else {
            i += 1;
        }
    }
    Ok(Document {
        source: content.to_string(),
        front_matter: fm.clone(),
        frontmatter: fm,
        heading,
        preamble,
        sections,
    })
}

pub fn render(doc: &Document) -> String {
    let mut lines: Vec<String> = Vec::new();
    if let Some(fm) = &doc.front_matter {
        lines.push("---".to_string());
        for l in fm.raw.split('\n') {
            lines.push(l.to_string());
        }
        // avoid double blank? raw has no trailing newline
        lines.push("---".to_string());
    }
    if let Some(h) = &doc.heading {
        lines.push(h.clone());
    }
    lines.extend(doc.preamble.clone());
    for section in &doc.sections {
        lines.push(section.heading.clone());
        lines.extend(section.body.clone());
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

// Convenience aliases
pub fn to_text(doc: &Document) -> String {
    render(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_render_identity_legacy() {
        let input = "# T Title\n\n## Description\nBody\n";
        let doc = parse(input).unwrap();
        assert_eq!(render(&doc), input);
    }

    #[test]
    fn parse_render_identity_front_matter() {
        let input = "---\nid: X\ntitle: T\n---\n# X T\n\n## Description\nBody\n";
        let doc = parse(input).unwrap();
        assert_eq!(render(&doc), input);
    }

    #[test]
    fn unknown_section_preserved() {
        let input = "# T Title\n\n## Description\nBody\n\n## Criticality\nMust have\n";
        let doc = parse(input).unwrap();
        assert!(doc.sections.iter().any(|s| s.heading == "## Criticality"));
        let out = render(&doc);
        assert!(out.contains("## Criticality\nMust have\n"));
    }

    #[test]
    fn front_matter_unknown_key_preserved_on_render() {
        let input = "---\nid: X\nforeign: 42\n---\n# X T\n\n## Description\nBody\n";
        let doc = parse(input).unwrap();
        assert_eq!(render(&doc), input);
        assert!(
            doc.front_matter
                .as_ref()
                .unwrap()
                .raw
                .contains("foreign: 42")
        );
    }

    #[test]
    fn malformed_front_matter_errors() {
        let input = "---\n: bad\n---\n# X T\n";
        assert!(parse(input).is_err());
    }
}
