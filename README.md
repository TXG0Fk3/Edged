# Edged 📐

> High-quality, fast, and faithful **PowerToys Screen Ruler** replica built natively for **Wayland** (Hyprland, GNOME, Sway, River, KDE Plasma, etc.).

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/display-Wayland-blue.svg)](https://wayland.freedesktop.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)

**Edged** is a pixel-accurate screen measuring tool designed specifically for modern Wayland environments. It allows you to measure distances, bounding boxes, margins, padding, and spacing between UI elements using real-time edge detection.

---

## ✨ Features

- ⚡ **One-Shot Activation Model**: Does **not** run as a persistent background daemon eating CPU or RAM. Launches instantly via terminal command and exits as soon as you copy your measurement or hit <kbd>Esc</kbd>.
- 🔀 **Dynamic Mode Switching During Use**:
  - Switch modes on the fly using <kbd>Tab</kbd> (or <kbd>Shift</kbd> + <kbd>Tab</kbd>).
  - Direct number keys: <kbd>1</kbd> (Bounds), <kbd>2</kbd> (Spacing), <kbd>3</kbd> (Horizontal), <kbd>4</kbd> (Vertical).
  - Mnemonic letter keys: <kbd>B</kbd>, <kbd>S</kbd>, <kbd>H</kbd>, <kbd>V</kbd>.
  - **Shift + Scroll** or **Ctrl + Scroll** to cycle modes seamlessly with the mouse wheel!
  - Fully configurable keybindings in `~/.config/edged/config.toml`.
  - Floating HUD notification shows the active mode when changed.
- 🎯 **Real-time Edge Detection**: Walks outwards from the cursor across 4 cardinal directions (up, down, left, right) based on Euclidean color differences.
- 🔄 **Dynamic Scroll-wheel Tolerance**: Scroll your mouse wheel up or down to adjust edge sensitivity in real-time with an instant HUD progress indicator.
- 📦 **Bounds Mode**: Click and drag to measure rectangular regions displaying `Width × Height`.
- 📌 **Shift to Pin**: Hold <kbd>Shift</kbd> in Bounds mode to pin previously drawn rectangles to the screen and measure multiple areas simultaneously.
- 📏 **Draw Feet**: Perpendicular tick marks at measurement line ends for clear boundary visibility.
- 📋 **Click to Copy**: Primary click copies the measurement directly to your Wayland clipboard (`wl-clipboard` / GTK) in configurable formats (`compact`, `verbose`, or `css`).
- 🖥️ **Native Wayland & Multi-Monitor**: Uses `wlr-layer-shell` on wlroots compositors (Hyprland, Sway, River) and seamless overlay fallback on GNOME Mutter.
- 🎯 **Intelligent Multi-Monitor Targeting**:
  - **Focused Monitor by Default**: Automatically detects and measures on the monitor in focus (where your cursor or active window is).
  - **Auto-Cropping & Alignment**: If screenshot tools capture the entire multi-monitor desktop, Edged automatically crops the capture to your focused monitor's bounds, eliminating misaligned coordinates or black areas.
  - **Configurable**: Choose between `focused` (default), `all` (span all displays), `primary`, or any specific connector name (`DP-1`, `HDMI-A-1`, etc.).
- 📸 **Freeze-frame Rendering (No Black Screen)**: Captures the screen before presenting the overlay, drawing the frozen desktop snapshot underneath the ruler with zero flicker.
- 🛠️ **Configurable Screenshot Tools**: Supports `grim`, `xdg-desktop-portal` (PipeWire), or any custom command (e.g. `wayshot`, `hyprshot`, `spectacle`).
- ⚙️ **Fully Configurable**: Generates a documented `~/.config/edged/config.toml` file automatically on first run.

---

## 🚀 CLI Usage

Edged follows the Unix philosophy of modern Wayland tools (like `slurp`, `grim`, `swappy`):

```bash
edged                 # Starts in default mode from config (default: spacing)
edged --bounds        # Bounds mode: click and drag rectangular bounding boxes
edged --spacing       # Spacing mode: 4-way cross measuring margins and padding
edged --horizontal    # Horizontal line measuring distance between color edges
edged --vertical      # Vertical line measuring distance between color edges
```

### Additional Flags

```bash
edged --tolerance 25                # Override initial edge detection tolerance (1 to 100%)
edged --monitor focused             # Measure on focused monitor (default)
edged --monitor all                 # Span overlay across all connected monitors
edged --monitor primary             # Measure on primary monitor
edged --monitor DP-1                # Measure on specific connector name
edged --config /path/to/config.toml # Use custom configuration file
edged --help                        # Show help information
edged --version                     # Print version
```

---

## 🎮 Overlay Controls

| Control | Action |
| :--- | :--- |
| **Move Cursor** | Updates edge detection lines or bounding box in real time |
| **Mouse Wheel Up / Down** | Adjusts edge detection tolerance (Spacing/Horizontal/Vertical modes) |
| **Shift + Scroll** *(or Ctrl+Scroll)* | **Cycles modes on the fly** (`Spacing` ↔ `Bounds` ↔ `Horizontal` ↔ `Vertical`) |
| **Tab / Shift + Tab** | **Cycles mode forward / backward** |
| **1 / 2 / 3 / 4** *(or B / S / H / V)* | **Directly switches to Bounds, Spacing, Horizontal, or Vertical** |
| **Left Click** | Copies active measurement to clipboard and exits |
| **Shift + Release** *(Bounds)* | Pins the rectangle to screen to measure additional regions |
| **Esc / Q Key** | Closes Edged immediately without copying |
| **Right Click** | Cancels and exits immediately |

---

## ⌨️ Window Manager Keybinds

### Hyprland (`~/.config/hypr/hyprland.conf`)

```ini
# Edged Screen Ruler Keybinds
bind = $mainMod, M, exec, edged --spacing
bind = $mainMod SHIFT, M, exec, edged --bounds
bind = $mainMod CTRL, H, exec, edged --horizontal
bind = $mainMod CTRL, V, exec, edged --vertical
```

### Sway (`~/.config/sway/config`)

```ini
# Edged Screen Ruler Keybinds
bindsym $mod+m exec edged --spacing
bindsym $mod+Shift+m exec edged --bounds
bindsym $mod+Control+h exec edged --horizontal
bindsym $mod+Control+v exec edged --vertical
```

### GNOME Shell

Add custom shortcuts in **Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts**:
- **Name**: `Edged Spacing Ruler` | **Command**: `edged --spacing` | **Shortcut**: <kbd>Super</kbd> + <kbd>M</kbd>
- **Name**: `Edged Bounds Ruler`  | **Command**: `edged --bounds`  | **Shortcut**: <kbd>Super</kbd> + <kbd>Shift</kbd> + <kbd>M</kbd>

Or configure via `gsettings` in terminal:

```bash
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ name 'Edged Ruler'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ command 'edged --spacing'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ binding '<Super>m'
```

---

## ⚙️ Configuration File

On first run, Edged automatically creates a configuration file at:

`~/.config/edged/config.toml`

Full configuration options:

```toml
[general]
# Default mode when starting without arguments: "bounds", "spacing", "horizontal", "vertical"
default_mode = "spacing"

# Whether to close the application immediately after copying a measurement
close_on_copy = true

[monitor]
# Multi-monitor target mode: "focused", "all", "primary", or specific connector name ("DP-1", "HDMI-A-1", "eDP-1")
# - "focused": (Default & Recommended) Always targets the currently active / focused monitor where your cursor or active window is.
#              Avoids multi-monitor glitches, scaling differences, and black/misaligned surfaces.
# - "all": Spans the ruler overlay across all connected monitors.
# - "primary": Targets the primary display.
# - "<name>": Targets a specific monitor by connector name (e.g. "DP-1", "HDMI-A-1", "Virtual-1").
target = "focused"

# When target is a single monitor ("focused", "primary", or "<name>"), automatically crop multi-monitor screenshots
# so that the ruler overlay matches the target monitor 1:1 without displaying pieces of other monitors.
crop_to_monitor = true

[capture]
# Screen capture method: "auto", "grim", "portal", "custom"
# - "auto": Tries 'grim' first (fastest on wlroots/Hyprland/Sway), falls back to xdg-desktop-portal (GNOME/KDE)
# - "grim": Forces using the 'grim' utility
# - "portal": Forces using XDG Desktop Portal (PipeWire / Screenshot portal)
# - "custom": Executes the custom command specified below
method = "auto"

# Custom shell command when method = "custom"
# Available path and monitor placeholders:
#   %s or %path     Full temporary output file path (e.g. /tmp/edged_custom_1234.png)
#   %d or %dir      Temporary directory (e.g. /tmp)
#   %n or %filename Temporary output filename without directory (e.g. edged_custom_1234.png)
#   %o or %output   Target monitor connector name (e.g. DP-1, Virtual-1)
#   %w or %width    Target monitor width in pixels
#   %h or %height   Target monitor height in pixels
#   %x              Target monitor x offset
#   %y              Target monitor y offset
#
# Verified Examples:
#   # Grim (standard Wayland / wlroots tool, takes output file directly):
#   command = "grim %s"
#
#   # Grim targeting specific monitor connector:
#   command = "grim -o %o %s"
#
#   # Hyprshot (Recommended: use --raw to write directly to stdout and redirect to %s):
#   command = "hyprshot -m output -s --raw > %s"
#
#   # Hyprshot (Alternative using -o for folder and -f for filename):
#   command = "hyprshot -m output -s -o %d -f %n"
#
#   # Wayshot (takes output file as positional argument, -o is for monitor name):
#   command = "wayshot %s"
#
#   # Grimblast (Hyprland grim/slurp wrapper):
#   command = "grimblast save screen %s"
#
#   # Spectacle (KDE: -b background non-interactive, -n no notify, -o output file):
#   command = "spectacle -b -n -o %s"
#
#   # Grimshot (Sway grim/slurp wrapper):
#   command = "grimshot save screen %s"
#
#   # Flameshot (fullscreen raw output redirected to %s):
#   command = "flameshot full -r > %s"
command = ""

# Timeout in seconds for custom capture command execution
timeout_seconds = 5

[shortcuts]
# Key to cycle through modes in order: "Tab" (or any key name like "grave", "space")
# Tip: Shift + Tab will cycle in reverse!
cycle_mode_key = "Tab"

# Direct number keys to switch modes
mode_bounds_key = "1"
mode_spacing_key = "2"
mode_horizontal_key = "3"
mode_vertical_key = "4"

# Alternative mnemonic letter keys to switch modes
alt_mode_bounds_key = "b"
alt_mode_spacing_key = "s"
alt_mode_horizontal_key = "h"
alt_mode_vertical_key = "v"

# Allow Shift + Mouse Scroll to cycle through modes during runtime
shift_scroll_cycles_mode = true

# Allow Ctrl + Mouse Scroll to cycle through modes during runtime
ctrl_scroll_cycles_mode = true

[ruler]
# Initial tolerance percentage for edge detection (1 to 100)
initial_tolerance = 15

# Tolerance adjustment step per mouse scroll wheel tick
tolerance_step = 2

# Draw feet (perpendicular tick marks at line ends showing exact measurement bounds)
draw_feet = true

# Size of the feet tick marks in pixels
feet_size = 10.0

# Thickness of the measurement lines in pixels
line_thickness = 2.0

# Display a subtle crosshair at the cursor position
draw_crosshair = true

[bounds]
# In Bounds mode, holding Shift keeps previously drawn rectangles pinned to the screen
shift_pins_bounds = true

# Border thickness for rectangles in Bounds mode
border_thickness = 2.0

[appearance]
# Dim overlay opacity (0.0 = completely transparent, 0.05 = subtle dimming)
overlay_opacity = 0.05

# Colors formatted as hex "#RRGGBB", "#RRGGBBAA", or "rgba(r, g, b, a)"
line_color = "#3B82F6"
line_shadow_color = "rgba(0, 0, 0, 0.6)"

bounds_fill_color = "rgba(59, 130, 246, 0.15)"
bounds_border_color = "#3B82F6"

# Label badges styling
label_bg_color = "rgba(15, 23, 42, 0.90)"
label_text_color = "#F8FAFC"
label_border_color = "#475569"
label_font_family = "sans-serif"
label_font_size = 12.0
label_border_radius = 5.0
label_padding_x = 7.0
label_padding_y = 3.5

[units]
# Show the "px" suffix on labels
show_pixel_unit = true

# Optional secondary unit: "none", "inch", "cm", "mm", "pt"
secondary_unit = "none"

# Screen DPI for secondary unit calculations
screen_dpi = 96.0

[clipboard]
# Text format copied to clipboard on click:
# - "compact": e.g. "1920x1080", "120"
# - "verbose": e.g. "1920 × 1080 px", "120 px"
# - "css":     e.g. "width: 1920px; height: 1080px;", "width: 120px;"
format = "compact"
```

---

## 🛠️ Build and Installation

### Prerequisites

Because Edged uses the pure Rust **egui** immediate-mode GPU engine, it does **not** require any heavy C desktop libraries (no GTK4, no Cairo, no Pango, no libgtk4-layer-shell). Only standard Rust and a Wayland environment are required:

#### Arch Linux
```bash
sudo pacman -S rust cargo
# Optional (recommended for Hyprland/Sway):
sudo pacman -S grim wl-clipboard
```

#### Fedora
```bash
sudo dnf install rust cargo
# Optional for wlroots compositors:
sudo dnf install grim wl-clipboard
```

#### Ubuntu / Debian
```bash
sudo apt install cargo
# Optional:
sudo apt install grim wl-clipboard
```

### Compiling Release Binary

```bash
git clone https://github.com/txg0fk3/Edged.git
cd Edged
cargo build --release
```

The compiled binary will be placed at `target/release/edged`.

### System Installation

```bash
# Copy binary to your user PATH
cp target/release/edged ~/.local/bin/

# Or install system-wide
sudo cp target/release/edged /usr/local/bin/

# Optional: Install desktop entry for app launchers (wofi, rofi, fuzzel)
cp edged.desktop ~/.local/share/applications/
```

---

## 🧪 Testing

To run the automated test suite:

```bash
cargo test
```

The test suite covers:
- Mode cycling logic (`next` / `prev`)
- Keyboard shortcut matching (Tab, Shift+Tab, single chars, named keys)
- Monitor target parsing (`focused`, `all`, `primary`, connector names)
- Multi-monitor screenshot auto-cropping to target monitor bounds
- Command placeholder expansion (`%s`, `%path`, `%d`, `%dir`, `%n`, `%filename`, `%o`, `%w`, `%h`, `%x`, `%y`)
- Directory fallback and newest image detection
- Synthetic box and edge boundary detection
- Gradient tolerance thresholds
- Bounds rectangle geometry and reverse drag normalization
- Clipboard formatting (`compact`, `verbose`, `css`)
- Color parser (hex, rgb, rgba)
- Configuration file creation, parsing, and validation
- Cairo surface conversion

---

## 📄 License

Distributed under either MIT License or Apache-2.0 License.
