mod backends;
mod models;
mod ui;
mod utils;

use backends::{get_available_backends};
use ui::progress::run_search_with_progress;
use ui::tui::{run_tui, AppState};

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .expect("Usage: fi <search term>");

    // Load available backends (dnf, flatpak, cargo, etc.)
    let backends = get_available_backends();

    // Perform async search on all available backends
    let results = run_search_with_progress(&query, &backends).await;

    // Group results by backend for the TUI
    let mut sections: Vec<(String, Vec<_>)> = Vec::new();

    for backend in &backends {
        let name = backend.name().to_string();

        let items: Vec<_> = results
            .iter()
            .filter(|r| r.backend == name)
            .cloned()
            .collect();

        sections.push((name, items));
    }

    // Create the application state
    let mut app = AppState::new(sections);

    // Run the ratatui interface
    if let Err(err) = run_tui(&mut app) {
        eprintln!("Error running TUI: {:?}", err);
        return;
    }

    // If the user selected a package, install it
    if let Some(selected) = app.selected_result {
        for backend in backends {
            if backend.name() == selected.backend {
                println!(
                    "Installing {} via {}...",
                    selected.name, backend.name()
                );

                match backend.install(&selected).await {
                    Ok(_) => println!("Installation complete."),
                    Err(err) => println!("Installation failed: {}", err),
                }
                return;
            }
        }

        println!("Error: backend not found.");
    }
}

