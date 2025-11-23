use std::fmt;

/// Errors that can occur during package search and installation.
#[derive(Debug)]
pub enum FiError {
    /// Backend installation failed
    InstallationFailed(String),
    /// Backend not found for selected package
    BackendNotFound(String),
    /// TUI error
    TuiError(String),
    /// Invalid command line arguments
    InvalidArguments(String),
}

impl fmt::Display for FiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FiError::InstallationFailed(msg) => write!(f, "Installation failed: {}", msg),
            FiError::BackendNotFound(backend) => write!(f, "Backend not found: {}", backend),
            FiError::TuiError(msg) => write!(f, "TUI error: {}", msg),
            FiError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl std::error::Error for FiError {}

