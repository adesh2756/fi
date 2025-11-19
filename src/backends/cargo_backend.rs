use tokio::process::Command;
use indicatif::ProgressBar;
use async_trait::async_trait;
use crate::models::result::SearchResult;
use super::Backend;

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

        pb.finish_with_message("Cargo search done");

        if let Ok(out) = output {
            return parse_cargo(&String::from_utf8_lossy(&out.stdout));
        }

        vec![]
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

fn parse_cargo(s: &str) -> Vec<SearchResult> {
    s.lines()
     .filter(|l| l.contains(" = "))
     .map(|line| {
         let name = line.split('=').next().unwrap().trim().to_string();
         SearchResult {
             backend: "cargo".into(),
             name: name.clone(),
             identifier: name,
             description: line.to_string(),
             version: None,
         }
     })
     .collect()
}

