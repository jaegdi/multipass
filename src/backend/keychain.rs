use super::{Backend, Entry, SearchOptions};
use anyhow::{anyhow, Context, Result};
use std::process::Command;

pub struct KeychainBackend;

impl KeychainBackend {
    pub fn new() -> Result<Self> {
        // Verify we're on macOS
        #[cfg(not(target_os = "macos"))]
        {
            return Err(anyhow!("Keychain backend is only available on macOS"));
        }

        // Verify security command is available
        #[cfg(target_os = "macos")]
        {
            Command::new("security")
                .arg("--help")
                .output()
                .context("Failed to execute 'security' command. Is it installed?")?;
        }

        Ok(Self)
    }

    /// Search keychain for items matching the query
    #[cfg(target_os = "macos")]
    fn search_keychain(&self, query: &str, options: &SearchOptions) -> Result<Vec<Entry>> {
        let output = Command::new("security")
            .arg("find-generic-password")
            .arg("-g") // Display password
            .arg("-s") // Service name
            .arg(query)
            .output()
            .context("Failed to search keychain")?;

        let mut results = Vec::new();

        // Parse security command output
        // Note: security outputs to stderr for password data
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // If we found something, parse it
        if output.status.success() || !stderr.is_empty() {
            let mut entry = Entry::new(query.to_string(), format!("/keychain/{}", query));

            // Parse account (username)
            if let Some(account_line) = stdout.lines().find(|l| l.contains("\"acct\"")) {
                if let Some(account) = Self::extract_value(account_line) {
                    entry.username = Some(account);
                }
            }

            // Parse password from stderr
            if let Some(password_line) = stderr.lines().find(|l| l.contains("password:")) {
                if let Some(password) = password_line.split("password:").nth(1) {
                    entry.password = Some(password.trim().trim_matches('"').to_string());
                }
            }

            // Parse service name (title)
            if let Some(service_line) = stdout.lines().find(|l| l.contains("\"svce\"")) {
                if let Some(service) = Self::extract_value(service_line) {
                    entry.title = service;
                }
            }

            results.push(entry);
        }

        Ok(results)
    }

    /// Extract value from security command output line
    fn extract_value(line: &str) -> Option<String> {
        // Format is typically: "acct"<blob>="value"
        if let Some(equals_pos) = line.find('=') {
            let value_part = &line[equals_pos + 1..];
            let trimmed = value_part.trim().trim_matches('"');
            return Some(trimmed.to_string());
        }
        None
    }

    /// List all keychain items (for non-exact searches)
    #[cfg(target_os = "macos")]
    fn list_all_items(&self) -> Result<Vec<String>> {
        let output = Command::new("security")
            .arg("dump-keychain")
            .output()
            .context("Failed to list keychain items")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            if line.contains("\"svce\"") {
                if let Some(service) = Self::extract_value(line) {
                    items.push(service);
                }
            }
        }

        Ok(items)
    }

    /// Check if a value matches the query
    fn matches(&self, value: &str, pattern: &str, options: &SearchOptions) -> bool {
        if options.case_sensitive {
            if options.exact_match {
                value == pattern
            } else {
                value.contains(pattern)
            }
        } else {
            let value_lower = value.to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            if options.exact_match {
                value_lower == pattern_lower
            } else {
                value_lower.contains(&pattern_lower)
            }
        }
    }
}

impl Backend for KeychainBackend {
    #[cfg(target_os = "macos")]
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<Entry>> {
        if options.exact_match {
            // Direct search for exact match
            self.search_keychain(query, options)
        } else {
            // List all items and filter
            let all_items = self.list_all_items()?;
            let mut results = Vec::new();

            for item in all_items {
                if self.matches(&item, query, options) {
                    // Fetch the full entry
                    if let Ok(mut entries) = self.search_keychain(&item, options) {
                        results.append(&mut entries);
                    }
                }
            }

            Ok(results)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn search(&self, _query: &str, _options: &SearchOptions) -> Result<Vec<Entry>> {
        Err(anyhow!("Keychain backend is only available on macOS"))
    }

    fn get_field(&self, entry: &Entry, field_name: &str) -> Result<String> {
        entry
            .get_field(field_name)
            .ok_or_else(|| anyhow!("Field '{}' not found", field_name))
    }
}
