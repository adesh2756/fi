use tokio::process::Command;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::models::result::SearchResult;
use super::Backend;

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
            .arg(query)
            .output()
            .await;

        pb.finish_with_message("DNF search done");

        if let Ok(out) = output {
            return parse_dnf(&String::from_utf8_lossy(&out.stdout));
        }

        vec![]
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

