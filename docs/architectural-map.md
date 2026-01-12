# ChromaCat Architecture Map

## 🏗️ Component Hierarchy & Relationships

```
┌─ main.rs (Entry Point)
│   └─ ChromaCat App
│       ├─ CLI Parser ────────────────── Validates & Parses Args
│       ├─ Terminal Setup ───────────── Raw Mode, Alternate Screen  
│       └─ Mode Router ──────────────── Playground vs Static/Streaming
│           │
│           ├─ [Playground Mode] ────── Interactive TUI Experience
│           │   └─ Renderer.run() ─────── Ownership Transfer
│           │       ├─ EventLoop ──────── 30fps Animation Loop
│           │       ├─ PlaygroundUI ──── Ratatui TUI Components
│           │       └─ InputHandler ──── Keyboard/Mouse Events
│           │
│           ├─ [Static Mode] ─────────── File Processing
│           │   └─ InputReader ────────── File/Stdin Content
│           │       └─ Renderer ──────── Single Frame Render
│           │
│           └─ [Streaming Mode] ──────── Real-time Processing
│               └─ StreamingInput ───── Buffered Line Processing
```

## 🎨 Pattern-Theme-Art Data Flow

```
CLI Args ──► PatternConfig ──► PatternEngine ──► Visual Output
    │             │                 │
    │             ├─ PatternParams  │
    │             └─ CommonParams   │
    │                               │
Theme File ──► GradientBuilder ────┘
    │             │
    ├─ ColorStop  │
    └─ Easing ────┘

Demo Art ──► InputReader ──► Content String ──► Renderer
    │           │                   │
    ├─ Matrix   ├─ File Reader      ├─ Line Processing
    ├─ Spiral   ├─ Demo Generator   └─ Character Mapping
    └─ Wave     └─ Stdin Reader
```

## 🔄 Rendering Pipeline

### Playground Mode (Interactive)
```
EventLoop (30fps) ──┬─► Input Events ──► InputHandler ──┬─► Pattern Change
                    │                                   ├─► Theme Change  
                    │                                   ├─► Art Change
                    │                                   └─► Parameter Adjust
                    │                                          │
                    └─► Render Tick ──► PatternEngine.update() ──┘
                                              │
                                              ▼
                        Ratatui Frame ──► PatternWidget ──► Terminal Buffer
                              │                │
                              ├─ Overlay UI   ├─ Pattern Values
                              ├─ Toast Msgs   ├─ Color Mapping
                              └─ Status Bar   └─ Character Rendering
```

### Static Mode (One-shot)
```
Content Input ──► PatternEngine ──► Color Calculation ──► Terminal Output
      │                │                    │                  │
      ├─ Lines         ├─ Pattern Value     ├─ RGB Values      ├─ ANSI Codes
      ├─ Characters    ├─ Normalized Coords ├─ Gradient        └─ Styled Text
      └─ Dimensions    └─ Time = 0          └─ Color Stops
```

## 🎛️ Component Interaction Patterns

### 1. Event Flow (Playground Mode)
```
Terminal Event ──► EventLoop ──► Renderer ──► InputHandler ──► Action
     │                │            │            │              │
     ├─ KeyEvent      ├─ Poll      ├─ Routing   ├─ Pattern     ├─ Apply Pattern
     ├─ MouseEvent    ├─ Read      ├─ State     ├─ Theme       ├─ Apply Theme
     └─ ResizeEvent   └─ Forward   └─ Update    ├─ Art         ├─ Apply Art
                                                └─ Parameter   └─ Adjust Param
```

### 2. State Updates
```
User Action ──► Component Update ──► Engine Refresh ──► Visual Refresh
     │               │                   │                 │
     ├─ Key Press    ├─ Selection        ├─ Pattern        ├─ Frame Render
     ├─ Mouse Click  ├─ Configuration    ├─ Gradient       ├─ Color Update
     └─ Selection    └─ Parameter        └─ Time Step      └─ Animation
```

### 3. Data Transformation Pipeline
```
Raw Input ──► Coordinate System ──► Pattern Function ──► Color Space ──► Terminal
    │              │                    │                  │             │
    ├─ Text        ├─ Pixel (x,y)       ├─ Pattern Value   ├─ RGB Color  ├─ ANSI
    ├─ Files       ├─ Normalized        ├─ [0.0, 1.0]      ├─ Gradient   ├─ Styled
    └─ Streams     └─ [-0.5, 0.5]       └─ + Time          └─ Mapping    └─ Output
```

## 🔧 Core Component Responsibilities

### **ChromaCat App** (Orchestrator)
- **Initializes:** Terminal state, raw mode, alternate screen
- **Routes:** Between playground/static/streaming modes  
- **Manages:** Application lifecycle, error handling, cleanup
- **Owns:** CLI configuration, terminal dimensions

### **Renderer** (Rendering Engine)
- **Coordinates:** Pattern engine, UI components, event handling
- **Maintains:** Animation state, content buffer, overlay state
- **Handles:** Frame rendering, user interactions, state transitions
- **Integrates:** Ratatui terminal UI with pattern generation

### **PatternEngine** (Color Generation)
- **Generates:** Pattern values from coordinates and time
- **Manages:** Animation timing, gradient mapping, pattern config
- **Calculates:** Normalized coordinates, pattern mathematics
- **Provides:** Thread-safe color lookups, real-time updates

### **PlaygroundUI** (Terminal Interface)
- **Renders:** Overlay panels, selection lists, toast messages
- **Manages:** Section navigation, item selection, scrolling
- **Provides:** Interactive pattern/theme/art browser
- **Handles:** Mouse clicks, keyboard navigation, layout

### **EventLoop** (Animation Controller)
- **Maintains:** 30fps render cycle, input polling
- **Coordinates:** Event handling, frame timing, delta calculation
- **Integrates:** Crossterm events with ratatui rendering
- **Manages:** Terminal lifecycle within playground mode

### **InputHandler** (Interaction Logic)
- **Processes:** Keyboard and mouse events into actions
- **Maps:** UI interactions to system changes
- **Provides:** Section navigation, item selection, shortcuts
- **Generates:** Pattern/theme/art change requests

## 🌊 Data Flow Patterns

### **Configuration Flow**
```
CLI ──► Validation ──► PatternConfig ──► PatternEngine
                  ├──► Theme Selection ──► Gradient
                  └──► Art Selection ──► Content
```

### **Animation Flow**  
```
Timer ──► Delta Time ──► Engine.update() ──► Pattern Values ──► Colors
```

### **Input Processing Flow**
```
Terminal Event ──► Action ──► State Change ──► Engine Update ──► Render
```

### **Content Processing Flow**
```
Input Source ──► InputReader ──► Content String ──► Character Matrix ──► Pattern Mapping
```

## 🧩 Terminal UI Organization (Playground Mode)

```
┌─────────────────────── Terminal Window ──────────────────────┐
│                     Pattern Background                        │
│                    (PatternWidget)                           │
│                                                              │
├─────────────────── Overlay Panel (1/4 height) ─────────────────┤
│ Patterns │ Params │ Themes │ Art     ◄── 4 Column Layout    │
│ -------- │ ------ │ ------ │ ---                            │
│ ▸wave    │ speed  │▸ocean  │▸matrix  ◄── Selection Lists    │
│  plasma  │ scale  │ fire   │ spiral                          │
│  aurora  │ phase  │ cyber  │ wave                            │
├──────────────────────────────────────────────────────────────┤
│ [Tab] switch • [↑↓] select • [Enter] apply • [q] quit       │ ◄── Controls
├──────────────────────────────────────────────────────────────┤
│ ChromaCat • Pattern: wave • Theme: ocean • [?] help         │ ◄── Status Bar
└──────────────────────────────────────────────────────────────┘
```

## 🎯 Update Cycles & Performance

### **Render Cycle (30fps)**
1. **Input Poll** (1ms timeout) → Event processing
2. **Time Update** → Animation delta calculation  
3. **Pattern Calculate** → Per-pixel color generation
4. **UI Render** → Ratatui frame composition
5. **Terminal Draw** → Buffer flush to terminal
6. **Frame Limit** → Sleep remainder to maintain 30fps

### **State Change Cycle**
1. **User Input** → Key/mouse event
2. **Action Parse** → InputHandler routing
3. **State Update** → Component modification
4. **Engine Refresh** → Pattern/gradient update
5. **Visual Update** → Next frame reflects changes

### **Memory Management**
- **Content Buffer**: Single String, reused across frames
- **Pattern Cache**: Gradient arc-shared across threads  
- **UI State**: Minimal selection indices and offsets
- **Event Buffer**: Crossterm handles internal buffering

## 🔀 Component Communication

### **Ownership Model**
```
App owns → Renderer owns → PatternEngine + PlaygroundUI
        ↓
    EventLoop owns → Renderer (consumed via move)
```

### **Data Sharing**
```
PatternEngine: Arc<Gradient> for thread-safe color access
PlaygroundUI: Owned state, no sharing needed
Content: String owned by Renderer, passed by reference
```

### **Event Propagation**
```
Terminal → EventLoop → Renderer → InputHandler → Action → Engine Update
```

This architecture balances interactive responsiveness with rendering performance, using Rust's ownership model to ensure memory safety while maintaining 30fps animation in playground mode.