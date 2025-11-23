//! DNF (Dandified YUM) backend for searching and installing RPM packages.

use tokio::process::Command;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::models::result::SearchResult;
use super::Backend;

/// Backend implementation for the DNF package manager.
pub struct DnfBackend;

#[async_trait]
impl Backend for DnfBackend {
    fn name(&self) -> &'static str { "dnf" }

    fn exists(&self) -> bool {
        which::which("dnf").is_ok()
    }

    async fn search(&self, query: &str, pb: ProgressBar) -> Vec<SearchResult> {
        pb.set_message("Searching DNF...");

        let output = Command::new("dnf")
            .arg("search")
            .arg("--assumeyes")
            .arg("--setopt=assumeyes=True")
            .arg(query)
            .output()
            .await;

        let results = if let Ok(out) = output {
            parse_dnf(&String::from_utf8_lossy(&out.stdout))
        } else {
            vec![]
        };

        pb.finish_with_message("DNF search done");
        results
    }

    async fn install(&self, pkg: &SearchResult) -> Result<(), String> {
        let status = Command::new("sudo")
            .arg("dnf")
            .arg("install")
            .arg("-y")
            .arg(&pkg.identifier)
            .status()
            .await;

        if let Ok(s) = status {
            if s.success() { return Ok(()); }
        }
        Err("DNF install failed".into())
    }
}

fn parse_dnf(s: &str) -> Vec<SearchResult> {
    // Very simple parser — you will refine later
    s.lines()
     .filter(|l| l.contains(":"))
     .map(|line| {
         let name = line.split(":").next().unwrap_or("").trim().to_string();

         SearchResult {
             backend: "dnf".into(),
             name: name.clone(),
             identifier: name,
             description: line.to_string(),
             version: None,
         }
     })
     .collect()
}

