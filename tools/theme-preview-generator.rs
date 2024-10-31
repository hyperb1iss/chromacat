//! Theme preview generator for ChromaCat
//!
//! Generates PNG preview images of all ChromaCat themes for documentation
//! and website use. Can be run as a standalone binary.

use anyhow::{Context, Result};
use clap::Parser;
use colorgrad::Color;
use image::{ImageBuffer, Rgb};
use log::{debug, info};
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

    /// Number of color blocks in the preview
    #[arg(short = 'B', long, default_value_t = 30)]
    pub blocks: u32,

    /// Height of the preview image
    #[arg(short = 'H', long, default_value_t = 50)]
    pub height: u32,

    /// Width of the preview image (calculated based on blocks if not provided)
    #[arg(short = 'W', long)]
    pub width: Option<u32>,

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

        // Determine image dimensions
        let blocks = self.args.blocks;
        let img_height = self.args.height;
        let img_width = self.args.width.unwrap_or(blocks * 10); // Default block width is 10 pixels
        let block_width = img_width / blocks;

        // Create image buffer
        let mut img = ImageBuffer::new(img_width, img_height);

        // Generate the preview by rendering blocks of colors
        for i in 0..blocks {
            let t = i as f32 / (blocks - 1) as f32;
            let color = gradient.at(t);
            let rgb = color_to_rgb(&color);

            let x_start = i * block_width;
            let x_end = if i == blocks - 1 {
                img_width
            } else {
                (i + 1) * block_width
            };

            for x in x_start..x_end {
                for y in 0..img_height {
                    img.put_pixel(x, y, rgb);
                }
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

                // Determine image dimensions
                let blocks = self.args.blocks;
                let img_height = self.args.height * theme_count as u32;
                let img_width = self.args.width.unwrap_or(blocks * 10); // Default block width is 10 pixels
                let block_width = img_width / blocks;

                let mut combined_img = ImageBuffer::new(img_width, img_height);

                // Generate each theme preview and add it to the combined image
                for (i, theme_name) in themes.iter().enumerate() {
                    let theme = chromacat::themes::get_theme(theme_name)?;
                    let gradient = theme.create_gradient()?;

                    let y_offset = i as u32 * self.args.height;

                    for j in 0..blocks {
                        let t = j as f32 / (blocks - 1) as f32;
                        let color = gradient.at(t);
                        let rgb = color_to_rgb(&color);

                        let x_start = j * block_width;
                        let x_end = if j == blocks - 1 {
                            img_width
                        } else {
                            (j + 1) * block_width
                        };

                        for x in x_start..x_end {
                            for y in y_offset..(y_offset + self.args.height) {
                                combined_img.put_pixel(x, y, rgb);
                            }
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
        (color.r * 255.0).clamp(0.0, 255.0) as u8,
        (color.g * 255.0).clamp(0.0, 255.0) as u8,
        (color.b * 255.0).clamp(0.0, 255.0) as u8,
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
