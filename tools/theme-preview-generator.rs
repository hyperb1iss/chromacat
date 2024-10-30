//! Theme preview generator for ChromaCat
//!
//! Generates PNG preview images of all ChromaCat themes for documentation
//! and website use. Can be run as a standalone binary.

use anyhow::{Context, Result};
use chromacat::themes;
use clap::Parser;
use colorgrad::Color;
use image::{ImageBuffer, Rgb};
use log::{debug, info};
use std::f32::consts::PI;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// CLI arguments for the preview generator
#[derive(Parser, Debug)]
#[command(
    name = "theme-preview-generator",
    about = "Generate preview images for ChromaCat themes",
    version
)]
pub struct Args {
    /// Output directory
    #[arg(short, long, default_value = "docs/theme-previews")]
    pub output_dir: PathBuf,

    /// Preview image width
    #[arg(short = 'W', long, default_value_t = 400)]
    pub width: u32,

    /// Preview image height
    #[arg(short = 'H', long, default_value_t = 100)]
    pub height: u32,

    /// Skip generating combined category previews
    #[arg(long)]
    pub skip_combined: bool,

    /// Specific theme to generate (generates all if not specified)
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Specific category to generate (generates all if not specified)
    #[arg(short, long)]
    pub category: Option<String>,
}

/// Generate preview images for ChromaCat themes
pub struct ThemePreviewGenerator {
    args: Args,
}

impl ThemePreviewGenerator {
    /// Create a new preview generator with the given arguments
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    /// Generate previews based on configuration
    pub fn generate(&self) -> Result<()> {
        let start = Instant::now();
        info!("Starting theme preview generation");
        debug!("Configuration: {:#?}", self.args);

        // Create output directory and its parents if they don't exist
        fs::create_dir_all(&self.args.output_dir).context("Failed to create output directory")?;

        // Filter themes based on arguments
        let themes = if let Some(theme_name) = &self.args.theme {
            vec![chromacat::themes::get_theme(theme_name)?]
        } else if let Some(category) = &self.args.category {
            let names = chromacat::themes::list_category(category).unwrap_or_default();
            names
                .into_iter()
                .filter_map(|name| chromacat::themes::get_theme(&name).ok())
                .collect()
        } else {
            chromacat::themes::all_themes()
        };

        info!("Generating previews for {} themes", themes.len());

        // Generate individual previews
        for theme in &themes {
            self.generate_preview(theme).with_context(|| {
                format!("Failed to generate preview for theme '{}'", theme.name)
            })?;
        }

        // Generate combined previews if not skipped
        if !self.args.skip_combined {
            self.generate_category_previews()
                .context("Failed to generate category previews")?;
        }

        info!(
            "Preview generation completed in {:.2}s",
            start.elapsed().as_secs_f64()
        );
        Ok(())
    }

    /// Generate preview for a specific theme
    fn generate_preview(&self, theme: &chromacat::themes::ThemeDefinition) -> Result<()> {
        debug!("Generating preview for theme: {}", theme.name);

        // Create gradient from theme
        let gradient = theme
            .create_gradient()
            .with_context(|| format!("Failed to create gradient for theme '{}'", theme.name))?;

        // Create image buffer
        let mut img = ImageBuffer::new(self.args.width, self.args.height);

        // Render horizontal gradient with theme's distribution and easing
        for x in 0..self.args.width {
            let raw_t = x as f32 / self.args.width as f32;

            // Apply theme's distribution and easing
            let t = theme.apply_easing(theme.apply_distribution(raw_t));

            // Handle repeat modes
            let t = match theme.repeat {
                themes::Repeat::Named(themes::RepeatMode::None) => t,
                themes::Repeat::Named(themes::RepeatMode::Mirror) => {
                    if (raw_t * 2.0) % 2.0 >= 1.0 {
                        1.0 - t
                    } else {
                        t
                    }
                }
                themes::Repeat::Named(themes::RepeatMode::Repeat) => t % 1.0,
                themes::Repeat::Function(ref name, rate) => {
                    match name.as_str() {
                        "rotate" => (t + 0.0 * rate).fract(), // time is 0 for static preview
                        "pulse" => {
                            let phase = (0.0 * rate * PI).sin(); // time is 0 for static preview
                            (t + phase) * 0.5
                        }
                        _ => t, // fallback
                    }
                }
            };

            let color = gradient.at(t);

            // Apply color to entire column
            for y in 0..self.args.height {
                img.put_pixel(x, y, color_to_rgb(&color));
            }
        }

        // Create theme category subdirectory
        let category = self.get_theme_category(&theme.name)?;
        let category_dir = self.args.output_dir.join(&category);
        fs::create_dir_all(&category_dir).context("Failed to create category directory")?;

        // Save image
        let filename = format!("{}.png", theme.name);
        let path = category_dir.join(filename);
        img.save(&path)
            .with_context(|| format!("Failed to save preview to '{}'", path.display()))?;

        debug!(
            "Generated preview for '{}' at '{}'",
            theme.name,
            path.display()
        );
        Ok(())
    }

    /// Generate combined preview images for each category
    fn generate_category_previews(&self) -> Result<()> {
        info!("Generating combined category previews");

        for category in chromacat::themes::list_categories() {
            // Skip if specific category requested and this isn't it
            if let Some(requested) = &self.args.category {
                if requested != &category {
                    continue;
                }
            }

            if let Some(themes) = chromacat::themes::list_category(&category) {
                let theme_count = themes.len();
                if theme_count == 0 {
                    continue;
                }

                // Create combined image
                let combined_height = self.args.height * theme_count as u32;
                let mut combined_img = ImageBuffer::new(self.args.width, combined_height);

                // Add each theme preview
                for (i, theme_name) in themes.iter().enumerate() {
                    let theme = chromacat::themes::get_theme(theme_name)?;
                    let gradient = theme.create_gradient()?;

                    // Calculate y position for this theme
                    let y_offset = i as u32 * self.args.height;

                    // Render theme gradient with proper settings
                    for x in 0..self.args.width {
                        let raw_t = x as f32 / self.args.width as f32;

                        // Apply theme's distribution and easing
                        let t = theme.apply_easing(theme.apply_distribution(raw_t));

                        // Handle repeat modes
                        let t = match theme.repeat {
                            themes::Repeat::Named(themes::RepeatMode::None) => t,
                            themes::Repeat::Named(themes::RepeatMode::Mirror) => {
                                if (raw_t * 2.0) % 2.0 >= 1.0 {
                                    1.0 - t
                                } else {
                                    t
                                }
                            }
                            themes::Repeat::Named(themes::RepeatMode::Repeat) => t % 1.0,
                            themes::Repeat::Function(ref name, rate) => {
                                match name.as_str() {
                                    "rotate" => (t + 0.0 * rate).fract(), // time is 0 for static preview
                                    "pulse" => {
                                        let phase = (0.0 * rate * PI).sin(); // time is 0 for static preview
                                        (t + phase) * 0.5
                                    }
                                    _ => t, // fallback
                                }
                            }
                        };

                        let color = gradient.at(t);

                        for y in 0..self.args.height {
                            combined_img.put_pixel(x, y + y_offset, color_to_rgb(&color));
                        }
                    }
                }

                // Save combined image
                let filename = format!("{}_all.png", category);
                let path = self.args.output_dir.join(&filename);
                combined_img.save(&path).with_context(|| {
                    format!(
                        "Failed to save combined preview for category '{}'",
                        category
                    )
                })?;

                debug!("Generated combined preview for category '{}'", category);
            }
        }

        Ok(())
    }

    /// Get the category for a theme
    fn get_theme_category(&self, theme_name: &str) -> Result<String> {
        for category in chromacat::themes::list_categories() {
            if let Some(themes) = chromacat::themes::list_category(&category) {
                if themes.contains(&theme_name.to_string()) {
                    return Ok(category);
                }
            }
        }
        Ok("default".to_string())
    }
}

/// Convert colorgrad Color to image::Rgb
fn color_to_rgb(color: &Color) -> Rgb<u8> {
    Rgb([
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
    ])
}

fn main() -> Result<()> {
    // Initialize logging with reasonable defaults
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    // Parse command line arguments
    let args = Args::parse();

    // Create generator and run
    let generator = ThemePreviewGenerator::new(args);
    generator.generate()
}
