use std::time::Duration;

use eframe::{CreationContext, Frame, egui::{self, CentralPanel, Color32, ColorImage, Context, Image, Key, Label, TextureOptions, Ui}};
use jiff::Timestamp; // TODO: I should probably just use the standard library for this
use moving_avg::MovingAverage;

use crate::chip8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH, PIXEL_ON};

const STARTING_SCALE: f32 = 10.0;
const FRAME_TIME_SAMPLES: usize = 60;

pub struct Chip8App {
    chip8: Chip8,
    display_texture: egui::TextureHandle,
    scale: f32,
    current_frame: Timestamp,
    previous_frame: Timestamp,
    frame_time: MovingAverage<f64>,
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
            current_frame: Timestamp::now(),
            previous_frame: Timestamp::now(),
            frame_time: MovingAverage::new(FRAME_TIME_SAMPLES),
        }
    }
}

impl eframe::App for Chip8App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        ui.ctx().request_repaint_after(Duration::from_millis(1000 / 60));
        self.previous_frame = self.current_frame;
        self.current_frame = Timestamp::now();
        let avg = self.frame_time.feed(self.current_frame.duration_since(self.previous_frame).as_millis_f64());
        
        self.chip8.process_input(poll_keyboard(ui.ctx()));
        for _ in 0..60 {
          self.chip8.cycle();
        }
        let image = framebuffer_to_image(self.chip8.get_video());
        self.display_texture.set(image, TextureOptions::NEAREST);

        CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("-")).clicked() {
                    self.scale -= 1.0;
                    self.scale = self.scale.max(1.0);
                }
                ui.add(Label::new(self.scale.to_string()));
                if ui.add(egui::Button::new("+")).clicked() {
                    self.scale += 1.0;
                    self.scale = self.scale.min(20.0);
                }
            });
            ui.add(Image::new(&self.display_texture).fit_to_original_size(self.scale));
            ui.label(format!("Frame time: {avg:.2}"));
            ui.label(format!("FPS: {:.2}", 1000.0 / avg));
            ui.label(format!("CPU Frequency: {:.2} Hz", 60000.0 / avg));
        });
        //ui.ctx().request_repaint();
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
