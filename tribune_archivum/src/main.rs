use clap::Parser;
use std::fs::File;
use std::io::Write;
use tribune_archivum::validate_epub;

/// EPUB validator implementing the six-phase validation architecture
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the EPUB file to validate
    #[arg()]
    epub_path: String,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Include warnings in output
    #[arg(long, default_value_t = false)]
    include_warnings: bool,
}

fn main() {
    let args = Args::parse();
    
    // Validate the EPUB
    let result = validate_epub(&args.epub_path);
    
    // Prepare output
    let output = print_text_output(&result, args.include_warnings);
    
    // Write to file or stdout
    if let Some(path) = args.output {
        let mut file = File::create(&path).expect("Failed to create output file");
        file.write_all(output.as_bytes()).expect("Failed to write to output file");
    } else {
        println!("{}", output);
    }
    
    // Exit with appropriate code
    if result.passed {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

/// Formats validation result as text
fn print_text_output(result: &tribune_archivum::core::ValidationResult, include_warnings: bool) -> String {
    let mut output = String::new();
    
    if result.passed {
        output.push_str("✓ EPUB validation passed\n");
    } else {
        output.push_str("✗ EPUB validation failed\n");
        output.push_str(&format!("  {} errors, {} warnings\n", result.error_count(), result.warning_count()));
        output.push('\n');
    }
    
    if !include_warnings && result.warning_count() > 0 {
        output.push_str("Warnings (use --include-warnings to show):\n");
        for warning in &result.warnings {
            output.push_str(&format!("  ⚠ [{}] {}: {}\n", 
                warning.code, 
                warning.location, 
                warning.message
            ));
        }
        output.push('\n');
    }
    
    if !result.errors.is_empty() {
        output.push_str("Errors:\n");
        for error in &result.errors {
            output.push_str(&format!("  ✗ [{}] {}: {}\n", 
                error.code, 
                error.location, 
                error.message
            ));
        }
        output.push('\n');
    }
    
    output
}


