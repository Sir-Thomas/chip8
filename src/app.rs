use std::time::{Duration, Instant};

use eframe::{CreationContext, Frame, egui::{self, CentralPanel, Color32, ColorImage, Context, Image, Key, TextureOptions, Ui}};
use moving_avg::MovingAverage;

use crate::chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_ON};

const STARTING_SCALE: f32 = 10.0;
const STARTING_CPU_FREQ: f64 = 1000.0;
const SAMPLE_SIZE: usize = 60;

pub struct Chip8App {
    chip8: Chip8,
    display_texture: egui::TextureHandle,
    scale: f32,
    timer_accumulator: Duration,
    timer_duration: Duration,
    cycle_accumulator: Duration,
    cpu_freq: f64,
    current_frame: Instant,
    previous_frame: Instant,
    frame_time: MovingAverage<f64>,
    cycles_per_sec: MovingAverage<f64>,
}

impl Chip8App {
    pub fn new(cc: &CreationContext<'_>, chip8: Chip8) -> Self {
        let display_texture = cc.egui_ctx.load_texture(
            "chip8-display",
            ColorImage::new([DISPLAY_WIDTH, DISPLAY_HEIGHT], vec![Color32::BLACK; 64 * 32]),
            TextureOptions::NEAREST,
        );

        Self {
            chip8,
            display_texture,
            scale: STARTING_SCALE,
            timer_accumulator: Duration::ZERO,
            timer_duration: Duration::from_secs_f64(1.0 / 60.0),
            cycle_accumulator: Duration::ZERO,
            cpu_freq: STARTING_CPU_FREQ,
            current_frame: Instant::now(),
            previous_frame: Instant::now(),
            frame_time: MovingAverage::new(SAMPLE_SIZE),
            cycles_per_sec: MovingAverage::new(SAMPLE_SIZE),
        }
    }
}

impl eframe::App for Chip8App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        //ui.ctx().request_repaint_after(Duration::from_secs_f64(1.0 / 60.0));
        self.previous_frame = self.current_frame;
        self.current_frame = Instant::now();
        let frame_time = self.current_frame.duration_since(self.previous_frame);
        self.timer_accumulator = self.timer_accumulator.saturating_add(frame_time);
        self.cycle_accumulator = self.cycle_accumulator.saturating_add(frame_time);
        
        self.chip8.process_input(poll_keyboard(ui.ctx()));
        let mut cycles = 0.0;
        let cycle_duration = Duration::from_secs_f64(1.0 / self.cpu_freq);
        while self.cycle_accumulator > cycle_duration {
          self.chip8.cycle();
          self.cycle_accumulator = self.cycle_accumulator.saturating_sub(cycle_duration);
          cycles += 1.0;
        }
        while self.timer_accumulator > self.timer_duration {
            self.chip8.decrement_timers();
            self.timer_accumulator = self.timer_accumulator.saturating_sub(self.timer_duration);
        }
        let image = framebuffer_to_image(self.chip8.get_video());
        self.display_texture.set(image, TextureOptions::NEAREST);

        CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Scale: ");
                if ui.add(egui::Button::new("-")).clicked() {
                    self.scale -= 1.0;
                    self.scale = self.scale.max(1.0);
                }
                ui.label(self.scale.to_string());
                if ui.add(egui::Button::new("+")).clicked() {
                    self.scale += 1.0;
                    self.scale = self.scale.min(20.0);
                }
            });
            ui.horizontal(|ui| {
                ui.label("CPU Frequency: ");
                if ui.add(egui::Button::new("-")).clicked() {
                    self.cpu_freq -= 100.0;
                }
                ui.label(self.cpu_freq.to_string());
                if ui.add(egui::Button::new("+")).clicked() {
                    self.cpu_freq += 100.0;
                }
            });
            ui.add(Image::new(&self.display_texture).fit_to_original_size(self.scale));
            let avg = self.frame_time.feed(frame_time.as_millis_f64());
            ui.label(format!("Frame time: {avg:.2} ms"));
            ui.label(format!("FPS: {:.2}", 1000.0 / avg));
            let freq = self.cycles_per_sec.feed(cycles / frame_time.as_secs_f64());
            ui.label(format!("CPU Frequency: {freq:.2} Hz"));
        });
        ui.ctx().request_repaint();
    }
}

fn framebuffer_to_image(framebuffer: &[u32; DISPLAY_WIDTH * DISPLAY_HEIGHT]) -> ColorImage {
    let pixels = framebuffer
        .iter()
        .map(|&pixel| if pixel == PIXEL_ON { Color32::WHITE } else { Color32::BLACK })
        .collect();

    ColorImage::new([DISPLAY_WIDTH, DISPLAY_HEIGHT], pixels)
}

fn poll_keyboard(ctx: &Context) -> [bool; 16] {
    // layout
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F
    ctx.input(|i| {[
        i.key_down(Key::X),
        i.key_down(Key::Num1),
        i.key_down(Key::Num2),
        i.key_down(Key::Num3),
        i.key_down(Key::Q),
        i.key_down(Key::W),
        i.key_down(Key::E),
        i.key_down(Key::A),
        i.key_down(Key::S),
        i.key_down(Key::D),
        i.key_down(Key::Z),
        i.key_down(Key::C),
        i.key_down(Key::Num4),
        i.key_down(Key::R),
        i.key_down(Key::F),
        i.key_down(Key::V),
    ]})
}
