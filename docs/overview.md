# 🌈 **ChromaCat** 🌈
*Enhance Your Terminal with Vibrant Gradients*

---

## 📚 Table of Contents

1. [Project Overview](#project-overview)
2. [Features](#features)
3. [Technical Specifications](#technical-specifications)
   - [Architecture](#architecture)
   - [Dependencies](#dependencies)
   - [Data Flow](#data-flow)
4. [Installation](#installation)
5. [Usage Guide](#usage-guide)
   - [Command-Line Interface](#command-line-interface)
   - [Examples](#examples)
6. [Built-in Themes](#built-in-themes)
7. [Customization Options](#customization-options)
8. [Error Handling](#error-handling)
9. [Testing](#testing)
10. [Performance Considerations](#performance-considerations)
11. [Future Enhancements](#future-enhancements)
12. [Contribution Guidelines](#contribution-guidelines)
13. [License](#license)
14. [Acknowledgements](#acknowledgements)

---

## 🎨 Project Overview

**ChromaCat** is a versatile and user-friendly command-line tool written in Rust, inspired by the popular `lolcat` utility. It transforms your terminal output by applying vibrant and customizable color gradients to text, making your command-line interactions more visually appealing. Unlike traditional tools that offer simple colorization, ChromaCat provides advanced gradient options, including horizontal and diagonal gradients across multi-line text, numerous built-in themes, and extensive customization capabilities.

---

## ✨ Features

- **Gradient Colorization:**
  - **Horizontal Gradients:** Smooth left-to-right color transitions within each line.
  - **Diagonal Gradients:** Dynamic color shifts across multiple lines based on specified angles.
  
- **Built-in Themes:**
  - A diverse collection of predefined color gradients (e.g., Rainbow, Ocean, Forest, Neon).
  
- **Customization Options:**
  - **Gradient Cycling:** Enable infinite cycling of gradient colors.
  - **Custom Gradient Angles:** Adjust the angle of diagonal gradients for varied effects.
  - **Color Output Control:** Enable or disable colored output based on user preference or terminal capabilities.
  
- **Input Flexibility:**
  - Accepts input via standard input (stdin) or from specified files.
  
- **Unicode Support:**
  - Accurate handling of Unicode characters, including multi-byte and grapheme clusters.
  
- **Performance Optimizations:**
  - Efficient processing of large text inputs with minimal latency.
  
- **Cross-Platform Compatibility:**
  - Seamless operation across major operating systems (Linux, macOS, Windows).
  
- **Intuitive Command-Line Interface:**
  - User-friendly flags and options for easy navigation and usage.

---

## 💻 Technical Specifications

### 🏗 Architecture

ChromaCat follows a **modular architecture** to ensure scalability, maintainability, and ease of development. The primary components include:

1. **CLI Parser:**
   - Utilizes the [`clap`](https://crates.io/crates/clap) crate to parse and manage command-line arguments and options.

2. **Gradient Engine:**
   - Leverages the [`colorgrad`](https://crates.io/crates/colorgrad) crate to generate and manage color gradients based on selected themes and user customizations.

3. **Input Handler:**
   - Manages input sources, reading from stdin or specified files with proper buffering.

4. **Colorizer:**
   - Processes text lines and applies the appropriate color transformations using ANSI escape codes, supporting both horizontal and diagonal gradients.

5. **Output Renderer:**
   - Handles colored and non-colored output to the terminal using the [`termcolor`](https://crates.io/crates/termcolor) crate, ensuring compatibility with various terminal types.

6. **Utility Modules:**
   - Includes helper functions for tasks like Unicode grapheme segmentation, angle validations, and error reporting.

### 📦 Dependencies

ChromaCat relies on several Rust crates to provide its functionalities:

- [`clap`](https://crates.io/crates/clap) (v4.1): Command-line argument parsing and management.
- [`colorgrad`](https://crates.io/crates/colorgrad) (v0.3): Creation and handling of color gradients.
- [`termcolor`](https://crates.io/crates/termcolor) (v1.2): Colored terminal output management.
- [`atty`](https://crates.io/crates/atty) (v0.2): Detection of terminal streams (e.g., stdout).
- [`unicode-segmentation`](https://crates.io/crates/unicode-segmentation) (v1.10): Accurate handling of Unicode grapheme clusters.
- [`serde`](https://crates.io/crates/serde) (Optional): For potential future features like configuration files.

### 🔄 Data Flow

1. **Initialization:**
   - The application starts by parsing command-line arguments using `clap`.

2. **Input Acquisition:**
   - Reads input from a file or stdin based on user input.

3. **Gradient Selection:**
   - Chooses a gradient theme from built-in options or custom specifications.

4. **Gradient Configuration:**
   - Applies user-defined settings like cycling and angle adjustments.

5. **Text Processing:**
   - Iterates through each character or grapheme cluster in the input, calculating its position-based gradient factor (`t`).

6. **Color Application:**
   - Assigns colors to each character based on the gradient factor using ANSI escape codes.

7. **Output Rendering:**
   - Displays the colorized text in the terminal, respecting user settings like color disabling.

---

## 🛠 Installation

ChromaCat can be installed using various methods, ensuring accessibility across different environments.

### 📝 Prerequisites

- **Rust Toolchain:** Ensure that Rust is installed on your system. If not, install it from [rustup.rs](https://rustup.rs/).

### 📥 Installation Methods

1. **Using Cargo (Recommended):**

   Install ChromaCat directly from [crates.io](https://crates.io/) using Cargo:

   ```bash
   cargo install chromacat
   ```

   This command downloads, compiles, and installs ChromaCat, making the `chromacat` executable available in your `$PATH`.

2. **From Source:**

   Clone the repository and build the project manually:

   ```bash
   git clone https://github.com/yourusername/chromacat.git
   cd chromacat
   cargo build --release
   ```

   The executable will be located at `target/release/chromacat`.

3. **Package Managers:**

   - **Homebrew (macOS and Linux):**

     ```bash
     brew install yourusername/tap/chromacat
     ```

   - **Other Package Managers:**

     Currently, ChromaCat may not be available on other package managers. Contributions to packaging for platforms like `apt`, `yum`, or `scoop` are welcome.

---

## 🚀 Usage Guide

ChromaCat provides a rich set of command-line options to customize text colorization. Below is a detailed guide on how to use ChromaCat effectively.

### 🔧 Command-Line Interface

```bash
chromacat [OPTIONS]
```

### ⚙️ Options

| Flag                | Short | Description                                                                                      | Default   |
|---------------------|-------|--------------------------------------------------------------------------------------------------|-----------|
| `--theme <THEME>`   | `-t`  | Select a built-in theme for the gradient.                                                       | `rainbow` |
| `--cycle`           | `-c`  | Enable infinite cycling of gradient colors.                                                      | `false`   |
| `--input <FILE>`    | `-i`  | Specify an input file to read text from. If not provided, reads from standard input (stdin).     | `None`    |
| `--no-color`        | `-n`  | Disable colored output, outputting plain text instead.                                         | `false`   |
| `--diagonal`        | `-d`  | Enable diagonal gradient mode across multiple lines.                                            | `false`   |
| `--angle <DEGREES>` | `-a`  | Adjust the angle of the diagonal gradient in degrees (applicable only if `--diagonal` is enabled).| `45`      |
| `--help`            | `-h`  | Show help information.                                                                           | `false`   |
| `--version`         | `-V`  | Show version information.                                                                        | `false`   |

### 📂 Examples

1. **Basic Horizontal Gradient:**

   Apply the default rainbow gradient to text from stdin.

   ```bash
   echo "Hello, ChromaCat!" | chromacat
   ```

2. **Selecting a Theme:**

   Choose the `ocean` theme for colorization.

   ```bash
   echo "Hello, Ocean!" | chromacat --theme ocean
   ```

3. **Cycling Gradient Colors:**

   Enable infinite cycling of gradient colors.

   ```bash
   echo "Cycling Colors" | chromacat --cycle
   ```

4. **Diagonal Gradient with Default Angle:**

   Apply a diagonal gradient with a 45-degree angle.

   ```bash
   echo -e "Hello,\nThis is\nChromaCat!" | chromacat --diagonal
   ```

5. **Diagonal Gradient with Custom Angle:**

   Set a 60-degree angle for the diagonal gradient.

   ```bash
   echo -e "Diagonal\nGradient\nTest" | chromacat --diagonal --angle 60
   ```

6. **Reading from a File with a Specific Theme:**

   Colorize the contents of `example.txt` using the `forest` theme.

   ```bash
   chromacat --input example.txt --theme forest
   ```

7. **Disabling Colored Output:**

   Output plain text without any colorization.

   ```bash
   echo "No Color Output" | chromacat --no-color
   ```

8. **Displaying Help Information:**

   View all available options and usage instructions.

   ```bash
   chromacat --help
   ```

---

## 🎨 Built-in Themes

ChromaCat includes a variety of built-in themes to cater to different aesthetic preferences. Each theme represents a unique color gradient.

| **Theme**    | **Description**                                  |
|--------------|--------------------------------------------------|
| **Rainbow**  | A classic multi-color rainbow gradient.          |
| **Heat**     | Warm colors transitioning from red to yellow.    |
| **Ocean**    | Cool blues and teals inspired by the ocean.      |
| **Forest**   | Greens and earthy tones resembling a forest.     |
| **Pastel**   | Soft, muted colors for a gentle appearance.      |
| **Neon**     | Bright, vibrant colors for a striking effect.    |
| **Autumn**   | Warm oranges and browns reminiscent of fall.     |
| **Sunrise**  | *Planned*: Gradients inspired by sunrise hues.   |
| **Sunset**   | *Planned*: Gradients inspired by sunset hues.    |
| **Custom**   | *Placeholder*: User-defined custom gradients.    |

> **Note:** The `Sunrise`, `Sunset`, and `Custom` themes are placeholders for future enhancements.

---

## 🎛 Customization Options

ChromaCat offers extensive customization to tailor the colorization process to user preferences:

1. **Gradient Cycling (`--cycle`):**
   - Enables the gradient to loop infinitely, creating a continuous color transition effect across the text.

2. **Diagonal Gradient Angle (`--angle`):**
   - Allows users to set the angle of diagonal gradients between `0` and `360` degrees.
   - Affects how colors transition across multiple lines, providing flexibility in visual presentation.

3. **Disabling Colors (`--no-color`):**
   - Useful for piping output to other programs or when color display is not desired.

4. **Input Source (`--input`):**
   - Users can specify a file to read text from, offering flexibility beyond standard input.

5. **Theme Selection (`--theme`):**
   - Choose from a variety of predefined themes to quickly apply desired color schemes.

6. **Unicode Support:**
   - Proper handling of multi-byte and complex Unicode characters ensures accurate colorization without breaking grapheme clusters.

---

## ❗ Error Handling

ChromaCat includes robust error handling to ensure smooth user experiences:

1. **Invalid Gradient Angles:**
   - If the `--angle` value is outside the `0-360` degrees range, the tool exits gracefully with an informative error message.

2. **File Access Issues:**
   - If the specified input file cannot be read (due to permissions, non-existence, etc.), ChromaCat notifies the user and exits.

3. **Unsupported Themes:**
   - If a user specifies a non-existent theme, the tool defaults to the `rainbow` theme and warns the user.

4. **Terminal Compatibility:**
   - Detects if the output stream supports colored output. If not, it either disables colors automatically or respects the `--no-color` flag.

5. **Graceful Degradation:**
   - In cases where color application fails (e.g., unsupported terminals), ChromaCat falls back to plain text output to prevent breaking user workflows.

---

## 🧪 Testing

Ensuring reliability and correctness is paramount. ChromaCat employs comprehensive testing strategies:

1. **Unit Tests:**
   - Validate individual components like gradient calculations, angle validations, and color assignments.
   - Test functions handling Unicode grapheme segmentation to ensure accurate processing.

2. **Integration Tests:**
   - Verify end-to-end functionality by simulating various command-line inputs and checking the output for correct colorization.
   - Include tests for different themes, gradient modes, and customization options.

3. **Cross-Platform Testing:**
   - Ensure consistent behavior across Linux, macOS, and Windows environments.
   - Test on various terminal emulators to verify ANSI color support.

4. **Performance Testing:**
   - Assess the tool's performance with large text inputs to ensure scalability and responsiveness.

5. **Error Scenario Testing:**
   - Simulate error conditions (e.g., invalid angles, inaccessible files) to confirm proper error handling and user notifications.

---

## ⚡ Performance Considerations

ChromaCat is optimized for efficient processing, even with large text inputs:

1. **Buffered I/O:**
   - Utilizes buffered reading and writing to minimize I/O overhead, enhancing performance during large-scale text processing.

2. **Efficient Gradient Computations:**
   - Precomputes gradient color values where possible to reduce redundant calculations during colorization.

3. **Unicode Processing Optimization:**
   - Employs efficient algorithms from the [`unicode-segmentation`](https://crates.io/crates/unicode-segmentation) crate to handle complex Unicode characters without significant performance penalties.

4. **Parallel Processing (Future Enhancement):**
   - Potential integration of multi-threading to process multiple lines concurrently, further boosting performance for extremely large inputs.

---

## 🔮 Future Enhancements

To continually improve ChromaCat, the following features and optimizations are planned:

1. **Additional Built-in Themes:**
   - Introduce more themes like `Sunrise`, `Sunset`, `Midnight`, and others to expand user choices.

2. **Custom Gradient Definitions:**
   - Allow users to define custom gradients by specifying a sequence of colors via command-line arguments or configuration files.

3. **Vertical and Radial Gradients:**
   - Expand gradient options to include vertical (top-to-bottom) and radial (center-outward) gradients for diverse visual effects.

4. **Interactive Mode:**
   - Develop an interactive mode where users can preview and adjust gradient settings in real-time.

5. **Configuration Files:**
   - Enable users to save default settings and custom themes in configuration files (e.g., TOML, YAML) for persistent customization.

6. **Advanced Color Options:**
   - Support additional color models (e.g., HSL, HSV) for more granular color manipulation.

7. **Performance Optimizations:**
   - Implement multi-threading or asynchronous processing to enhance performance further, especially for very large text inputs.

8. **Shell Integration:**
   - Provide shell aliases or functions to integrate ChromaCat seamlessly into user workflows.

9. **GUI Frontend:**
   - Develop a graphical user interface for users who prefer not to interact via the command line.

10. **Enhanced Documentation:**
    - Expand documentation with tutorials, FAQs, and detailed usage scenarios to assist users in maximizing ChromaCat's potential.

---

## 🤝 Contribution Guidelines

ChromaCat welcomes contributions from the community. To maintain a collaborative and productive environment, please adhere to the following guidelines:

1. **Fork the Repository:**
   - Create a personal fork of the ChromaCat repository to develop and test your changes.

2. **Branching:**
   - Use descriptive branch names (e.g., `feature/custom-themes`, `bugfix/angle-validation`) to organize your work.

3. **Commit Messages:**
   - Write clear and concise commit messages that describe the changes made.

4. **Pull Requests:**
   - Submit pull requests from your forked repository to the main repository.
   - Ensure that your code adheres to the project's coding standards and passes all tests.

5. **Code Reviews:**
   - Participate in the code review process by providing constructive feedback and addressing requested changes promptly.

6. **Issue Reporting:**
   - Report bugs or suggest features by creating issues with detailed descriptions and, if applicable, reproduction steps.

7. **Documentation:**
   - Update or add documentation as needed to reflect your contributions and ensure comprehensive coverage.

8. **Respect and Collaboration:**
   - Maintain a respectful and collaborative attitude in all interactions. Follow the project's Code of Conduct.

---

## 📜 License

ChromaCat is released under the **MIT License**, granting users extensive freedom to use, modify, and distribute the software. A copy of the license is included in the project repository.

---

## 🙏 Acknowledgements

ChromaCat leverages several open-source Rust crates and tools that make its functionalities possible:

- [**clap**](https://crates.io/crates/clap) for command-line argument parsing.
- [**colorgrad**](https://crates.io/crates/colorgrad) for creating and managing color gradients.
- [**termcolor**](https://crates.io/crates/termcolor) for handling colored terminal output.
- [**atty**](https://crates.io/crates/atty) for detecting terminal streams.
- [**unicode-segmentation**](https://crates.io/crates/unicode-segmentation) for accurate Unicode character handling.
- [**lolcat**](https://github.com/busyloop/lolcat) for inspiring the initial concept of colorizing terminal output.

Special thanks to the Rust community for their continuous contributions and support, enabling the development of powerful and efficient tools like ChromaCat.

---

## 🌟 Conclusion

**ChromaCat** stands as a robust and aesthetically pleasing tool for enhancing terminal outputs with dynamic color gradients. Its thoughtful design, extensive customization options, and performance optimizations make it a valuable addition to any command-line enthusiast's toolkit. With a clear roadmap for future enhancements and a welcoming stance towards community contributions, ChromaCat is poised for continual growth and improvement.

Whether you're looking to add flair to your scripts, logs, or command outputs, ChromaCat provides the flexibility and functionality to transform plain text into vibrant, colorful displays.

**Happy colorizing! 🎨🐱**
```