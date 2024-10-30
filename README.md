# 😺 ChromaCat ✨

> _Because your terminal deserves to be fabulous_ ✨

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/chromacat)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

ChromaCat is a turbocharged terminal colorizer written in Rust that brings stunning gradient patterns and animations to your command-line experience. Think `lolcat` but with superpowers! 🚀

## ✨ Features

- 🎨 **Rich Pattern Library**: Nine distinct pattern types from simple gradients to psychedelic plasma effects
- 🌈 **40+ Built-in Themes**: Everything from classic rainbow to custom color schemes
- 🔄 **Smooth Animations**: Breathe life into your terminal with fluid color transitions
- 🎮 **Interactive Mode**: Real-time control over animations and effects
- 🎯 **Precise Control**: Fine-tune every aspect of your gradients
- 🦀 **Blazing Fast**: Optimized Rust implementation with minimal overhead
- 🌍 **Full Unicode Support**: Works beautifully with emojis and international text
- 📱 **Terminal-Aware**: Adapts to terminal dimensions and capabilities

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

# Use a specific pattern
echo "Wave pattern!" | chromacat -p wave --param amplitude=1.5
```

## 🎨 Pattern Types

ChromaCat offers several pattern types for dynamic colorization:

- `horizontal` - Classic left-to-right gradient (default)
- `diagonal` - Angled gradient with customizable direction
- `plasma` - Psychedelic plasma effect using sine waves
- `ripple` - Concentric circles emanating from center
- `wave` - Flowing wave distortion pattern
- `spiral` - Hypnotic spiral pattern from center
- `checkerboard` - Alternating gradient colors in a grid
- `diamond` - Diamond-shaped gradient pattern
- `perlin` - Organic, cloud-like noise pattern

## 🌈 Theme Categories

ChromaCat includes many themed gradients organized by category:

### Space Themes

- `nebula` - Cosmic nebula colors
- `cosmos` - Deep space with stars
- `aurora` - Northern lights in motion
- `galaxy` - Spiral galaxy with star clusters

### Tech Themes

- `matrix` - Digital rain aesthetic
- `cyberpunk` - High-tech urban future vibes
- `terminal` - Classic computer terminal look
- `hackerman` - Retro hacker aesthetic
- `quantum` - Quantum superposition states

### Nature Themes

- `ocean` - Cool blue tones of the sea
- `forest` - Natural green tones
- `autumn` - Warm fall colors
- `sunset` - Evening sky colors
- `desert` - Warm earth tones

### Aesthetic Themes

- `pastel` - Soft, muted colors
- `neon` - Bright, vibrant colors that pop
- `retrowave` - 80s-inspired synthwave aesthetic
- `vaporwave` - 90s-inspired aesthetic

### Party Themes

- `rave` - Intense party colors that pulse
- `disco` - 70s disco-inspired colors
- `festival` - Vibrant festival color palette
- `carnival` - Bright carnival celebration colors

### Abstract Themes

- `heat` - Warm colors transitioning from red to yellow
- `ice` - Cool, frozen tones
- `fire` - Hot, intense flames
- `toxic` - Radioactive greens and acid colors
- `plasma` - Electric plasma-like effect

### Pride Themes
- `trans` - Transgender pride flag colors
- `pan` - Pansexual pride flag colors
- `nonbinary` - Non-binary pride flag colors
- `lesbian` - Lesbian pride flag colors
- `progress` - Progress pride flag with inclusive colors

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

### Pattern Selection and Customization

```bash
# Diagonal gradient at 45 degrees
chromacat -p diagonal --param angle=45 file.txt

# Plasma effect with custom settings
chromacat -p plasma --param complexity=3.0,scale=1.5 file.txt

# Wave pattern with customization
chromacat -p wave --param amplitude=1.5,frequency=2.0 file.txt

# Ripple pattern from center
chromacat -p ripple --param wavelength=1.0,damping=0.5 file.txt
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
- `T` - Cycle through themes
- `P` - Cycle through patterns
- `Q` or `Esc` - Quit
- `←` `→` - Adjust animation speed
- `↑` `↓` - Scroll through content

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
chromacat -p plasma --param complexity=3.0,scale=1.5,blend_mode=add

# Ripple
chromacat -p ripple --param wavelength=1.0,damping=0.5,center_x=0.5,center_y=0.5

# Wave
chromacat -p wave --param amplitude=1.0,frequency=2.0,phase=0.0,offset=0.5

# Spiral
chromacat -p spiral --param density=2.0,rotation=90,expansion=1.5

# Checkerboard
chromacat -p checkerboard --param size=2,blur=0.1,rotation=45
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

## 📝 Custom Themes

ChromaCat supports custom theme files in YAML format:

```yaml
- name: custom-theme
  desc: A beautiful custom gradient
  colors:
    - [1.0, 0.0, 0.0, 0.0, red]
    - [0.0, 1.0, 0.0, 0.5, green]
    - [0.0, 0.0, 1.0, 1.0, blue]
  dist: even
  repeat: mirror
  speed: 1.0
  ease: smooth
```

Load custom themes with:

```bash
chromacat --theme-file my-themes.yaml -t custom-theme
```

## 🎯 Performance Considerations

ChromaCat is designed to be fast and efficient:

- Pre-computed lookup tables for pattern generation
- Efficient buffering for large inputs
- Smart terminal handling and state management
- Optional performance modes for resource-constrained environments

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

Created by [Stefanie Jane 🌠](https://github.com/hyperb1iss)

If you find ChromaCat useful, [buy me a Monster Ultra Violet](https://ko-fi.com/hyperb1iss)! ⚡️

</div>
