# Edged 📐

> A pixel-accurate screen ruler for Wayland desktops, inspired by Windows PowerToys Screen Ruler.

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Release](https://github.com/TXG0Fk3/Edged/actions/workflows/release.yml/badge.svg)](https://github.com/TXG0Fk3/Edged/actions/workflows/release.yml)
[![Wayland](https://img.shields.io/badge/display-Wayland-00599C.svg?logo=wayland)](https://wayland.freedesktop.org/)
[![UI](https://img.shields.io/badge/GUI-egui%200.37-blue.svg)](https://github.com/emilk/egui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Vibecoded with Antigravity](https://img.shields.io/badge/vibecoded%20with-Google%20Antigravity-7C3AED.svg)](https://deepmind.google/)

> [!NOTE]
> **Project Context**: Edged is an open-source tool developed as an experimental project **vibecoded with [Google Antigravity](https://deepmind.google/)** (an advanced agentic AI coding assistant). It explores AI-assisted desktop tooling for Linux, maintained through human-AI pair programming rather than handcrafted from scratch.

---

**Edged** brings intuitive, edge-aware measurement to Linux Wayland desktops (Hyprland, Sway, River, Niri, GNOME Shell, KDE Plasma). It captures screen boundaries using color contrast differences, supports rectangular selection, adjusts sensitivity on the fly, and copies dimensions directly to your clipboard.

## Table of Contents

- [Features](#features)
- [Modes of Operation](#modes-of-operation)
- [Controls Cheatsheet](#controls-cheatsheet)
- [Installation](#installation)
  - [Nix & NixOS (Flake & Modules)](#1-nix--nixos-flake--modules)
  - [Precompiled Packages](#2-precompiled-packages-github-releases)
  - [Building from Source](#3-building-from-source)
- [Compositor Keybindings](#compositor-keybindings)
- [CLI Reference](#cli-reference)
- [Configuration](#configuration)
  - [Capture Command Recipes](#capture-command-recipes)
- [Architecture](#architecture)
- [Troubleshooting & FAQ](#troubleshooting--faq)
- [Testing](#testing)
- [License](#license)

---

## Features

- **One-shot execution**: Follows the Unix tool philosophy of `slurp` and `grim`—launches on demand, measures, copies to the clipboard, and terminates immediately without running an idle daemon.
- **Zero-flicker freeze-frame**: Takes an instant desktop snapshot before presenting the overlay, eliminating black flashes, screen tearing, or display-server feedback loops.
- **Euclidean edge detection**: Ray-marches pixels outward along cardinal axes in 32-bit RGBA color space with real-time feedback.
- **Scroll sensitivity adjustment**: Mouse wheel adjusts detection tolerance by 2% per notch, accompanied by a floating HUD.
- **4 measurement modes**: Spacing (crosshair), Bounds (drag-to-measure), Horizontal, and Vertical, with in-flight switching via <kbd>Tab</kbd>, hotkeys, or mouse scroll.
- **Shift-to-Pin rectangles**: Hold <kbd>Shift</kbd> on release in Bounds mode to pin multiple bounding boxes on screen.
- **Multi-monitor & fractional scaling**: Automatically detects the focused monitor, queries compositor IPC, and crops composite screenshots to prevent coordinate distortion.
- **Pure Rust rendering**: Built with `egui` and `eframe` on OpenGL (`glow`), dynamically linking only to `libc` and `libm` without GTK runtime dependencies.
- **Flexible clipboard output**: Formats measurements as `compact` (`1920x1080`), `verbose` (`1920 × 1080 px`), or `css` (`width: 1920px; height: 1080px;`).

---

## Modes of Operation

```
+-------------------------------------------------------------------------------+
| 1. SPACING MODE (Crosshair)          | 2. BOUNDS MODE (Click & Drag)          |
|                                      |                                        |
|              top_y                   |       (x, y) +------------------+      |
|                |                     |              |                  |      |
|                |                     |              |    W × H px      |      |
|    left_x -----+----- right_x        |              |  (Shift to Pin)  |      |
|                |                     |              +------------------+      |
|                |                     |                                        |
|             bottom_y                 |                                        |
+--------------------------------------+----------------------------------------+
| 3. HORIZONTAL MODE                   | 4. VERTICAL MODE                       |
|                                      |                                        |
|    | <--------- W px ---------> |    |                  --+-- top_y           |
|                                      |                    |                   |
|                                      |                    | H px              |
|                                      |                    |                   |
|                                      |                  --+-- bottom_y        |
+-------------------------------------------------------------------------------+
```

1. **Spacing Mode** (`edged --spacing`, default): 4-way crosshair extending outward until contrast boundaries exceed the tolerance threshold. Displays horizontal and vertical distances simultaneously.
2. **Bounds Mode** (`edged --bounds`): Click and drag to measure rectangular areas. Displays real-time dimensions (`W × H`). Hold <kbd>Shift</kbd> upon release to pin the box to the screen.
3. **Horizontal Mode** (`edged --horizontal`): Measures horizontal distance exclusively between contrast edges (ideal for column widths, buttons, and padding).
4. **Vertical Mode** (`edged --vertical`): Measures vertical distance exclusively (ideal for line-heights, header heights, and navbar spans).

---

## Controls Cheatsheet

| Input | Action | Modes |
| :--- | :--- | :--- |
| **Move Mouse** | Recalculates edge detection in real time | All |
| **Scroll Up / Down** | Adjusts edge tolerance by **±2%** per wheel notch | Spacing, Horizontal, Vertical |
| **Shift + Scroll** / **Ctrl + Scroll** | Cycles operating modes forward / backward | All |
| **<kbd>Tab</kbd>** / **<kbd>Shift</kbd> + <kbd>Tab</kbd>** | Cycle to next / previous measurement mode | All |
| **<kbd>1</kbd>** / **<kbd>B</kbd>** | Switch to **Bounds Mode** | All |
| **<kbd>2</kbd>** / **<kbd>S</kbd>** | Switch to **Spacing Mode** | All |
| **<kbd>3</kbd>** / **<kbd>H</kbd>** | Switch to **Horizontal Mode** | All |
| **<kbd>4</kbd>** / **<kbd>V</kbd>** | Switch to **Vertical Mode** | All |
| **Left Click** | Copies active measurement to clipboard and exits | All |
| **Drag + Release** | Copies bounding box dimensions to clipboard and exits | Bounds |
| **<kbd>Shift</kbd> + Release** | Pins rectangle to screen for multi-element analysis | Bounds |
| **<kbd>Esc</kbd>** / **<kbd>Q</kbd>** / **Right Click** | Closes Edged immediately without altering clipboard | All |

---

## Installation

### 1. Nix & NixOS (Flake & Modules)

Edged provides a Nix Flake using official precompiled release tarballs (no compilation needed). It automatically wraps all runtime Wayland, OpenGL, and clipboard dependencies (`libGL`, `libxkbcommon`, `wayland`, `vulkan-loader`, `grim`, `wl-clipboard`) and self-updates on every release.

#### Quick Run (Zero Install)
```bash
nix run github:TXG0Fk3/Edged
# Or pass mode flags directly:
nix run github:TXG0Fk3/Edged -- --bounds
```

#### Flake Input
```nix
inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  edged.url = "github:TXG0Fk3/Edged";
};
```

#### Home Manager Module (User Configuration)
```nix
{ inputs, ... }: {
  imports = [ inputs.edged.homeManagerModules.default ];

  programs.edged = {
    enable = true;
    settings = {
      general = {
        default_mode = "spacing";
        close_on_copy = true;
      };
      ruler = {
        initial_tolerance = 15;
        continuous = true;
        draw_feet = true;
      };
      appearance = {
        line_color = "#3B82F6";
      };
    };
  };
}
```

#### NixOS Module (System-Wide Installation)
```nix
{ inputs, ... }: {
  imports = [ inputs.edged.nixosModules.default ];

  programs.edged = {
    enable = true;
    settings = {
      general.default_mode = "spacing";
    };
  };
}
```

---

### 2. Precompiled Packages (GitHub Releases)

Precompiled packages are built on push to `master` and available on the [**GitHub Releases**](https://github.com/TXG0Fk3/Edged/releases) page:

| Format / Distro | Package File | Direct Installation Command |
| :--- | :--- | :--- |
| **Nix / NixOS** | `flake.nix` | `nix run github:TXG0Fk3/Edged` |
| **Standalone Binary** | `edged` | `chmod +x edged && ./edged` |
| **Arch Linux / Manjaro** | `edged-*-1-x86_64.pkg.tar.zst` | `sudo pacman -U edged-*-1-x86_64.pkg.tar.zst` |
| **Debian / Ubuntu / Mint** | `edged_*_amd64.deb` | `sudo apt install ./edged_*_amd64.deb` |
| **Fedora / RHEL / openSUSE** | `edged-*-1.x86_64.rpm` | `sudo dnf install ./edged-*-1.x86_64.rpm` |
| **Universal AppImage** | `edged-*-x86_64.AppImage` | `chmod +x edged-*.AppImage && ./edged-*.AppImage` |
| **Tarball Archive** | `edged-*-x86_64-linux.tar.gz` | `tar -xzf edged-*.tar.gz && cd edged-* && sudo ./install.sh` |

---

### 3. Building from Source

Requires Rust 1.80 or later:

```bash
git clone https://github.com/TXG0Fk3/Edged.git
cd Edged
cargo build --release

# System-wide installation (/usr/local/bin, desktop entry, icons)
sudo ./install.sh

# Or install for current user only (~/.local)
./install.sh
```

---

## Compositor Keybindings

Add Edged to your compositor's keybinding configuration:

### Hyprland (`~/.config/hypr/hyprland.conf`)
```ini
bind = $mainMod, M, exec, edged --spacing
bind = $mainMod SHIFT, M, exec, edged --bounds
bind = $mainMod CTRL, H, exec, edged --horizontal
bind = $mainMod CTRL, V, exec, edged --vertical
```
> [!TIP]
> Edged automatically sends dynamic window rules via `hyprctl keyword windowrulev2` on launch to float, fullscreen, and disable launch animations for `class:^(edged)$`. No manual window rules are required.

### Sway (`~/.config/sway/config`)
```ini
bindsym $mod+m exec edged --spacing
bindsym $mod+Shift+m exec edged --bounds
bindsym $mod+Control+h exec edged --horizontal
bindsym $mod+Control+v exec edged --vertical
```

### River (`~/.config/river/init`)
```bash
riverctl map normal Super M spawn "edged --spacing"
riverctl map normal Super+Shift M spawn "edged --bounds"
```

### Niri (`~/.config/niri/config.kdl`)
```kdl
binds {
    Mod+M { spawn "edged" "--spacing"; }
    Mod+Shift+M { spawn "edged" "--bounds"; }
}
```

### GNOME Shell
Configure in **Settings → Keyboard → Shortcuts → Custom Shortcuts**, or via CLI:
```bash
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ name 'Edged Ruler'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ command 'edged --spacing'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ binding '<Super>m'
```

### KDE Plasma 6
Configure in **System Settings → Shortcuts → Custom Shortcuts → Add New Command**:
- Command: `edged --spacing`
- Shortcut: <kbd>Meta</kbd> + <kbd>M</kbd>

---

## CLI Reference

```
Usage: edged [OPTIONS]

Options:
      --bounds               Bounds mode: click and drag to measure rectangular areas
      --spacing              Spacing mode: 4-way cross measuring margins and element dimensions
      --horizontal           Horizontal mode: measures horizontal distance between color edges
      --vertical             Vertical mode: measures vertical distance between color edges
  -t, --tolerance <PERCENT>  Override initial edge detection tolerance (1 to 100)
  -m, --monitor <TARGET>     Target monitor: 'focused' (default), 'all', 'primary', or connector ('DP-1')
  -c, --config <PATH>        Path to a custom configuration file
      --log-level <LEVEL>    Override log level: 'trace', 'debug', 'info', 'warn', 'error'
      --log-dir              Print the active log directory path and exit
      --view-logs            Print the contents of the latest log file to stdout and exit
      --clear-logs           Clear all stored Edged log files and exit
  -h, --help                 Print help information
  -V, --version              Print version
```

### Examples
```bash
edged                         # Launch in default mode (Spacing)
edged --bounds                # Launch directly into Bounds mode
edged --spacing --tolerance 35# Measure with custom sensitivity threshold of 35%
edged --monitor HDMI-A-1      # Target specific monitor connector
edged --view-logs             # View session logs
edged --clear-logs            # Purge stored logs
```

---

## Configuration

On first run, Edged creates a documented configuration file at:
```bash
~/.config/edged/config.toml
```
System-wide configuration is also read from `/etc/xdg/edged/config.toml` (or `$XDG_CONFIG_DIRS`) if no user configuration exists.

### Overview

```toml
[general]
default_mode = "spacing"    # "spacing", "bounds", "horizontal", "vertical"
close_on_copy = true        # Exit overlay immediately upon clicking to copy

[monitor]
target = "focused"          # "focused", "all", "primary", or connector name ("DP-1")
crop_to_monitor = true      # Crop composite multi-monitor screenshots to target bounds

[capture]
method = "auto"             # "auto", "grim", "portal", "custom"
command = ""                # Custom capture shell command (when method = "custom")
timeout_seconds = 5

[ruler]
initial_tolerance = 15      # Color difference threshold (1-100)
tolerance_step = 2          # Sensitivity adjustment per wheel scroll notch
draw_feet = true            # Render perpendicular tick marks at endpoints
line_thickness = 2.0
draw_crosshair = true

[bounds]
shift_pins_bounds = true    # Holding Shift upon release pins rectangle to screen
border_thickness = 2.0

[appearance]
overlay_opacity = 0.05
line_color = "#3B82F6"      # Hex (#RRGGBB) or rgba(r, g, b, a)
label_font_size = 12.0

[units]
show_pixel_unit = true      # Appends "px" suffix to badges
secondary_unit = "none"     # "none", "inch", "cm", "mm", "pt"
screen_dpi = 96.0

[clipboard]
format = "compact"          # "compact" (1920x1080), "verbose" (1920 × 1080 px), "css"

[logging]
enabled = true
level = "info"
retention_days = 4          # Automatically purges daily logs older than 4 days
```

> 📖 See [**`config.example.toml`**](config.example.toml) for the complete, line-by-line annotated configuration template.

---

### Capture Command Recipes

When `capture.method = "custom"`, Edged replaces `%s` (or `%path`) with the temporary image destination path, `%o` with the monitor connector, and `%w`/`%h`/`%x`/`%y` with monitor geometry:

| Tool | Recommended Recipe in `config.toml` | Notes |
| :--- | :--- | :--- |
| **`grim`** | `command = "grim %s"` | Standard wlroots tool. Fast and direct. |
| **`grim` (per-monitor)** | `command = "grim -o %o %s"` | Captures only the targeted output connector. |
| **`hyprshot`** | `command = "hyprshot -m output -s --raw > %s"` | Recommended for Hyprland. Redirects raw stream to `%s`. |
| **`wayshot`** | `command = "wayshot %s"` | Direct file destination. |
| **`grimblast`** | `command = "grimblast save screen %s"` | Hyprland community wrapper. |
| **`grimshot`** | `command = "grimshot save screen %s"` | Sway community wrapper. |
| **`spectacle`** | `command = "spectacle -b -n -o %s"` | Background capture for KDE Plasma. |
| **`flameshot`** | `command = "flameshot full -r > %s"` | Fullscreen capture redirected to target file. |

---

## Architecture

```
+-----------------------------------------------------------------------------------+
|                              USER INVOCATION (CLI / KEYBIND)                       |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| 1. MONITOR DETECTION & SCREEN CAPTURE                                             |
|    - Queries compositor IPC (Hyprland / Sway / Mutter) for active monitor geometry|
|    - Captures desktop snapshot via grim, xdg-desktop-portal, or custom command     |
|    - Auto-crops multi-display canvas to focused monitor dimensions                |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| 2. COMPOSITOR DYNAMIC IPC WINDOW RULES                                            |
|    - Hyprland: hyprctl keyword windowrulev2 float, fullscreen, noanim, noborder   |
|    - Sway: swaymsg for_window [app_id="edged"] floating enable, fullscreen enable |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| 3. EGUI GPU RENDERING ENGINE (Pure Rust)                                          |
|    - Zero GTK C dependencies: links dynamically only to libc/libm                  |
|    - Streams captured snapshot into egui::ColorImage texture (zero black screen)  |
|    - Precise 1:1 scale mapping between viewport coordinates and image pixels      |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| 4. REAL-TIME EUCLIDEAN EDGE DETECTOR                                              |
|    - Evaluates Euclidean color distance: sqrt(dr^2 + dg^2 + db^2 + da^2) / 4.41    |
|    - 4-way raymarching with discrete 2% scroll tolerance adjustments              |
|    - Direct clipboard copy on primary click -> Clean termination                  |
+-----------------------------------------------------------------------------------+
```

### Freeze-Frame Snapshot
Rather than capturing a live screencast (which causes tearing, input latency, or feedback loops on Wayland), Edged acquires a single freeze-frame desktop snapshot before opening its window, rendering it as a static GPU background texture under the measurement overlay.

### Euclidean Color Distance
Edges are detected by comparing pixels against the origin pixel in 32-bit RGBA color space:

$$\Delta E = \frac{\sqrt{(R_1 - R_2)^2 + (G_1 - G_2)^2 + (B_1 - B_2)^2 + (A_1 - A_2)^2}}{\sqrt{255^2 \times 4}} \times 100$$

When the delta exceeds the user's active tolerance threshold, the coordinate is locked.

---

## Troubleshooting & FAQ

### Black screen on startup
Indicates that the screen capture backend could not access the compositor output:
- **Hyprland / Sway**: Ensure `grim` is installed (`pacman -S grim` or `dnf install grim`).
- **GNOME / KDE**: Ensure `xdg-desktop-portal` and `pipewire` are running.
- In `config.toml`, verify `capture.method = "auto"` or configure a working custom command.

### `hyprshot` fails with exit code 1
`hyprshot` treats `-o` as an output *directory*, not a filename. Use the stdout redirection recipe:
```toml
[capture]
method = "custom"
command = "hyprshot -m output -s --raw > %s"
```

### Measuring in physical units (cm, mm, inches)
Set `secondary_unit` and `screen_dpi` in `~/.config/edged/config.toml`:
```toml
[units]
secondary_unit = "mm"  # "none", "inch", "cm", "mm", "pt"
screen_dpi = 96.0      # Your display's physical DPI
```
Badges will display both pixel dimensions and converted units: `120 px (31.8 mm)`.

---

## Testing

### Automated Test Suite
Edged includes unit and integration tests covering geometry, edge detection math, color parsing, and configuration resolution:
```bash
cargo test --all-targets --verbose
```

### Manual QA Checklist
For compositor surface behavior, hardware capture backends, and multi-monitor scaling validation, see [**`MANUAL_TESTING.md`**](MANUAL_TESTING.md).

### AI & Developer Guidelines
For architectural rules, release engineering, and vibecoding context for future developers or AI agents, refer to [**`AGENTS.md`**](AGENTS.md).

---

## License

Edged is open-source software licensed under the **MIT License**. Copyright © 2026 **TXG0Fk3**. See [LICENSE](LICENSE) for details.

*Inspired by Microsoft PowerToys Screen Ruler. Vibecoded with Google Antigravity.*
