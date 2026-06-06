use crate::compatibility_checker::{CompatibilityChecker, CompatibilityStatus};
use crate::dependency_scanner::DependencyLicense;
use crate::license_db::LicenseDb;
use serde::Serialize;
use std::io::Write;

/// A single entry in the compliance report.
#[derive(Debug, Serialize)]
pub struct ReportEntry {
    pub name: String,
    pub version: String,
    pub license: String,
    pub compatibility: String,
    pub license_type: String,
    pub requires_attribution: bool,
    pub notes: Vec<String>,
}

/// Summary statistics for the report.
#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub total_dependencies: usize,
    pub compatible: usize,
    pub incompatible: usize,
    pub unknown: usize,
    pub attribution_required: Vec<String>,
}

/// The full compliance report.
#[derive(Debug, Serialize)]
pub struct ComplianceReport {
    pub project_license: String,
    pub entries: Vec<ReportEntry>,
    pub summary: ReportSummary,
}

/// Generates license compliance reports.
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate a compliance report from dependency checks.
    pub fn generate(
        project_license: &str,
        checks: &[crate::compatibility_checker::DependencyCheck],
    ) -> ComplianceReport {
        let db = LicenseDb::new();
        let mut entries = Vec::new();
        let mut compatible = 0;
        let mut incompatible = 0;
        let mut unknown = 0;
        let mut attribution_required = Vec::new();

        for check in checks {
            let (compat_str, compat_count) = match &check.status {
                CompatibilityStatus::Compatible => ("✅ Compatible".into(), true),
                CompatibilityStatus::Incompatible { reason } => {
                    (format!("❌ Incompatible: {reason}"), false)
                }
                CompatibilityStatus::Unknown => ("⚠️ Unknown".into(), false),
            };

            match &check.status {
                CompatibilityStatus::Compatible => compatible += 1,
                CompatibilityStatus::Incompatible { .. } => incompatible += 1,
                CompatibilityStatus::Unknown => unknown += 1,
            }

            let lic_type = check
                .license_type
                .as_ref()
                .map(|lt| lt.to_string())
                .unwrap_or_else(|| "Unknown".into());

            if check.requires_attribution {
                attribution_required.push(check.dep.name.clone());
            }

            entries.push(ReportEntry {
                name: check.dep.name.clone(),
                version: check.dep.version.clone(),
                license: check
                    .dep
                    .license_expression
                    .clone()
                    .unwrap_or_else(|| "Unknown".into()),
                compatibility: compat_str,
                license_type: lic_type,
                requires_attribution: check.requires_attribution,
                notes: check.notes.clone(),
            });
        }

        ComplianceReport {
            project_license: project_license.to_string(),
            entries,
            summary: ReportSummary {
                total_dependencies: checks.len(),
                compatible,
                incompatible,
                unknown,
                attribution_required,
            },
        }
    }

    /// Format report as a human-readable text table.
    pub fn format_text(report: &ComplianceReport) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "License Compliance Report\n\
             ========================\n\
             Project License: {}\n\n",
            report.project_license
        ));

        out.push_str("Dependencies:\n");
        out.push_str(&format!(
            "{:<30} {:<12} {:<35} {:<15} {}\n",
            "CRATE", "VERSION", "LICENSE", "TYPE", "STATUS"
        ));
        out.push_str(&"-".repeat(110));
        out.push('\n');

        for entry in &report.entries {
            out.push_str(&format!(
                "{:<30} {:<12} {:<35} {:<15} {}\n",
                truncate(&entry.name, 30),
                truncate(&entry.version, 12),
                truncate(&entry.license, 35),
                truncate(&entry.license_type, 15),
                truncate(&entry.compatibility, 30),
            ));
            for note in &entry.notes {
                out.push_str(&format!("  → {note}\n"));
            }
        }

        out.push('\n');
        out.push_str("Summary:\n");
        out.push_str(&format!("  Total dependencies:  {}\n", report.summary.total_dependencies));
        out.push_str(&format!("  Compatible:          {} ✅\n", report.summary.compatible));
        out.push_str(&format!("  Incompatible:        {} ❌\n", report.summary.incompatible));
        out.push_str(&format!("  Unknown:             {} ⚠️\n", report.summary.unknown));

        if !report.summary.attribution_required.is_empty() {
            out.push_str("\nAttribution Required:\n");
            for name in &report.summary.attribution_required {
                out.push_str(&format!("  • {name}\n"));
            }
        }

        out
    }

    /// Format report as JSON.
    pub fn format_json(report: &ComplianceReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    /// Write the report to a file.
    pub fn write_report(report: &ComplianceReport, path: &std::path::Path, json: bool) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        if json {
            let content = Self::format_json(report)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            write!(file, "{content}")?;
        } else {
            write!(file, "{}", Self::format_text(report))?;
        }
        Ok(())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compatibility_checker::{CompatibilityStatus, DependencyCheck};
    use crate::dependency_scanner::DependencyLicense;
    use crate::license_db::LicenseType;

    fn make_dep(name: &str, license: Option<&str>) -> DependencyLicense {
        DependencyLicense {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            license_expression: license.map(|s| s.to_string()),
            license_ids: vec![],
            unknown_licenses: vec![],
        }
    }

    fn make_check(dep: DependencyLicense, status: CompatibilityStatus) -> DependencyCheck {
        DependencyCheck {
            dep,
            status,
            license_type: Some(LicenseType::Permissive),
            requires_attribution: true,
            notes: vec![],
        }
    }

    #[test]
    fn test_generate_report() {
        let checks = vec![
            make_check(
                make_dep("serde", Some("MIT OR Apache-2.0")),
                CompatibilityStatus::Compatible,
            ),
            make_check(
                make_dep("gpl-crate", Some("GPL-3.0")),
                CompatibilityStatus::Incompatible {
                    reason: "GPL-3.0 (Copyleft) is not compatible with MIT".into(),
                },
            ),
        ];

        let report = ReportGenerator::generate("MIT", &checks);
        assert_eq!(report.summary.total_dependencies, 2);
        assert_eq!(report.summary.compatible, 1);
        assert_eq!(report.summary.incompatible, 1);
    }

    #[test]
    fn test_format_text() {
        let checks = vec![make_check(
            make_dep("serde", Some("MIT OR Apache-2.0")),
            CompatibilityStatus::Compatible,
        )];
        let report = ReportGenerator::generate("MIT", &checks);
        let text = ReportGenerator::format_text(&report);
        assert!(text.contains("serde"));
        assert!(text.contains("Compatible"));
        assert!(text.contains("Summary"));
    }

    #[test]
    fn test_format_json() {
        let checks = vec![make_check(
            make_dep("serde", Some("MIT")),
            CompatibilityStatus::Compatible,
        )];
        let report = ReportGenerator::generate("MIT", &checks);
        let json = ReportGenerator::format_json(&report).unwrap();
        assert!(json.contains("\"serde\""));
        assert!(json.contains("Compatible"));
    }

    #[test]
    fn test_attribution_list() {
        let checks = vec![
            make_check(
                make_dep("serde", Some("MIT")),
                CompatibilityStatus::Compatible,
            ),
            make_check(
                make_dep("other", Some("MIT")),
                CompatibilityStatus::Compatible,
            ),
        ];
        let report = ReportGenerator::generate("MIT", &checks);
        assert_eq!(report.summary.attribution_required.len(), 2);
        assert!(report.summary.attribution_required.contains(&"serde".to_string()));
    }
}
