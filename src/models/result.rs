use serde::{Deserialize, Serialize};

/// Represents a package search result from any backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The name of the backend that found this package (e.g., "dnf", "flatpak").
    pub backend: String,
    /// The display name of the package.
    pub name: String,
    /// The identifier used to install the package (e.g., package name, application ID).
    pub identifier: String,
    /// A description of the package.
    pub description: String,
    /// The version of the package, if available.
    pub version: Option<String>,
}

