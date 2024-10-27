# 😺 ChromaCat ✨

> _Because your terminal deserves to be fabulous_ ✨

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/chromacat)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

ChromaCat is a turbocharged terminal colorizer written in Rust that brings stunning gradient patterns and animations to your command-line experience. Think `lolcat` but with superpowers! 🚀

## ✨ Features

- 🎨 **Rich Pattern Library**: From simple horizontal gradients to psychedelic plasma effects
- 🌈 **40+ Built-in Themes**: Everything from classic rainbow to cyberpunk aesthetics
- 🔄 **Smooth Animations**: Breathe life into your terminal with fluid color transitions
- 🎮 **Interactive Mode**: Real-time control over animations and effects
- 🎯 **Precise Control**: Fine-tune every aspect of your gradients
- 🦀 **Blazing Fast**: Written in Rust for optimal performance
- 🌍 **Full Unicode Support**: Works beautifully with emojis and international text

## 🚀 Installation

### Using Cargo (Recommended)

```bash
cargo install chromacat
```

### From Source

```bash
git clone https://github.com/hyperb1iss/chromacat
cd chromacat
cargo build --release
```

### Homebrew

```bash
brew install hyperb1iss/tap/chromacat
```

## 🎯 Quick Start

```bash
# Basic usage
echo "Hello, ChromaCat!" | chromacat

# Choose a theme
ls -la | chromacat -t cyberpunk

# Add some animation
cat your_file.txt | chromacat -a
```

## 🎨 Pattern Types

ChromaCat offers several pattern types to make your terminal output pop:

- `horizontal` - Classic left-to-right gradient (default)
- `diagonal` - Angled gradient with customizable direction
- `plasma` - Psychedelic plasma effect using sine waves
- `ripple` - Concentric circles emanating from center
- `wave` - Flowing wave distortion pattern
- `spiral` - Hypnotic spiral pattern from center
- `checkerboard` - Alternating gradient colors in a grid
- `diamond` - Diamond-shaped gradient pattern
- `perlin` - Organic, cloud-like noise pattern

## 🌈 Available Themes

Here's a taste of the available themes (there are many more!):

### Classic Themes

- `rainbow` - The classic ROY G. BIV
- `grayscale` - Smooth black to white transitions
- `sepia` - Vintage brownscale vibes

### Tech Themes

- `matrix` - Digital rain aesthetic
- `cyberpunk` - High-tech urban future vibes
- `hackerman` - Old-school terminal feel

### Nature Themes

- `ocean` - Cool blues of the deep
- `forest` - Earthy green gradients
- `aurora` - Northern lights inspired

### Aesthetic Themes

- `vaporwave` - 90s aesthetic with pink and cyan
- `retrowave` - 80s-inspired synthwave colors
- `neon` - Bright, vibrant pop

## 💫 Usage Examples

### Basic Text Coloring

```bash
# Simple gradient
echo "Hello, World!" | chromacat

# Using a specific theme
echo "Ocean vibes" | chromacat -t ocean

# Multiple files
chromacat file1.txt file2.txt
```

### Animation Effects

```bash
# Basic animation
cat your_file.txt | chromacat -a

# Smooth animation with custom FPS
ls -la | chromacat -a --fps 60 --smooth

# Infinite animation
chromacat --animate --duration 0 file.txt
```

### Pattern Customization

```bash
# Diagonal gradient at 45 degrees
chromacat -p diagonal --angle 45 file.txt

# Plasma effect with custom settings
chromacat -p plasma --complexity 3.0 --scale 1.5 file.txt

# Wave pattern with custom parameters
chromacat -p wave --height 1.5 --count 3 file.txt
```

### Advanced Usage

```bash
# Combine with other commands
git status | chromacat -p ripple -t neon

# Custom animation speed
find . -type f | chromacat -a --speed 0.5

# Progress logging with style
yarn build | chromacat -t cyberpunk
```

## 🎮 Interactive Controls

When running in animation mode (`-a`):

- `Space` - Pause/Resume animation
- `R` - Reset animation time
- `Q` or `Esc` - Quit
- `←` `→` - Adjust animation speed
- `↑` `↓` - Modify pattern parameters

## 🛠 Configuration Options

### Common Parameters

- `--frequency <0.1-10.0>` - Base pattern frequency
- `--amplitude <0.1-2.0>` - Pattern intensity
- `--speed <0.0-1.0>` - Animation speed

### Animation Settings

- `--fps <1-144>` - Frames per second
- `--duration <seconds>` - Animation duration (0 for infinite)
- `--smooth` - Enable smooth transitions
- `--no-color` - Disable colored output

### Pattern-Specific Parameters

```bash
# Plasma
chromacat -p plasma --complexity <1.0-10.0> --scale <0.1-5.0>

# Ripple
chromacat -p ripple --wavelength <0.1-5.0> --damping <0.0-1.0>

# Wave
chromacat -p wave --height <0.1-2.0> --count <0.1-5.0>

# Spiral
chromacat -p spiral --density <0.1-5.0> --expansion <0.1-2.0>

# Checkerboard
chromacat -p checkerboard --size <1-10> --blur <0.0-1.0>
```

## 🔧 Integration Tips

### Shell Aliases

```bash
# Add to your .bashrc or .zshrc
alias cat="chromacat"
alias ls="ls --color=always | chromacat -t ocean"
alias gl="git log --oneline --graph | chromacat -p wave -t neon"
```

### Build Logs

```bash
# Make your build logs fabulous
npm run build | chromacat -t cyberpunk
cargo build 2>&1 | chromacat -p plasma -t matrix
```

### System Monitoring

```bash
# Colorful system monitoring
watch -n1 "ps aux | sort -rn -k 3,3 | head -n 5 | chromacat -t heat"
```

## 🎯 Performance Considerations

ChromaCat is designed to be fast and efficient, but here are some tips for optimal performance:

- Use static rendering for large files instead of animation mode
- Adjust FPS based on your terminal's capabilities
- Consider using simpler patterns (like horizontal or diagonal) for very large outputs
- The `--no-color` flag can be used to bypass processing when needed

## 🤝 Contributing

Contributions are super welcome! Here's how you can help:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

Please make sure to update tests as appropriate and follow the existing coding style.

## 🐛 Known Issues

- Some terminals might not support all color combinations
- Very large files might cause performance issues in animation mode
- Pattern parameters might need adjustment based on terminal size

## 🙏 Acknowledgements

ChromaCat leverages several open-source Rust crates and tools that make its functionalities possible:

- [**clap**](https://crates.io/crates/clap) for command-line argument parsing
- [**colorgrad**](https://crates.io/crates/colorgrad) for creating and managing color gradients
- [**termcolor**](https://crates.io/crates/termcolor) for handling colored terminal output
- [**atty**](https://crates.io/crates/atty) for detecting terminal streams
- [**unicode-segmentation**](https://crates.io/crates/unicode-segmentation) for accurate Unicode character handling
- [**lolcat**](https://github.com/busyloop/lolcat) for inspiring the initial concept of colorizing terminal output

Special thanks to the Rust community for their continuous contributions and support.

## 📝 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

Created by [Stefanie Jane 🌠](https://github.com/hyperb1iss)

If you find ChromaCat useful, [buy me a Monster Ultra Violet](https://ko-fi.com/hyperb1iss)! ⚡️

</div>

## 🎨 Custom Themes

ChromaCat allows you to create and load your own custom theme files in YAML format. Custom themes can be used alongside built-in themes and support all the same features.

### Loading Custom Themes

```bash
# Load a custom theme file
chromacat --theme-file my-themes.yaml -t my-custom-theme

# You can combine custom themes with any pattern
chromacat --theme-file themes/neon.yaml -t cyberpunk-nights -p wave
```

### Custom Theme Format

Custom themes are defined in YAML files. Each file can contain multiple themes:

```yaml
- name: my-custom-theme
  desc: A beautiful custom gradient
  colors:
    - [1.0, 0.0, 0.0, 0.0, red] # [R, G, B, position, name]
    - [0.0, 1.0, 0.0, 0.5, green] # position and name are optional
    - [0.0, 0.0, 1.0, 1.0, blue]
  dist: even # Distribution pattern
  repeat: mirror # Repeat mode
  speed: 1.0 # Animation speed
  ease: smooth # Easing function

- name: another-theme
  desc: Another custom theme
  colors:
    - [0.8, 0.2, 0.8] # Minimal color definition
    - [0.2, 0.8, 0.8]
    - [0.8, 0.8, 0.2]
  # Optional fields will use defaults
```

### Theme Properties

#### Colors

- RGB values must be between 0.0 and 1.0
- Position (optional) defines color stop position (0.0 to 1.0)
- Name (optional) is for reference only

#### Distribution Types (`dist`)

- `even` - Colors evenly distributed (default)
- `front` - Colors concentrated at start
- `back` - Colors concentrated at end
- `center` - Colors concentrated in middle
- `alt` - Alternating distribution

#### Repeat Modes (`repeat`)

- `none` - No repetition (default)
- `mirror` - Reverse pattern at end
- `repeat` - Repeat pattern
- `pulse(rate)` - Pulsing effect with rate (e.g., `pulse(0.5)`)
- `rotate(rate)` - Rotating effect with rate (e.g., `rotate(1.0)`)

#### Easing Functions (`ease`)

- `linear` - No easing (default)
- `smooth` - Smooth transition
- `smoother` - Extra smooth transition
- `sine` - Sinusoidal easing
- `exp` - Exponential easing
- `elastic` - Elastic bounce effect

#### Other Properties

- `speed` - Animation speed multiplier (default: 1.0)
- `desc` - Theme description (optional)

### Example Theme File

Here's a more complex example showing various features:

```yaml
- name: sunset-dream
  desc: Warm sunset colors with smooth transitions
  colors:
    - [1.0, 0.2, 0.0, 0.0, deep-orange]
    - [1.0, 0.6, 0.2, 0.3, light-orange]
    - [0.8, 0.4, 0.6, 0.7, purple]
    - [0.4, 0.2, 0.8, 1.0, deep-blue]
  dist: back
  repeat: pulse(0.3)
  speed: 0.8
  ease: smoother

- name: neon-flash
  desc: Bright neon colors with sharp transitions
  colors:
    - [1.0, 0.0, 1.0, 0.0, magenta]
    - [0.0, 1.0, 1.0, 0.33, cyan]
    - [1.0, 1.0, 0.0, 0.66, yellow]
    - [0.0, 1.0, 0.0, 1.0, green]
  dist: alt
  repeat: rotate(0.8)
  speed: 1.2
  ease: exp
```

### Tips for Creating Themes

1. Start with 2-4 colors for simple gradients
2. Use position values to control color distribution
3. Match the distribution type to your desired effect
4. Experiment with repeat modes for dynamic effects
5. Choose easing functions that complement your pattern
6. Test themes with different patterns and animations

### Validation

ChromaCat validates custom themes when loading them and will report specific errors if:

- Color values are outside the valid range (0.0-1.0)
- Required fields are missing
- Position values are invalid
- Speed value is not positive
- Theme has fewer than 2 colors
