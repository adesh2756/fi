use dialoguer::Select;
use crate::models::result::SearchResult;

pub async fn display_results(results: Vec<SearchResult>) -> Option<SearchResult> {
    if results.is_empty() {
        println!("No results found.");
        return None;
    }

    let items: Vec<String> = results
        .iter()
        .map(|r| format!("[{}] {} - {}", r.backend, r.name, r.description))
        .collect();

    let selection = Select::new()
        .with_prompt("Choose a package to install")
        .items(&items)
        .interact()
        .unwrap();

    Some(results[selection].clone())
}

