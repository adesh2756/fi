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

/// Parses DNF search output.
///
/// DNF search output format: `PackageName.arch\tDescription`
/// Example: `python3.x86_64\tPython 3 interpreter`
fn parse_dnf(s: &str) -> Vec<SearchResult> {
    s.lines()
        .filter(|line| {
            // Skip header lines and empty lines
            let line = line.trim();
            !line.is_empty()
                && !line.starts_with("Updating")
                && !line.starts_with("Repositories")
                && !line.starts_with("Matched fields")
                && line.contains('\t')
        })
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                return None;
            }

            let full_name = parts[0].trim();
            let description = parts[1].trim();

            // Skip if empty
            if full_name.is_empty() || description.is_empty() {
                return None;
            }

            // Extract package name and architecture
            // Format: PackageName.arch (e.g., "test.x86_64" or "test.noarch")
            let (pkg_name, arch) = if let Some(dot_pos) = full_name.rfind('.') {
                let (name, arch_part) = full_name.split_at(dot_pos);
                (name, Some(&arch_part[1..])) // Skip the dot
            } else {
                (full_name, None)
            };

            // Use just the package name as identifier (DNF installs by name, not arch)
            let identifier = pkg_name.to_string();
            
            // Create display name with architecture if present
            let display_name = if let Some(arch) = arch {
                format!("{} ({})", pkg_name, arch)
            } else {
                pkg_name.to_string()
            };

            Some(SearchResult {
                backend: "dnf".into(),
                name: display_name,
                identifier,
                description: description.to_string(),
                version: None, // DNF search doesn't show versions
            })
        })
        .collect()
}

