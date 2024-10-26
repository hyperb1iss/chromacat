use chromacat::cli::Cli;
use chromacat::ChromaCat;
use clap::Parser;
use env_logger;
use std::process;

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle --list flag
    if cli.list_available {
        Cli::print_available_options();
        process::exit(0);
    }

    // Create and run ChromaCat
    let mut cat = ChromaCat::new(cli);
    if let Err(e) = cat.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
