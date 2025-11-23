//! # fi - Unified Package Manager
//!
//! A unified package search and installation tool for Linux that searches across
//! multiple package managers (DNF, Flatpak, Cargo) and provides a TUI interface
//! to select and install packages.

pub mod backends;
pub mod error;
pub mod models;
pub mod ui;

use backends::{get_available_backends, Backend};
use error::FiError;
use models::result::SearchResult;
use ui::progress::run_search_with_progress;
use ui::tui::{run_tui, AppState};

/// Runs the main application workflow: search, display, and optionally install.
///
/// # Arguments
///
/// * `query` - The search query string
///
/// # Returns
///
/// Returns `Ok(())` if the operation completed successfully, or an error if
/// something went wrong during the process.
pub async fn run(query: &str) -> Result<(), FiError> {
    // Load available backends (dnf, flatpak, cargo, etc.)
    let backends = get_available_backends();

    if backends.is_empty() {
        return Err(FiError::InvalidArguments(
            "No package managers found. Please install at least one: dnf, flatpak, or cargo".into(),
        ));
    }

    // Perform async search on all available backends
    let results = run_search_with_progress(&query, &backends).await;

    // Group results by backend for the TUI
    let sections = group_results_by_backend(&results, &backends);

    // Create the application state
    let mut app = AppState::new(sections);

    // Run the ratatui interface
    run_tui(&mut app).map_err(|e| FiError::TuiError(e.to_string()))?;

    // If the user selected a package, install it
    if let Some(selected) = app.selected_result {
        install_package(&selected, backends).await?;
    }

    Ok(())
}

/// Groups search results by their backend.
fn group_results_by_backend(
    results: &[SearchResult],
    backends: &[Box<dyn Backend>],
) -> Vec<(String, Vec<SearchResult>)> {
    let mut sections: Vec<(String, Vec<SearchResult>)> = Vec::new();

    for backend in backends {
        let name = backend.name().to_string();

        let items: Vec<SearchResult> = results
            .iter()
            .filter(|r| r.backend == name)
            .cloned()
            .collect();

        sections.push((name, items));
    }

    sections
}

/// Installs a package using the appropriate backend.
async fn install_package(
    selected: &SearchResult,
    backends: Vec<Box<dyn Backend>>,
) -> Result<(), FiError> {
    for backend in backends {
        if backend.name() == selected.backend {
            println!(
                "Installing {} via {}...",
                selected.name, backend.name()
            );

            backend
                .install(selected)
                .await
                .map_err(|e| FiError::InstallationFailed(e))?;

            println!("Installation complete.");
            return Ok(());
        }
    }

    Err(FiError::BackendNotFound(selected.backend.clone()))
}

