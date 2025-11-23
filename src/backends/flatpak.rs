//! Flatpak backend for searching and installing Flatpak applications.

use tokio::process::Command;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::models::result::SearchResult;
use super::Backend;

/// Backend implementation for the Flatpak package manager.
pub struct FlatpakBackend;

#[async_trait]
impl Backend for FlatpakBackend {
    fn name(&self) -> &'static str { "flatpak" }

    fn exists(&self) -> bool {
        which::which("flatpak").is_ok()
    }

    async fn search(&self, query: &str, pb: ProgressBar) -> Vec<SearchResult> {
        pb.set_message("Searching Flatpak...");

        let output = Command::new("flatpak")
            .arg("search")
            .arg(query)
            .output()
            .await;

        let results = if let Ok(out) = output {
            parse_flatpak(&String::from_utf8_lossy(&out.stdout))
        } else {
            vec![]
        };

        pb.finish_with_message("Flatpak search done");
        results
    }

    async fn install(&self, pkg: &SearchResult) -> Result<(), String> {
        let status = Command::new("flatpak")
            .arg("install")
            .arg("flathub")
            .arg(&pkg.identifier)
            .status()
            .await;

        if let Ok(s) = status {
            if s.success() { return Ok(()); }
        }
        Err("Flatpak install failed".into())
    }
}

/// Parses Flatpak search output.
///
/// Flatpak search output format: `Name\tDescription\tApplicationID\tVersion\tBranch\tRemote`
/// Example: `Discord\tChat and voice client\tcom.discordapp.Discord\t1.0.0\tstable\tflathub`
fn parse_flatpak(s: &str) -> Vec<SearchResult> {
    s.lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && line.contains('\t')
        })
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').map(|s| s.trim()).collect();
            if parts.len() < 3 {
                return None;
            }

            let name = parts[0];
            let description = parts.get(1).copied().unwrap_or("");
            let application_id = parts.get(2).copied().unwrap_or("");
            let version = parts.get(3).copied().filter(|v| !v.is_empty());

            // Skip if essential fields are missing
            if name.is_empty() || application_id.is_empty() {
                return None;
            }

            Some(SearchResult {
                backend: "flatpak".into(),
                name: name.to_string(),
                identifier: application_id.to_string(),
                description: if description.is_empty() {
                    "No description available".to_string()
                } else {
                    description.to_string()
                },
                version: version.map(|v| v.to_string()),
            })
        })
        .collect()
}

