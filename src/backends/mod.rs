mod dnf;
mod flatpak;
mod cargo_backend;

use async_trait::async_trait;
use indicatif::ProgressBar;
use crate::models::result::SearchResult;

#[async_trait]
pub trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn exists(&self) -> bool;
    async fn search(&self, query: &str, pb: ProgressBar) -> Vec<SearchResult>;
    async fn install(&self, pkg: &SearchResult) -> Result<(), String>;
}

pub fn get_available_backends() -> Vec<Box<dyn Backend>> {
    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(dnf::DnfBackend),
        Box::new(flatpak::FlatpakBackend),
        Box::new(cargo_backend::CargoBackend),
    ];

    backends
        .into_iter()
        .filter(|b| b.exists())
        .collect()
}


