use std::io::Cursor;

use clap::Parser;
use env_logger::Env;
use icns::{IconFamily, PixelFormat};

mod diagnostics;
mod engine;
mod env;
mod jre;
mod mods;
mod process;
mod pwr;
mod storage;
mod ui;
mod updater;
mod util;

#[derive(Parser, Debug)]
#[command(
    name = "HRS Launcher",
    author,
    version,
    about = "Community launcher for Hytale with integrated diagnostics and mod downloads"
)]
struct Cli {
    /// Print launcher version and exit without starting the UI.
    #[arg(long)]
    version_only: bool,
}

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    if cli.version_only {
        println!("HRS Launcher {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_icon(app_icon())
            .with_inner_size(eframe::egui::vec2(1240.0, 760.0)),
        ..Default::default()
    };
    eframe::run_native(
        "HRS Launcher",
        options,
        Box::new(|cc| Ok(Box::new(ui::LauncherApp::new(cc)))),
    )
}

fn app_icon() -> eframe::egui::IconData {
    load_app_icon().unwrap_or_else(default_icon)
}

fn load_app_icon() -> Option<eframe::egui::IconData> {
    let bytes = include_bytes!("../assets/AppIcon.icns");
    let family = IconFamily::read(Cursor::new(bytes)).ok()?;

    let mut best: Option<icns::Image> = None;
    for icon_type in family.available_icons() {
        if let Ok(img) = family.get_icon_with_type(icon_type) {
            let new_area = img.width() * img.height();
            let should_replace = best
                .as_ref()
                .is_none_or(|current| new_area > current.width() * current.height());
            if should_replace {
                best = Some(img);
            }
        }
    }

    let img = best?;
    let rgba_img = img.convert_to(PixelFormat::RGBA);
    let (width, height) = (rgba_img.width(), rgba_img.height());
    Some(eframe::egui::IconData {
        rgba: rgba_img.into_data().into_vec(),
        width,
        height,
    })
}

fn default_icon() -> eframe::egui::IconData {
    // Simple 2x2 icon: dark background with a cyan accent.
    let rgba: Vec<u8> = vec![
        20, 24, 32, 255, 30, 196, 220, 255, //
        20, 24, 32, 255, 20, 150, 180, 255,
    ];
    eframe::egui::IconData {
        rgba,
        width: 2,
        height: 2,
    }
}
