use chromacat::cli::Cli;
use chromacat::error::Result;
use chromacat::ChromaCat;
use clap::Parser;
use std::process;

fn main() -> Result<()> {
    // Set up panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("ChromaCat panicked: {panic_info}");
        if let Some(location) = panic_info.location() {
            eprintln!("Location: {}:{}", location.file(), location.line());
        }
    }));

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

    // Validate CLI arguments early for fail-fast behavior
    cli.validate()?;

    // Create and run ChromaCat
    let mut cat = ChromaCat::new(cli);
    match cat.run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
    }

    Ok(())
}
