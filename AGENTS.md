# AGENTS.md — Developer & AI Agent Guidelines for Edged 📐

> Essential context, business logic, architectural constraints, testing protocols, and development preferences for any human developer or AI Agent working on the **Edged** codebase.

---

## 1. Project Overview & Identity

- **Project Name**: Edged
- **Repository**: `https://github.com/TXG0Fk3/Edged`
- **Primary Maintainer**: TXG0Fk3 (`txg0fk3@proton.me`)
- **Primary Branch**: `master` (All work merges directly into `master`; no stale feature branches)
- **License**: MIT (2026, TXG0Fk3)
- **Nature of the Project**: High-performance, pixel-accurate PowerToys Screen Ruler replica natively designed for Linux Wayland desktops (Hyprland, Sway, River, Niri, GNOME Shell, KDE Plasma).

### 🤖 Vibecoding Disclosure (Crucial Constraint)
> [!IMPORTANT]
> **Edged is explicitly developed as an experiment vibecoded with Google Antigravity** (an advanced agentic AI coding assistant).
> - **Always preserve** this transparent notice in the `README.md`, release descriptions, and project metadata.
> - **Never describe Edged as "handcrafted"** or "manually written from scratch". Always state honestly that it is an experimental vibecoded tool explored and maintained via agentic AI collaboration.

---

## 2. Core Architectural Principles (Non-Negotiables)

1. **Pure Rust GPU Rendering (Zero GTK C Dependencies)**:
   - The UI is built entirely with [`egui`](https://github.com/emilk/egui) and [`eframe`](https://docs.rs/eframe) running on `glow` (OpenGL) and `winit` over Wayland.
   - **NEVER reintroduce GTK, Cairo, Pango, or GObject C libraries.**
   - The compiled release binary must remain self-contained, dynamically linking only to `libc` and `libm`.

2. **One-Shot Execution Model (No Daemons)**:
   - Edged is **not** a resident daemon. It follows the Unix philosophy of Wayland tools like `slurp` and `grim`:
     $$\text{Launch} \longrightarrow \text{Freeze Screen} \longrightarrow \text{Measure} \longrightarrow \text{Copy to Clipboard} \longrightarrow \text{Terminate}$$
   - Do not convert Edged into a background service consuming idle RAM or CPU cycles.

3. **Zero-Flicker Freeze-Frame Capture**:
   - Before presenting the overlay, Edged captures the desktop snapshot into an `image::RgbaImage` and converts it into an `egui::ColorImage` texture.
   - The overlay renders the frozen image as its background, completely eliminating Wayland screen tearing, black bars, or recursive screencast feedback loops.

4. **Compositor Window Rules & Tiling IPC**:
   - Edged sends dynamic compositor IPC commands on launch (`hyprctl keyword windowrulev2` on Hyprland, `swaymsg` on Sway) to ensure the overlay surface opens:
     - Floating (never tiled into a split workspace)
     - Borderless and unshadowed
     - Pinned to the active monitor with zero launch animations.

5. **Multi-Monitor & Fractional Scaling Precision**:
   - Always query compositor IPC to target the display currently holding the user's cursor or focused window.
   - When third-party capture utilities return a composite canvas of multiple monitors, Edged automatically crops the image down to the physical bounds of the active monitor (`crop_image_if_needed`).

---

## 3. Codebase Anatomy & Responsibilities

```
Edged/
├── .github/
│   └── workflows/
│       ├── ci.yml                 # PR and Push test runner (Arch Linux container)
│       └── release.yml            # Multi-distro release packager (gated by test job)
├── assets/                        # Application icons (SVG and 256x256 PNG)
├── scripts/
│   └── build-arch-packages.sh     # Comprehensive packaging script (.pkg.tar.zst, .deb, .rpm, .AppImage)
├── src/
│   ├── main.rs                    # Entry point, CLI routing, compositor IPC, window init
│   ├── cli.rs                     # Clap argument definitions and flags
│   ├── config.rs                  # TOML configuration loader, defaults, color parser
│   ├── capture.rs                 # Capture pipeline (custom cmd, grim, hyprshot, portal)
│   ├── monitor.rs                 # Compositor monitor discovery and image cropping
│   ├── clipboard.rs               # arboard + wl-copy clipboard management
│   ├── logger.rs                  # Structured logging (console + daily log files) with 4-day retention
│   ├── ruler/
│   │   ├── mod.rs                 # RulerMode state machine & EdgeDetector algorithm
│   │   └── bounds.rs              # Rect geometry, center calculation, point containment
│   └── ui/
│       ├── mod.rs                 # OverlayApp state, hotkeys, input handling, HUD timers
│       └── renderer.rs            # egui rendering for crosshairs, badges, feet, and pinned boxes
├── tests/
│   └── ruler_tests.rs             # Unit and integration test suite (41 tests)
├── Cargo.toml                     # Manifest, dependencies, and packaging metadata
├── config.example.toml            # Fully commented reference configuration
├── install.sh                     # Automated user/system installer script
├── MANUAL_TESTING.md              # Hardware/compositor manual QA test plan & checklist
└── README.md                      # Public documentation and cheatsheet
```

---

## 4. Key Subsystem Implementation Details

### Edge Detection Algorithm (`src/ruler/mod.rs`)
- Cardinal ray-marching extends outward from `(cx, cy)` in 4 directions: Left, Right, Up, Down.
- Distance metric is squared Euclidean color distance in RGB space:
  $$\Delta E^2 = (R - R_{\text{base}})^2 + (G - G_{\text{base}})^2 + (B - B_{\text{base}})^2$$
- Converted threshold:
  $$\text{threshold}^2 = \left( \frac{\text{tolerance\_pct}}{100.0} \times \sqrt{255^2 \times 3} \right)^2$$
- Cursor positions are clamped to `[0, width - 1]` and `[0, height - 1]` to prevent out-of-bounds panics.

### Screen Capture Pipeline (`src/capture.rs`)
- Fallback hierarchy for `CaptureMethod::Auto`:
  1. Configured custom capture command (if non-empty)
  2. `grim` (direct wlroots protocol, fastest, zero user prompts)
  3. `hyprshot -m output -s` (on Hyprland sessions)
  4. XDG Desktop Portal via `ashpd::desktop::screenshot` (works on GNOME, KDE, and sandboxes)
- Template placeholders for custom commands:
  - `%s` / `%path`: Full absolute path to target temporary image file
  - `%d` / `%dir`: Directory of the output file
  - `%n` / `%filename`: Filename of the output file
  - `%o`: Monitor connector name (e.g. `DP-1`)
  - `%x`, `%y`, `%w`, `%h`: Monitor physical geometry

### Structured Logging & 4-Day Retention (`src/logger.rs`)
- Logs to two targets simultaneously:
  1. **Standard Error (Console)**: ANSI colored output for real-time terminal debugging.
  2. **Persistent Daily Files**: Plaintext logs located at `~/.local/state/edged/logs/edged-YYYY-MM-DD.log`.
  3. Symlink `latest.log` points to today's active log.
- **Strict 4-Day Retention**: On every startup, `prune_old_logs(dir, 4)` scans the log directory:
  - Evaluates both file modification timestamp (`SystemTime`) and calendar date in filename (`edged-YYYY-MM-DD.log`).
  - Purges log files older than 4 days.
  - Leaves `latest.log`, subdirectories, and non-Edged files untouched.
- CLI logging tools:
  - `edged --view-logs`: Prints active log file contents.
  - `edged --log-dir`: Prints absolute log folder path.
  - `edged --clear-logs`: Wipes all Edged logs on demand.
  - `edged --log-level <LEVEL>`: Overrides verbosity (`trace`, `debug`, `info`, `warn`, `error`, `off`).

---

## 5. Coding & Contribution Rules

1. **Strictly English**:
   - Write all code, comments, docstrings, commit messages, and documentation in English.
   - User communication may be in Portuguese or the user's preferred language, but repository code remains 100% English.

2. **Zero Clippy Warnings**:
   - Always run and pass:
     ```bash
     cargo clippy --all-targets -- -D warnings
     ```
   - Resolve all linter suggestions (e.g. `strip_prefix`, `sort_by_key`, collapsed `if` blocks, redundant casts) rather than suppressing them.

3. **Zero Test Regressions**:
   - Run the complete test suite before committing:
     ```bash
     cargo test --all-targets --verbose
     ```
   - All tests must pass cleanly.

4. **Conventional Commits**:
   - Format commit messages cleanly:
     - `feat: ...` for new features
     - `fix: ...` for bug fixes
     - `test: ...` for test suite additions
     - `ci: ...` for CI/CD workflow updates
     - `docs: ...` for documentation changes
     - `refactor: ...` for non-functional code refactoring

---

## 6. Release & Versioning Engineering

### 3-Part Date SemVer Requirement
Cargo strictly enforces 3-part SemVer (`MAJOR.MINOR.PATCH`). **Never use 4-part dot versions** like `2026.10.25.1` (Cargo will fail to parse `Cargo.toml`).
- Always format date versions as:
  $$\text{v}YYYY.MMDD.BUILD \quad \text{(e.g. } \text{v}2026.1025.1 \text{)}$$
- `scripts/build-arch-packages.sh` and `.github/workflows/release.yml` automatically compute and inject this format.

### Two-Stage Release Gating
- In `.github/workflows/release.yml`, the `release` job explicitly declares `needs: test`.
- If the `test` job fails (compiler error, clippy warning, or unit test failure), the `release` job is **automatically cancelled**.
- `scripts/build-arch-packages.sh` contains an internal `cargo test --release` assertion as a second safety net.

### Supported Distribution Packages
The packaging pipeline generates 6 distinct distribution formats in `dist/`:
1. `edged`: Standalone executable
2. `edged-*-x86_64.pkg.tar.zst`: Native Arch Linux package (built via standard `makepkg`)
3. `edged_*_amd64.deb`: Debian / Ubuntu / Linux Mint package (via `cargo-deb`)
4. `edged-*.x86_64.rpm`: Fedora / RHEL / openSUSE package (via `cargo-generate-rpm`)
5. `edged-*-x86_64.AppImage`: Standalone AppImage for all Linux distributions
6. `edged-*-x86_64-linux.tar.gz`: Universal portable tarball with `install.sh`

---

## 7. Testing Strategy for Future Agents

When modifying features:
1. **Algorithms, Math, Config, Logging**: Add automated tests to `tests/ruler_tests.rs`.
2. **Compositor, Wayland Protocols, Capture, UI Hardware**: Follow the protocols defined in `MANUAL_TESTING.md`.
3. If an automated test cannot recreate a bug because it requires a live Wayland display server, document the manual verification procedure in `MANUAL_TESTING.md`.
