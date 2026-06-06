use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Classification of a license type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    Permissive,
    Copyleft,
    WeakCopyleft,
    Proprietary,
    PublicDomain,
    Unknown,
}

impl fmt::Display for LicenseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseType::Permissive => write!(f, "Permissive"),
            LicenseType::Copyleft => write!(f, "Copyleft"),
            LicenseType::WeakCopyleft => write!(f, "Weak Copyleft"),
            LicenseType::Proprietary => write!(f, "Proprietary"),
            LicenseType::PublicDomain => write!(f, "Public Domain"),
            LicenseType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a single license.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub id: String,
    pub full_name: String,
    pub license_type: LicenseType,
    pub compatible_with_mit: bool,
    pub compatible_with_apache2: bool,
    pub requires_attribution: bool,
    pub requires_source_disclosure: bool,
    pub requires_license_notice: bool,
}

/// Database of known open-source licenses.
pub struct LicenseDb {
    licenses: HashMap<String, LicenseInfo>,
}

impl LicenseDb {
    /// Create a new license database populated with common licenses.
    pub fn new() -> Self {
        let mut licenses = HashMap::new();

        let entries = vec![
            LicenseInfo {
                id: "MIT".into(),
                full_name: "MIT License".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "Apache-2.0".into(),
                full_name: "Apache License 2.0".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "BSD-2-Clause".into(),
                full_name: "BSD 2-Clause Simplified License".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "BSD-3-Clause".into(),
                full_name: "BSD 3-Clause New/Revised License".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "ISC".into(),
                full_name: "ISC License".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "0BSD".into(),
                full_name: "Zero-Clause BSD".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: false,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "Unlicense".into(),
                full_name: "The Unlicense".into(),
                license_type: LicenseType::PublicDomain,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: false,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "CC0-1.0".into(),
                full_name: "Creative Commons Zero v1.0 Universal".into(),
                license_type: LicenseType::PublicDomain,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: false,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "MPL-2.0".into(),
                full_name: "Mozilla Public License 2.0".into(),
                license_type: LicenseType::WeakCopyleft,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "LGPL-2.1".into(),
                full_name: "GNU Lesser General Public License v2.1".into(),
                license_type: LicenseType::WeakCopyleft,
                compatible_with_mit: false,
                compatible_with_apache2: false,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "LGPL-3.0".into(),
                full_name: "GNU Lesser General Public License v3.0".into(),
                license_type: LicenseType::WeakCopyleft,
                compatible_with_mit: false,
                compatible_with_apache2: false,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "GPL-2.0".into(),
                full_name: "GNU General Public License v2.0".into(),
                license_type: LicenseType::Copyleft,
                compatible_with_mit: false,
                compatible_with_apache2: false,
                requires_attribution: true,
                requires_source_disclosure: true,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "GPL-3.0".into(),
                full_name: "GNU General Public License v3.0".into(),
                license_type: LicenseType::Copyleft,
                compatible_with_mit: false,
                compatible_with_apache2: false,
                requires_attribution: true,
                requires_source_disclosure: true,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "AGPL-3.0".into(),
                full_name: "GNU Affero General Public License v3.0".into(),
                license_type: LicenseType::Copyleft,
                compatible_with_mit: false,
                compatible_with_apache2: false,
                requires_attribution: true,
                requires_source_disclosure: true,
                requires_license_notice: true,
            },
            LicenseInfo {
                id: "BSL-1.0".into(),
                full_name: "Boost Software License 1.0".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "Zlib".into(),
                full_name: "zlib License".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "MIT-0".into(),
                full_name: "MIT No Attribution".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: false,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
            LicenseInfo {
                id: "BlueOak-1.0.0".into(),
                full_name: "Blue Oak Model License 1.0.0".into(),
                license_type: LicenseType::Permissive,
                compatible_with_mit: true,
                compatible_with_apache2: true,
                requires_attribution: true,
                requires_source_disclosure: false,
                requires_license_notice: false,
            },
        ];

        for info in entries {
            licenses.insert(info.id.clone(), info);
        }

        Self { licenses }
    }

    /// Look up a license by its SPDX identifier (case-insensitive).
    pub fn lookup(&self, id: &str) -> Option<&LicenseInfo> {
        // Exact match first
        if let Some(info) = self.licenses.get(id) {
            return Some(info);
        }
        // Case-insensitive fallback
        self.licenses
            .values()
            .find(|info| info.id.eq_ignore_ascii_case(id))
    }

    /// Get all known licenses.
    pub fn all(&self) -> Vec<&LicenseInfo> {
        self.licenses.values().collect()
    }

    /// Check if a given license ID is known.
    pub fn is_known(&self, id: &str) -> bool {
        self.lookup(id).is_some()
    }
}

impl Default for LicenseDb {
    fn default() -> Self {
        Self::new()
    }
}
