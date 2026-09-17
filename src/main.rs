#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chip8;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let mut args = std::env::args();
    let _ = args.next().unwrap_or_else(|| String::new());
    let rom: String = args.next().unwrap_or_else(|| String::from("roms/particle_demo.ch8"));
    let _delay: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let mut chip8 = chip8::Chip8::new();
    chip8.load_fontset();
    chip8.load_rom(&rom);

    eframe::run_native(
        "Chip-8",
        options,
        Box::new(|cc| {
            Ok(Box::new(app::Chip8App::new(cc, chip8)))
        }),
    )
}


/*
fn init() {
    print!("\x1B[2J\x1B[H");
    let mut args = std::env::args();
    let _ = args.next().unwrap_or_else(|| String::new());
    let rom: String = args.next().unwrap_or_else(|| String::from("roms/particle_demo.ch8"));
    let delay: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let _scale: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut chip8 = chip8::Chip8::new();

    chip8.load_fontset();
    chip8.load_rom(&rom);

    let mut quit = false;

    let mut last_cycle_time = jiff::Timestamp::now();

    while !quit {
        let current_time = jiff::Timestamp::now();
        let delta = current_time - last_cycle_time;
        if delta.get_milliseconds() > delay {
            last_cycle_time = current_time;
            chip8.cycle();
            display_video(chip8.get_video());
            play_audio();
            update_keyboard();
        }
    }
}

fn display_video(video: &[u32; chip8::DISPLAY_WIDTH * chip8::DISPLAY_HEIGHT]) {
    print!("\x1B[2J\x1B[H");
    //print!("\x1B[H");
    
    for row in video.chunks_exact(chip8::DISPLAY_WIDTH) {
        for &pixel in row {
            print!("{}", if pixel == chip8::PIXEL_ON { "█" } else { " " });
        }
        println!();
    }
}

fn play_audio() {
}

fn update_keyboard() {
}
*/
