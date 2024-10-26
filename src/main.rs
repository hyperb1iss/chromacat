use chromacat::cli::Cli;
use chromacat::ChromaCat;
use clap::Parser;
use std::process;

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle --list-themes flag
    if cli.list_themes {
        println!("Available themes:");
        for (name, description) in Cli::theme_descriptions() {
            println!("  {:<8} - {}", name, description);
        }
        process::exit(0);
    }

    // Create and run ChromaCat
    let cat = ChromaCat::new(cli);
    if let Err(e) = cat.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}