use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::common::{
    find_github_files, parse_frontmatter,
    CursorMetadata, GithubMetadata
};

pub fn convert_github_to_cursor(from_dir: &Path, to_dir: &Path) -> Result<()> {
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
