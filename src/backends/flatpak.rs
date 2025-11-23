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

fn parse_flatpak(s: &str) -> Vec<SearchResult> {
    s.lines()
     .filter(|l| l.contains("\t"))
     .map(|line| {
         let parts: Vec<&str> = line.split('\t').collect();
         SearchResult {
             backend: "flatpak".into(),
             name: parts.get(1).unwrap_or(&"").to_string(),
             identifier: parts.get(0).unwrap_or(&"").to_string(),
             description: parts.get(2).unwrap_or(&"").to_string(),
             version: None,
         }
     })
     .collect()
}

