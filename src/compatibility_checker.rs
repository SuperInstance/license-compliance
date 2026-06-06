use crate::dependency_scanner::DependencyLicense;
use crate::license_db::{LicenseDb, LicenseType};
use crate::spdx_parser::SpdxParser;

/// Compatibility status for a single dependency.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum CompatibilityStatus {
    Compatible,
    Incompatible { reason: String },
    Unknown,
}

/// Result of checking a single dependency.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyCheck {
    pub dep: DependencyLicense,
    pub status: CompatibilityStatus,
    pub license_type: Option<LicenseType>,
    pub requires_attribution: bool,
    pub notes: Vec<String>,
}

/// Checks if dependency licenses are compatible with the project license.
pub struct CompatibilityChecker {
    db: LicenseDb,
    spdx: SpdxParser,
}

impl CompatibilityChecker {
    pub fn new() -> Self {
        Self {
            db: LicenseDb::new(),
            spdx: SpdxParser::new(),
        }
    }

    /// Check all dependencies against the project's license.
    pub fn check_all(
        &self,
        deps: &[DependencyLicense],
        project_license: &str,
    ) -> Vec<DependencyCheck> {
        deps.iter()
            .map(|dep| self.check_one(dep, project_license))
            .collect()
    }

    /// Check a single dependency against the project's license.
    pub fn check_one(&self, dep: &DependencyLicense, project_license: &str) -> DependencyCheck {
        let mut notes = Vec::new();
        let mut requires_attribution = false;
        let mut license_type = None;

        let Some(expr) = &dep.license_expression else {
            return DependencyCheck {
                dep: dep.clone(),
                status: CompatibilityStatus::Unknown,
                license_type: None,
                requires_attribution: false,
                notes: vec!["No license information found".into()],
            };
        };

        let parsed = self.spdx.parse(expr);
        let license_ids = parsed.all_license_ids();

        // Determine overall compatibility
        let mut any_incompatible = false;
        let mut incompatible_reasons = Vec::new();

        for id in &license_ids {
            if let Some(info) = self.db.lookup(id) {
                license_type = Some(info.license_type.clone());
                if info.requires_attribution {
                    requires_attribution = true;
                }

                // Check compatibility based on project license
                let compatible = match project_license {
                    "MIT" => info.compatible_with_mit,
                    "Apache-2.0" => info.compatible_with_apache2,
                    _ => {
                        // For dual-licensed projects (MIT OR Apache-2.0), both must be compatible
                        info.compatible_with_mit && info.compatible_with_apache2
                    }
                };

                if !compatible {
                    any_incompatible = true;
                    incompatible_reasons.push(format!(
                        "{id} ({}) is not compatible with {project_license}",
                        info.license_type
                    ));
                }

                if info.requires_source_disclosure {
                    notes.push(format!("{id} requires source code disclosure"));
                }
                if info.requires_license_notice {
                    notes.push(format!("{id} requires retaining the license notice"));
                }
            } else {
                notes.push(format!("Unknown license identifier: {id}"));
            }
        }

        // For OR expressions, any branch being compatible is sufficient
        let status = if parsed.has_or() {
            // With OR, if any option is compatible, it's fine
            let any_compatible = self.check_or_compatibility(&parsed, project_license);
            if any_compatible {
                // Still check attribution requirements from all branches
                for id in &license_ids {
                    if let Some(info) = self.db.lookup(id) {
                        if info.requires_attribution {
                            requires_attribution = true;
                        }
                    }
                }
                CompatibilityStatus::Compatible
            } else if any_incompatible {
                CompatibilityStatus::Incompatible {
                    reason: incompatible_reasons.join("; "),
                }
            } else {
                CompatibilityStatus::Unknown
            }
        } else if any_incompatible {
            CompatibilityStatus::Incompatible {
                reason: incompatible_reasons.join("; "),
            }
        } else if license_ids.is_empty() {
            CompatibilityStatus::Unknown
        } else {
            CompatibilityStatus::Compatible
        };

        DependencyCheck {
            dep: dep.clone(),
            status,
            license_type,
            requires_attribution,
            notes,
        }
    }

    /// Check if any branch of an OR expression is compatible.
    fn check_or_compatibility(&self, parsed: &crate::spdx_parser::SpdxExpr, project_license: &str) -> bool {
        for id in parsed.all_license_ids() {
            if let Some(info) = self.db.lookup(&id) {
                let compatible = match project_license {
                    "MIT" => info.compatible_with_mit,
                    "Apache-2.0" => info.compatible_with_apache2,
                    _ => info.compatible_with_mit && info.compatible_with_apache2,
                };
                if compatible {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two specific licenses are compatible.
    pub fn are_compatible(&self, license_a: &str, license_b: &str) -> bool {
        let info_b = match self.db.lookup(license_b) {
            Some(info) => info,
            None => return false,
        };

        match license_a {
            "MIT" => info_b.compatible_with_mit,
            "Apache-2.0" => info_b.compatible_with_apache2,
            _ => info_b.compatible_with_mit && info_b.compatible_with_apache2,
        }
    }
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}
