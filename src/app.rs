pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const PIXEL_ON: u32 = 0xFFFFFFFF;

use eframe::egui::{self, Context};

use crate::chip8;

pub struct Chip8App {
    chip8: chip8::Chip8,
    display_texture: egui::TextureHandle,
    scale: usize,
}

impl Chip8App {
    pub fn new(cc: &eframe::CreationContext<'_>, chip8: chip8::Chip8) -> Self {
        let display_texture = cc.egui_ctx.load_texture(
            "chip8-display",
            egui::ColorImage::new([DISPLAY_WIDTH, DISPLAY_HEIGHT], vec![egui::Color32::BLACK; 64 * 32]),
            egui::TextureOptions::NEAREST,
        );

        Self {
            chip8: chip8,
            display_texture: display_texture,
            scale: 10,
        }
    }
}

impl eframe::App for Chip8App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for _ in 0..60 {
          self.chip8.process_input(poll_keyboard(ui.ctx()));
          self.chip8.cycle();
        }
        let image = framebuffer_to_image(self.chip8.get_video());
        self.display_texture.set(image, egui::TextureOptions::NEAREST);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("-")).clicked() {
                    self.scale -= 1;
                }
                // to_string().as_str() is needed because we first have to convert the usize to a
                // String, then convert the String to &str to make the TextEdit show the number
                // without allowing manual entries
                ui.add(egui::TextEdit::singleline(&mut self.scale.to_string().as_str())
                    .desired_width(0.0)
                    .clip_text(false)
                );
                if ui.add(egui::Button::new("+")).clicked() {
                    self.scale += 1;
                }
            });
            ui.add(egui::Image::new(&self.display_texture).fit_to_original_size(self.scale as f32));
        });

        ui.ctx().request_repaint();
    }
}

fn framebuffer_to_image(array: [u32; DISPLAY_WIDTH * DISPLAY_HEIGHT]) -> egui::ColorImage {
    let pixels = array
        .iter()
        .map(|&pixel| if pixel == PIXEL_ON { egui::Color32::WHITE } else { egui::Color32::BLACK })
        .collect();

    egui::ColorImage::new([DISPLAY_WIDTH, DISPLAY_HEIGHT], pixels)
}

fn poll_keyboard(ctx: &Context) -> [bool; 16] {
    // layout
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F
    ctx.input(|i| {[
        i.key_down(egui::Key::X),
        i.key_down(egui::Key::Num1),
        i.key_down(egui::Key::Num2),
        i.key_down(egui::Key::Num3),
        i.key_down(egui::Key::Q),
        i.key_down(egui::Key::W),
        i.key_down(egui::Key::E),
        i.key_down(egui::Key::A),
        i.key_down(egui::Key::S),
        i.key_down(egui::Key::D),
        i.key_down(egui::Key::Z),
        i.key_down(egui::Key::C),
        i.key_down(egui::Key::Num4),
        i.key_down(egui::Key::R),
        i.key_down(egui::Key::F),
        i.key_down(egui::Key::V),
    ]})
}
