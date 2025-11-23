//! Package manager backends for searching and installing packages.
//!
//! This module provides a trait-based interface for different package managers
//! (DNF, Flatpak, Cargo) and automatically detects which ones are available
//! on the system.

mod dnf;
mod flatpak;
mod cargo;

use async_trait::async_trait;
use indicatif::ProgressBar;
use crate::models::result::SearchResult;

/// Trait for package manager backends.
///
/// Each backend implements this trait to provide search and installation
/// capabilities for a specific package manager.
#[async_trait]
pub trait Backend: Send + Sync {
    /// Returns the name of the backend (e.g., "dnf", "flatpak", "cargo").
    fn name(&self) -> &'static str;

    /// Checks if the backend's package manager is available on the system.
    fn exists(&self) -> bool;

    /// Searches for packages matching the query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string
    /// * `pb` - Progress bar for displaying search progress
    ///
    /// # Returns
    ///
    /// A vector of search results matching the query.
    async fn search(&self, query: &str, pb: ProgressBar) -> Vec<SearchResult>;

    /// Installs a package.
    ///
    /// # Arguments
    ///
    /// * `pkg` - The package to install
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    async fn install(&self, pkg: &SearchResult) -> Result<(), String>;
}

/// Returns a list of all available backends on the system.
///
/// This function checks which package managers are installed and returns
/// only the backends that are available.
pub fn get_available_backends() -> Vec<Box<dyn Backend>> {
    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(dnf::DnfBackend),
        Box::new(flatpak::FlatpakBackend),
        Box::new(cargo::CargoBackend),
    ];

    backends
        .into_iter()
        .filter(|b| b.exists())
        .collect()
}


