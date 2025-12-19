use super::{Backend, Entry, SearchOptions};
use anyhow::{anyhow, Context, Result};
use keepass::{Database, DatabaseKey};
use std::fs::File;
use std::path::Path;

pub struct KeePassBackend {
    db: Database,
}

impl KeePassBackend {
    /// Open a KeePass database from a file path
    pub fn new(path: &str, password: &str) -> Result<Self> {
        let path = Path::new(path);
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open database file: {:?}", path))?;

        let key = DatabaseKey::new().with_password(password);
        let db = Database::open(&mut file, key)
            .with_context(|| "Failed to open KeePass database. Check password or keyfile.")?;

        Ok(Self { db })
    }

    /// Convert keepass::db::Entry to our unified Entry
    fn convert_entry(&self, entry: &keepass::db::Entry, path: String) -> Entry {
        let mut unified = Entry::new(entry.get_title().unwrap_or("").to_string(), path);

        unified.username = entry.get_username().map(|s| s.to_string());
        unified.password = entry.get_password().map(|s| s.to_string());
        unified.url = entry.get_url().map(|s| s.to_string());
        unified.notes = entry.get("Notes").map(|s| s.to_string());

        // Add all custom fields
        for key in entry.fields.keys() {
            // Skip standard fields
            if !matches!(
                key.as_str(),
                "Title" | "UserName" | "Password" | "URL" | "Notes"
            ) {
                if let Some(value) = entry.get(key) {
                    unified.custom_fields.insert(key.clone(), value.to_string());
                }
            }
        }

        unified
    }

    /// Recursively search through groups
    fn search_recursive(
        &self,
        group: &keepass::db::Group,
        current_path: &str,
        query: &str,
        options: &SearchOptions,
        results: &mut Vec<Entry>,
    ) {
        let group_name = &group.name;

        let group_path = if current_path.is_empty() {
            if group_name == "Root" || group_name.is_empty() {
                String::new()
            } else {
                group_name.to_string()
            }
        } else {
            format!("{}/{}", current_path, group_name)
        };

        for entry in group.entries() {
            let title = entry.get_title().unwrap_or("");
            if self.matches(title, query, options) {
                let full_path = if group_path.is_empty() {
                    title.to_string()
                } else {
                    format!("{}/{}", group_path, title)
                };

                results.push(self.convert_entry(entry, format!("/{}", full_path)));
            }
        }

        for child_group in group.groups() {
            self.search_recursive(child_group, &group_path, query, options, results);
        }
    }

    /// Check if a value matches the query based on search options
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

impl Backend for KeePassBackend {
    fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<Entry>> {
        let mut results = Vec::new();
        let root = &self.db.root;
        self.search_recursive(root, "", query, options, &mut results);
        Ok(results)
    }

    fn get_field(&self, entry: &Entry, field_name: &str) -> Result<String> {
        entry
            .get_field(field_name)
            .ok_or_else(|| anyhow!("Field '{}' not found", field_name))
    }
}
