mod chip8;

fn main() {
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
        quit = chip8.process_input();
        let current_time = jiff::Timestamp::now();
        let delta = current_time - last_cycle_time;
        if delta.get_milliseconds() > delay {
            last_cycle_time = current_time;
            chip8.cycle();
            display_video(&chip8.video);
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
