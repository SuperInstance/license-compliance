use crate::cargo_parser::{self, CargoInfo};
use crate::license_db::LicenseDb;
use crate::spdx_parser::SpdxParser;
use std::path::Path;

/// Information about a single dependency's license status.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyLicense {
    pub name: String,
    pub version: String,
    pub license_expression: Option<String>,
    pub license_ids: Vec<String>,
    pub unknown_licenses: Vec<String>,
}

/// Scans a project's dependency tree for license information.
pub struct DependencyScanner {
    db: LicenseDb,
    spdx: SpdxParser,
}

impl DependencyScanner {
    pub fn new() -> Self {
        Self {
            db: LicenseDb::new(),
            spdx: SpdxParser::new(),
        }
    }

    /// Scan a project directory for all dependency license info.
    pub fn scan(&self, project_path: &Path) -> Vec<DependencyLicense> {
        let mut results = Vec::new();

        // Get dependencies from Cargo.lock
        let deps = cargo_parser::CargoParser::scan_dependencies(project_path);

        for (name, info) in deps {
            let license_expr = self.resolve_license(&name, &info);
            let (ids, unknowns) = self.resolve_license_ids(&license_expr);

            results.push(DependencyLicense {
                name,
                version: info.version,
                license_expression: license_expr,
                license_ids: ids,
                unknown_licenses: unknowns,
            });
        }

        results
    }

    /// Resolve the license expression for a dependency.
    fn resolve_license(&self, name: &str, info: &CargoInfo) -> Option<String> {
        // First check the Cargo.toml info
        if info.license.is_some() {
            return info.license.clone();
        }

        // Check known crates mapping
        if let Some(lic) = cargo_parser::lookup_known_crate_license(name) {
            return Some(lic.to_string());
        }

        // Try the registry
        cargo_parser::CargoParser::find_license_for_dep(name)
    }

    /// Parse a license expression into known and unknown license IDs.
    fn resolve_license_ids(&self, expr: &Option<String>) -> (Vec<String>, Vec<String>) {
        let Some(expr) = expr else {
            return (vec![], vec![]);
        };

        let parsed = self.spdx.parse(expr);
        let mut known = Vec::new();
        let mut unknown = Vec::new();

        for id in parsed.all_license_ids() {
            if self.db.is_known(&id) {
                if !known.contains(&id) {
                    known.push(id);
                }
            } else {
                if !unknown.contains(&id) {
                    unknown.push(id);
                }
            }
        }

        (known, unknown)
    }
}

impl Default for DependencyScanner {
    fn default() -> Self {
        Self::new()
    }
}
