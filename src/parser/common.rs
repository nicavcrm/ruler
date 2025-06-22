use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CursorMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_globs"
    )]
    pub globs: Option<Vec<String>>,
    #[serde(rename = "alwaysApply", skip_serializing_if = "Option::is_none")]
    pub always_apply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "applyTo", skip_serializing_if = "Option::is_none")]
    pub apply_to: Option<String>,
    #[serde(skip_deserializing)]
    pub description_present: bool,
    #[serde(skip_deserializing)]
    pub apply_to_present: bool,
}

// Custom deserializer to handle multiple formats for globs:
// - Array: ["glob1", "glob2"]
// - Single string: "glob1"
// - Comma-separated string: "glob1,glob2"
// - Multiple quoted strings: "glob1", "glob2"
pub fn deserialize_globs<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct GlobsVisitor;

    impl<'de> Visitor<'de> for GlobsVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, array of strings, or comma-separated values")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Split by comma and trim whitespace, removing quotes if present
            if value.contains(',') {
                let globs: Vec<String> = value
                    .split(',')
                    .map(|s| {
                        let trimmed = s.trim();
                        // Remove surrounding quotes if present
                        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
                           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
                            trimmed[1..trimmed.len()-1].to_string()
                        } else {
                            trimmed.to_string()
                        }
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok(Some(globs))
            } else {
                // Single string, remove quotes if present
                let cleaned = if (value.starts_with('"') && value.ends_with('"')) ||
                                 (value.starts_with('\'') && value.ends_with('\'')) {
                    value[1..value.len()-1].to_string()
                } else {
                    value.to_string()
                };
                Ok(Some(vec![cleaned]))
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                vec.push(value);
            }
            Ok(Some(vec))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(GlobsVisitor)
}

pub fn find_cursor_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str.eq_ignore_ascii_case("mdc") || ext_str.eq_ignore_ascii_case("md") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

pub fn find_github_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".instructions.md") || file_name.ends_with(".md") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

pub fn parse_frontmatter(content: &str) -> Result<(Option<String>, String)> {
    let (frontmatter, body, _) = parse_frontmatter_with_field_info(content)?;
    Ok((frontmatter, body))
}

pub fn parse_frontmatter_with_field_info(content: &str) -> Result<(Option<String>, String, FieldInfo)> {
    let content = content.trim();

    if !content.starts_with("---") {
        return Ok((None, content.to_string(), FieldInfo::default()));
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, content.to_string(), FieldInfo::default()));
    }

    // Find the closing ---
    let mut frontmatter_end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    match frontmatter_end {
        Some(end) => {
            let frontmatter = lines[1..end].join("\n");
            let body = if end + 1 < lines.len() {
                lines[end + 1..].join("\n").trim_start().to_string()
            } else {
                String::new()
            };

            // Analyze which fields are present
            let field_info = analyze_frontmatter_fields(&frontmatter);

            Ok((Some(frontmatter), body, field_info))
        }
        None => Ok((None, content.to_string(), FieldInfo::default())),
    }
}

#[derive(Debug, Default)]
pub struct FieldInfo {
    pub description_present: bool,
    pub globs_present: bool,
}

fn analyze_frontmatter_fields(frontmatter: &str) -> FieldInfo {
    let mut info = FieldInfo::default();

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("description:") {
            info.description_present = true;
        } else if trimmed.starts_with("globs:") {
            info.globs_present = true;
        }
    }

    info
}

pub fn preprocess_frontmatter(frontmatter: &str) -> String {
    let mut result = String::new();

    for line in frontmatter.lines() {
        if let Some(colon_pos) = line.find(':') {
            let key = &line[..colon_pos];
            let value = &line[colon_pos + 1..].trim();

            // Special handling for globs field with comma-separated strings
            if key.trim() == "globs" && value.contains(',') && !value.starts_with('[') {
                // Handle two formats:
                // 1. "string1", "string2" (multiple quoted strings)
                // 2. "string1,string2,string3" (single quoted string with commas)

                let mut array_items = Vec::new();

                // Check if it's format 1: multiple quoted strings separated by commas
                if value.contains("\", \"") || value.contains("', '") {
                    // Split on commas but preserve quoted strings
                    for item in value.split(',') {
                        let trimmed = item.trim();
                        if !trimmed.is_empty() {
                            array_items.push(trimmed.to_string());
                        }
                    }
                } else {
                    // Handle format 2: single string with comma-separated values
                    // First remove outer quotes if present
                    let unquoted = if (value.starts_with('"') && value.ends_with('"')) ||
                                     (value.starts_with('\'') && value.ends_with('\'')) {
                        &value[1..value.len()-1]
                    } else {
                        value
                    };

                    // Split by comma and quote each item
                    for item in unquoted.split(',') {
                        let trimmed = item.trim();
                        if !trimmed.is_empty() {
                            array_items.push(format!("\"{}\"", trimmed));
                        }
                    }
                }

                if !array_items.is_empty() {
                    result.push_str(&format!("{}: [{}]\n", key, array_items.join(", ")));
                    continue;
                }
            }
        }

        // For all other lines, keep as is
        result.push_str(line);
        result.push('\n');
    }

    result
}
