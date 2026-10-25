# Edged — Nix & NixOS Documentation

> Complete guide for installing, configuring, and packaging Edged declaratively using Nix, NixOS, and Home Manager.

---

## Table of Contents

- [Overview](#overview)
- [Flake Outputs](#flake-outputs)
- [Quick Run (`nix run`)](#quick-run-nix-run)
- [Adding to Flake Inputs](#adding-to-flake-inputs)
- [Home Manager Module](#home-manager-module)
  - [Module Options](#module-options)
  - [Complete Configuration Example](#complete-configuration-example)
- [NixOS System Module](#nixos-system-module)
- [Nixpkgs Overlay](#nixpkgs-overlay)
- [Packaging Implementation & Runtime Environment](#packaging-implementation--runtime-environment)
- [Automated Source Updating](#automated-source-updating)
- [Troubleshooting](#troubleshooting)

---

## Overview

Edged provides a first-class **Nix Flake** and modular integrations for both **Home Manager** and **NixOS**.

### Key Architectural Characteristics
- **Zero Local Compilation**: The derivation fetches the official precompiled release tarball (`edged-${VERSION}-x86_64-linux.tar.gz`), eliminating Rust compilation overhead on client machines.
- **Dynamic Wayland Patching**: Uses `autoPatchelfHook` to patch binary ELF headers and `wrapProgram` to prefix `LD_LIBRARY_PATH` with runtime graphics and Wayland libraries (`libGL`, `libxkbcommon`, `wayland`, `vulkan-loader`).
- **Bundled Runtime Tools**: `wrapProgram` prefixes `PATH` with `grim` (Wayland screencopy) and `wl-clipboard` (`wl-copy`), ensuring screenshotting and clipboard copying work out of the box on minimal NixOS setups.
- **Declarative TOML Generation**: Uses `pkgs.formats.toml` to serialize native Nix attribute sets directly into valid TOML configuration files.
- **Continuous Self-Updating**: Every GitHub Actions release computes the release tarball's SHA-256 hash, updates `nix/sources.json`, and commits back to `master`.

---

## Flake Outputs

The flake at the root of this repository exports the following attributes:

| Output | Description |
| :--- | :--- |
| `packages.x86_64-linux.default` | Standalone packaged derivation of Edged. |
| `packages.x86_64-linux.edged` | Alias to `packages.x86_64-linux.default`. |
| `apps.x86_64-linux.default` | Executable app for `nix run`. |
| `apps.x86_64-linux.edged` | Alias to `apps.x86_64-linux.default`. |
| `homeManagerModules.default` | Home Manager module for per-user installation and configuration. |
| `homeManagerModules.edged` | Alias to `homeManagerModules.default`. |
| `nixosModules.default` | NixOS module for system-wide installation and configuration. |
| `nixosModules.edged` | Alias to `nixosModules.default`. |
| `overlays.default` | Nixpkgs overlay providing `pkgs.edged`. |
| `devShells.x86_64-linux.default` | Development shell with Rust toolchain, Wayland headers, and libraries. |

---

## Quick Run (`nix run`)

You can launch Edged immediately on any system with Nix installed (flakes enabled) without installing it:

```bash
# Launch in default mode (Spacing)
nix run github:TXG0Fk3/Edged

# Launch with CLI arguments (e.g., Bounds mode)
nix run github:TXG0Fk3/Edged -- --bounds

# Launch with custom sensitivity tolerance
nix run github:TXG0Fk3/Edged -- --spacing --tolerance 30

# Inspect version
nix run github:TXG0Fk3/Edged -- --version
```

To run directly from a local clone of this repository:
```bash
nix run . -- --bounds
```

---

## Adding to Flake Inputs

In your system or home flake (`flake.nix`), add Edged to your inputs:

```nix
{
  description = "NixOS / Home Manager configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    edged = {
      url = "github:TXG0Fk3/Edged";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, edged, ... }@inputs: {
    # NixOS or Home Manager configurations...
  };
}
```

> [!TIP]
> Setting `inputs.edged.inputs.nixpkgs.follows = "nixpkgs"` ensures Edged shares the exact same package set and shared libraries as your host system, minimizing closure size.

---

## Home Manager Module

The Home Manager module (`inputs.edged.homeManagerModules.default`) installs the wrapped package into `home.packages` and renders your declarative settings into:
```
$XDG_CONFIG_HOME/edged/config.toml  (typically ~/.config/edged/config.toml)
```

### Module Options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `programs.edged.enable` | `bool` | `false` | Whether to enable Edged screen ruler. |
| `programs.edged.package` | `package` | Flake package | The package to install (automatically resolves to the flake derivation without requiring an overlay or `pkgs.edged`). |
| `programs.edged.settings` | `attrsOf anything` | `{ }` | Declarative configuration serialized to TOML via `pkgs.formats.toml`. |

If `programs.edged.settings` is left empty (`{ }`), no configuration file is written, allowing Edged to use its internal defaults or a manual file.

### Complete Configuration Example

Below is a comprehensive configuration demonstrating all available settings in Home Manager:

```nix
{ inputs, pkgs, ... }:

{
  imports = [
    inputs.edged.homeManagerModules.default
  ];

  programs.edged = {
    enable = true;

    # Optional: override package derivation if using an overlay
    # package = pkgs.edged;

    settings = {
      general = {
        # Default measurement mode on launch: "spacing", "bounds", "horizontal", "vertical"
        default_mode = "spacing";

        # Automatically terminate overlay immediately after copying measurement
        close_on_copy = true;
      };

      monitor = {
        # Display selection: "focused" (recommended), "all", "primary", or connector identifier ("DP-1")
        target = "focused";

        # Crop composite multi-monitor screenshots to active monitor physical bounds
        crop_to_monitor = true;
      };

      capture = {
        # Capture backend: "auto", "grim", "portal", "custom"
        method = "auto";

        # Shell command when method = "custom" (supports %s, %o, %w, %h, %x, %y placeholders)
        command = "";

        # Command timeout in seconds
        timeout_seconds = 5;
      };

      shortcuts = {
        # Cycle through modes (Shift + key cycles backwards)
        cycle_mode_key = "Tab";

        # Numeric direct mode shortcuts
        mode_bounds_key = "1";
        mode_spacing_key = "2";
        mode_horizontal_key = "3";
        mode_vertical_key = "4";

        # Mnemonic letter shortcuts
        alt_mode_bounds_key = "b";
        alt_mode_spacing_key = "s";
        alt_mode_horizontal_key = "h";
        alt_mode_vertical_key = "v";

        # Mouse wheel mode cycling
        shift_scroll_cycles_mode = true;
        ctrl_scroll_cycles_mode = true;
      };

      ruler = {
        # Initial color difference tolerance (1 to 100)
        initial_tolerance = 15;

        # Percentage adjustment per mouse wheel notch
        tolerance_step = 2;

        # Render perpendicular tick marks ('feet') at measurement line endpoints
        draw_feet = true;
        feet_size = 10.0;
        line_thickness = 2.0;

        # Render crosshair guide under cursor
        draw_crosshair = true;
      };

      bounds = {
        # Hold Shift upon releasing drag to pin rectangle to screen
        shift_pins_bounds = true;
        border_thickness = 2.0;
      };

      appearance = {
        # Background dimming factor (0.0 to 1.0)
        overlay_opacity = 0.05;

        # Measurement line colors (#RRGGBB, #RRGGBBAA, or rgba(...))
        line_color = "#3B82F6";
        line_shadow_color = "rgba(0, 0, 0, 0.60)";

        # Bounds mode rectangle styling
        bounds_fill_color = "rgba(59, 130, 246, 0.15)";
        bounds_border_color = "#3B82F6";

        # Dimension badge styling
        label_bg_color = "rgba(15, 23, 42, 0.90)";
        label_text_color = "#F8FAFC";
        label_border_color = "#475569";
        label_font_family = "sans-serif";
        label_font_size = 12.0;
        label_border_radius = 5.0;
        label_padding_x = 7.0;
        label_padding_y = 3.5;
      };

      units = {
        # Append "px" suffix on badges
        show_pixel_unit = true;

        # Secondary unit conversion: "none", "inch", "cm", "mm", "pt"
        secondary_unit = "none";

        # Physical DPI for unit conversion
        screen_dpi = 96.0;
      };

      clipboard = {
        # Output format: "compact" (1920x1080), "verbose" (1920 × 1080 px), "css"
        format = "compact";
      };

      logging = {
        enabled = true;
        level = "info";
        retention_days = 4;
        log_to_console = true;
        log_to_file = true;
      };
    };
  };
}
```

---

## NixOS System Module

If you prefer installing Edged system-wide rather than through Home Manager, import `inputs.edged.nixosModules.default` in `configuration.nix`.

The NixOS module:
1. Adds `cfg.package` to `environment.systemPackages`.
2. Generates the system-wide configuration file at `/etc/xdg/edged/config.toml`.

```nix
{ inputs, pkgs, ... }:

{
  imports = [
    inputs.edged.nixosModules.default
  ];

  programs.edged = {
    enable = true;

    settings = {
      general = {
        default_mode = "spacing";
        close_on_copy = true;
      };

      ruler = {
        initial_tolerance = 15;
      };
    };
  };
}
```

### Configuration Precedence Order
Edged checks configuration paths in the following priority:
1. `--config <path>` command-line argument (highest priority).
2. User configuration: `~/.config/edged/config.toml` (`$XDG_CONFIG_HOME/edged/config.toml`).
3. System-wide configuration: `$XDG_CONFIG_DIRS/edged/config.toml` or `/etc/xdg/edged/config.toml`.
4. Embedded default fallback configuration.

If a user defines a local config in Home Manager or `~/.config/edged/config.toml`, it automatically overrides the system-wide NixOS settings.

---

## Nixpkgs Overlay

If you manage packages via overlays rather than modules, you can add `inputs.edged.overlays.default` to your `nixpkgs.overlays`:

```nix
{ inputs, ... }:

{
  nixpkgs.overlays = [
    inputs.edged.overlays.default
  ];

  # Now pkgs.edged is available anywhere in your configuration:
  environment.systemPackages = [
    pkgs.edged
  ];
}
```

---

## Packaging Implementation & Runtime Environment

The derivation definition is located at [`nix/package.nix`](package.nix).

### Build Details
```nix
stdenv.mkDerivation {
  pname = "edged";
  inherit (sources) version;
  src = fetchurl { inherit (sources) url sha256; };
  dontBuild = true;
  dontConfigure = true;
  nativeBuildInputs = [ autoPatchelfHook makeWrapper ];
  buildInputs = [ stdenv.cc.cc.lib libGL libxkbcommon wayland vulkan-loader ];
}
```

### Dynamic Linking & Wrapping
1. **ELF Interpreter & Runpath**: `autoPatchelfHook` inspects the release binary, matches required dynamic symbols (`libc.so.6`, `libgcc_s.so.1`, `libm.so.6`), and embeds the proper Nix store paths into the binary's `RPATH`.
2. **Wayland & OpenGL dlopen Wrapping**: Wayland libraries (`libwayland-client.so.0`, `libxkbcommon.so.0`, `libGL.so.1`, `libvulkan.so.1`) are loaded dynamically by `glow` and `winit` at runtime. The derivation generates a shell wrapper (`bin/edged`) that prepends these libraries to `LD_LIBRARY_PATH`.
3. **Screenshot & Clipboard Tools**: The wrapper injects `grim` and `wl-clipboard` (`wl-copy`) into `PATH`.

---

## Automated Source Updating

The package source metadata is stored in [`nix/sources.json`](sources.json):

```json
{
  "version": "v2026.1025.2",
  "clean_version": "2026.1025.2",
  "url": "https://github.com/TXG0Fk3/Edged/releases/download/v2026.1025.2/edged-v2026.1025.2-x86_64-linux.tar.gz",
  "sha256": "1cd899bb19320989149f3a2a9606c2e0b4151f5ecdd85dd7ea7c17ab5a59846c"
}
```

### Continuous Integration Pipeline
In `.github/workflows/release.yml`, immediately after a release build passes all gated tests and publishes to GitHub Releases:
1. The workflow computes the exact SHA-256 hash of `edged-${VERSION}-x86_64-linux.tar.gz`.
2. Rewrites `nix/sources.json`.
3. Pushes a commit back to `master` with `[skip ci]`.

### Manual Update Script
Developers or maintainers can manually check and update the release pointer using [`nix/update.sh`](update.sh):

```bash
# Automatically query GitHub API for the latest release and update sources.json
./nix/update.sh

# Or target a specific release tag
./nix/update.sh v2026.1025.2
```

---

## Troubleshooting

### `experimental Nix feature 'nix-command' is disabled`
Enable flakes and the new CLI in `/etc/nix/nix.conf` or `~/.config/nix/nix.conf`:
```text
experimental-features = nix-command flakes
```

### OpenGL / Hardware Acceleration on NixOS
Because Edged uses GPU hardware rendering via `egui_glow` / OpenGL:
- On NixOS 24.11 and newer: Ensure `hardware.graphics.enable = true;` is set in `configuration.nix`.
- On older NixOS versions: Ensure `hardware.opengl.enable = true;`.

### Verifying Generated Configuration
To inspect the exact TOML configuration file produced by Home Manager:
```bash
cat ~/.config/edged/config.toml
```
Or for NixOS system-wide configuration:
```bash
cat /etc/xdg/edged/config.toml
```

### Inspecting Local Flake Derivation
```bash
# Validate all flake outputs and module structures
nix flake check

# View the full evaluation tree
nix flake show

# Build local package into ./result
nix build -L
```
