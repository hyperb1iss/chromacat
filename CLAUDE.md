# ChromaCat Development Guide for AI Agents

This document encapsulates essential knowledge for AI coding agents working with the ChromaCat codebase. It focuses on architecture, patterns, and practices rather than code specifics.

## Project Philosophy

ChromaCat is a sophisticated terminal colorization tool that balances simplicity with power. It operates on the principle of **progressive disclosure** - simple use cases work immediately, while advanced features reveal themselves to those who seek them.

### Core Values
- **Performance First**: 30fps smooth animations, efficient memory usage
- **User Delight**: Beautiful defaults, instant feedback, engaging interactions
- **Extensibility**: Plugin-based patterns, theme system, modular architecture
- **Safety**: Rust's ownership model, comprehensive error handling, resource cleanup

## Architectural Overview

### System Layers
```
CLI Interface → Application Controller → Pattern Engine → Renderer → Terminal
                                     ↓
                              Theme System → Gradient Engine
```

### Mode Architecture
ChromaCat operates in three distinct modes:

1. **Static Mode**: Single-shot file processing (`chromacat file.txt`)
2. **Streaming Mode**: Real-time stdin processing (`tail -f log | chromacat`)  
3. **Playground Mode**: Interactive TUI (default for interactive terminals)

### Key Design Patterns

- **Strategy Pattern**: Swappable pattern algorithms via registry
- **Factory Pattern**: Pattern creation by name
- **Builder Pattern**: Configuration builders for patterns/animations
- **Observer Pattern**: Event-driven playground UI
- **RAII Pattern**: Automatic resource cleanup via Drop traits

## Component Architecture

### Core Components

**ChromaCat (app.rs)**
- Application orchestrator
- Terminal setup/cleanup
- Mode selection logic
- Resource lifecycle management

**PatternEngine (pattern/engine.rs)**
- Pattern value calculation
- Gradient color mapping
- Animation time tracking
- Normalized coordinate system

**Renderer (renderer/core.rs)**
- Content rendering
- Playground UI management
- Event handling delegation
- State synchronization

**PlaygroundUI (renderer/playground.rs)**
- TUI overlay rendering
- Selection state management
- Toast notifications
- Terminal size tracking

### Data Flow Patterns

**Initialization Flow**:
```
CLI Args → Validation → Theme Loading → Pattern Config → Engine Creation → Renderer Setup
```

**Rendering Pipeline**:
```
Text Input → Line Processing → Pattern Calculation → Color Mapping → Terminal Output
```

**Event Processing**:
```
Keyboard/Mouse → EventLoop → Input Handler → State Update → Re-render
```

## Coding Conventions

### Error Handling
- Use `Result<T>` for all fallible operations
- Create specific error types in error modules
- Implement `From` traits for error conversion
- Provide context in error messages
- Use `?` for error propagation

### Resource Management
- Implement `Drop` for cleanup guarantees
- Use `Arc` for shared ownership
- Prefer stack allocation over heap
- Pre-allocate buffers when size is known
- Clear terminal state in all exit paths

### Validation Patterns
```rust
// Multi-layer validation approach
CLI::validate() → Pattern::validate() → Parameter::validate()
```

### Testing Strategy
- Unit tests for algorithms
- Integration tests for CLI
- Property tests for parameters
- Mock environment for terminal tests
- Performance benchmarks for critical paths

## UX Design Patterns

### Progressive Disclosure
1. Basic usage works immediately
2. `--help` reveals common options
3. `--list` shows available choices
4. `--pattern-help` provides deep details
5. Playground mode offers full control

### Feedback Mechanisms
- **Immediate**: Pattern changes render instantly
- **Contextual**: Toast messages for actions
- **Persistent**: Status bar for current state
- **Visual**: Color previews in help text

### Smart Defaults
- **Pattern**: "diagonal" (visually interesting)
- **Theme**: "rainbow" (showcases capabilities)
- **Mode**: Playground (if terminal supports)
- **FPS**: 30 (smooth without excess CPU)

## Playground Mode Architecture

### Component Layout
```
┌─────────────────────────────────────┐
│          Pattern Display            │
│                                     │
├─────────────────────────────────────┤
│ Patterns │ Params │ Themes │ Art   │ ← Overlay (toggle with ;)
├─────────────────────────────────────┤
│         Status Bar                  │
└─────────────────────────────────────┘
```

### Event Loop
- 30fps animation loop
- Non-blocking event polling (1ms timeout)
- Double-buffered rendering
- Dirty region optimization

### Input Handling
- **Tab**: Cycle sections
- **Arrows**: Navigate lists
- **Enter**: Apply selection
- **;**: Toggle overlay
- **q/Esc**: Exit

## Pattern System

### Pattern Types
- **Geometric**: diagonal, horizontal, vertical, radial
- **Wave-based**: wave, ripple, spiral
- **Noise-based**: perlin, plasma
- **Complex**: aurora (turbulence + folding)

### Parameter System
- Type-safe parameter definitions
- Range validation with min/max
- Default values for all parameters
- Compile-time registration via macros

### Animation Model
- Time-based evolution
- Speed parameter controls rate
- Frame delta for smooth motion
- Pattern-specific animation logic

## Theme System

### Theme Structure
```yaml
name: "theme_name"
desc: "Description"
author: "Author"
colors:
  - [r, g, b]  # 0.0-1.0 RGB values
```

### Gradient Generation
- Interpolation between color stops
- 256-color terminal fallback
- True color (24-bit) support
- Smooth color transitions

## Performance Optimizations

### Memory Management
- Pre-allocated buffers
- Object pooling for patterns
- Arc-shared gradients
- Capacity-aware growth

### Rendering Optimizations
- Double buffering
- Dirty region tracking
- Batch terminal writes
- Lazy state updates

### Computation Optimizations
- Lookup tables for trigonometry
- SIMD-friendly data layout
- Cache-aware access patterns
- Minimal allocations in hot paths

## Security Practices

### Input Validation
- CLI argument validation
- File path sanitization
- Parameter range checking
- Theme file validation

### Resource Safety
- Automatic cleanup via RAII
- Bounds checking on all arrays
- No unsafe code in core logic
- Panic-safe error handling

### Terminal Safety
- State restoration on exit
- Signal handling for cleanup
- Graceful degradation
- Feature detection

## Development Workflow

### Adding New Patterns
1. Create pattern module in `src/pattern/patterns/`
2. Implement Pattern trait
3. Define parameters with macros
4. Register in pattern registry
5. Add tests for parameter validation

### Adding New Themes
1. Create YAML file in `themes/`
2. Define color stops
3. Test gradient generation
4. Add to theme categories

### Modifying Playground UI
1. Update PlaygroundUI struct
2. Modify render methods
3. Update input handling
4. Test with different terminal sizes

## Common Pitfalls to Avoid

1. **Terminal State**: Always restore terminal on exit
2. **Resource Leaks**: Use RAII for all resources
3. **Panic Handling**: Set panic hook for cleanup
4. **Color Space**: Validate RGB ranges (0.0-1.0)
5. **Unicode**: Handle multi-byte characters properly
6. **Performance**: Profile before optimizing
7. **Testing**: Test with redirected stdin/stdout

## Key Files and Responsibilities

- `src/app.rs` - Application orchestration
- `src/cli.rs` - Argument parsing and validation
- `src/pattern/engine.rs` - Pattern calculation engine
- `src/renderer/core.rs` - Main renderer logic
- `src/renderer/playground.rs` - TUI components
- `src/renderer/event_loop.rs` - Animation loop
- `src/themes.rs` - Theme loading and management
- `src/error.rs` - Error type definitions

## Testing Approach

### Test Categories
- **Unit**: Individual functions and methods
- **Integration**: CLI and configuration
- **System**: End-to-end workflows
- **Performance**: Benchmarks for critical paths

### Test Environment
- Mock terminal dimensions (80x24)
- Disabled playground mode (`RUST_TEST=1`)
- Isolated theme loading
- Predictable random seeds

## Future Architecture Considerations

### Potential Enhancements
- GPU acceleration for patterns
- Network streaming support
- Plugin system for patterns
- Configuration persistence
- Collaborative features

### Scalability Points
- Pattern calculation parallelization
- Streaming buffer optimization
- Theme caching system
- Terminal capability detection

## Quick Reference

### Command Patterns
```bash
# Basic usage
chromacat file.txt

# With pattern and theme
chromacat -p plasma -t neon file.txt

# Playground mode
chromacat  # Opens with demo content

# List available options
chromacat --list
```

### Common Tasks
- **Run playground**: `cargo run`
- **Run tests**: `cargo test`
- **Check lints**: `cargo clippy`
- **Format code**: `cargo fmt`

## Philosophy for AI Agents

When working with ChromaCat:

1. **Respect the architecture**: Understand component boundaries
2. **Maintain type safety**: Let the compiler catch errors
3. **Preserve UX quality**: Test visual output and interactions
4. **Follow patterns**: Use existing patterns for consistency
5. **Document intent**: Code should be self-explanatory
6. **Test thoroughly**: Cover edge cases and error paths
7. **Profile first**: Don't optimize without measurements

ChromaCat exemplifies how terminal applications can be both powerful and delightful. Maintain this balance in all modifications.