use super::{Backend, Entry, SearchOptions};
use anyhow::{anyhow, Context, Result};
use std::process::Command;

pub struct BitwardenBackend {
    session: Option<String>,
}

impl BitwardenBackend {
    pub fn new() -> Result<Self> {
        // Verify bw CLI is installed
        Command::new("bw")
            .arg("--version")
            .output()
            .context("Failed to execute 'bw' command. Is Bitwarden CLI installed?")?;

        // Check if already logged in
        let status_output = Command::new("bw")
            .arg("status")
            .output()
            .context("Failed to check Bitwarden status")?;

        let status_str = String::from_utf8_lossy(&status_output.stdout);

        // Check if unlocked
        let session = if status_str.contains("\"status\":\"unlocked\"") {
            // Already unlocked, get session from environment
            std::env::var("BW_SESSION").ok()
        } else {
            None
        };

        Ok(Self { session })
    }

    /// Execute bw command with session if available
    fn execute_bw(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("bw");

        // Add session if we have one
        if let Some(session) = &self.session {
            cmd.arg("--session").arg(session);
        }

        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output().context("Failed to execute bw command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Bitwarden command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse Bitwarden JSON item to Entry
    fn parse_item(&self, json_str: &str) -> Result<Entry> {
        // Simple JSON parsing (in production, use serde_json)
        // For now, we'll do basic string parsing

        let mut entry = Entry::new(String::new(), String::new());

        // Extract name/title
        if let Some(name_start) = json_str.find("\"name\":\"") {
            let name_offset = name_start + 8;
            if let Some(name_end) = json_str[name_offset..].find('"') {
                entry.title = json_str[name_offset..name_offset + name_end].to_string();
            }
        }

        // Extract username
        if let Some(user_start) = json_str.find("\"username\":\"") {
            let user_offset = user_start + 12;
            if let Some(user_end) = json_str[user_offset..].find('"') {
                entry.username = Some(json_str[user_offset..user_offset + user_end].to_string());
            }
        }

        // Extract password
        if let Some(pass_start) = json_str.find("\"password\":\"") {
            let pass_offset = pass_start + 12;
            if let Some(pass_end) = json_str[pass_offset..].find('"') {
                entry.password = Some(json_str[pass_offset..pass_offset + pass_end].to_string());
            }
        }

        // Extract URL
        if let Some(url_start) = json_str.find("\"uri\":\"") {
            let url_offset = url_start + 7;
            if let Some(url_end) = json_str[url_offset..].find('"') {
                entry.url = Some(json_str[url_offset..url_offset + url_end].to_string());
            }
        }

        // Extract notes
        if let Some(notes_start) = json_str.find("\"notes\":\"") {
            let notes_offset = notes_start + 9;
            if let Some(notes_end) = json_str[notes_offset..].find('"') {
                entry.notes = Some(json_str[notes_offset..notes_offset + notes_end].to_string());
            }
        }

        // Set path
        entry.path = format!("/bitwarden/{}", entry.title);

        Ok(entry)
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

impl Backend for BitwardenBackend {
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<Entry>> {
        // Use bw list items --search query
        let output = self.execute_bw(&["list", "items", "--search", query])?;

        let mut results = Vec::new();

        // Parse JSON array (simple approach - split by objects)
        // In production, use serde_json
        if output.trim().starts_with('[') {
            // Split by },{ to get individual items
            let items_str = output.trim().trim_start_matches('[').trim_end_matches(']');

            if !items_str.is_empty() {
                for item_str in items_str.split("},{") {
                    let mut item_json = item_str.to_string();
                    if !item_json.starts_with('{') {
                        item_json = format!("{{{}", item_json);
                    }
                    if !item_json.ends_with('}') {
                        item_json = format!("{}}}", item_json);
                    }

                    if let Ok(entry) = self.parse_item(&item_json) {
                        // Apply additional filtering based on options
                        if self.matches(&entry.title, query, options) {
                            results.push(entry);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn get_field(&self, entry: &Entry, field_name: &str) -> Result<String> {
        entry
            .get_field(field_name)
            .ok_or_else(|| anyhow!("Field '{}' not found", field_name))
    }
}
