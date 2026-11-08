//! The only mutation API for records.

use crate::error::AppError;
use crate::record::document::{self, Document, Section};
use crate::record::frontmatter::{self, FrontMatterValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatchOp {
    SetFrontMatter(String, String),
    SetFrontMatterValue(String, FrontMatterValue),
    ClearFrontMatter(String),
    SetSection(String, String),
    ClearSection(String),
    SetHeadingTitle(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub ops: Vec<PatchOp>,
}

impl Patch {
    pub fn new(ops: Vec<PatchOp>) -> Self {
        Self { ops }
    }

    pub fn apply(&self, doc: &Document) -> Result<String, AppError> {
        apply_patch(doc, &self.ops)
    }
}

pub fn apply_patch(doc: &Document, ops: &[PatchOp]) -> Result<String, AppError> {
    if ops.is_empty() {
        return Ok(doc.source.clone());
    }
    let mut fm = doc.front_matter.clone();
    let mut heading = doc.heading.clone();
    let preamble = doc.preamble.clone();
    let mut sections = doc.sections.clone();
    let mut changed = false;
    let mut body_changed = false;

    for op in ops {
        match op {
            PatchOp::SetFrontMatter(key, value) => {
                if fm.is_none() {
                    fm = Some(frontmatter::FrontMatter {
                        raw: String::new(),
                        text: String::new(),
                        value: yaml_serde::Value::Mapping(yaml_serde::Mapping::new()),
                        data: yaml_serde::Value::Mapping(yaml_serde::Mapping::new()),
                    });
                }
                let f = fm.as_mut().unwrap();
                let noop = if frontmatter::contains_key(&f.raw, key) {
                    // scalar no-op
                    if frontmatter::current_value_string(&f.raw, key).as_deref()
                        == Some(value.as_str())
                    {
                        true
                    } else if let Some(block) = frontmatter::current_key_block(&f.raw, key) {
                        block == frontmatter::format_kv_pub(key, value)
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !noop {
                    let (new_raw, _) = frontmatter::patch_raw(&f.raw, key, Some(value));
                    let new_value: yaml_serde::Value =
                        yaml_serde::from_str(&new_raw).unwrap_or_else(|_| f.value.clone());
                    f.raw = new_raw;
                    f.value = new_value;
                    changed = true;
                }
            }
            PatchOp::SetFrontMatterValue(key, value) => {
                if fm.is_none() {
                    fm = Some(frontmatter::FrontMatter {
                        raw: String::new(),
                        text: String::new(),
                        value: yaml_serde::Value::Mapping(yaml_serde::Mapping::new()),
                        data: yaml_serde::Value::Mapping(yaml_serde::Mapping::new()),
                    });
                }
                let f = fm.as_mut().unwrap();
                let noop = if frontmatter::contains_key(&f.raw, key) {
                    match value {
                        FrontMatterValue::Null => frontmatter::current_value_string(&f.raw, key)
                            .map(|v| v.is_empty())
                            .unwrap_or(true),
                        FrontMatterValue::Integer(n) => {
                            frontmatter::current_value_string(&f.raw, key).as_deref()
                                == Some(&n.to_string())
                        }
                        FrontMatterValue::String(s) => {
                            if frontmatter::current_value_string(&f.raw, key).as_deref()
                                == Some(s.as_str())
                            {
                                true
                            } else if let Some(block) = frontmatter::current_key_block(&f.raw, key)
                            {
                                block == frontmatter::format_kv_pub(key, s)
                            } else {
                                false
                            }
                        }
                    }
                } else {
                    // Always add a missing key, even when the value is null.
                    false
                };
                if !noop {
                    let (new_raw, _) = frontmatter::patch_raw_value(&f.raw, key, value);
                    let new_value: yaml_serde::Value =
                        yaml_serde::from_str(&new_raw).unwrap_or_else(|_| f.value.clone());
                    f.raw = new_raw;
                    f.value = new_value;
                    changed = true;
                }
            }
            PatchOp::ClearFrontMatter(key) => {
                if let Some(f) = &mut fm
                    && frontmatter::contains_key(&f.raw, key)
                {
                    let (new_raw, _) = frontmatter::patch_raw(&f.raw, key, None);
                    let new_value: yaml_serde::Value =
                        yaml_serde::from_str(&new_raw).unwrap_or_else(|_| f.value.clone());
                    f.raw = new_raw;
                    f.value = new_value;
                    changed = true;
                }
            }
            PatchOp::SetSection(h, value) => {
                if let Some(sec) = sections.iter_mut().find(|s| s.heading == *h) {
                    if sec.body.join("\n") != *value {
                        sec.body = value.split('\n').map(String::from).collect();
                        changed = true;
                    }
                } else {
                    sections.push(Section {
                        heading: h.clone(),
                        body: value.split('\n').map(String::from).collect(),
                    });
                    changed = true;
                }
                body_changed = true;
            }
            PatchOp::ClearSection(h) => {
                if let Some(idx) = sections.iter().position(|s| s.heading == *h) {
                    sections.remove(idx);
                    changed = true;
                }
                body_changed = true;
            }
            PatchOp::SetHeadingTitle(title) => {
                let new_heading = if let Some(h) = &heading {
                    if let Some(rest) = h.strip_prefix("# ") {
                        if let Some((id, _)) = rest.split_once(' ') {
                            format!("# {} {}", id, title)
                        } else {
                            format!("# {} {}", rest, title)
                        }
                    } else {
                        title.clone()
                    }
                } else {
                    title.clone()
                };
                if heading.as_deref() != Some(new_heading.as_str()) {
                    heading = Some(new_heading);
                    changed = true;
                }
                body_changed = true;
            }
        }
    }

    if !changed {
        return Ok(doc.source.clone());
    }
    // If only front matter changed, preserve the original body bytes exactly.
    if !body_changed && let Some(f) = &fm {
        if let Some((_, _, body_start)) = frontmatter::find_block(&doc.source) {
            let body = &doc.source[body_start..];
            return Ok(frontmatter::wrap(&f.raw, body));
        }
        // No original front matter: the whole source becomes the body.
        return Ok(frontmatter::wrap(&f.raw, &doc.source));
    }
    let new_doc = Document {
        source: String::new(),
        front_matter: fm.clone(),
        frontmatter: fm,
        heading,
        preamble,
        sections,
    };
    Ok(document::render(&new_doc))
}

// aliases
pub fn apply(doc: &Document, ops: &[PatchOp]) -> Result<String, AppError> {
    apply_patch(doc, ops)
}

pub fn apply_patches(doc: &Document, ops: &[PatchOp]) -> Result<String, AppError> {
    apply_patch(doc, ops)
}

pub fn patch(doc: &Document, ops: &[PatchOp]) -> Result<String, AppError> {
    apply_patch(doc, ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::document;

    #[test]
    fn empty_patch_byte_identical() {
        let input = "---\nid: X\ntitle: T\n---\n# X T\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let out = apply_patch(&doc, &[]).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn unchanged_values_byte_identical() {
        let input = "---\nid: X\ntitle: T\n---\n# X T\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let ops = vec![
            PatchOp::SetFrontMatter("title".into(), "T".into()),
            PatchOp::SetSection("## Description".into(), "Body".into()),
            PatchOp::SetHeadingTitle("T".into()),
        ];
        let out = apply_patch(&doc, &ops).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn patch_clear_frontmatter_noop_when_missing() {
        let input = "# T Title\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let ops = vec![PatchOp::ClearFrontMatter("title".into())];
        let out = apply_patch(&doc, &ops).unwrap();
        assert_eq!(out, input);
    }

    #[test]
    fn unknown_key_preserved_on_patch() {
        let input = "---\nid: X\nforeign: 42\n---\n# X T\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let ops = vec![PatchOp::SetFrontMatter("title".into(), "New".into())];
        let out = apply_patch(&doc, &ops).unwrap();
        assert!(out.contains("foreign: 42"));
    }

    #[test]
    fn unknown_section_preserved_on_patch() {
        let input = "# T Title\n\n## Description\nBody\n\n## Criticality\nMust have\n";
        let doc = document::parse(input).unwrap();
        let ops = vec![PatchOp::SetFrontMatter("title".into(), "X".into())];
        // no front matter; SetFrontMatter creates block, but sections preserved
        let out = apply_patch(&doc, &ops).unwrap();
        assert!(out.contains("## Criticality\nMust have"));
    }

    #[test]
    fn patch_op_variants_exist() {
        let ops = [
            PatchOp::SetFrontMatter("k".into(), "v".into()),
            PatchOp::ClearFrontMatter("k".into()),
            PatchOp::SetSection("## S".into(), "v".into()),
            PatchOp::ClearSection("## S".into()),
            PatchOp::SetHeadingTitle("t".into()),
        ];
        assert_eq!(ops.len(), 5);
    }

    #[test]
    fn patch_struct_apply_matches() {
        let input = "# T Title\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let p = Patch::new(vec![]);
        assert_eq!(p.apply(&doc).unwrap(), input);
    }

    #[test]
    fn set_front_matter_value_renders_null_and_integer() {
        use crate::record::frontmatter::FrontMatterValue;
        let input = "---\nid: X\ntitle: T\n---\n# X T\n\n## Description\nBody\n";
        let doc = document::parse(input).unwrap();
        let ops = vec![
            PatchOp::SetFrontMatterValue("release".into(), FrontMatterValue::Null),
            PatchOp::SetFrontMatterValue("ordinal".into(), FrontMatterValue::Integer(7)),
        ];
        let out = apply_patch(&doc, &ops).unwrap();
        assert!(out.contains("release: null"), "got: {}", out);
        assert!(out.contains("ordinal: 7"), "got: {}", out);
        assert!(!out.contains("\"null\""), "got: {}", out);
        assert!(!out.contains("\"7\""), "got: {}", out);
    }

    #[test]
    fn front_matter_only_patch_preserves_body_whitespace() {
        let body = "\n## Description\n\nMilestone body\n\n";
        let input = format!("---\nid: m-2\ntitle: Title\n---\n{}", body);
        let doc = document::parse(&input).unwrap();
        let ops = vec![PatchOp::SetFrontMatter("release".into(), "null".into())];
        let out = apply_patch(&doc, &ops).unwrap();
        // Body bytes should be unchanged apart from the front-matter block.
        let (_, out_body) = crate::record::frontmatter::split(&out).unwrap();
        assert_eq!(out_body, body, "body changed: {:?} vs {:?}", out_body, body);
    }
}
