//! Якоря и Dock. Расчёт общий для программы и дизайнера.

use std::ops::BitOr;

use crate::Rect;

/// Стороны родителя, расстояние до которых контрол сохраняет при изменении его размера.
///
/// `LEFT | RIGHT` растягивает по ширине; только `RIGHT` двигает за правым краем;
/// без обеих контрол смещается на половину изменения. По вертикали так же.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

impl Anchor {
    pub const NONE: Anchor = Anchor { left: false, top: false, right: false, bottom: false };
    pub const LEFT: Anchor = Anchor { left: true, ..Anchor::NONE };
    pub const TOP: Anchor = Anchor { top: true, ..Anchor::NONE };
    pub const RIGHT: Anchor = Anchor { right: true, ..Anchor::NONE };
    pub const BOTTOM: Anchor = Anchor { bottom: true, ..Anchor::NONE };
}

impl Default for Anchor {
    /// Как в WinForms: к левому верхнему углу.
    fn default() -> Self {
        Anchor::LEFT | Anchor::TOP
    }
}

impl BitOr for Anchor {
    type Output = Anchor;

    fn bitor(self, other: Anchor) -> Anchor {
        Anchor {
            left: self.left || other.left,
            top: self.top || other.top,
            right: self.right || other.right,
            bottom: self.bottom || other.bottom,
        }
    }
}

/// Прижать к стороне родителя на всю её длину или занять оставшееся место.
///
/// Прижимаются по очереди добавления: первый оказывается у самого края.
/// `Fill` получает то, что осталось после всех остальных.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dock {
    #[default]
    None,
    Top,
    Bottom,
    Left,
    Right,
    Fill,
}

/// Новое положение после того, как родитель вырос на `dw` × `dh`.
pub fn anchored(mut rect: Rect, anchor: Anchor, dw: f32, dh: f32) -> Rect {
    fn axis(start: &mut f32, length: &mut f32, near: bool, far: bool, delta: f32) {
        match (near, far) {
            (true, true) => *length += delta,
            (false, true) => *start += delta,
            (false, false) => *start += delta / 2.0,
            (true, false) => {}
        }
    }
    axis(&mut rect.x, &mut rect.width, anchor.left, anchor.right, dw);
    axis(&mut rect.y, &mut rect.height, anchor.top, anchor.bottom, dh);
    rect
}

/// Размещает прижатые контролы одного родителя размером `size`. Каждый элемент —
/// способ прижатия и текущее положение, которое и меняется.
pub fn dock(size: (f32, f32), children: &mut [(Dock, Rect)]) {
    let mut free = Rect::new(0.0, 0.0, size.0, size.1);
    for (side, rect) in children.iter_mut() {
        match side {
            Dock::Top => {
                *rect = Rect::new(free.x, free.y, free.width, rect.height);
                free.y += rect.height;
                free.height -= rect.height;
            }
            Dock::Bottom => {
                free.height -= rect.height;
                *rect = Rect::new(free.x, free.y + free.height, free.width, rect.height);
            }
            Dock::Left => {
                *rect = Rect::new(free.x, free.y, rect.width, free.height);
                free.x += rect.width;
                free.width -= rect.width;
            }
            Dock::Right => {
                free.width -= rect.width;
                *rect = Rect::new(free.x + free.width, free.y, rect.width, free.height);
            }
            Dock::None | Dock::Fill => {}
        }
    }
    for (side, rect) in children.iter_mut() {
        if *side == Dock::Fill {
            *rect = Rect::new(free.x, free.y, free.width.max(0.0), free.height.max(0.0));
        }
    }
}
