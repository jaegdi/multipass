use anyhow::Result;
use std::collections::HashMap;

pub mod keepass;

#[cfg(target_os = "macos")]
pub mod keychain;

pub mod bitwarden;

/// Unified entry representation across all backends
#[derive(Debug, Clone)]
pub struct Entry {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub custom_fields: HashMap<String, String>,
    pub path: String,
}

impl Entry {
    pub fn new(title: String, path: String) -> Self {
        Self {
            title,
            username: None,
            password: None,
            url: None,
            notes: None,
            custom_fields: HashMap::new(),
            path,
        }
    }

    /// Get a field value by name (case-insensitive)
    pub fn get_field(&self, field_name: &str) -> Option<String> {
        match field_name.to_lowercase().as_str() {
            "title" => Some(self.title.clone()),
            "username" => self.username.clone(),
            "password" => self.password.clone(),
            "url" => self.url.clone(),
            "notes" => self.notes.clone(),
            _ => self.custom_fields.get(field_name).cloned(),
        }
    }
}

/// Search options for querying backends
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub exact_match: bool,
}

/// Backend trait that all password database backends must implement
pub trait Backend {
    /// Search for entries matching the query
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<Entry>>;

    /// Get a specific field value from an entry
    fn get_field(&self, entry: &Entry, field_name: &str) -> Result<String>;
}

/// Backend type enum for selection
#[derive(Debug, Clone, PartialEq)]
pub enum BackendType {
    KeePass,
    Keychain,
    Bitwarden,
}

impl BackendType {
    /// Detect backend type from database path
    pub fn from_path(path: &str) -> Self {
        match path.to_lowercase().as_str() {
            "keychain" => BackendType::Keychain,
            "bitwarden" => BackendType::Bitwarden,
            _ => BackendType::KeePass,
        }
    }
}
