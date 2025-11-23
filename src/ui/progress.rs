//! Progress indicators for concurrent backend searches.

use indicatif::{MultiProgress, ProgressBar};
use std::time::Duration;
use futures::future::join_all;

use crate::backends::Backend;
use crate::models::result::SearchResult;

/// Runs searches across all backends concurrently with progress indicators.
///
/// # Arguments
///
/// * `query` - The search query string
/// * `backends` - List of backends to search
///
/// # Returns
///
/// A combined vector of all search results from all backends.
pub async fn run_search_with_progress(
    query: &str,
    backends: &Vec<Box<dyn Backend>>
) -> Vec<SearchResult> {

    let mp = MultiProgress::new();

    // Build a list of futures
    let futures = backends.iter().map(|backend| {
        let pb = mp.add(ProgressBar::new_spinner());
        pb.enable_steady_tick(Duration::from_millis(80));

        let name = backend.name().to_string();
        let q = query.to_string();

        async move {
            let results = backend.search(&q, pb.clone()).await;
            // Note: progress bar is already finished by the backend's search method
            (name, results)
        }
    });

    // Run all futures concurrently
    let results = join_all(futures).await;

    // Combine all results
    let mut combined = vec![];

    for (_, list) in results {
        combined.extend(list);
    }

    combined
}

