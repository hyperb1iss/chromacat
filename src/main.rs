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
        println!("Available Themes:\n");
        
        let themes = Cli::theme_descriptions();
        let mut current_category = String::new();
        
        for (name, description) in themes {
            // Extract category from name (assuming format like "nature/forest")
            let category = if name.contains('/') {
                name.split('/').next().unwrap()
            } else {
                "Classic"
            };
            
            // Print category header when it changes
            if category != current_category {
                if !current_category.is_empty() {
                    println!();  // Add spacing between categories
                }
                println!("{}:", category.to_uppercase());
                current_category = category.to_string();
            }
            
            // Print theme info with proper padding
            println!("  {:<15} - {}", name, description);
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