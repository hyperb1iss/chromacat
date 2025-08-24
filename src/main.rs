use chromacat::cli::Cli;
use chromacat::error::Result;
use chromacat::ChromaCat;
use clap::Parser;
use std::process;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    if cli.pattern_help {
        Cli::print_pattern_help();
        return Ok(());
    }

    if cli.list_available {
        Cli::print_available_options();
        return Ok(());
    }

    // Create and run ChromaCat
    let mut cat = ChromaCat::new(cli);
    if let Err(e) = cat.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}
