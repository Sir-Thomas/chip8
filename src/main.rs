#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chip8;

use eframe::{NativeOptions, Result, egui::ViewportBuilder, run_native};

use crate::chip8::Chip8;

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let mut args = std::env::args();
    args.next();
    let rom: String = args.next().unwrap_or_else(|| String::from("roms/particle_demo.ch8"));
    let mut chip8 = Chip8::new();
    chip8.load_fontset();
    chip8.load_rom(&rom);

    run_native(
        "Chip-8",
        options,
        Box::new(|cc| {
            Ok(Box::new(app::Chip8App::new(cc, chip8)))
        }),
    )
}
