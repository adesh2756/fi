//! Cargo backend for searching and installing Rust crates.

use tokio::process::Command;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::models::result::SearchResult;
use super::Backend;

/// Backend implementation for the Cargo package manager.
pub struct CargoBackend;

#[async_trait]
impl Backend for CargoBackend {
    fn name(&self) -> &'static str { "cargo" }

    fn exists(&self) -> bool {
        which::which("cargo").is_ok()
    }

    async fn search(&self, query: &str, pb: ProgressBar) -> Vec<SearchResult> {
        pb.set_message("Searching Cargo...");

        let output = Command::new("cargo")
            .arg("search")
            .arg(query)
            .output()
            .await;

        let results = if let Ok(out) = output {
            parse_cargo(&String::from_utf8_lossy(&out.stdout))
        } else {
            vec![]
        };

        pb.finish_with_message("Cargo search done");
        results
    }

    async fn install(&self, pkg: &SearchResult) -> Result<(), String> {
        let status = Command::new("cargo")
            .arg("install")
            .arg(&pkg.identifier)
            .status()
            .await;

        if let Ok(s) = status {
            if s.success() { return Ok(()); }
        }
        Err("Cargo install failed".into())
    }
}

/// Parses Cargo search output.
///
/// Cargo search output format: `name = "version" # description`
/// Example: `test = "0.1.0" # A testing framework for Rust`
fn parse_cargo(s: &str) -> Vec<SearchResult> {
    s.lines()
        .filter(|line| {
            let line = line.trim();
            // Skip empty lines, notes, and summary lines
            !line.is_empty()
                && !line.starts_with("...")
                && !line.starts_with("note:")
                && line.contains(" = ")
        })
        .filter_map(|line| {
            
            let trimmed = line.trim();
            
            // Find the equals sign
            let Some(eq_pos) = trimmed.find(" = ") else {
                return None;
            };
            
            let name = trimmed[..eq_pos].trim();
            let rest = &trimmed[eq_pos + 3..]; // Skip " = "
            
            // Extract version (quoted string)
            let (version, description) = if let Some(quote_start) = rest.find('"') {
                let after_quote = &rest[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    let version = &after_quote[..quote_end];
                    let after_version = &after_quote[quote_end + 1..].trim_start();
                    
                    // Extract description (after #)
                    let desc = if let Some(desc_start) = after_version.find('#') {
                        after_version[desc_start + 1..].trim()
                    } else {
                        ""
                    };
                    
                    (Some(version.to_string()), desc.to_string())
                } else {
                    (None, rest.trim().to_string())
                }
            } else {
                // No quoted version, try to parse anyway
                let parts: Vec<&str> = rest.split('#').collect();
                let version = parts.get(0).map(|s| s.trim()).filter(|s| !s.is_empty());
                let description = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();
                (version.map(|s| s.to_string()), description)
            };
            
            if name.is_empty() {
                return None;
            }

            Some(SearchResult {
                backend: "cargo".into(),
                name: name.to_string(),
                identifier: name.to_string(),
                description: if description.is_empty() {
                    "No description available".to_string()
                } else {
                    description
                },
                version,
            })
        })
        .collect()
}

