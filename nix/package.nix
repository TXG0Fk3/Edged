{ lib
, stdenv
, fetchurl
, autoPatchelfHook
, makeWrapper
, wayland
, libxkbcommon
, libGL
, vulkan-loader
, grim
, wl-clipboard
}:

let
  sources = builtins.fromJSON (builtins.readFile ./sources.json);
in
stdenv.mkDerivation {
  pname = "edged";
  inherit (sources) version;

  src = fetchurl {
    inherit (sources) url sha256;
  };

  # Source tarball contains precompiled release artifacts
  dontBuild = true;
  dontConfigure = true;

  nativeBuildInputs = [
    autoPatchelfHook
    makeWrapper
  ];

  buildInputs = [
    stdenv.cc.cc.lib
    libGL
    libxkbcommon
    wayland
    vulkan-loader
  ];

  installPhase = ''
    runHook preInstall

    # 1. Main executable
    BIN_PATH=$(find . -maxdepth 2 -name "edged" -type f -perm -0100 2>/dev/null | head -n 1)
    if [ -z "$BIN_PATH" ]; then
      BIN_PATH=$(find . -maxdepth 2 -name "edged" -type f 2>/dev/null | head -n 1)
    fi
    install -Dm755 "$BIN_PATH" "$out/bin/edged"

    # 2. Desktop file
    DESKTOP_PATH=$(find . -name "edged.desktop" -type f 2>/dev/null | head -n 1)
    if [ -n "$DESKTOP_PATH" ]; then
      install -Dm644 "$DESKTOP_PATH" "$out/share/applications/edged.desktop"
    fi

    # 3. Application Icons
    ICON_PNG=$(find . -name "edged.png" -type f 2>/dev/null | head -n 1)
    if [ -n "$ICON_PNG" ]; then
      install -Dm644 "$ICON_PNG" "$out/share/icons/hicolor/256x256/apps/edged.png"
    fi

    ICON_SVG=$(find . -name "edged.svg" -type f 2>/dev/null | head -n 1)
    if [ -n "$ICON_SVG" ]; then
      install -Dm644 "$ICON_SVG" "$out/share/icons/hicolor/scalable/apps/edged.svg"
    fi

    # 4. Documentation & Configuration reference
    CONF_EXAMPLE=$(find . -name "config.example.toml" -type f 2>/dev/null | head -n 1)
    if [ -n "$CONF_EXAMPLE" ]; then
      install -Dm644 "$CONF_EXAMPLE" "$out/share/doc/edged/config.example.toml"
    fi

    README_PATH=$(find . -name "README.md" -type f 2>/dev/null | head -n 1)
    if [ -n "$README_PATH" ]; then
      install -Dm644 "$README_PATH" "$out/share/doc/edged/README.md"
    fi

    LICENSE_PATH=$(find . -name "LICENSE" -type f 2>/dev/null | head -n 1)
    if [ -n "$LICENSE_PATH" ]; then
      install -Dm644 "$LICENSE_PATH" "$out/share/licenses/edged/LICENSE"
    fi

    runHook postInstall
  '';

  postFixup = ''
    wrapProgram $out/bin/edged \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [
        libGL
        libxkbcommon
        wayland
        vulkan-loader
      ]} \
      --prefix PATH : ${lib.makeBinPath [
        grim
        wl-clipboard
      ]}
  '';

  meta = with lib; {
    description = "High-performance, pixel-accurate PowerToys Screen Ruler replica natively designed for Linux Wayland desktops";
    longDescription = ''
      Edged is an ultra-fast, zero-GTK Screen Ruler built with pure Rust (egui/eframe/glow).
      Designed natively for Wayland compositors (Hyprland, Sway, River, Niri, GNOME, KDE),
      it provides freeze-frame precision measurement modes (spacing, bounds, horizontal, vertical)
      with multi-monitor and fractional scaling support.
    '';
    homepage = "https://github.com/TXG0Fk3/Edged";
    license = licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "edged";
  };
}
