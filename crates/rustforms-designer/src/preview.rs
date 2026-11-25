//! Картинка формы, нарисованная самим движком RustForms, — вид как у готовой программы.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use rustforms_model::FormModel;

use crate::render::Renderer;

#[derive(Default)]
pub struct Preview {
    renderer: Renderer,
    drawn: Option<(FormModel, f32)>,
    texture: Option<TextureHandle>,
}

impl Preview {
    pub fn texture(&mut self, ctx: &egui::Context, model: &FormModel) -> Option<&TextureHandle> {
        let scale = ctx.pixels_per_point();
        if self.drawn.as_ref().is_none_or(|(m, s)| m != model || *s != scale) {
            let pixmap = self.renderer.render(model, scale)?;
            let size = [pixmap.width() as usize, pixmap.height() as usize];
            let image = ColorImage::from_rgba_premultiplied(size, pixmap.data());
            self.texture = Some(ctx.load_texture("form", image, TextureOptions::LINEAR));
            self.drawn = Some((model.clone(), scale));
        }
        self.texture.as_ref()
    }
}
