use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

mod parser;

use parser::{convert_cursor_to_github, convert_github_to_cursor};

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

#[cfg(test)]
mod tests {
    use super::parser::common::{parse_frontmatter, preprocess_frontmatter, CursorMetadata};

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
