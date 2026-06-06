mod license_db;
mod cargo_parser;
mod dependency_scanner;
mod compatibility_checker;
mod spdx_parser;
mod report_generator;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use license_db::LicenseDb;
use cargo_parser::CargoParser;
use dependency_scanner::DependencyScanner;
use compatibility_checker::CompatibilityChecker;
use report_generator::ReportGenerator;

#[derive(Parser)]
#[command(name = "license-compliance")]
#[command(about = "Check open-source license compatibility across dependency trees")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a project's dependencies for license compatibility
    Check {
        /// Path to the project directory (containing Cargo.toml)
        #[arg(default_value = ".")]
        project_path: PathBuf,

        /// Project license (SPDX identifier)
        #[arg(short, long, default_value = "MIT OR Apache-2.0")]
        license: String,

        /// Output format: text or json
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Write report to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List all known licenses in the database
    ListLicenses {
        /// Filter by license type (permissive, copyleft, weak-copyleft, public-domain)
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Look up a specific license by SPDX ID
    Lookup {
        /// SPDX license identifier
        license_id: String,
    },
    /// Show what a given SPDX expression parses to
    ParseExpr {
        /// SPDX license expression
        expression: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            project_path,
            license,
            format,
            output,
        } => {
            let cargo_path = project_path.join("Cargo.toml");
            if !cargo_path.exists() {
                eprintln!("Error: No Cargo.toml found in {}", project_path.display());
                std::process::exit(1);
            }

            // Parse project Cargo.toml
            let cargo_info = match CargoParser::parse_file(&cargo_path) {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error parsing Cargo.toml: {e}");
                    std::process::exit(1);
                }
            };

            eprintln!("Scanning {} ({})...", cargo_info.name, license);

            // Scan dependencies
            let scanner = DependencyScanner::new();
            let deps = scanner.scan(&project_path);

            if deps.is_empty() {
                eprintln!("No dependencies found.");
                return;
            }

            // Check compatibility
            let checker = CompatibilityChecker::new();
            let checks = checker.check_all(&deps, &license);

            // Generate report
            let report = ReportGenerator::generate(&license, &checks);

            let is_json = format == "json";

            if let Some(path) = output {
                match ReportGenerator::write_report(&report, &path, is_json) {
                    Ok(()) => eprintln!("Report written to {}", path.display()),
                    Err(e) => {
                        eprintln!("Error writing report: {e}");
                        std::process::exit(1);
                    }
                }
            } else if is_json {
                match ReportGenerator::format_json(&report) {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        eprintln!("Error generating JSON: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                print!("{}", ReportGenerator::format_text(&report));
            }

            // Exit with error if any incompatibilities found
            if report.summary.incompatible > 0 {
                std::process::exit(2);
            }
        }
        Commands::ListLicenses { filter } => {
            let db = LicenseDb::new();
            let licenses = db.all();

            println!("Known Licenses ({total}):\n", total = licenses.len());
            println!(
                "{:<20} {:<40} {:<15} {:<10} {:<10} {:<12}",
                "ID", "NAME", "TYPE", "MIT OK", "AP2 OK", "ATTRIB"
            );
            println!("{}", "-".repeat(110));

            for lic in licenses {
                if let Some(ref f) = filter {
                    let f_lower = f.to_lowercase();
                    let type_str = lic.license_type.to_string().to_lowercase().replace(' ', "-");
                    if !type_str.contains(&f_lower) && !lic.id.to_lowercase().contains(&f_lower) {
                        continue;
                    }
                }

                println!(
                    "{:<20} {:<40} {:<15} {:<10} {:<10} {:<12}",
                    lic.id,
                    truncate_str(&lic.full_name, 40),
                    lic.license_type,
                    if lic.compatible_with_mit { "✅" } else { "❌" },
                    if lic.compatible_with_apache2 { "✅" } else { "❌" },
                    if lic.requires_attribution { "yes" } else { "no" },
                );
            }
        }
        Commands::Lookup { license_id } => {
            let db = LicenseDb::new();
            match db.lookup(&license_id) {
                Some(lic) => {
                    println!("License: {}", lic.full_name);
                    println!("  ID:                    {}", lic.id);
                    println!("  Type:                  {}", lic.license_type);
                    println!("  Compatible with MIT:   {}", if lic.compatible_with_mit { "Yes" } else { "No" });
                    println!("  Compatible w/ Apache2: {}", if lic.compatible_with_apache2 { "Yes" } else { "No" });
                    println!("  Requires attribution:  {}", if lic.requires_attribution { "Yes" } else { "No" });
                    println!("  Source disclosure:      {}", if lic.requires_source_disclosure { "Yes" } else { "No" });
                    println!("  License notice:        {}", if lic.requires_license_notice { "Yes" } else { "No" });
                }
                None => {
                    eprintln!("Unknown license: {license_id}");
                    std::process::exit(1);
                }
            }
        }
        Commands::ParseExpr { expression } => {
            let parser = spdx_parser::SpdxParser::new();
            let parsed = parser.parse(&expression);
            println!("Expression: {expression}");
            println!("Parsed:     {parsed:?}");
            println!("License IDs: {:?}", parsed.all_license_ids());
            println!("Has OR:     {}", parsed.has_or());
        }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
