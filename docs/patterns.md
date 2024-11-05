# 🌈 ChromaCat Pattern Guide

> _A deep dive into creating mesmerizing terminal art_ ✨

## Table of Contents

1. [Pattern Overview](#pattern-overview)
2. [Core Patterns](#core-patterns)
   - [Horizontal Gradient](#horizontal-gradient)
   - [Diagonal Gradient](#diagonal-gradient)
3. [Wave-Based Patterns](#wave-based-patterns)
   - [Wave Pattern](#wave-pattern)
   - [Ripple Pattern](#ripple-pattern)
4. [Complex Patterns](#complex-patterns)
   - [Plasma Pattern](#plasma-pattern)
   - [Perlin Noise](#perlin-noise)
5. [Geometric Patterns](#geometric-patterns)
   - [Spiral Pattern](#spiral-pattern)
   - [Diamond Pattern](#diamond-pattern)
   - [Checkerboard Pattern](#checkerboard-pattern)
6. [Dynamic Effects](#dynamic-effects)
   - [Digital Rain](#digital-rain)
   - [Fire Effect](#fire-effect)
   - [Aurora Borealis](#aurora-borealis)
   - [Kaleidoscope](#kaleidoscope)

## Pattern Overview

ChromaCat's pattern system is built on several key concepts:

- **Normalized Coordinates**: All patterns work in a normalized coordinate space (-0.5 to 0.5)
- **Animation Time**: Patterns can use the global time value for animation
- **Value Range**: All patterns output values between 0.0 and 1.0 which are mapped to colors
- **Parameter Modulation**: Most parameters can be animated or modulated for dynamic effects

Common parameters available to all patterns:

- `frequency` (0.1-10.0): Base pattern frequency
- `amplitude` (0.1-2.0): Pattern intensity
- `speed` (0.0-1.0): Animation speed multiplier

## Core Patterns

### Horizontal Gradient

The simplest but most versatile pattern, perfect for text highlighting and clean transitions.

```bash
chromacat -p horizontal --param invert=false
```

**Parameters:**

- `invert` (boolean): Reverse gradient direction

**Creative Uses:**

- Basic text highlighting with clean, readable gradients
- Foundation for layered effects when combined with other patterns
- Status bar and progress indicators
- Gentle color flow for logs and output

**Tips:**

- Use with `--smooth` for buttery transitions
- Combine with high-contrast themes for emphasis
- Great for readability when using light themes

### Diagonal Gradient

A dynamic angled gradient that adds visual interest to text.

```bash
chromacat -p diagonal --param "angle=45,frequency=1.0"
```

**Parameters:**

- `angle` (0-360): Direction of the gradient
- `frequency` (0.1-10.0): Number of gradient repeats

**Creative Uses:**

- Direction-based emphasis
- Creating a sense of motion
- Highlighting important sections
- Simulating light sources

**Tips:**

- Use shallow angles (15-30°) for subtle effects
- Higher frequencies create striped patterns
- Combine with animation for flowing effects

## Wave-Based Patterns

### Wave Pattern

Creates fluid, undulating waves of color perfect for dynamic displays.

```bash
chromacat -p wave --param "amplitude=1.0,frequency=2.0,phase=0.0,offset=0.5,base_freq=1.0"
```

**Parameters:**

- `amplitude` (0.1-2.0): Height of the waves
- `frequency` (0.1-5.0): Number of waves
- `phase` (0.0-2π): Wave offset
- `offset` (0.0-1.0): Vertical position
- `base_freq` (0.1-10.0): Animation speed multiplier

**Creative Uses:**

- Audio visualizer-style effects
- Smooth transitions
- Water and fluid simulations
- Rhythmic animations

**Advanced Techniques:**

1. Sound Wave Simulation:

   ```bash
   # Create audio-like waveforms
   chromacat -p wave --param "amplitude=0.8,frequency=3.0,phase=0.0"
   ```

2. Ocean Waves:

   ```bash
   # Layered waves with different speeds
   chromacat -p wave --param "amplitude=0.5,frequency=1.5,base_freq=0.7"
   ```

3. Interference Patterns:
   ```bash
   # Complex wave interactions
   chromacat -p wave --param "frequency=2.5,phase=1.57,base_freq=1.2"
   ```

### Ripple Pattern

Creates concentric waves emanating from a central point.

```bash
chromacat -p ripple --param "center_x=0.5,center_y=0.5,wavelength=1.0,damping=0.5,frequency=1.0"
```

**Parameters:**

- `center_x` (0.0-1.0): Horizontal center of ripples
- `center_y` (0.0-1.0): Vertical center of ripples
- `wavelength` (0.1-5.0): Distance between ripple peaks
- `damping` (0.0-1.0): How quickly ripples fade with distance
- `frequency` (0.1-10.0): Animation speed

**Creative Uses:**

- Impact effects
- Radar/sonar displays
- Emphasis animations
- Water drop effects

**Advanced Techniques:**

1. Raindrop Effect:

   ```bash
   # Quick, tight ripples with high damping
   chromacat -p ripple --param "wavelength=0.5,damping=0.8,frequency=2.0"
   ```

2. Energy Pulse:

   ```bash
   # Slow, powerful waves with low damping
   chromacat -p ripple --param "wavelength=2.0,damping=0.2,frequency=0.5"
   ```

3. Multiple Sources:
   ```bash
   # Offset center for interesting interference
   chromacat -p ripple --param "center_x=0.3,center_y=0.7,wavelength=1.0"
   ```

## Complex Patterns

### Plasma Pattern

Creates psychedelic, organic flowing patterns reminiscent of plasma globes.

```bash
chromacat -p plasma --param "complexity=3.0,scale=1.5,frequency=1.0,blend_mode=add"
```

**Parameters:**

- `complexity` (1.0-10.0): Number of plasma layers
- `scale` (0.1-5.0): Size of plasma features
- `frequency` (0.1-10.0): Animation speed
- `blend_mode`: How layers combine (`add`/`multiply`/`max`)

**Creative Uses:**

- Psychedelic backgrounds
- Energy field effects
- Dynamic transitions
- Abstract art generation

**Advanced Techniques:**

1. Alien Energy:

   ```bash
   # High complexity with multiplicative blending
   chromacat -p plasma --param "complexity=7.0,scale=2.0,blend_mode=multiply"
   ```

2. Smooth Flow:

   ```bash
   # Low complexity with large scale
   chromacat -p plasma --param "complexity=2.0,scale=3.0,frequency=0.5"
   ```

3. Electric Field:
   ```bash
   # High frequency with additive blending
   chromacat -p plasma --param "complexity=4.0,frequency=3.0,blend_mode=add"
   ```

### Perlin Noise

Generates smooth, natural-looking random patterns.

```bash
chromacat -p perlin --param "octaves=4,persistence=0.5,scale=1.0,seed=0"
```

**Parameters:**

- `octaves` (1-8): Detail levels in the noise
- `persistence` (0.0-1.0): How much detail carries through
- `scale` (0.1-5.0): Size of noise features
- `seed`: Random seed for consistent patterns

**Creative Uses:**

- Natural texture generation
- Smooth random transitions
- Cloud and smoke effects
- Terrain-like patterns

**Advanced Techniques:**

1. Cloud Cover:

   ```bash
   # Smooth, large-scale noise
   chromacat -p perlin --param "octaves=3,persistence=0.7,scale=2.0"
   ```

2. Detailed Texture:

   ```bash
   # High detail with strong persistence
   chromacat -p perlin --param "octaves=6,persistence=0.8,scale=1.0"
   ```

3. Gentle Flow:
   ```bash
   # Low octaves with large scale
   chromacat -p perlin --param "octaves=2,persistence=0.3,scale=3.0"
   ```

## Geometric Patterns

### Spiral Pattern

Creates hypnotic spiral patterns rotating from the center.

```bash
chromacat -p spiral --param "density=1.0,rotation=0,expansion=1.0,clockwise=true,frequency=1.0"
```

**Parameters:**

- `density` (0.1-5.0): How tightly wound the spiral is
- `rotation` (0-360): Base rotation angle
- `expansion` (0.1-2.0): How quickly spiral expands
- `clockwise` (boolean): Rotation direction
- `frequency` (0.1-10.0): Animation speed

**Creative Uses:**

- Loading animations
- Hypnotic effects
- Circular progress indicators
- Vortex simulations

**Advanced Techniques:**

1. Tight Vortex:

   ```bash
   # Dense, fast-moving spiral
   chromacat -p spiral --param "density=3.0,expansion=0.5,frequency=2.0"
   ```

2. Galaxy Arm:

   ```bash
   # Loose, elegant spiral
   chromacat -p spiral --param "density=0.5,expansion=1.5,frequency=0.5"
   ```

3. Double Helix:
   ```bash
   # Medium density with slow rotation
   chromacat -p spiral --param "density=2.0,rotation=180,frequency=0.7"
   ```

### Diamond Pattern

Creates diamond-shaped patterns with dynamic animations.

```bash
chromacat -p diamond --param "size=1.0,offset=0.5,sharpness=2.0,rotation=45,speed=1.0,mode=zoom"
```

**Parameters:**

- `size` (0.1-5.0): Size of diamond shapes
- `offset` (0.0-1.0): Pattern offset
- `sharpness` (0.1-5.0): Edge definition
- `rotation` (0-360): Pattern rotation
- `speed` (0.0-5.0): Animation speed
- `mode`: Animation type (`zoom`/`scroll`/`static`)

**Creative Uses:**

- Geometric backgrounds
- Crystalline effects
- Dynamic transitions
- Abstract patterns

**Advanced Techniques:**

1. Crystal Formation:

   ```bash
   # Sharp edges with zoom animation
   chromacat -p diamond --param "size=2.0,sharpness=4.0,mode=zoom"
   ```

2. Flowing Gems:

   ```bash
   # Smooth edges with scroll animation
   chromacat -p diamond --param "size=1.5,sharpness=1.0,mode=scroll"
   ```

3. Static Mosaic:
   ```bash
   # Large diamonds with rotation
   chromacat -p diamond --param "size=3.0,rotation=30,mode=static"
   ```

### Checkerboard Pattern

Creates a dynamic checkerboard pattern with various effects.

```bash
chromacat -p checkerboard --param "size=2,blur=0.1,rotation=45,scale=1.0"
```

**Parameters:**

- `size` (1-10): Size of checker squares
- `blur` (0.0-1.0): Edge softness
- `rotation` (0-360): Pattern rotation
- `scale` (0.1-5.0): Overall pattern size

**Creative Uses:**

- Pattern backgrounds
- Transition effects
- Texture generation
- Grid-based displays

**Advanced Techniques:**

1. Soft Grid:

   ```bash
   # Large squares with high blur
   chromacat -p checkerboard --param "size=4,blur=0.8,scale=1.5"
   ```

2. Sharp Diamonds:

   ```bash
   # Rotated checkers with no blur
   chromacat -p checkerboard --param "size=2,blur=0.0,rotation=45"
   ```

3. Fine Pattern:
   ```bash
   # Small squares with medium blur
   chromacat -p checkerboard --param "size=8,blur=0.3,scale=0.5"
   ```

## Dynamic Effects

### Digital Rain

Creates a Matrix-style digital rain effect.

```bash
chromacat -p rain --param "speed=1.0,density=1.0,length=3.0,glitch=true,glitch_freq=1.0"
```

**Parameters:**

- `speed` (0.1-5.0): Rain fall speed
- `density` (0.1-2.0): Amount of rain
- `length` (1.0-10.0): Length of rain streaks
- `glitch` (boolean): Enable glitch effects
- `glitch_freq` (0.1-5.0): Frequency of glitches

**Creative Uses:**

- Cyberpunk effects
- Data visualization
- Loading screens
- Digital transitions

**Advanced Techniques:**

1. Heavy Downpour:

   ```bash
   # Fast, dense rain
   chromacat -p rain --param "speed=3.0,density=1.8,length=5.0"
   ```

2. Glitch Storm:

   ```bash
   # Moderate rain with heavy glitching
   chromacat -p rain --param "speed=1.5,glitch=true,glitch_freq=3.0"
   ```

3. Light Drizzle:
   ```bash
   # Slow, sparse rain
   chromacat -p rain --param "speed=0.5,density=0.5,length=2.0"
   ```

### Fire Effect

Creates dynamic fire simulation with realistic movement.

```bash
chromacat -p fire --param "intensity=1.0,speed=1.0,turbulence=0.5,height=1.0,wind=true,wind_strength=0.3"
```

**Parameters:**

- `intensity` (0.1-2.0): Brightness of flames
- `speed` (0.1-5.0): Animation speed
- `turbulence` (0.0-1.0): Flame chaos
- `height` (0.1-2.0): Flame height
- `wind` (boolean): Enable wind effect
- `wind_strength` (0.0-1.0): Wind intensity

**Creative Uses:**

- Dynamic backgrounds
- Loading effects
- Energy visualization
- Atmospheric effects

**Advanced Techniques:**

1. Raging Inferno:

   ```bash
   # Intense, turbulent flames
   chromacat -p fire --param "intensity=1.8,turbulence=0.9,height=1.5"
   ```

2. Gentle Campfire:

   ```bash
   # Low, steady flames
   chromacat -p fire --param "intensity=0.8,turbulence=0.3,height=0.7"
   ```

3. Windswept Blaze:
   ```bash
   # Strong wind effect
   chromacat -p fire --param "wind=true,wind_strength=0.8,turbulence=0.6"
   ```

### Aurora Borealis

Simulates the flowing curtains of the northern lights.

```bash
chromacat -p aurora --param "intensity=1.0,speed=1.0,waviness=1.0,layers=3,height=0.5,spread=0.3"
```

**Parameters:**

- `intensity` (0.1-2.0): Brightness of aurora
- `speed` (0.1-5.0): Movement speed
- `waviness` (0.1-2.0): Amount of wave distortion
- `layers` (1-5): Number of aurora curtains
- `height` (0.1-1.0): Vertical thickness
- `spread` (0.1-1.0): Vertical spacing between curtains

**Creative Uses:**

- Ethereal backgrounds
- Atmospheric effects
- Flowing transitions
- Dream sequences

**Advanced Techniques:**

1. Cosmic Dance:

   ```bash
   # Multiple dynamic curtains
   chromacat -p aurora --param "layers=4,waviness=1.5,speed=1.2,spread=0.4"
   ```

2. Subtle Shimmer:

   ```bash
   # Gentle, low-key effect
   chromacat -p aurora --param "intensity=0.7,speed=0.5,waviness=0.8,layers=2"
   ```

3. Storm of Lights:
   ```bash
   # Intense, chaotic display
   chromacat -p aurora --param "intensity=1.8,speed=2.0,waviness=2.0,layers=5"
   ```

**Tips for Aurora:**

- Use with blue/green/purple themes for realistic effects
- Lower speeds create more mystical atmospheres
- Higher waviness works well for energetic displays
- Combine layers with different spreads for depth

### Kaleidoscope

Creates mesmerizing symmetrical patterns with dynamic animations.

```bash
chromacat -p kaleidoscope --param "segments=6,rotation_speed=1.0,zoom=1.5,complexity=2.0,color_flow=1.0,distortion=0.3"
```

**Parameters:**

- `segments` (3-12): Number of mirror segments
- `rotation_speed` (0.1-5.0): Speed of pattern rotation
- `zoom` (0.5-3.0): Pattern scale
- `complexity` (1.0-5.0): Detail level of patterns
- `color_flow` (0.0-2.0): Color transition speed
- `distortion` (0.0-1.0): Amount of pattern warping

**Creative Uses:**

- Psychedelic displays
- Mandala generation
- Loading animations
- Abstract art

**Advanced Techniques:**

1. Crystal Kaleidoscope:

   ```bash
   # Sharp, geometric patterns
   chromacat -p kaleidoscope --param "segments=8,complexity=4.0,distortion=0.1"
   ```

2. Flowing Mandala:

   ```bash
   # Smooth, organic movement
   chromacat -p kaleidoscope --param "segments=6,rotation_speed=0.5,color_flow=1.5,distortion=0.4"
   ```

3. Fractal Explosion:
   ```bash
   # Complex, dynamic patterns
   chromacat -p kaleidoscope --param "segments=12,complexity=5.0,zoom=2.0,color_flow=2.0"
   ```

## Advanced Pattern Usage

### Pattern Combinations

ChromaCat's patterns can be enhanced by thoughtful theme selection and parameter combinations. Here are some powerful combinations:

1. Fire + Kaleidoscope Theme:

   ```bash
   chromacat -p fire --param "intensity=1.5" -t kaleidoscope
   ```

   Creates a mesmerizing symmetrical inferno effect.

2. Aurora + Wave Pattern:

   ```bash
   chromacat -p aurora --param "layers=3" -t wave
   ```

   Produces flowing, wave-like northern lights.

3. Digital Rain + Neon Theme:
   ```bash
   chromacat -p rain --param "density=1.5" -t neon
   ```
   Creates vibrant cyberpunk effects.

### Animation Tips

1. Speed Modulation:

   - Use slower speeds (0.3-0.7) for hypnotic effects
   - Higher speeds (1.5-2.0) work well for energetic displays
   - Very slow speeds (0.1-0.3) create subtle ambient movement

2. Smooth Transitions:

   ```bash
   chromacat --smooth -p wave --param "frequency=0.5"
   ```

   Enables frame interpolation for buttery-smooth animation.

3. Pattern Timing:
   - Match animation speed to content type
   - Use faster animations for alerts/notifications
   - Slower animations for background effects

### Theme Selection

Patterns work best with specific theme types:

1. Geometric Patterns (Diamond, Checkerboard, Spiral):

   - High contrast themes
   - Complementary colors
   - Triadic color schemes

2. Organic Patterns (Plasma, Perlin, Aurora):

   - Analogous color schemes
   - Gradient-based themes
   - Nature-inspired palettes

3. Dynamic Effects (Fire, Rain):
   - Monochromatic themes
   - Theme colors matching effect type
   - High saturation for intensity

### Performance Optimization

1. Pattern Complexity:

   - Reduce complexity for large outputs
   - Lower animation speeds for smoother performance
   - Use simpler patterns for real-time updates

2. Buffer Management:

   ```bash
   chromacat --buffer-size 4096 -p plasma
   ```

   Adjust buffer size for optimal streaming performance.

3. Resource Usage:
   - Geometric patterns are typically fastest
   - Complex effects (Fire, Aurora) use more CPU
   - Consider pattern complexity for different use cases

### Creative Applications

1. Status Displays:

   ```bash
   # Progress bar with wave effect
   progress | chromacat -p wave --param "frequency=0.5,amplitude=0.3"
   ```

2. Log Enhancement:

   ```bash
   # Error logs with fire effect
   tail -f error.log | chromacat -p fire -t heat
   ```

3. Build Output:
   ```bash
   # Build process with rain effect
   build.sh | chromacat -p rain --param "density=0.7"
   ```

### Custom Pattern Development

For developers looking to create custom patterns:

1. Pattern Interface:

   ```rust
   pub trait Pattern {
       fn generate(&self, x: f64, y: f64, time: f64, params: &PatternParams) -> f64;
   }
   ```

2. Parameter Definition:

   ```rust
   define_param!(
       num CustomPattern,
       MyParam,
       "param_name",
       "Parameter description",
       0.0, // min
       1.0, // max
       0.5  // default
   );
   ```

3. Pattern Registration:
   ```rust
   registry.register("custom", CustomPattern::new());
   ```

## Best Practices

1. Pattern Selection:

   - Match pattern to content type
   - Consider readability requirements
   - Think about animation purpose

2. Parameter Tuning:

   - Start with defaults and adjust gradually
   - Test different parameter combinations
   - Consider the viewing context

3. Theme Integration:

   - Choose themes that enhance pattern effect
   - Use contrasting colors for emphasis
   - Consider color accessibility

4. Animation Timing:
   - Match speed to user attention span
   - Use appropriate timing for context
   - Consider system performance

Remember: The most effective patterns often combine simplicity with purposeful parameter choices. Start with basic patterns and gradually explore more complex effects as you become familiar with the system.

The true power of ChromaCat comes from understanding how patterns, themes, and parameters interact to create engaging visual experiences that enhance rather than distract from your terminal content.
