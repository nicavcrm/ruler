use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::common::{
    find_cursor_files, parse_frontmatter, preprocess_frontmatter,
    CursorMetadata, GithubMetadata
};

pub fn convert_cursor_to_github(from_dir: &Path, to_dir: &Path) -> Result<()> {
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
