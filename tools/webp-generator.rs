//! WebP generator for ChromaCat
//!
//! Generates animated WebP previews of ChromaCat patterns and themes.
//! Can be run as a standalone binary.

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// CLI arguments for the WebP generator
#[derive(Parser, Debug)]
#[command(
    name = "webp-generator",
    about = "Generate animated WebP previews for ChromaCat patterns",
    version
)]
pub struct Args {
    /// Output directory
    #[arg(short, long, default_value = "docs/pattern-previews")]
    pub output_dir: PathBuf,

    /// Width of the WebP
    #[arg(short = 'W', long, default_value_t = 320)]
    pub width: u32,

    /// Height of the WebP
    #[arg(short = 'H', long, default_value_t = 240)]
    pub height: u32,

    /// Frames per second
    #[arg(long, default_value_t = 30)]
    pub fps: u32,

    /// Animation duration in seconds
    #[arg(long, default_value_t = 5.0)]
    pub duration: f64,

    /// Pattern to generate (defaults to all patterns)
    #[arg(short, long)]
    pub pattern: Option<String>,

    /// Theme to use (defaults to rainbow)
    #[arg(short, long, default_value = "rainbow")]
    pub theme: String,

    /// Pattern-specific parameters
    #[arg(long = "param", value_name = "KEY=VALUE")]
    pub params: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Generate WebP previews for ChromaCat patterns
pub struct WebPGenerator {
    args: Args,
}

impl WebPGenerator {
    /// Create a new WebP generator with the given arguments
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    /// Generate WebPs based on configuration
    pub fn generate(&self) -> Result<()> {
        let start = Instant::now();
        info!("Starting WebP generation");
        info!("Output directory: {}", self.args.output_dir.display());
        info!("Size: {}x{}", self.args.width, self.args.height);
        info!("FPS: {}, Duration: {}s", self.args.fps, self.args.duration);
        debug!("Full configuration: {:#?}", self.args);

        // Create output directory if it doesn't exist
        info!("Creating output directory...");
        std::fs::create_dir_all(&self.args.output_dir)
            .context("Failed to create output directory")?;

        // Get list of patterns to generate
        let patterns = if let Some(pattern) = &self.args.pattern {
            info!("Generating single pattern: {}", pattern);
            vec![pattern.clone()]
        } else {
            info!("Generating all patterns");
            vec![
                "diagonal".to_string(),
                "horizontal".to_string(),
                "plasma".to_string(),
                "ripple".to_string(),
                "wave".to_string(),
                "spiral".to_string(),
                "checkerboard".to_string(),
                "diamond".to_string(),
                "perlin".to_string(),
                "rain".to_string(),
                "fire".to_string(),
                "aurora".to_string(),
                "kaleidoscope".to_string(),
            ]
        };

        // Process patterns in parallel with a limited number of threads
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4) // Limit to 4 patterns at a time
            .build()?;

        pool.install(|| {
            patterns.par_iter().try_for_each(|pattern| {
                let theme = if self.args.theme != "rainbow" {
                    &self.args.theme
                } else {
                    get_recommended_theme(pattern)
                };

                info!("Starting pattern: {} (theme: {})", pattern, theme);

                let pattern_config = chromacat::pattern::PatternConfig {
                    common: chromacat::pattern::CommonParams {
                        frequency: 1.0,
                        amplitude: 1.0,
                        speed: 1.0,
                        correct_aspect: true,
                        aspect_ratio: 0.5,
                        theme_name: Some(theme.to_string()),
                    },
                    params: chromacat::pattern::REGISTRY
                        .create_pattern_params(pattern)
                        .ok_or_else(|| anyhow::anyhow!("Invalid pattern: {}", pattern))?,
                };

                self.generate_pattern_webp(pattern, pattern_config)
                    .with_context(|| format!("Failed to generate WebP for pattern '{}'", pattern))
            })
        })?;

        let elapsed = start.elapsed();
        info!(
            "WebP generation completed in {:.2}s ({} patterns)",
            elapsed.as_secs_f64(),
            patterns.len()
        );
        info!("Output directory: {}", self.args.output_dir.display());
        Ok(())
    }

    /// Generate WebP for a specific pattern
    fn generate_pattern_webp(
        &self,
        pattern: &str,
        config: chromacat::pattern::PatternConfig,
    ) -> Result<()> {
        let start = Instant::now();

        // Calculate frames and timing - adjusted for slower, app-like speed
        let total_frames = (self.args.fps as f64 * self.args.duration) as u32;
        let frame_delay = (100.0 / self.args.fps as f64) as u16;

        // Adjusted cycle time for slower animation
        let cycle_time = 0.5 * std::f64::consts::PI; // Reduced to 1/4 of original speed
        let time_scale = cycle_time / (total_frames as f64);

        debug!(
            "Animation settings: {} frames, {}ms delay, cycle time: {:.3}s",
            total_frames,
            frame_delay * 10,
            cycle_time
        );

        // Create pattern engine with app-matched speed config
        let mut config = config;
        config.common.speed *= 0.025; // Significantly reduced from 0.1 to match app speed

        // Clone config for parallel processing
        let config = Arc::new(config);

        // Create output file
        let filename = format!("{}.webp", pattern);
        let path = self.args.output_dir.join(filename);

        // Collect all frame data first
        let frames: Vec<Result<Vec<u8>>> = (0..total_frames)
            .into_par_iter()
            .map(|frame_idx| {
                let progress = frame_idx as f64 / (total_frames - 1) as f64;
                if frame_idx % 10 == 0 {
                    info!(
                        "  {} Frame {}/{} ({:.1}%)",
                        if frame_idx % 20 < 10 { "→" } else { "⇢" },
                        frame_idx + 1,
                        total_frames,
                        progress * 100.0,
                    );
                }

                // Calculate time position in the cycle (0 to 2π)
                let cycle_position = frame_idx as f64 * time_scale;
                let config = config.clone();

                // Create a new engine for this frame
                let mut frame_engine = chromacat::pattern::engine::PatternEngine::new(
                    chromacat::themes::get_theme(config.common.theme_name.as_ref().unwrap())?
                        .create_gradient()
                        .context("Failed to create gradient")?,
                    (*config).clone(),
                    self.args.width as usize,
                    self.args.height as usize,
                );

                // Set time based on cycle position
                frame_engine.set_time(cycle_position);

                // Create frame data
                let mut frame_data = vec![0u8; (self.args.width * self.args.height * 3) as usize];

                // Process chunks sequentially within each frame
                let chunk_size: u32 = 32;
                for chunk_y in (0..self.args.height).step_by(chunk_size as usize) {
                    for chunk_x in (0..self.args.width).step_by(chunk_size as usize) {
                        let chunk_width =
                            ((chunk_x + chunk_size).min(self.args.width) - chunk_x) as usize;
                        let chunk_height =
                            ((chunk_y + chunk_size).min(self.args.height) - chunk_y) as usize;

                        // Process pixels within chunk
                        for y in chunk_y..chunk_y + chunk_height as u32 {
                            for x in chunk_x..chunk_x + chunk_width as u32 {
                                let value = frame_engine.get_value_at(x as usize, y as usize)?;
                                let color = frame_engine.gradient().at(value as f32);

                                let pixel_idx = ((y * self.args.width + x) * 3) as usize;
                                frame_data[pixel_idx] = (color.r * 255.0) as u8;
                                frame_data[pixel_idx + 1] = (color.g * 255.0) as u8;
                                frame_data[pixel_idx + 2] = (color.b * 255.0) as u8;
                            }
                        }
                    }
                }

                Ok(frame_data)
            })
            .collect();

        // Create WebP encoder
        let mut encoder = webp_animation::Encoder::new((self.args.width, self.args.height))
            .context("Failed to create WebP encoder")?;

        // Set default encoding config
        encoder
            .set_default_encoding_config(webp_animation::EncodingConfig {
                quality: 75.0,
                encoding_type: webp_animation::EncodingType::Lossless,
                ..Default::default()
            })
            .context("Failed to set encoding config")?;

        // Generate and add frames
        for (i, raw_pixels) in frames.into_iter().enumerate() {
            let raw_pixels = raw_pixels?;

            // Convert RGB to RGBA by adding alpha channel
            let mut rgba_data =
                Vec::with_capacity((self.args.width * self.args.height * 4) as usize);
            for chunk in raw_pixels.chunks(3) {
                rgba_data.extend_from_slice(chunk); // RGB
                rgba_data.push(255); // Alpha
            }

            // Calculate timestamp for this frame
            let timestamp_ms = ((i as f64 * 1000.0) / self.args.fps as f64) as i32;

            // Add frame to encoder
            encoder
                .add_frame(&rgba_data, timestamp_ms)
                .with_context(|| format!("Failed to add frame {}", i))?;
        }

        // Calculate final timestamp (total duration in ms)
        let final_timestamp_ms = (self.args.duration * 1000.0) as i32;

        // Finalize and get the WebP data
        let webp_data = encoder
            .finalize(final_timestamp_ms)
            .context("Failed to encode WebP animation")?;

        // Write the WebP file
        std::fs::write(&path, webp_data)
            .with_context(|| format!("Failed to write WebP file '{}'", path.display()))?;

        let total_time = start.elapsed();
        info!(
            "Completed pattern '{}' in {:.2}s - Output: {}",
            pattern,
            total_time.as_secs_f64(),
            path.display()
        );
        Ok(())
    }
}

/// Get the best theme for a given pattern
fn get_recommended_theme(pattern: &str) -> &'static str {
    match pattern {
        // Abstract/Fluid patterns
        "plasma" => "plasma", // Electric plasma effect matches the pattern perfectly
        "ripple" => "ocean",  // Water-like ripples with ocean colors
        "wave" => "serenity", // Gentle waves with calming colors
        "spiral" => "galaxy", // Spiral pattern with space colors

        // Geometric patterns
        "checkerboard" => "complementary", // High contrast works well with sharp edges
        "diamond" => "retrowave",          // Diamond shapes pop with retro colors
        "diagonal" => "triadic",           // Clean lines work well with distinct colors
        "horizontal" => "pride",           // Simple gradient works well with pride flag colors

        // Noise-based patterns
        "perlin" => "nebula", // Organic noise works well with nebula colors

        // Dynamic patterns
        "rain" => "hackerman",   // Digital rain effect with matrix colors
        "fire" => "fire",     // Fire pattern with matching heat colors
        "aurora" => "neon", // Aurora pattern with matching colors

        // Default to rainbow if no specific recommendation
        _ => "rainbow",
    }
}

fn main() -> Result<()> {
    // Initialize logging with more detailed format
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{:>5}] {}",
                buf.timestamp(),
                record.level(),
                record.args()
            )
        })
        .init();

    // Parse command line arguments
    let args = Args::parse();

    // Create generator and run
    let generator = WebPGenerator::new(args);
    generator.generate()
}
