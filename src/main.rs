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
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_globs")]
    globs: Option<Vec<String>>,
    #[serde(rename = "alwaysApply", skip_serializing_if = "Option::is_none")]
    always_apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GithubMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "applyTo", skip_serializing_if = "Option::is_none")]
    apply_to: Option<String>,
}

// Custom deserializer to handle both string and array formats for globs
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
            formatter.write_str("a string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![value.to_string()]))
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
            let from_dir = cli.from_folder.unwrap_or_else(|| PathBuf::from(".cursor/rules"));
            let to_dir = cli.to_folder.unwrap_or_else(|| PathBuf::from(".github/instructions"));
            convert_cursor_to_github(&from_dir, &to_dir)
        }
        ConversionMode::G2c => {
            let from_dir = cli.from_folder.unwrap_or_else(|| PathBuf::from(".github/instructions"));
            let to_dir = cli.to_folder.unwrap_or_else(|| PathBuf::from(".cursor/rules"));
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

    // Find all .mdc files in the source directory
    let mdc_files = find_files_with_extension(from_dir, "mdc")?;

    if mdc_files.is_empty() {
        println!("No .mdc files found in {}", from_dir.display());
        return Ok(());
    }

    for mdc_file in mdc_files {
        let relative_path = mdc_file.strip_prefix(from_dir)
            .with_context(|| "Failed to get relative path")?;

        // Change extension from .mdc to .instructions.md
        let mut target_path = to_dir.join(relative_path);
        let file_stem = target_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        target_path.set_file_name(format!("{}.instructions.md", file_stem));

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        convert_mdc_to_md(&mdc_file, &target_path)?;
        println!("Converted: {} -> {}", mdc_file.display(), target_path.display());
    }

    println!("Conversion completed successfully!");
    Ok(())
}

fn convert_github_to_cursor(from_dir: &Path, to_dir: &Path) -> Result<()> {
    println!("Converting GitHub Copilot instructions to Cursor rules...");
    println!("From: {}", from_dir.display());
    println!("To: {}", to_dir.display());

    // Create target directory if it doesn't exist
    fs::create_dir_all(to_dir)
        .with_context(|| format!("Failed to create directory: {}", to_dir.display()))?;

    // Find all .instructions.md files in the source directory
    let md_files = find_instruction_files(from_dir)?;

    if md_files.is_empty() {
        println!("No .instructions.md files found in {}", from_dir.display());
        return Ok(());
    }

    for md_file in md_files {
        let relative_path = md_file.strip_prefix(from_dir)
            .with_context(|| "Failed to get relative path")?;

        // Change extension from .instructions.md to .mdc
        let mut target_path = to_dir.join(relative_path);
        if let Some(file_name) = target_path.file_name().and_then(|n| n.to_str()) {
            if let Some(base_name) = file_name.strip_suffix(".instructions.md") {
                target_path.set_file_name(format!("{}.mdc", base_name));
            } else {
                // Fallback for files that don't end with .instructions.md
                target_path.set_extension("mdc");
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        convert_md_to_mdc(&md_file, &target_path)?;
        println!("Converted: {} -> {}", md_file.display(), target_path.display());
    }

    println!("Conversion completed successfully!");
    Ok(())
}

fn find_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().eq_ignore_ascii_case(extension) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

fn find_instruction_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".instructions.md") {
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
        let cursor_meta: CursorMetadata = serde_yaml::from_str(&fm)
            .with_context(|| "Failed to parse Cursor frontmatter")?;

        let mut github_meta = GithubMetadata::default();
        github_meta.description = cursor_meta.description;

        // Convert globs and alwaysApply to applyTo
        if cursor_meta.always_apply == Some(true) {
            github_meta.apply_to = Some("**".to_string());
        } else if let Some(globs) = cursor_meta.globs {
            if !globs.is_empty() {
                github_meta.apply_to = Some(globs.join(","));
            }
        }

        Some(github_meta)
    } else {
        None
    };

    // Write the converted file
    let output_content = if let Some(meta) = github_metadata {
        let frontmatter_yaml = serde_yaml::to_string(&meta)
            .with_context(|| "Failed to serialize GitHub metadata")?;
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
        let github_meta: GithubMetadata = serde_yaml::from_str(&fm)
            .with_context(|| "Failed to parse GitHub frontmatter")?;

        let mut cursor_meta = CursorMetadata::default();
        cursor_meta.description = github_meta.description;

        // Convert applyTo to globs and alwaysApply
        if let Some(apply_to) = github_meta.apply_to {
            if apply_to == "**" {
                cursor_meta.always_apply = Some(true);
                cursor_meta.globs = Some(vec![]);
            } else {
                cursor_meta.always_apply = Some(false);
                cursor_meta.globs = Some(
                    apply_to.split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                );
            }
        }

        Some(cursor_meta)
    } else {
        None
    };

    // Write the converted file
    let output_content = if let Some(meta) = cursor_metadata {
        let frontmatter_yaml = serde_yaml::to_string(&meta)
            .with_context(|| "Failed to serialize Cursor metadata")?;
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
}
