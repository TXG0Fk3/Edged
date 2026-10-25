use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "edged",
    author = "Edged Contributors",
    version,
    about = "High-quality PowerToys Screen Ruler replica natively built for Wayland (Hyprland, GNOME, Sway)",
    long_about = "Edged is a fast, pixel-accurate screen measuring tool for Wayland.\n\
                  It replicates the PowerToys Screen Ruler with real-time edge detection,\n\
                  bounds measuring, scroll-wheel dynamic tolerance, and clipboard copying."
)]
pub struct Cli {
    /// Bounds mode: click and drag to measure rectangular areas
    #[arg(long, conflicts_with_all = ["spacing", "horizontal", "vertical"])]
    pub bounds: bool,

    /// Spacing mode: 4-way cross measuring margins and element dimensions
    #[arg(long, conflicts_with_all = ["bounds", "horizontal", "vertical"])]
    pub spacing: bool,

    /// Horizontal mode: measures horizontal distance between color edges
    #[arg(long, conflicts_with_all = ["bounds", "spacing", "vertical"])]
    pub horizontal: bool,

    /// Vertical mode: measures vertical distance between color edges
    #[arg(long, conflicts_with_all = ["bounds", "spacing", "horizontal"])]
    pub vertical: bool,

    /// Override initial edge detection tolerance percentage (1 to 100)
    #[arg(short, long)]
    pub tolerance: Option<u8>,

    /// Target monitor: 'focused' (default), 'all', 'primary', or connector name (e.g. 'DP-1')
    #[arg(short, long)]
    pub monitor: Option<String>,

    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
