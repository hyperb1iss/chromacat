#!/usr/bin/env python3
"""Release management script for ChromaCat."""

# ruff: noqa: E501
# pylint: disable=broad-exception-caught, line-too-long

import re
import shutil
import subprocess
import sys
from typing import List, Tuple

from colorama import Style, init
from wcwidth import wcswidth

# Initialize colorama for cross-platform colored output
init(autoreset=True)

# Constants
PROJECT_NAME = "ChromaCat"
REPO_NAME = "hyperb1iss/chromacat"
PROJECT_LINK = f"https://github.com/{REPO_NAME}"
ISSUE_TRACKER = f"{PROJECT_LINK}/issues"

# ANSI Color Constants
COLOR_RESET = Style.RESET_ALL
COLOR_BORDER = "\033[38;2;75;0;130m"  # Indigo
COLOR_STAR = "\033[38;2;255;20;147m"  # DeepPink
COLOR_ERROR = "\033[38;2;255;69;0m"  # OrangeRed
COLOR_SUCCESS = "\033[38;2;50;205;50m"  # LimeGreen
COLOR_BUILD_SUCCESS = "\033[38;2;65;105;225m"  # RoyalBlue
COLOR_VERSION_PROMPT = "\033[38;2;218;112;214m"  # Orchid
COLOR_STEP = "\033[38;2;72;209;204m"  # MediumTurquoise
COLOR_WARNING = "\033[38;2;255;165;0m"  # Orange

# Gradient colors for the banner
GRADIENT_COLORS = [
    (75, 0, 130),  # Indigo
    (138, 43, 226),  # BlueViolet
    (148, 0, 211),  # DarkViolet
    (199, 21, 133),  # MediumVioletRed
    (255, 20, 147),  # DeepPink
    (255, 105, 180),  # HotPink
    (218, 112, 214),  # Orchid
    (186, 85, 211),  # MediumOrchid
    (138, 43, 226),  # BlueViolet
    (75, 0, 130),  # Indigo
]


def print_colored(message: str, color: str) -> None:
    """Print a message with a specific color."""
    print(f"{color}{message}{COLOR_RESET}")


def print_step(step: str) -> None:
    """Print a step in the process with a specific color."""
    print_colored(f"\n✨ {step}", COLOR_STEP)


def print_error(message: str) -> None:
    """Print an error message with a specific color."""
    print_colored(f"❌ Error: {message}", COLOR_ERROR)


def print_success(message: str) -> None:
    """Print a success message with a specific color."""
    print_colored(f"✅ {message}", COLOR_SUCCESS)


def print_warning(message: str) -> None:
    """Print a warning message with a specific color."""
    print_colored(f"⚠️  {message}", COLOR_WARNING)


def generate_gradient(colors: List[Tuple[int, int, int]], steps: int) -> List[str]:
    """Generate a list of color codes for a smooth multi-color gradient."""
    gradient = []
    segments = len(colors) - 1
    steps_per_segment = max(1, steps // segments)

    for i in range(segments):
        start_color = colors[i]
        end_color = colors[i + 1]
        for j in range(steps_per_segment):
            t = j / steps_per_segment
            r = int(start_color[0] * (1 - t) + end_color[0] * t)
            g = int(start_color[1] * (1 - t) + end_color[1] * t)
            b = int(start_color[2] * (1 - t) + end_color[2] * t)
            gradient.append(f"\033[38;2;{r};{g};{b}m")

    return gradient


def strip_ansi(text: str) -> str:
    """Remove ANSI color codes from a string."""
    ansi_escape = re.compile(r"\x1B[@-_][0-?]*[ -/]*[@-~]")
    return ansi_escape.sub("", text)


def apply_gradient(text: str, gradient: List[str], line_number: int) -> str:
    """Apply gradient colors diagonally to text."""
    return "".join(
        f"{gradient[(i + line_number) % len(gradient)]}{char}"
        for i, char in enumerate(text)
    )


def center_text(text: str, width: int) -> str:
    """Center text, accounting for ANSI color codes and Unicode widths."""
    visible_length = wcswidth(strip_ansi(text))
    padding = max((width - visible_length) // 2, 0)
    return f"{' ' * padding}{text}{' ' * (width - padding - visible_length)}"


def center_block(block: List[str], width: int) -> List[str]:
    """Center a block of text within a given width."""
    return [center_text(line, width) for line in block]


def create_banner() -> str:
    """Create a beautiful cosmic-themed banner with diagonal gradient."""
    banner_width = 80
    content_width = banner_width - 4  # Accounting for border characters
    cosmic_gradient = generate_gradient(GRADIENT_COLORS, banner_width)

    logo = [
        " ▄▄·  ▄ .▄▄▄▄        • ▌ ▄ ·.  ▄▄▄·  ▄▄·  ▄▄▄· ▄▄▄▄▄",
        "▐█ ▌▪██▪▐█▀▄ █·▪     ·██ ▐███▪▐█ ▀█ ▐█ ▌▪▐█ ▀█ •██   ",
        "██ ▄▄██▀▐█▐▀▀▄  ▄█▀▄ ▐█ ▌▐▌▐█·▄█▀▀█ ██ ▄▄▄█▀▀█  ▐█.▪ ",
        "▐███▌██▌▐▀▐█•█▌▐█▌.▐▌██ ██▌▐█▌▐█ ▪▐▌▐███▌▐█ ▪▐▌ ▐█▌· ",
        "·▀▀▀ ▀▀▀ ·.▀  ▀ ▀█▄▀▪▀▀  █▪▀▀▀ ▀  ▀ ·▀▀▀  ▀  ▀  ▀▀▀  ",
        center_text(
            "🌌 Elevate Your Terminal with Pure Color and Art 🌌", content_width
        ),
    ]

    centered_logo = center_block(logo, content_width)

    banner = [
        center_text(
            f"{COLOR_STAR}･ ｡ ☆ ∴｡　　･ﾟ*｡★･ ∴｡　　･ﾟ*｡☆ ･ ｡ ☆ ∴｡", banner_width
        ),
        f"{COLOR_BORDER}╭{'─' * (banner_width - 2)}╮",
    ]

    for line_number, line in enumerate(centered_logo):
        gradient_line = apply_gradient(line, cosmic_gradient, line_number)
        banner.append(
            f"{COLOR_BORDER}│ {gradient_line}{' ' * (content_width - wcswidth(strip_ansi(line)))} {COLOR_BORDER}│"
        )

    release_manager_text = COLOR_STEP + "Release Manager"

    banner.extend(
        [
            f"{COLOR_BORDER}╰{'─' * (banner_width - 2)}╯",
            center_text(
                f"{COLOR_STAR}∴｡　　･ﾟ*｡☆ {release_manager_text}{COLOR_STAR} ☆｡*ﾟ･　 ｡∴",
                banner_width,
            ),
            center_text(
                f"{COLOR_STAR}･ ｡ ☆ ∴｡　　･ﾟ*｡★･ ∴｡　　･ﾟ*｡☆ ･ ｡ ☆ ∴｡", banner_width
            ),
        ]
    )

    return "\n".join(banner)


def print_logo() -> None:
    """Print the banner/logo for the release manager."""
    print(create_banner())


def check_tool_installed(tool_name: str) -> None:
    """Check if a tool is installed."""
    if shutil.which(tool_name) is None:
        print_error(f"{tool_name} is not installed. Please install it and try again.")
        sys.exit(1)


def check_branch() -> None:
    """Ensure we're on the main branch."""
    current_branch = (
        subprocess.check_output(["git", "rev-parse", "--abbrev-ref", "HEAD"])
        .decode()
        .strip()
    )
    if current_branch != "main":
        print_error("You must be on the main branch to release.")
        sys.exit(1)


def check_uncommitted_changes() -> None:
    """Check for uncommitted changes."""
    result = subprocess.run(
        ["git", "diff-index", "--quiet", "HEAD", "--"], capture_output=True
    )
    if result.returncode != 0:
        print_error(
            "You have uncommitted changes. Please commit or stash them before releasing."
        )
        sys.exit(1)


def get_current_version() -> str:
    """Get the current version from Cargo.toml."""
    with open("Cargo.toml", "r") as f:
        content = f.read()
    match = re.search(r'version\s*=\s*"(\d+\.\d+\.\d+)"', content)
    if match:
        return match.group(1)
    print_error("Could not find version in Cargo.toml")
    sys.exit(1)


def update_version(new_version: str) -> None:
    """Update the version in Cargo.toml."""
    with open("Cargo.toml", "r") as f:
        content = f.read()
    updated_content = re.sub(
        r'^(version\s*=\s*)"(\d+\.\d+\.\d+)"',
        f'\\1"{new_version}"',
        content,
        flags=re.MULTILINE,
    )
    with open("Cargo.toml", "w") as f:
        f.write(updated_content)
    print_success(f"Updated version in Cargo.toml to {new_version}")


def run_checks() -> None:
    """Run cargo check and cargo test."""
    print_step("Running cargo check")
    subprocess.run(["cargo", "check"], check=True)
    print_step("Running cargo test")
    subprocess.run(["cargo", "test"], check=True)
    print_success("All checks passed")


def show_changes() -> bool:
    """Show changes and ask for confirmation."""
    print_warning("The following files will be modified:")
    subprocess.run(["git", "status", "--porcelain"])
    confirmation = input(
        f"{COLOR_VERSION_PROMPT}Do you want to proceed with these changes? (y/N): {COLOR_RESET}"
    ).lower()
    return confirmation == "y"


def commit_and_push(version: str) -> None:
    """Commit and push changes to the repository."""
    print_step("Committing and pushing changes")
    try:
        subprocess.run(["git", "add", "Cargo.*"], check=True)
        subprocess.run(
            ["git", "commit", "-m", f":rocket: Release version {version}"], check=True
        )
        subprocess.run(["git", "push"], check=True)
        subprocess.run(["git", "tag", f"v{version}"], check=True)
        subprocess.run(["git", "push", "--tags"], check=True)
        print_success(f"Changes committed and pushed for version {version}")
    except subprocess.CalledProcessError as e:
        print_error(f"Git operations failed: {str(e)}")
        sys.exit(1)


def is_valid_version(version: str) -> bool:
    """Validate version format."""
    return re.match(r"^\d+\.\d+\.\d+$", version) is not None


def main() -> None:
    """Main function to handle the release process."""
    print_logo()
    print_step(f"Starting release process for {PROJECT_NAME}")

    for tool in ["git", "cargo"]:
        check_tool_installed(tool)

    check_branch()
    check_uncommitted_changes()

    current_version = get_current_version()
    new_version = input(
        f"{COLOR_VERSION_PROMPT}Current version is {current_version}. What should the new version be? {COLOR_RESET}"
    )

    if not is_valid_version(new_version):
        print_error(
            "Invalid version format. Please use semantic versioning (e.g., 1.2.3)."
        )
        sys.exit(1)

    update_version(new_version)
    run_checks()

    if not show_changes():
        print_error("Release cancelled.")
        sys.exit(1)

    commit_and_push(new_version)

    print_success(
        f"\n🎉✨ {PROJECT_NAME} v{new_version} has been successfully released! ✨🎉"
    )


if __name__ == "__main__":
    main()
