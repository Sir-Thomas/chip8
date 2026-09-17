pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const PIXEL_ON: u32 = 0xFFFFFFFF;

use eframe::egui;

use crate::chip8;

pub struct Chip8App {
    chip8: chip8::Chip8,
    display_texture: egui::TextureHandle,
    scale: usize,
}

impl Chip8App {
    pub fn new(cc: &eframe::CreationContext<'_>, chip8: chip8::Chip8, scale: usize) -> Self {
        let display_texture = cc.egui_ctx.load_texture(
            "chip8-display",
            egui::ColorImage::new([DISPLAY_WIDTH, DISPLAY_HEIGHT], vec![egui::Color32::BLACK; 64 * 32]),
            egui::TextureOptions::NEAREST,
        );

        Self {
            chip8: chip8,
            display_texture: display_texture,
            scale: scale,
        }
    }
}

impl eframe::App for Chip8App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.chip8.cycle();
        let image = framebuffer_to_image(self.chip8.video);
        self.display_texture.set(image, egui::TextureOptions::NEAREST);

        egui::CentralPanel::default().show(ui, |ui| {
            let size = egui::vec2((DISPLAY_WIDTH * self.scale) as f32, (DISPLAY_HEIGHT * self.scale) as f32);
            ui.add(
                egui::Image::new(&self.display_texture)
                    .fit_to_exact_size(size)
            );
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
