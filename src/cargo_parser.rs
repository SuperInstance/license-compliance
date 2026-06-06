use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<PackageSection>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    name: Option<String>,
    version: Option<String>,
    license: Option<String>,
    authors: Option<Vec<String>>,
    description: Option<String>,
}

/// Result of parsing a Cargo.toml file.
#[derive(Debug, Clone)]
pub struct CargoInfo {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
}

/// Parses Cargo.toml files for license information.
pub struct CargoParser;

impl CargoParser {
    /// Parse a Cargo.toml file and extract license info.
    pub fn parse_file(path: &Path) -> Result<CargoInfo, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse_content(&content)
    }

    /// Parse Cargo.toml content from a string.
    pub fn parse_content(content: &str) -> Result<CargoInfo, String> {
        let cargo: CargoToml =
            toml::from_str(content).map_err(|e| format!("Failed to parse Cargo.toml: {e}"))?;

        let pkg = cargo
            .package
            .ok_or("Missing [package] section in Cargo.toml")?;

        Ok(CargoInfo {
            name: pkg.name.unwrap_or_default(),
            version: pkg.version.unwrap_or_else(|| "0.0.0".into()),
            license: pkg.license,
            authors: pkg.authors.unwrap_or_default(),
            description: pkg.description,
        })
    }

    /// Find and parse all dependency Cargo.toml files in a directory tree.
    /// Looks for Cargo.toml in subdirectories of a given path.
    pub fn scan_dependencies(root: &Path) -> Vec<(String, CargoInfo)> {
        let mut results = Vec::new();
        let cargo_lock = root.join("Cargo.lock");

        // If there's a Cargo.lock, we can use cargo metadata instead,
        // but for simplicity we scan the lockfile for crate names and versions
        if cargo_lock.exists() {
            if let Ok(lock_content) = std::fs::read_to_string(&cargo_lock) {
                results.extend(Self::parse_lockfile(&lock_content));
            }
        }

        results
    }

    /// Parse a Cargo.lock file for dependency names and versions.
    fn parse_lockfile(content: &str) -> Vec<(String, CargoInfo)> {
        let mut results = Vec::new();
        // Simple parsing: look for [[package]] entries
        let mut in_package = false;
        let mut name = String::new();
        let mut version = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "[[package]]" {
                // Save previous entry
                if !name.is_empty() {
                    results.push((
                        name.clone(),
                        CargoInfo {
                            name: name.clone(),
                            version: version.clone(),
                            license: None,
                            authors: Vec::new(),
                            description: None,
                        },
                    ));
                }
                in_package = true;
                name.clear();
                version.clear();
                continue;
            }

            if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
                // New section, save last package if any
                if in_package && !name.is_empty() {
                    results.push((
                        name.clone(),
                        CargoInfo {
                            name: name.clone(),
                            version: version.clone(),
                            license: None,
                            authors: Vec::new(),
                            description: None,
                        },
                    ));
                    name.clear();
                    version.clear();
                }
                in_package = false;
                continue;
            }

            if in_package {
                if let Some(val) = trimmed.strip_prefix("name = ") {
                    name = val.trim_matches('"').to_string();
                } else if let Some(val) = trimmed.strip_prefix("version = ") {
                    version = val.trim_matches('"').to_string();
                }
            }
        }

        // Don't forget the last entry
        if !name.is_empty() {
            results.push((
                name.clone(),
                CargoInfo {
                    name: name.clone(),
                    version: version.clone(),
                    license: None,
                    authors: Vec::new(),
                    description: None,
                },
            ));
        }

        results
    }

    /// Try to find license info for a dependency from common locations:
    /// the cargo registry cache or local path.
    pub fn find_license_for_dep(dep_name: &str) -> Option<String> {
        // Check cargo registry for license files
        let registry_dir = dirs_cargo_registry();
        if let Some(dir) = registry_dir {
            // Look for the crate directory
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(dep_name) {
                        let manifest = entry.path().join("Cargo.toml");
                        if manifest.exists() {
                            if let Ok(info) = Self::parse_file(&manifest) {
                                if info.name == dep_name && info.license.is_some() {
                                    return info.license;
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// Try to locate the cargo registry cache directory.
fn dirs_cargo_registry() -> Option<std::path::PathBuf> {
    let home = std::env::var("CARGO_HOME")
        .ok()
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.cargo")))?;
    let path = std::path::PathBuf::from(home).join("registry/src");
    if path.exists() {
        // Find the first subdirectory (usually the index hash)
        if let Ok(mut entries) = std::fs::read_dir(&path) {
            if let Some(entry) = entries.next().and_then(|e| e.ok()) {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Known license mappings for common crates (fallback when registry isn't available).
pub static KNOWN_CRATE_LICENSES: &[(&str, &str)] = &[
    ("serde", "MIT OR Apache-2.0"),
    ("serde_json", "MIT OR Apache-2.0"),
    ("serde_derive", "MIT OR Apache-2.0"),
    ("toml", "MIT OR Apache-2.0"),
    ("clap", "MIT OR Apache-2.0"),
    ("walkdir", "Unlicense OR MIT"),
    ("unicode-ident", "MIT OR Apache-2.0"),
    ("proc-macro2", "MIT OR Apache-2.0"),
    ("quote", "MIT OR Apache-2.0"),
    ("syn", "MIT OR Apache-2.0"),
    ("libc", "MIT OR Apache-2.0"),
    ("log", "MIT OR Apache-2.0"),
    ("memchr", "Unlicense OR MIT"),
    ("aho-corasick", "Unlicense OR MIT"),
    ("regex", "MIT OR Apache-2.0"),
    ("regex-syntax", "MIT OR Apache-2.0"),
    ("regex-automata", "MIT OR Apache-2.0"),
    ("thiserror", "MIT OR Apache-2.0"),
    ("anyhow", "MIT OR Apache-2.0"),
    ("once_cell", "MIT OR Apache-2.0"),
    ("cfg-if", "MIT OR Apache-2.0"),
    ("bitflags", "MIT OR Apache-2.0"),
    ("rustc-hash", "Apache-2.0"),
    ("indexmap", "Apache-2.0"),
    ("hashbrown", "MIT OR Apache-2.0"),
    ("ryu", "Apache-2.0 OR BSL-1.0"),
    ("itoa", "MIT OR Apache-2.0"),
];

/// Look up a known crate's license from the static mapping.
pub fn lookup_known_crate_license(name: &str) -> Option<&'static str> {
    KNOWN_CRATE_LICENSES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, l)| *l)
}
