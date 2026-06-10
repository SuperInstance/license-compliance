# license-compliance

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![SuperInstance](https://img.shields.io/badge/part%20of-SuperInstance-purple.svg)](https://github.com/SuperInstance)

CLI tool to check open-source license compatibility across dependency trees.

## What It Does

`license-compliance` scans a Rust project's `Cargo.lock`, resolves each dependency's license from its `Cargo.toml` metadata, parses SPDX license expressions (handling `AND`, `OR`, `WITH`, `+`, and parenthesized groups), checks compatibility against your project's license, and generates a report listing compatible, incompatible, and unknown dependencies along with attribution requirements.

The conservation law **γ + η = C** applies: compatible licenses (γ) plus incompatible/unknown ones (η) sum to the total dependency count C. The goal is η = 0.

## Architecture

```
┌──────────────────────────────────────────────────┐
│                   CLI (clap)                      │
│  check / list-licenses / lookup / parse-expr     │
├──────────┬──────────────┬────────────────────────┤
│SpdxParser│  LicenseDb   │  CargoParser           │
│          │              │                        │
│ parse()  │  18 licenses │  scan_dependencies()   │
│ AND/OR/  │  MIT, Apache,│  find_license_for_dep()│
│ WITH/+   │  GPL, BSD... │  lookup_known_crate_   │
│          │              │  license()             │
├──────────┴──────────────┴────────────────────────┤
│          DependencyScanner                        │
│  scan(path) → Vec<DependencyLicense>             │
│  resolve_license() → SPDX expression             │
│  resolve_license_ids() → (known, unknown)        │
├───────────────────────────────────────────────────┤
│       CompatibilityChecker                        │
│  check_all(deps, project_license)                │
│  → Vec<DependencyCheck>                          │
│  check_one() handles:                            │
│    - OR expressions (any branch compatible)      │
│    - AND expressions (all must be compatible)    │
│    - WITH exceptions                             │
│    - Attribution/source disclosure flags         │
├───────────────────────────────────────────────────┤
│          ReportGenerator                          │
│  generate() → ComplianceReport                   │
│  format_text() → human-readable table            │
│  format_json() → structured output               │
│  write_report() → file output                    │
└───────────────────────────────────────────────────┘
```

## Installation

```bash
git clone https://github.com/SuperInstance/license-compliance.git
cd license-compliance
cargo build --release
```

## Usage

### Check a project's dependencies

```bash
# Default: assumes MIT OR Apache-2.0, text output
license-compliance check ./my-project

# Specify project license and output format
license-compliance check ./my-project --license MIT --format json --output report.json

# Check against GPL-3.0
license-compliance check ./my-project --license GPL-3.0
```

Output example:

```
License Compliance Report
========================
Project License: MIT OR Apache-2.0

Dependencies:
CRATE                          VERSION      LICENSE                TYPE            STATUS
--------------------------------------------------------------------------------------------------------------
serde                          1.0.200      MIT OR Apache-2.0      Permissive      ✅ Compatible
tokio                          1.38.0       MIT                    Permissive      ✅ Compatible
some-gpl-crate                 0.1.0        GPL-3.0                Copyleft        ❌ Incompatible

Summary:
  Total dependencies:  42
  Compatible:          40 ✅
  Incompatible:        1 ❌
  Unknown:             1 ⚠️
```

### List known licenses

```bash
# All licenses
license-compliance list-licenses

# Filter by type
license-compliance list-licenses --filter copyleft
```

### Look up a specific license

```bash
license-compliance lookup GPL-3.0
# Outputs: type (Copyleft), MIT compat (false), Apache compat (false),
#          attribution (required), source disclosure (required)
```

### Parse SPDX expressions

```bash
license-compliance parse-expr "(MIT OR Apache-2.0) AND BSD-3-Clause"
# Shows parsed tree: And(Or(MIT, Apache-2.0), BSD-3-Clause)
```

## API Reference

### `LicenseDb` — Built-in license database

18 licenses pre-loaded:

| License | Type | MIT compat | Apache compat | Attribution | Source disclosure |
|---------|------|-----------|---------------|-------------|-------------------|
| MIT | Permissive | ✅ | ✅ | Yes | No |
| Apache-2.0 | Permissive | ✅ | ✅ | Yes | Yes (notice) |
| BSD-2/3-Clause | Permissive | ✅ | ✅ | Yes | No |
| ISC | Permissive | ✅ | ✅ | Yes | No |
| MPL-2.0 | Weak Copyleft | ✅ | ✅ | Yes | Yes (notice) |
| LGPL-2.1/3.0 | Weak Copyleft | ❌ | ❌ | Yes | Yes (notice) |
| GPL-2.0/3.0 | Copyleft | ❌ | ❌ | Yes | Yes |
| AGPL-3.0 | Copyleft | ❌ | ❌ | Yes | Yes |
| Unlicense / CC0-1.0 | Public Domain | ✅ | ✅ | No | No |

### `SpdxParser` — SPDX expression parser

Parses the full SPDX expression grammar:

```rust
let parser = SpdxParser::new();

// Simple
let expr = parser.parse("MIT");
// → SpdxExpr::License { id: "MIT", plus: false }

// OR (either applies)
let expr = parser.parse("MIT OR Apache-2.0");
// → SpdxExpr::Or(MIT, Apache-2.0)

// WITH exception
let expr = parser.parse("GPL-2.0 WITH Classpath-exception-2.0");
// → SpdxExpr::WithException { license: GPL-2.0, exception: "Classpath-exception-2.0" }

// Complex nested
let expr = parser.parse("(MIT OR Apache-2.0) AND BSD-3-Clause");
expr.all_license_ids(); // → ["MIT", "Apache-2.0", "BSD-3-Clause"]
expr.has_or();          // → true
```

### `CompatibilityChecker` — License compatibility logic

```rust
let checker = CompatibilityChecker::new();
let deps = scanner.scan(project_path);
let checks = checker.check_all(&deps, "MIT");

for check in &checks {
    match &check.status {
        CompatibilityStatus::Compatible => { /* ✅ */ }
        CompatibilityStatus::Incompatible { reason } => { /* ❌: reason */ }
        CompatibilityStatus::Unknown => { /* ⚠️ */ }
    }
}
```

For `OR` expressions, any compatible branch is sufficient. For `AND` expressions, all branches must be compatible.

### `ReportGenerator` — Output formatting

```rust
let report = ReportGenerator::generate("MIT", &checks);
let text = ReportGenerator::format_text(&report);
let json = ReportGenerator::format_json(&report).unwrap();
ReportGenerator::write_report(&report, Path::new("report.txt"), false).unwrap();
```

## Supported Licenses

MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, 0BSD, MIT-0, Unlicense, CC0-1.0, MPL-2.0, LGPL-2.1, LGPL-3.0, GPL-2.0, GPL-3.0, AGPL-3.0, BSL-1.0, Zlib, BlueOak-1.0.0.

## Related Crates (SuperInstance Ecosystem)

- **meta-agent** — Multi-agent task coordination
- **ternary-fleet** — Fleet-wide license compliance via forgemaster orchestration
- **forgemaster** — GPU fleet manager, uses license-compliance for fleet audits
