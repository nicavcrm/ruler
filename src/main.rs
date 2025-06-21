use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "ruler")]
#[command(about = "Convert between Cursor rules and GitHub Copilot instructions")]
#[command(version = "0.1.0")]
struct Cli {
    /// Conversion mode
    #[arg(value_enum)]
    mode: ConversionMode,

    /// Source directory (defaults: c2g=.cursor/rules, g2c=.github/instructions)
    #[arg(short = 'f', long = "from")]
    from_folder: Option<PathBuf>,

    /// Target directory (defaults: c2g=.github/instructions, g2c=.cursor/rules)
    #[arg(short = 't', long = "to")]
    to_folder: Option<PathBuf>,
}

#[derive(Clone, ValueEnum)]
enum ConversionMode {
    /// Convert Cursor rules to GitHub Copilot instructions
    C2g,
    /// Convert GitHub Copilot instructions to Cursor rules
    G2c,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CursorMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_globs"
    )]
    globs: Option<Vec<String>>,
    #[serde(rename = "alwaysApply", skip_serializing_if = "Option::is_none")]
    always_apply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GithubMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "applyTo", skip_serializing_if = "Option::is_none")]
    apply_to: Option<String>,
}

// Custom deserializer to handle multiple formats for globs:
// - Array: ["glob1", "glob2"]
// - Single string: "glob1"
// - Comma-separated string: "glob1,glob2"
// - Multiple quoted strings: "glob1", "glob2"
fn deserialize_globs<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.mode {
        ConversionMode::C2g => {
            let from_dir = cli
                .from_folder
                .unwrap_or_else(|| PathBuf::from(".cursor/rules"));
            let to_dir = cli
                .to_folder
                .unwrap_or_else(|| PathBuf::from(".github/instructions"));
            convert_cursor_to_github(&from_dir, &to_dir)
        }
        ConversionMode::G2c => {
            let from_dir = cli
                .from_folder
                .unwrap_or_else(|| PathBuf::from(".github/instructions"));
            let to_dir = cli
                .to_folder
                .unwrap_or_else(|| PathBuf::from(".cursor/rules"));
            convert_github_to_cursor(&from_dir, &to_dir)
        }
    }
}

fn convert_cursor_to_github(from_dir: &Path, to_dir: &Path) -> Result<()> {
    println!("Converting Cursor rules to GitHub Copilot instructions...");
    println!("From: {}", from_dir.display());
    println!("To: {}", to_dir.display());

    // Create target directory if it doesn't exist
    fs::create_dir_all(to_dir)
        .with_context(|| format!("Failed to create directory: {}", to_dir.display()))?;

    // Find all .mdc and .md files in the source directory
    let source_files = find_cursor_files(from_dir)?;

    if source_files.is_empty() {
        println!("No .mdc or .md files found in {}", from_dir.display());
        return Ok(());
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for source_file in source_files {
        let relative_path = source_file
            .strip_prefix(from_dir)
            .with_context(|| "Failed to get relative path")?;

        // Change extension from .mdc/.md to .instructions.md
        let mut target_path = to_dir.join(relative_path);
        let file_stem = target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        target_path.set_file_name(format!("{}.instructions.md", file_stem));

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                continue;
            }
        }

        match convert_mdc_to_md(&source_file, &target_path) {
            Ok(()) => {
                println!(
                    "Converted: {} -> {}",
                    source_file.display(),
                    target_path.display()
                );
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Error converting {}: {}", source_file.display(), e);
                error_count += 1;
                continue;
            }
        }
    }

    if error_count > 0 {
        println!(
            "Conversion completed with {} successes and {} errors.",
            success_count, error_count
        );
    } else {
        println!("Conversion completed successfully!");
    }
    Ok(())
}

fn convert_github_to_cursor(from_dir: &Path, to_dir: &Path) -> Result<()> {
    println!("Converting GitHub Copilot instructions to Cursor rules...");
    println!("From: {}", from_dir.display());
    println!("To: {}", to_dir.display());

    // Create target directory if it doesn't exist
    fs::create_dir_all(to_dir)
        .with_context(|| format!("Failed to create directory: {}", to_dir.display()))?;

    // Find all .md and .instructions.md files in the source directory
    let source_files = find_github_files(from_dir)?;

    if source_files.is_empty() {
        println!("No .md or .instructions.md files found in {}", from_dir.display());
        return Ok(());
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for source_file in source_files {
        let relative_path = source_file
            .strip_prefix(from_dir)
            .with_context(|| "Failed to get relative path")?;

        // Change extension from .instructions.md/.md to .mdc
        let mut target_path = to_dir.join(relative_path);
        if let Some(file_name) = target_path.file_name().and_then(|n| n.to_str()) {
            if let Some(base_name) = file_name.strip_suffix(".instructions.md") {
                target_path.set_file_name(format!("{}.mdc", base_name));
            } else if let Some(base_name) = file_name.strip_suffix(".md") {
                target_path.set_file_name(format!("{}.mdc", base_name));
            } else {
                // Fallback
                target_path.set_extension("mdc");
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                continue;
            }
        }

        match convert_md_to_mdc(&source_file, &target_path) {
            Ok(()) => {
                println!(
                    "Converted: {} -> {}",
                    source_file.display(),
                    target_path.display()
                );
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Error converting {}: {}", source_file.display(), e);
                error_count += 1;
                continue;
            }
        }
    }

    if error_count > 0 {
        println!(
            "Conversion completed with {} successes and {} errors.",
            success_count, error_count
        );
    } else {
        println!("Conversion completed successfully!");
    }
    Ok(())
}

fn find_cursor_files(dir: &Path) -> Result<Vec<PathBuf>> {
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

fn find_github_files(dir: &Path) -> Result<Vec<PathBuf>> {
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

fn convert_mdc_to_md(source: &Path, target: &Path) -> Result<()> {
    let content = fs::read_to_string(source)
        .with_context(|| format!("Failed to read file: {}", source.display()))?;

    let (frontmatter, body) = parse_frontmatter(&content)?;

    // Convert Cursor metadata to GitHub metadata
    let github_metadata = if let Some(fm) = frontmatter {
        // Try to handle the non-standard YAML format by preprocessing it
        let preprocessed_fm = preprocess_frontmatter(&fm);

        let cursor_meta: CursorMetadata = serde_yaml::from_str(&preprocessed_fm)
            .with_context(|| format!("Failed to parse Cursor frontmatter after preprocessing: {}", preprocessed_fm))?;

        let github_meta = GithubMetadata {
            description: cursor_meta.description,
            apply_to: if cursor_meta.always_apply == Some(true) {
                Some("**".to_string())
            } else if let Some(globs) = cursor_meta.globs {
                if !globs.is_empty() {
                    Some(globs.join(","))
                } else {
                    None
                }
            } else {
                None
            },
        };

        Some(github_meta)
    } else {
        None
    };

    // Write the converted file
    let output_content = if let Some(meta) = github_metadata {
        let frontmatter_yaml =
            serde_yaml::to_string(&meta).with_context(|| "Failed to serialize GitHub metadata")?;
        format!("---\n{}---\n\n{}", frontmatter_yaml, body)
    } else {
        body
    };

    fs::write(target, output_content)
        .with_context(|| format!("Failed to write file: {}", target.display()))?;

    Ok(())
}

fn convert_md_to_mdc(source: &Path, target: &Path) -> Result<()> {
    let content = fs::read_to_string(source)
        .with_context(|| format!("Failed to read file: {}", source.display()))?;

    let (frontmatter, body) = parse_frontmatter(&content)?;

    // Convert GitHub metadata to Cursor metadata
    let cursor_metadata = if let Some(fm) = frontmatter {
        let github_meta: GithubMetadata =
            serde_yaml::from_str(&fm).with_context(|| "Failed to parse GitHub frontmatter")?;

        let mut cursor_meta = CursorMetadata::default();
        cursor_meta.description = github_meta.description;

        // Convert applyTo to globs and alwaysApply
        if let Some(apply_to) = github_meta.apply_to {
            if apply_to == "**" {
                cursor_meta.always_apply = Some(true);
                cursor_meta.globs = Some(vec![]);
            } else {
                cursor_meta.always_apply = Some(false);
                cursor_meta.globs =
                    Some(apply_to.split(',').map(|s| s.trim().to_string()).collect());
            }
        }

        Some(cursor_meta)
    } else {
        None
    };

    // Write the converted file
    let output_content = if let Some(meta) = cursor_metadata {
        let frontmatter_yaml =
            serde_yaml::to_string(&meta).with_context(|| "Failed to serialize Cursor metadata")?;
        format!("---\n{}---\n\n{}", frontmatter_yaml, body)
    } else {
        body
    };

    fs::write(target, output_content)
        .with_context(|| format!("Failed to write file: {}", target.display()))?;

    Ok(())
}

fn parse_frontmatter(content: &str) -> Result<(Option<String>, String)> {
    let content = content.trim();

    if !content.starts_with("---") {
        return Ok((None, content.to_string()));
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 {
        return Ok((None, content.to_string()));
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
            Ok((Some(frontmatter), body))
        }
        None => Ok((None, content.to_string())),
    }
}

fn preprocess_frontmatter(frontmatter: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
description: "Test rule"
globs: ["*.ts"]
---

This is the rule content."#;

        let (frontmatter, body) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_some());
        assert_eq!(body.trim(), "This is the rule content.");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "Just some rule content without frontmatter.";
        let (frontmatter, body) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_comma_separated_globs() {
        let content = r#"---
description: "Test comma-separated globs"
globs: "**/optimization*/**,**/integration*/**"
alwaysApply: false
---

This is a test rule with comma-separated globs."#;

        let (frontmatter, body) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_some());

        // Test that the frontmatter can be parsed correctly
        let cursor_meta: CursorMetadata = serde_yaml::from_str(&frontmatter.unwrap()).unwrap();
        assert_eq!(
            cursor_meta.description,
            Some("Test comma-separated globs".to_string())
        );
        assert_eq!(
            cursor_meta.globs,
            Some(vec![
                "**/optimization*/**".to_string(),
                "**/integration*/**".to_string()
            ])
        );
        assert_eq!(cursor_meta.always_apply, Some(false));
        assert_eq!(
            body.trim(),
            "This is a test rule with comma-separated globs."
        );
    }

    #[test]
    fn test_multiple_quoted_strings_globs() {
        let content = r#"---
description: "Test multiple quoted strings"
globs: "**/mode-transition*/**", "**/context-preservation*/**"
alwaysApply: false
---

This is a test rule with multiple quoted strings format."#;

        let (frontmatter, body) = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_some());

        // Preprocess the frontmatter to handle the non-standard format
        let preprocessed_fm = preprocess_frontmatter(&frontmatter.unwrap());

        // Test that the frontmatter can be parsed correctly after preprocessing
        let cursor_meta: CursorMetadata = serde_yaml::from_str(&preprocessed_fm).unwrap();
        assert_eq!(
            cursor_meta.description,
            Some("Test multiple quoted strings".to_string())
        );
        assert_eq!(
            cursor_meta.globs,
            Some(vec![
                "**/mode-transition*/**".to_string(),
                "**/context-preservation*/**".to_string()
            ])
        );
        assert_eq!(cursor_meta.always_apply, Some(false));
        assert_eq!(
            body.trim(),
            "This is a test rule with multiple quoted strings format."
        );
    }
}
