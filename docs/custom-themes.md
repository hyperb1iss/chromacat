# 🎨 ChromaCat Custom Themes

> _Paint your terminal with custom color gradients_ ✨

Create your own stunning terminal color schemes with ChromaCat's theme system. This guide will help you craft the perfect gradient for your terminal experience.

## 📚 Table of Contents

1. [Quick Start](#quick-start)
2. [Theme File Format](#theme-file-format)
3. [Color Configuration](#color-configuration)
4. [Distribution Types](#distribution-types)
5. [Examples](#examples)
6. [Tips & Best Practices](#tips--best-practices)

## ⚡ Quick Start

Create your first theme in two easy steps:

1. Create a theme file `mythemes.yaml`:

```yaml
- name: my-sunset
  desc: Warm sunset colors
  colors:
    - [1.0, 0.4, 0.0, 0.0, orange] # Deep orange
    - [0.7, 0.0, 0.7, 1.0, purple] # Royal purple
  dist: even
  ease: smooth
```

2. Use your theme:

```bash
chromacat --theme-file mythemes.yaml -t my-sunset input.txt
```

## 📋 Theme File Format

Themes are defined in YAML files with this structure:

```yaml
# Single theme definition
- name: theme-name # Required: Unique identifier
  desc: Theme description # Required: Brief description
  colors: # Required: At least 2 color stops
    - [r, g, b, pos, name] # RGB values (0-1), position (0-1), name
  dist: distribution-type # Optional: Color distribution method
  ease: easing-function # Optional: Transition type between colors

# You can define multiple themes in one file
- name: another-theme
  desc: Another cool theme
  colors:
    - [1.0, 0.0, 0.0, 0.0] # Red at start
    - [0.0, 0.0, 1.0, 1.0] # Blue at end
```

### Required Fields

| Field    | Description                | Example                                         |
| -------- | -------------------------- | ----------------------------------------------- |
| `name`   | Unique theme identifier    | `"ocean-depths"`                                |
| `desc`   | Short theme description    | `"Deep ocean blues"`                            |
| `colors` | Array of color definitions | See [Color Configuration](#color-configuration) |

### Optional Fields

| Field  | Default    | Valid Values                                  | Description            |
| ------ | ---------- | --------------------------------------------- | ---------------------- |
| `dist` | `"even"`   | `even`, `front`, `back`, `center`, `alt`      | How colors distribute  |
| `ease` | `"linear"` | `linear`, `smooth`, `smoother`, `sine`, `exp` | Color transition style |

## 🎨 Color Configuration

Colors can be defined in several ways:

```yaml
colors:
  # Full definition: RGB, position, name
  - [1.0, 0.0, 0.0, 0.0, red] # Red at start
  - [0.0, 1.0, 0.0, 0.5, green] # Green in middle
  - [0.0, 0.0, 1.0, 1.0, blue] # Blue at end

  # Position only: RGB, position
  - [0.8, 0.2, 0.8, 0.25] # Purple at 25%

  # Basic: Just RGB
  - [1.0, 1.0, 0.0] # Yellow, position auto-calculated
```

### RGB Values

- Range from `0.0` to `1.0`
- Common color recipes:
  ```yaml
  [1.0, 1.0, 1.0] # White
  [0.0, 0.0, 0.0] # Black
  [0.5, 0.5, 0.5] # Gray
  [1.0, 0.0, 0.0] # Red
  [0.0, 1.0, 0.0] # Green
  [0.0, 0.0, 1.0] # Blue
  [1.0, 1.0, 0.0] # Yellow
  [1.0, 0.0, 1.0] # Magenta
  [0.0, 1.0, 1.0] # Cyan
  ```

### Position Values

- Range from `0.0` (start) to `1.0` (end)
- Optional - colors are evenly spaced if omitted
- Example with explicit positioning:
  ```yaml
  colors:
    - [1.0, 0.0, 0.0, 0.0] # Red at 0%
    - [1.0, 1.0, 1.0, 0.3] # White at 30%
    - [0.0, 0.0, 1.0, 0.7] # Blue at 70%
    - [0.0, 0.0, 0.0, 1.0] # Black at 100%
  ```

## 🎭 Distribution Types

The `dist` field controls how colors spread across the gradient:

```yaml
# Even distribution (default)
dist: even    # Colors spread uniformly
- [Red] -------- [Green] -------- [Blue]

# Front-loaded
dist: front   # Colors concentrate at start
- [Red] -- [Green] ---- [Blue] --------

# Back-loaded
dist: back    # Colors concentrate at end
- [Red] -------- [Green] -- [Blue]

# Center-focused
dist: center  # Colors concentrate in middle
- [Red] ---- [Green] -- [Blue] ----

# Alternating
dist: alt     # Colors alternate sinusoidally
- [Red] ~ [Green] ~ [Blue] ~ [Red]
```

## 🌟 Examples

### Ocean Theme

```yaml
- name: deep-ocean
  desc: Calming ocean depths
  colors:
    - [0.0, 0.2, 0.4, 0.0, deep-blue] # Deep water
    - [0.0, 0.4, 0.6, 0.3, medium-blue] # Mid depths
    - [0.0, 0.6, 0.8, 0.7, light-blue] # Shallow water
    - [0.4, 0.8, 1.0, 1.0, surface-blue] # Surface
  dist: smooth
  ease: smooth
```

### Forest Theme

```yaml
- name: forest
  desc: Natural green tones
  colors:
    - [0.08, 0.32, 0.16, 0.0, dark-green] # Dark foliage
    - [0.18, 0.54, 0.34, 0.3, forest-green] # Forest shade
    - [0.13, 0.54, 0.13, 0.7, green] # Sunlit leaves
    - [0.60, 0.80, 0.20, 1.0, yellow-green] # Dappled light
  dist: back
  ease: smoother
```

### Sunset Theme

```yaml
- name: sunset
  desc: Evening sky colors
  colors:
    - [0.98, 0.31, 0.42, 0.0, coral] # Horizon
    - [0.99, 0.62, 0.45, 0.3, peach] # Lower sky
    - [0.97, 0.76, 0.44, 0.7, light-orange] # Mid sky
    - [0.56, 0.28, 0.58, 1.0, purple] # Upper sky
  dist: center
  ease: smooth
```

## 💡 Tips & Best Practices

### Color Selection

- Use 3-5 colors for best results
- Test your theme in both light and dark terminals
- Consider colorblind accessibility
- Use RGB color pickers to find exact values

### Theme Organization

```yaml
# Group related themes together
- name: forest-day
  desc: Daytime forest colors
  colors:
    - [0.13, 0.54, 0.13, 0.0] # Light green
    - [0.18, 0.54, 0.34, 1.0] # Dark green

- name: forest-night
  desc: Nighttime forest colors
  colors:
    - [0.05, 0.15, 0.05, 0.0] # Dark green
    - [0.10, 0.25, 0.10, 1.0] # Deeper green
```

### Best Practices

- Choose descriptive theme names
- Add clear descriptions
- Group related themes in the same file
- Use comments to explain color choices
- Keep color values between 0.0 and 1.0
- Test themes in different terminal types

## 🚀 Usage Examples

```bash
# Basic usage
chromacat --theme-file mythemes.yaml -t my-theme input.txt

# Multiple theme files
chromacat --theme-file themes1.yaml --theme-file themes2.yaml -t theme-name

# Pipe from another command
ls -la | chromacat --theme-file mythemes.yaml -t my-theme
```

## 🤝 Contributing

If you create an amazing theme, consider contributing it back to ChromaCat! Submit a pull request with your theme file added to the `themes/` directory.
