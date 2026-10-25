# Edged 📐

> A blazing-fast, pixel-accurate **PowerToys Screen Ruler** replica designed natively for **Wayland** (Hyprland, Sway, River, GNOME Shell, KDE Plasma, Niri).

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Release](https://github.com/TXG0Fk3/Edged/actions/workflows/release.yml/badge.svg)](https://github.com/TXG0Fk3/Edged/actions/workflows/release.yml)
[![GUI Engine](https://img.shields.io/badge/GUI-egui%200.37-blue.svg)](https://github.com/emilk/egui)
[![Wayland](https://img.shields.io/badge/display-Wayland-00599C.svg?logo=wayland)](https://wayland.freedesktop.org/)
[![Pure Rust](https://img.shields.io/badge/dependencies-pure%20rust%20(zero%20GTK)-success.svg)](#architecture--design-principles)
[![Vibecoded with Antigravity](https://img.shields.io/badge/vibecoded%20with-Google%20Antigravity-7C3AED.svg)](https://deepmind.google/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> [!NOTE]
> **Experimental Project / Vibecoding Notice**:
> Edged is an experimental project **vibecoded with [Google Antigravity](https://deepmind.google/)** (an advanced agentic AI coding assistant) in collaboration with **TXG0Fk3**. It is an exploration into AI-assisted desktop tooling for Linux—rapidly prototyping, migrating, and optimizing a high-performance Wayland screen ruler in pure Rust and `egui`. While fully functional, actively tested, and maintained, it is transparently developed via AI pair-programming iterations rather than manual handcrafting.

**Edged** brings the intuitive, edge-aware measurement capabilities of Windows PowerToys Screen Ruler to modern Linux Wayland desktops. Whether you are fine-tuning UI component spacing, inspecting web layouts, auditing margins and padding, or dragging rectangular bounds across visual elements, Edged provides real-time measurement lines, dynamic contrast sensitivity, and instant clipboard export—with **zero persistent background daemons**, **zero GTK runtime dependencies**, and **zero screen flicker**.

---

## 📑 Table of Contents

- [Key Features](#-key-features)
- [Modes of Operation](#-modes-of-operation)
- [Interactive Controls Cheatsheet](#-interactive-controls-cheatsheet)
- [Installation](#-installation)
  - [Arch Linux](#arch-linux)
  - [Fedora](#fedora)
  - [Ubuntu / Debian](#ubuntu--debian)
  - [From Source (Cargo)](#from-source-cargo)
  - [Desktop Entry](#desktop-entry)
- [CLI Reference](#-cli-reference)
- [Compositor & Window Manager Setup](#-compositor--window-manager-setup)
  - [Hyprland](#hyprland)
  - [Sway](#sway)
  - [River](#river)
  - [Niri](#niri)
  - [GNOME Shell](#gnome-shell)
  - [KDE Plasma 6](#kde-plasma-6)
- [Configuration Reference](#-configuration-reference)
  - [Configuration File Path](#configuration-file-path)
  - [General Settings (`[general]`)](#general-settings-general)
  - [Multi-Monitor Configuration (`[monitor]`)](#multi-monitor-configuration-monitor)
  - [Screen Capture Configuration (`[capture]`)](#screen-capture-configuration-capture)
  - [Capture Command Recipes](#capture-command-recipes)
  - [Keyboard & Mouse Shortcuts (`[shortcuts]`)](#keyboard--mouse-shortcuts-shortcuts)
  - [Ruler & Edge Detection (`[ruler]`)](#ruler--edge-detection-ruler)
  - [Bounds Mode (`[bounds]`)](#bounds-mode-bounds)
  - [Visual Appearance (`[appearance]`)](#visual-appearance-appearance)
  - [Measurement Units (`[units]`)](#measurement-units-units)
  - [Clipboard Export Formats (`[clipboard]`)](#clipboard-export-formats-clipboard)
- [Architecture & Design Principles](#-architecture--design-principles)
- [Troubleshooting & FAQ](#-troubleshooting--faq)
- [Testing](#-testing)
- [License](#-license)

---

## ✨ Key Features

- ⚡ **One-Shot Invocation Model**: Follows the classic Unix philosophy of modern Wayland tools like `slurp` and `grim`. Does **not** run as an idle background service consuming RAM or CPU cycles. Launches on demand, takes the measurement, copies to the clipboard, and immediately terminates.
- 🦀 **Pure Rust GPU Rendering (Zero GTK C Dependencies)**: Built completely in Rust using [`egui`](https://github.com/emilk/egui) and [`eframe`](https://docs.rs/eframe). Requires no GTK4, Cairo, Pango, or GObject C libraries installed on the host system. The compiled release binary only links dynamically to `libc` and `libm`.
- 🔀 **In-Flight Mode Switching**: Switch between Spacing, Bounds, Horizontal, and Vertical modes during execution without restarting:
  - Tap <kbd>Tab</kbd> (or <kbd>Shift</kbd> + <kbd>Tab</kbd> for reverse).
  - Press numeric hotkeys: <kbd>1</kbd> (Bounds), <kbd>2</kbd> (Spacing), <kbd>3</kbd> (Horizontal), <kbd>4</kbd> (Vertical).
  - Press mnemonic letters: <kbd>B</kbd>, <kbd>S</kbd>, <kbd>H</kbd>, <kbd>V</kbd>.
  - Scroll with <kbd>Shift</kbd> + **Mouse Wheel** or <kbd>Ctrl</kbd> + **Mouse Wheel** to rotate modes seamlessly.
- 🎯 **Real-Time Euclidean Edge Detection**: Explores pixels along cardinal axes using a Euclidean color distance metric in 32-bit RGBA color space. Instant visual feedback updates continuously with crosshair tracking.
- 🔄 **Discrete 2% Scroll Tolerance Adjustment**: Scroll the mouse wheel up or down to adjust detection sensitivity by exactly 2% per notch, complete with a temporary floating heads-up display (HUD).
- 📸 **Zero-Flicker Freeze-Frame Capture**: Takes an instantaneous snapshot of the desktop prior to presenting the measurement surface. The frozen snapshot is streamed directly into an egui GPU texture, completely eliminating the black screens or visual tearing common in Wayland screencasters.
- 🖥️ **Intelligent Focused-Monitor Target & Auto-Cropping**:
  - Automatically queries compositor IPC (Hyprland, Sway, Mutter) to measure directly on the monitor holding active user focus.
  - If third-party screenshot utilities produce a combined canvas of all monitors, Edged automatically crops the canvas to your focused display, preventing coordinate distortion.
- 🪟 **Tiling Compositor Native Integration**: Sends dynamic IPC keywords on startup (`hyprctl keyword windowrulev2` on Hyprland, `swaymsg` on Sway) to ensure the overlay opens floating, borderless, unshadowed, unrounded, without launch animations, and strictly pinned to the active output.
- 📌 **Shift-to-Pin Multi-Box Measurement**: Hold <kbd>Shift</kbd> upon releasing a rectangular drag in Bounds mode to keep the bounding box pinned to your screen while drawing additional boxes.
- 📋 **Flexible Clipboard Formats**: Primary click immediately copies the measurement in `compact` (`1920x1080` / `120`), `verbose` (`1920 × 1080 px` / `120 px`), or `css` (`width: 1920px; height: 1080px;`).

---

## 📐 Modes of Operation

Edged provides four distinct measuring modes inspired by PowerToys Screen Ruler:

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

### 1. Spacing Mode (`edged --spacing`, Default)
The primary mode for UI inspection. Crosshairs extend outwards in all four directions (Up, Down, Left, Right) from the mouse pointer until they encounter a visual boundary that exceeds the current color tolerance threshold. Displays horizontal and vertical distances simultaneously.

### 2. Bounds Mode (`edged --bounds`)
Allows you to click and drag to delineate any rectangular region. As you drag, an overlay box displays real-time dimensions (`Width × Height`). Releasing the primary mouse button copies the dimensions to the clipboard. Holding <kbd>Shift</kbd> upon release pins the rectangle to your screen, allowing multiple areas to be measured concurrently.

### 3. Horizontal Mode (`edged --horizontal`)
Measures horizontal distance exclusively. A single horizontal line expands left and right until it detects significant color changes. Useful for inspecting row widths, column grids, and button horizontal spans.

### 4. Vertical Mode (`edged --vertical`)
Measures vertical distance exclusively. A single vertical line expands upwards and downwards from your cursor. Ideal for line-height verification, paragraph spacing, header heights, and navbar thickness checks.

---

## 🎮 Interactive Controls Cheatsheet

| Input Gesture | Function | Supported Modes |
| :--- | :--- | :--- |
| **Move Mouse** | Dynamically recalculates edge detection and dimensions | All Modes |
| **Scroll Up / Down** | Adjusts edge tolerance up or down by **2%** per wheel notch | Spacing, Horizontal, Vertical |
| **Shift + Scroll** | **Cycles operating modes forward / backward** | All Modes |
| **Ctrl + Scroll** | **Cycles operating modes forward / backward** | All Modes |
| **<kbd>Tab</kbd>** | Cycle to the next measurement mode | All Modes |
| **<kbd>Shift</kbd> + <kbd>Tab</kbd>** | Cycle to the previous measurement mode | All Modes |
| **<kbd>1</kbd>** / **<kbd>B</kbd>** | Immediately switches to **Bounds Mode** | All Modes |
| **<kbd>2</kbd>** / **<kbd>S</kbd>** | Immediately switches to **Spacing Mode** | All Modes |
| **<kbd>3</kbd>** / **<kbd>H</kbd>** | Immediately switches to **Horizontal Mode** | All Modes |
| **<kbd>4</kbd>** / **<kbd>V</kbd>** | Immediately switches to **Vertical Mode** | All Modes |
| **Primary Click** | Copies active measurement to clipboard and exits | All Modes |
| **Drag + Release** | Copies bounding box dimensions to clipboard and exits | Bounds Mode |
| **<kbd>Shift</kbd> + Release** | Pins the drawn rectangle to the screen for multi-element analysis | Bounds Mode |
| **<kbd>Esc</kbd>** or **<kbd>Q</kbd>** | Closes Edged immediately without altering the clipboard | All Modes |
| **Secondary Click** | Cancels active measurement and closes overlay | All Modes |

---

## 📦 Installation & Prebuilt Packages

Every push to the repository automatically triggers automated builds creating packages for all major Linux distributions on the [**GitHub Releases**](https://github.com/TXG0Fk3/Edged/releases) page.

### 1. Prebuilt Packages (Recommended)

| Format / Distro | Package File | Direct Installation Command |
| :--- | :--- | :--- |
| **Standalone Binary** | `edged` | `chmod +x edged && ./edged` |
| **Debian / Ubuntu / Mint** | `edged_*_amd64.deb` | `sudo apt install ./edged_*_amd64.deb` |
| **Fedora / RHEL / openSUSE** | `edged-*-1.x86_64.rpm` | `sudo dnf install ./edged-*-1.x86_64.rpm` |
| **Arch Linux / Manjaro** | `edged-*-1-x86_64.pkg.tar.zst` | `sudo pacman -U edged-*-1-x86_64.pkg.tar.zst` |
| **Universal AppImage** | `edged-*-x86_64.AppImage` | `chmod +x edged-*.AppImage && ./edged-*.AppImage` |
| **Tarball Archive** | `edged-*-x86_64-linux.tar.gz` | `tar -xzf edged-*.tar.gz && cd edged-* && sudo ./install.sh` |

---

### 2. Arch Linux (AUR)

```bash
paru -S edged-git
# or
yay -S edged-git
```

### 3. From Source (Cargo)

Ensure you have Rust 1.80 or later installed:

```bash
# Clone the repository
git clone https://github.com/TXG0Fk3/Edged.git
cd Edged

# Build optimized release binary
cargo build --release

# Run automated installer script (installs binary, desktop entry, and icons)
sudo ./install.sh
```

To install without `sudo` into `~/.local`:
```bash
./install.sh
```

---

## 💻 CLI Reference

```
Edged is a fast, pixel-accurate screen measuring tool for Wayland.
It replicates the PowerToys Screen Ruler with real-time edge detection,
bounds measuring, scroll-wheel dynamic tolerance, and clipboard copying.

Usage: edged [OPTIONS]

Options:
      --bounds               Bounds mode: click and drag to measure rectangular areas
      --spacing              Spacing mode: 4-way cross measuring margins and element dimensions
      --horizontal           Horizontal mode: measures horizontal distance between color edges
      --vertical             Vertical mode: measures vertical distance between color edges
  -t, --tolerance <PERCENT>  Override initial edge detection tolerance (1 to 100)
  -m, --monitor <TARGET>     Target monitor: 'focused' (default), 'all', 'primary', or connector ('DP-1')
  -c, --config <PATH>        Path to a custom configuration file
  -h, --help                 Print help information
  -V, --version              Print version
```

### Examples

```bash
# Launch in default mode (configured in config.toml, default: Spacing)
edged

# Launch directly into Bounds mode for rectangle selection
edged --bounds

# Measure with a custom sensitivity threshold of 35%
edged --spacing --tolerance 35

# Measure explicitly on a secondary display connector
edged --monitor HDMI-A-1

# Span overlay across all attached monitors
edged --monitor all
```

---

## ⌨️ Compositor & Window Manager Setup

Bind Edged to keyboard shortcuts in your compositor configuration:

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```ini
# Edged Screen Ruler Keybindings
bind = $mainMod, M, exec, edged --spacing
bind = $mainMod SHIFT, M, exec, edged --bounds
bind = $mainMod CTRL, H, exec, edged --horizontal
bind = $mainMod CTRL, V, exec, edged --vertical
```

> [!TIP]
> Edged automatically sends dynamic window rules via `hyprctl keyword windowrulev2` at startup to float, fullscreen, and disable animations for `class:^(edged)$`. You do not need to manually configure window rules in `hyprland.conf`.

### Sway

Add to `~/.config/sway/config`:

```ini
# Edged Screen Ruler Keybindings
bindsym $mod+m exec edged --spacing
bindsym $mod+Shift+m exec edged --bounds
bindsym $mod+Control+h exec edged --horizontal
bindsym $mod+Control+v exec edged --vertical
```

### River

Add to `~/.config/river/init`:

```bash
riverctl map normal Super M spawn "edged --spacing"
riverctl map normal Super+Shift M spawn "edged --bounds"
```

### Niri

Add to `~/.config/niri/config.kdl`:

```kdl
binds {
    Mod+M { spawn "edged" "--spacing"; }
    Mod+Shift+M { spawn "edged" "--bounds"; }
}
```

### GNOME Shell

Configure via GUI in **Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts**, or via CLI:

```bash
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ name 'Edged Ruler'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ command 'edged --spacing'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ binding '<Super>m'
```

### KDE Plasma 6

Create custom shortcuts in **System Settings → Shortcuts → Custom Shortcuts → Add New Command**:
- Command: `edged --spacing`
- Shortcut: <kbd>Meta</kbd> + <kbd>M</kbd>

---

## ⚙️ Configuration Reference

### Configuration File Path

On initial execution, Edged automatically initializes a commented configuration file at:

```
~/.config/edged/config.toml
```

You can pass an alternative configuration file at any time using `--config /path/to/custom.toml`.

---

### General Settings (`[general]`)

```toml
[general]
# Default mode when invoked without CLI flags: "spacing", "bounds", "horizontal", "vertical"
default_mode = "spacing"

# Automatically terminate the overlay as soon as a measurement is copied
close_on_copy = true
```

---

### Multi-Monitor Configuration (`[monitor]`)

```toml
[monitor]
# Strategy used to determine the active display:
# - "focused": (Default & Recommended) Automatically detects and measures on the monitor
#              currently in focus (holding cursor or active window). Eliminates cross-display distortion.
# - "all":     Spans the measurement canvas across all connected displays simultaneously.
# - "primary": Targets the primary output reported by the compositor.
# - "<name>":  Targets a specific connector identifier (e.g., "DP-1", "HDMI-A-1", "eDP-1").
target = "focused"

# When targeting a single monitor, automatically crops composite multi-monitor screenshots
# down to the target monitor's physical bounds, ensuring pixel-for-pixel coordinate alignment.
crop_to_monitor = true
```

---

### Screen Capture Configuration (`[capture]`)

```toml
[capture]
# Capture backend to acquire the freeze-frame desktop snapshot:
# - "auto":   Tries 'grim' first (fastest on wlroots/Hyprland/Sway), falls back to XDG Desktop Portal.
# - "grim":   Forces the use of the 'grim' command-line utility.
# - "portal": Forces screenshot capture through org.freedesktop.portal.Screenshot (PipeWire).
# - "custom": Executes a user-defined shell command specified below.
method = "auto"

# Shell command invoked when method = "custom". Leave empty to use auto-detection.
command = ""

# Maximum timeout in seconds before canceling a hanging custom capture command
timeout_seconds = 5
```

#### Custom Capture Command Placeholders

When `method = "custom"`, Edged expands the following placeholders before executing your command:

| Placeholder | Expanded Value Description | Example Output |
| :--- | :--- | :--- |
| `%s` or `%path` | Full temporary output file destination path | `/tmp/edged_capture_1234.png` |
| `%d` or `%dir` | Directory containing the temporary output file | `/tmp` |
| `%n` or `%filename` | Base filename without directory component | `edged_capture_1234.png` |
| `%o` or `%output` | Target monitor connector name | `DP-1` |
| `%w` or `%width` | Target monitor pixel width | `1920` |
| `%h` or `%height` | Target monitor pixel height | `1080` |
| `%x` | Target monitor X coordinate offset | `0` |
| `%y` | Target monitor Y coordinate offset | `0` |

---

### Capture Command Recipes

If you want to use custom screenshot tools, Edged provides verified recipes:

| Tool | Recommended Recipe in `config.toml` | Notes |
| :--- | :--- | :--- |
| **`grim`** | `command = "grim %s"` | Standard wlroots tool. Pixel-accurate and fast. |
| **`grim` (per-monitor)** | `command = "grim -o %o %s"` | Captures only the targeted output connector. |
| **`hyprshot`** | `command = "hyprshot -m output -s --raw > %s"` | Recommended for Hyprland: outputs raw image to `%s`. |
| **`wayshot`** | `command = "wayshot %s"` | Takes positional output filepath directly. |
| **`grimblast`** | `command = "grimblast save screen %s"` | Popular Hyprland wrapper. |
| **`grimshot`** | `command = "grimshot save screen %s"` | Sway community wrapper script. |
| **`spectacle`** | `command = "spectacle -b -n -o %s"` | Non-interactive background capture for KDE Plasma. |
| **`flameshot`** | `command = "flameshot full -r > %s"` | Fullscreen raw capture redirected to target file. |

---

### Keyboard & Mouse Shortcuts (`[shortcuts]`)

```toml
[shortcuts]
# Key used to cycle through measurement modes (Shift + Key cycles backwards)
cycle_mode_key = "Tab"

# Direct number keys
mode_bounds_key = "1"
mode_spacing_key = "2"
mode_horizontal_key = "3"
mode_vertical_key = "4"

# Alternative mnemonic letter keys
alt_mode_bounds_key = "b"
alt_mode_spacing_key = "s"
alt_mode_horizontal_key = "h"
alt_mode_vertical_key = "v"

# Allow Shift + Mouse Wheel to cycle through modes in real time
shift_scroll_cycles_mode = true

# Allow Ctrl + Mouse Wheel to cycle through modes in real time
ctrl_scroll_cycles_mode = true
```

---

### Ruler & Edge Detection (`[ruler]`)

```toml
[ruler]
# Initial tolerance percentage for color difference (1 to 100)
# Lower values detect subtle gradients; higher values bridge softer edges.
initial_tolerance = 15

# Percentage points added or subtracted per mouse wheel scroll notch
tolerance_step = 2

# Draw perpendicular tick marks ('feet') at the ends of measurement lines
draw_feet = true

# Length of the feet tick marks in pixels
feet_size = 10.0

# Stroke thickness of the ruler lines in pixels
line_thickness = 2.0

# Render a crosshair guide at the cursor position
draw_crosshair = true
```

---

### Bounds Mode (`[bounds]`)

```toml
[bounds]
# When dragging in Bounds mode, holding Shift pins the rectangle to the screen
shift_pins_bounds = true

# Stroke thickness for rectangle borders in pixels
border_thickness = 2.0
```

---

### Visual Appearance (`[appearance]`)

Colors can be defined in `#RRGGBB`, `#RRGGBBAA`, or `rgba(r, g, b, a)` format:

```toml
[appearance]
# Background dimming factor (0.0 = transparent, 0.05 = subtle 5% dimming)
overlay_opacity = 0.05

# Measurement line colors
line_color = "#3B82F6"
line_shadow_color = "rgba(0, 0, 0, 0.60)"

# Bounds mode rectangle styling
bounds_fill_color = "rgba(59, 130, 246, 0.15)"
bounds_border_color = "#3B82F6"

# Measurement badge styling
label_bg_color = "rgba(15, 23, 42, 0.90)"
label_text_color = "#F8FAFC"
label_border_color = "#475569"
label_font_family = "sans-serif"
label_font_size = 12.0
label_border_radius = 5.0
label_padding_x = 7.0
label_padding_y = 3.5
```

---

### Measurement Units (`[units]`)

```toml
[units]
# Append 'px' unit suffix to on-screen badges
show_pixel_unit = true

# Optional secondary unit conversion: "none", "inch", "cm", "mm", "pt"
secondary_unit = "none"

# Monitor DPI used for physical unit calculation
screen_dpi = 96.0
```

---

### Clipboard Export Formats (`[clipboard]`)

```toml
[clipboard]
# Format written to the system clipboard upon left-click:
# - "compact": e.g. "1920x1080" (Bounds) or "120" (Spacing)
# - "verbose": e.g. "1920 × 1080 px" or "120 px"
# - "css":     e.g. "width: 1920px; height: 1080px;" or "width: 120px;"
format = "compact"
```

---

## 🏛️ Architecture & Design Principles

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
| 3. EGUIDE GPU RENDERING ENGINE (Pure Rust)                                        |
|    - Zero C dependencies: links only to libc/libm                                  |
|    - Streams captured snapshot into egui::ColorImage texture (zero black screen)  |
|    - Precise 1:1 scale mapping between viewport coordinates and image pixels      |
+-----------------------------------------+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
| 4. REAL-TIME EUCLIDEAN EDGE DETECTOR                                              |
|    - Evaluates Euclidean color distance: sqrt(dr^2 + dg^2 + db^2 + da^2) / 4.41    |
|    - 4-way raymarching with discrete 2% scroll tolerance adjustments              |
|    - Direct clipboard copy on primary click -> Clean termination                 |
+-----------------------------------------------------------------------------------+
```

### 1. Instant Freeze-Frame Architecture
Standard screen rulers often exhibit visual tearing, coordinate lag, or black frames when capturing live Wayland surfaces. Edged completely avoids this by taking a high-speed desktop snapshot **before** opening the overlay window, then rendering that snapshot as a static background layer underneath the interactive measurement lines.

### 2. Euclidean Color Distance
Edge boundaries are detected using Euclidean distance in four-channel RGBA color space:

$$\Delta E = \frac{\sqrt{(R_1 - R_2)^2 + (G_1 - G_2)^2 + (B_1 - B_2)^2 + (A_1 - A_2)^2}}{\sqrt{255^2 \times 4}} \times 100$$

When the delta between the reference pixel and an adjacent raymarching pixel exceeds the user's active tolerance threshold, the edge coordinate is locked.

### 3. Pure Rust Desktop Pipeline
By migrating completely to `egui` and `eframe`, Edged eliminates the complex web of C dependencies (`gtk4`, `gtk4-layer-shell`, `cairo`, `pango`, `glib`). The resulting binary is smaller, completely immune to ABI drift across Linux distributions, and offers instant startup times.

---

## ❓ Troubleshooting & FAQ

### Q: Why does my screen turn black when launching Edged?
**A:** This indicates that your screenshot capture backend could not access your Wayland compositor's display output.
- **On Hyprland / Sway**: Ensure `grim` is installed (`sudo pacman -S grim` or `sudo dnf install grim`).
- **On GNOME / KDE**: Ensure `xdg-desktop-portal` and `pipewire` are running.
- In `~/.config/edged/config.toml`, make sure `capture.method = "auto"` or specify a verified custom command.

### Q: Why did `hyprshot` fail with exit code 1?
**A:** When using `hyprshot` with `-o /path/to/file.png`, `hyprshot` interprets `-o` as the output *directory* rather than the output file. Use the recommended stdout redirection recipe instead:
```toml
[capture]
method = "custom"
command = "hyprshot -m output -s --raw > %s"
```

### Q: Why does the overlay not cover the full screen on Hyprland?
**A:** Ensure you are running the latest version of Edged. Edged automatically issues dynamic window rules via `hyprctl keyword windowrulev2` on startup to force the window into floating, fullscreen, borderless, and non-animated mode. If you have custom conflicting rules in `hyprland.conf` with `override`, remove or adjust them.

### Q: How do I measure in centimeters, millimeters, or inches?
**A:** Set `secondary_unit` in `~/.config/edged/config.toml`:
```toml
[units]
secondary_unit = "mm"  # Options: "none", "inch", "cm", "mm", "pt"
screen_dpi = 96.0      # Set to your display's physical DPI
```
On-screen badges will display both pixel dimensions and converted physical units: `120 px (31.8 mm)`.

---

## 🧪 Testing

Edged contains a comprehensive automated test suite. Run all unit and integration tests with:

```bash
cargo test
```

Test coverage includes:
- Mode cycling and bidirectional navigation logic.
- Keyboard shortcut matching (Tab, single keys, named keys, case-insensitivity).
- Multi-monitor target parsing and canvas cropping algorithms.
- Custom capture command placeholder substitution.
- Raymarching edge detection thresholds and synthetic box geometry.
- Reverse drag normalization in Bounds mode.
- Color parser verification (Hex 3/4/6/8-digit, RGB, RGBA).
- Configuration parser integrity and default settings validity.

---

## 📄 License

Edged is open-source software licensed under the **MIT License**.

Copyright © 2026 **TXG0Fk3**. See the [LICENSE](LICENSE) file for the full text.

---

*Inspired by Microsoft PowerToys Screen Ruler. Vibecoded with Google Antigravity.*
