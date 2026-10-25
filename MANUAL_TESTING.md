# Edged — Manual Quality Assurance & Testing Plan 📐

> Comprehensive manual testing protocol designed to validate hardware, compositor, screen capture, and interactive features that cannot be verified by automated unit tests.

---

## 🎯 Purpose

While Edged includes an extensive automated test suite covering algorithm mathematics, geometric projections, color parsing, and configuration loading, **desktop integration on Wayland involves hardware-level interactions that require manual validation**:

- Compositor surface layering (`zwlr_layer_shell_v1` / floating window rules)
- Hardware-accelerated screen capture pipelines (`grim`, `hyprshot`, XDG Desktop Portal via `ashpd`)
- Multi-monitor coordinate translation and fractional scaling (1.0x, 1.25x, 1.5x, 2.0x)
- Wayland clipboard data-source exchange (`arboard` / `wl-copy`)
- Visual freeze-frame synchronization without screen tearing or black flickers
- Keyboard input capture and immediate process termination

This document provides a step-by-step manual test plan. At the end of the document is a **standardized checklist** formatted for easy copy-pasting directly to an AI agent for triage.

---

## 🛠️ Prerequisites & Setup

Ensure the latest version is compiled locally before running manual tests:

```bash
# Compile optimized debug or release binary
cargo build --release

# Alias for convenient testing
alias edged="./target/release/edged"
```

Verify your desktop session:
```bash
echo "Compositor: $XDG_CURRENT_DESKTOP | Session: $XDG_SESSION_TYPE | Display: $WAYLAND_DISPLAY"
```

---

## 🧪 Test Scenarios

### Test Suite 1: Screen Capture & Freeze-Frame

#### Case 1.1: Default Capture Execution
- **Objective**: Verify that Edged captures the screen instantly with zero flicker or black frames.
- **Steps**:
  1. Open a web browser or text editor with visible content.
  2. Launch Edged from a terminal: `edged`
  3. Observe the screen.
- **Expected Result**:
  - The screen freezes instantaneously.
  - The measurement crosshair appears immediately beneath the cursor.
  - No black flash, window repositioning jump, or visual tearing occurs.

#### Case 1.2: Grim Direct Capture
- **Objective**: Ensure native wlroots capture works cleanly.
- **Steps**:
  1. Run: `edged --capture-method grim`
- **Expected Result**: Immediate capture without user prompt on Hyprland, Sway, or River.

#### Case 1.3: XDG Desktop Portal Fallback
- **Objective**: Verify portal compatibility on GNOME, KDE, or sandbox environments.
- **Steps**:
  1. Run: `edged --capture-method portal`
- **Expected Result**:
  - Desktop portal authorization dialog appears (if not previously granted).
  - Upon granting permission, the snapshot is captured and overlay presented.

#### Case 1.4: Custom Capture Command with Placeholders
- **Objective**: Verify custom CLI tool invocation with `%s` target expansion.
- **Steps**:
  1. Run: `edged --capture-command "grim %s"`
- **Expected Result**: Edged uses the custom command template, captures successfully, and presents the overlay.

---

### Test Suite 2: Spacing Mode (Crosshair & Padding Inspection)

#### Case 2.1: Element Boundary Tracking
- **Objective**: Confirm 4-way crosshair stops at contrast borders.
- **Steps**:
  1. Launch Edged: `edged --mode spacing`
  2. Move cursor inside a distinct UI element (e.g., a button or card with borders).
- **Expected Result**:
  - Four orthogonal colored measurement lines extend from the cursor.
  - Lines stop precisely at the outer contrast edges of the button.
  - Perpendicular tick marks ('feet') render at the endpoints.
  - A badge shows `Width × Height` in pixels.

#### Case 2.2: Free Space Expansion
- **Objective**: Confirm crosshair reaches screen edges when in solid color background.
- **Steps**:
  1. Move the cursor over a solid empty background.
- **Expected Result**: Lines extend all the way to screen boundaries without artifacting or freezing.

---

### Test Suite 3: Bounds Mode & Shift-to-Pin

#### Case 3.1: Rectangular Click-and-Drag
- **Objective**: Verify dynamic rectangle measurement.
- **Steps**:
  1. Switch to Bounds mode: Press <kbd>1</kbd> or <kbd>B</kbd>.
  2. Left-click and drag across any window or icon.
- **Expected Result**:
  - A semi-transparent blue rectangle fills the dragged area.
  - A badge at the center shows the current dimensions (e.g. `240 × 160 px`).
  - Dimensions update smoothly in real time during drag.

#### Case 3.2: Shift-to-Pin Feature
- **Objective**: Test keeping rectangles persistent on screen while drawing others.
- **Steps**:
  1. While in Bounds mode, click and drag a rectangle.
  2. **Hold <kbd>Shift</kbd>** before releasing the mouse button.
  3. Release the mouse button while still holding <kbd>Shift</kbd>.
- **Expected Result**:
  - The drawn rectangle remains pinned on the screen.
  - You can immediately click and drag a second rectangle elsewhere.
  - Both rectangles remain visible simultaneously.

#### Case 3.3: Selection of Pinned Rectangle
- **Objective**: Verify copying dimensions from an existing pinned box.
- **Steps**:
  1. Click inside one of the previously pinned rectangles without holding Shift.
- **Expected Result**: Its dimensions are copied to the clipboard, and Edged closes.

---

### Test Suite 4: Horizontal & Vertical 1D Modes

#### Case 4.1: Horizontal Mode
- **Objective**: Test single-axis width measurement.
- **Steps**:
  1. Press <kbd>3</kbd> or <kbd>H</kbd> to enter Horizontal mode.
  2. Hover between two vertical borders (e.g. a column or sidebar).
- **Expected Result**: Only a single horizontal measurement line is displayed with its width badge.

#### Case 4.2: Vertical Mode
- **Objective**: Test single-axis height measurement.
- **Steps**:
  1. Press <kbd>4</kbd> or <kbd>V</kbd> to enter Vertical mode.
  2. Hover between two horizontal bars (e.g. a topbar and a card header).
- **Expected Result**: Only a single vertical measurement line is displayed with its height badge.

---

### Test Suite 5: In-Flight Mode Cycling

#### Case 5.1: Keyboard Hotkeys
- **Objective**: Ensure seamless switching during an active session.
- **Steps**:
  1. Launch Edged.
  2. Tap <kbd>Tab</kbd> repeatedly.
  3. Tap <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>.
  4. Tap <kbd>B</kbd>, <kbd>S</kbd>, <kbd>H</kbd>, <kbd>V</kbd>.
- **Expected Result**:
  - Mode switches immediately on each keystroke in the expected order:
    `Spacing` $\to$ `Bounds` $\to$ `Horizontal` $\to$ `Vertical` $\to$ `Spacing`.
  - Floating HUD notification momentarily shows the active mode name.

#### Case 5.2: Mouse Wheel Mode Rotation
- **Objective**: Test cycling modes via wheel with modifiers.
- **Steps**:
  1. Hold <kbd>Shift</kbd> and scroll the mouse wheel up and down.
  2. Hold <kbd>Ctrl</kbd> and scroll the mouse wheel up and down.
- **Expected Result**: Active mode cycles forward and backward without requiring keyboard access.

---

### Test Suite 6: Edge Detection Sensitivity / Tolerance

#### Case 6.1: Real-Time Tolerance Adjustment
- **Objective**: Adjust edge sensitivity threshold on gradient surfaces.
- **Steps**:
  1. Open an image or wallpaper with a color gradient.
  2. Launch Edged in Spacing mode over the gradient.
  3. Scroll mouse wheel **UP** 5 notches.
  4. Scroll mouse wheel **DOWN** 5 notches.
- **Expected Result**:
  - Scrolling UP increases tolerance (+2% per notch), causing lines to extend further through subtle contrast variations.
  - Scrolling DOWN tightens tolerance (-2% per notch), causing lines to stop at subtle shade transitions.
  - A centered HUD displays the current tolerance percentage (e.g. `Tolerance: 15%`).

---

### Test Suite 7: Multi-Monitor & Fractional Scaling

#### Case 7.1: Focused Display Targeting
- **Objective**: Ensure overlay launches on the active monitor without black borders.
- **Steps**:
  1. Connect two monitors.
  2. Move cursor to Monitor 2 and focus an active window.
  3. Launch Edged from hotkey or terminal.
- **Expected Result**:
  - Edged opens exclusively on Monitor 2.
  - Coordinates align pixel-for-pixel (no black borders, no monitor bleed).

#### Case 7.2: High-DPI & Fractional Scaling
- **Objective**: Verify 1.25x / 1.5x scale factor visual accuracy.
- **Steps**:
  1. Set display scale to 125% or 150% in compositor settings.
  2. Launch Edged and measure a known 100x100 px box.
- **Expected Result**:
  - Overlay renders sharply without blurry UI elements.
  - Measurement reflects accurate physical pixel dimensions.

---

### Test Suite 8: Clipboard Formatting

#### Case 8.1: Compact Format (Default)
- **Steps**: Run `edged --format compact`, measure a 200x100 element, left-click.
- **Expected Result**: Paste (`Ctrl+V`) outputs: `200x100`.

#### Case 8.2: Verbose Format
- **Steps**: Run `edged --format verbose`, measure a 200x100 element, left-click.
- **Expected Result**: Paste (`Ctrl+V`) outputs: `200 × 100 px`.

#### Case 8.3: CSS Format
- **Steps**: Run `edged --format css`, measure a 200x100 element, left-click.
- **Expected Result**: Paste (`Ctrl+V`) outputs: `width: 200px; height: 100px;`.

---

### Test Suite 9: Exit & Dismissal Lifecycle

#### Case 9.1: Dismiss via Escape / Q / Right-Click
- **Objective**: Abort measurement without copying.
- **Steps**:
  1. Launch Edged.
  2. Press <kbd>Esc</kbd>, or press <kbd>Q</kbd>, or **Right-Click**.
- **Expected Result**:
  - Edged immediately closes.
  - Clipboard contents are left unchanged.
  - No background process remains (`ps aux | grep edged` returns nothing).

#### Case 9.2: Persistent Mode (`--no-close-on-copy`)
- **Objective**: Keep overlay open after copying.
- **Steps**:
  1. Launch `edged --no-close-on-copy`.
  2. Left-click to copy a measurement.
- **Expected Result**:
  - Dimensions are copied to the clipboard.
  - Overlay remains active until manually dismissed with <kbd>Esc</kbd>.

---

### Test Suite 10: Logging & Diagnostic CLI Commands

#### Case 10.1: View Logs CLI
- **Steps**: Run `edged --view-logs`
- **Expected Result**: Terminal displays active session logs with timestamps, log levels, and system metadata.

#### Case 10.2: Automatic 4-Day Log Pruning
- **Steps**:
  1. Inspect log directory: `ls -la $(edged --log-dir)`
- **Expected Result**:
  - Logs are organized as `edged-YYYY-MM-DD.log`.
  - A `latest.log` symlink points to today's log.
  - Files older than 4 days are automatically purged on launch.

#### Case 10.3: Clear Logs CLI
- **Steps**: Run `edged --clear-logs`
- **Expected Result**: Terminal confirms deleted log count, and the log folder is emptied cleanly.

---

### Test Suite 11: Nix Flake, Home Manager & NixOS Packaging

#### Case 11.1: Direct Nix Run Execution
- **Steps**: Run `nix run github:TXG0Fk3/Edged` on a NixOS or Nix-enabled system.
- **Expected Result**:
  - The precompiled binary package is fetched without compiling locally.
  - Wayland OpenGL overlay launches smoothly without missing `.so` library errors.

#### Case 11.2: Home Manager Module Configuration
- **Steps**:
  1. Add `programs.edged.enable = true` and `programs.edged.settings.general.default_mode = "bounds"` to home-manager configuration.
  2. Run `home-manager switch`.
  3. Inspect `~/.config/edged/config.toml`.
- **Expected Result**: File is generated with `default_mode = "bounds"` and Edged defaults to bounds mode on startup.

#### Case 11.3: NixOS System-Wide Module Configuration
- **Steps**:
  1. Add `programs.edged.enable = true` to `configuration.nix`.
  2. Run `nixos-rebuild switch`.
  3. Verify `/etc/xdg/edged/config.toml` presence if settings are defined.
- **Expected Result**: Edged is accessible system-wide in `PATH`.

---

## 📋 AI Report Checklist (Copy & Paste to AI)

> 💡 **Instructions for User**: Copy the checklist below, check the boxes `[x]` for items that worked and leave `[ ]` for items that failed. Paste it into the conversation with the AI along with any notes.

```markdown
# Edged Manual QA Results

**Environment Information**:
- Compositor: [e.g. Hyprland 0.44 / Sway 1.9 / GNOME 47 / KDE 6.2]
- Architecture / OS: [e.g. Arch Linux x86_64]
- Scale Factor: [e.g. 1.0x / 1.25x / 1.5x / 2.0x]
- Monitor Count: [e.g. 1 / 2 (stacked) / 2 (side-by-side)]

---

### Test Results:

- [ ] **1. Screen Capture & Freeze-Frame**
  - [ ] 1.1 Default capture is instantaneous with zero screen flicker
  - [ ] 1.2 Grim direct capture works without prompt
  - [ ] 1.3 Portal capture works with system prompt
  - [ ] 1.4 Custom capture command (%s) functions properly

- [ ] **2. Spacing Mode**
  - [ ] 2.1 Crosshair lines stop precisely at contrast borders
  - [ ] 2.2 Tick marks ('feet') render at line endpoints
  - [ ] 2.3 Lines span to monitor borders over solid backgrounds

- [ ] **3. Bounds Mode**
  - [ ] 3.1 Dragging creates visible semi-transparent rectangle with badge
  - [ ] 3.2 Shift + Drag release keeps rectangles pinned to screen
  - [ ] 3.3 Clicking a pinned rectangle copies its dimensions

- [ ] **4. 1D Distance Modes**
  - [ ] 4.1 Horizontal mode (3/H) shows only width between boundaries
  - [ ] 4.2 Vertical mode (4/V) shows only height between boundaries

- [ ] **5. In-Flight Mode Switching**
  - [ ] 5.1 Tab and hotkeys (1, 2, 3, 4 / B, S, H, V) cycle modes cleanly
  - [ ] 5.2 Shift/Ctrl + Mouse Wheel rotates modes

- [ ] **6. Tolerance & Sensitivity Adjustment**
  - [ ] 6.1 Mouse wheel up/down changes tolerance by +/- 2% per notch
  - [ ] 6.2 Floating tolerance HUD appears and reflects current percentage

- [ ] **7. Multi-Monitor & Scaling**
  - [ ] 7.1 Overlay targets the focused display holding the cursor
  - [ ] 7.2 No black borders, coordinate offsets, or image stretching
  - [ ] 7.3 Pixel dimensions match physical scale on fractional HiDPI

- [ ] **8. Clipboard Export**
  - [ ] 8.1 Left-click copies measurement to clipboard
  - [ ] 8.2 Compact format outputs "WxH" / "W"
  - [ ] 8.3 Verbose format outputs "W × H px"
  - [ ] 8.4 CSS format outputs "width: Wpx; height: Hpx;"

- [ ] **9. Exit & Lifecycle**
  - [ ] 9.1 Esc / Q / Right-click closes immediately without copying
  - [ ] 9.2 Left-click closes immediately after copying (default)
  - [ ] 9.3 `--no-close-on-copy` keeps overlay open after click
  - [ ] 9.4 No orphaned processes left running in background

- [ ] **10. Logging System**
  - [ ] 10.1 `edged --view-logs` outputs formatted log entries
  - [ ] 10.2 Logs older than 4 days are automatically pruned
  - [ ] 10.3 `edged --clear-logs` clears the log directory cleanly

- [ ] **11. Nix Flake & Modules**
  - [ ] 11.1 `nix run github:TXG0Fk3/Edged` runs without compiling from source
  - [ ] 11.2 Home Manager module writes settings to `~/.config/edged/config.toml`
  - [ ] 11.3 NixOS module provides system-wide package and config integration

---

### Notes / Observed Issues:
<!-- Describe any bugs, visual glitches, unexpected behavior, or terminal errors here -->
```
