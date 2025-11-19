mod backends;
mod models;
mod ui;
mod utils;

use backends::{Backend, get_available_backends};
use ui::progress::run_search_with_progress;
use ui::display::display_results;

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .expect("Usage: fi <search term>");

    // Load available backends (dnf, flatpak, cargo)
    let backends = get_available_backends();

    // Run async search
    let results = run_search_with_progress(&query, &backends).await;

    // Ask user which to install
    if let Some(selected) = display_results(results).await {

        // Find matching backend object
        for backend in backends {
            if backend.name() == selected.backend {
                println!("Installing {} via {}...", selected.name, backend.name());
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

