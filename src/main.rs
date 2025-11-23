//! # fi - Unified Package Manager CLI
//!
//! Command-line interface for the fi package manager.

use fi::run;

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .expect("Usage: fi <search term>");

    if let Err(e) = run(&query).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
