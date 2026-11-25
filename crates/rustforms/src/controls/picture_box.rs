use tiny_skia::Pixmap;

use crate::{Color, Control, ControlBase, ControlImpl, EventCx, Painter, Point, Rect};

/// Как картинка занимает контрол.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeMode {
    /// Целиком, с сохранением пропорций.
    #[default]
    Zoom,
    /// Растянуть на весь контрол.
    Stretch,
    /// Исходный размер по центру.
    Center,
}

/// Картинка PNG.
#[derive(Control)]
pub struct PictureBox {
    base: ControlBase,
    image: Option<Pixmap>,
    pub size_mode: SizeMode,
}

impl PictureBox {
    pub fn new() -> Self {
        Self { base: ControlBase::new(120.0, 90.0, false), image: None, size_mode: SizeMode::Zoom }
    }

    /// Картинка из байтов PNG, например `include_bytes!("logo.png")`.
    /// Возвращает `false`, если это не PNG.
    pub fn set_image_png(&mut self, png: &[u8]) -> bool {
        self.image = Pixmap::decode_png(png).ok();
        self.image.is_some()
    }

    /// Загружает PNG с диска.
    pub fn load(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let bytes = std::fs::read(path)?;
        if self.set_image_png(&bytes) {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "это не PNG"))
        }
    }

    pub fn clear(&mut self) {
        self.image = None;
    }

    /// Размер картинки в пикселях.
    pub fn image_size(&self) -> Option<(u32, u32)> {
        self.image.as_ref().map(|i| (i.width(), i.height()))
    }
}

impl Default for PictureBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlImpl for PictureBox {
    fn base(&self) -> &ControlBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ControlBase {
        &mut self.base
    }

    fn on_paint(&mut self, p: &mut Painter) {
        let area = p.area();
        if let Some(back) = p.back {
            p.fill_rect(area, back);
        }
        let Some(image) = &self.image else {
            p.stroke_rect(area, Color::BORDER, 1.0);
            return;
        };
        let (w, h) = (image.width() as f32, image.height() as f32);
        let (w, h) = match self.size_mode {
            SizeMode::Stretch => (area.width, area.height),
            SizeMode::Center => (w, h),
            SizeMode::Zoom => {
                let k = (area.width / w).min(area.height / h);
                (w * k, h * k)
            }
        };
        let target = Rect::new((area.width - w) / 2.0, (area.height - h) / 2.0, w, h);
        p.image(image, target);
    }

    fn on_mouse_up(&mut self, _at: Point, inside: bool, cx: &mut EventCx) {
        if inside {
            cx.click();
        }
    }
}
