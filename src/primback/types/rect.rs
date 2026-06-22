use crate::primback::types::{
    color::{Color, ColorWithAlpha},
    rect_transform::Anchor,
};
use glam::{Vec2, vec2};

/// A UI rectangle structure supporting rounding, outlines, and shadows.
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub size: Vec2,
    pub color: ColorWithAlpha,
    pub radius: [f32; 4], // [top_left, top_right, bottom_right, bottom_left]
    pub outline_width: Option<f32>,
    pub outline_color: Option<ColorWithAlpha>,
    pub shadow_offset: Option<Vec2>,
    pub shadow_color: Option<ColorWithAlpha>,
    pub anchor_x: Anchor,
    pub anchor_y: Anchor,
}

impl Rect {
    /// Creates a new `Rect` with the specified size.
    pub fn new(size: impl Into<Vec2>) -> Self {
        Self {
            size: size.into(),
            color: Color::WHITE.into(),
            radius: [0.0; 4],
            outline_width: None,
            outline_color: None,
            shadow_offset: None,
            shadow_color: None,
            anchor_x: Anchor::Start,
            anchor_y: Anchor::Start,
        }
    }

    /// Sets the horizontal anchor.
    pub fn anchor_x(mut self, anchor_x: Anchor) -> Self {
        self.anchor_x = anchor_x;
        self
    }

    /// Sets the vertical anchor.
    pub fn anchor_y(mut self, anchor_y: Anchor) -> Self {
        self.anchor_y = anchor_y;
        self
    }

    /// Sets both the horizontal and vertical anchors.
    pub fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.anchor_x = anchor_x;
        self.anchor_y = anchor_y;
        self
    }

    /// Sets the background color.
    pub fn color(mut self, color: impl Into<ColorWithAlpha>) -> Self {
        self.color = color.into();
        self
    }

    /// Sets the corner rounding radius for all four corners.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = [radius; 4];
        self
    }

    /// Sets the top-left corner rounding radius.
    pub fn radius_tl(mut self, radius: f32) -> Self {
        self.radius[0] = radius;
        self
    }

    /// Sets the top-right corner rounding radius.
    pub fn radius_tr(mut self, radius: f32) -> Self {
        self.radius[1] = radius;
        self
    }

    /// Sets the bottom-right corner rounding radius.
    pub fn radius_br(mut self, radius: f32) -> Self {
        self.radius[2] = radius;
        self
    }

    /// Sets the bottom-left corner rounding radius.
    pub fn radius_bl(mut self, radius: f32) -> Self {
        self.radius[3] = radius;
        self
    }

    // Outline builders
    /// Sets the outline width.
    pub fn outline_width(mut self, width: f32) -> Self {
        self.outline_width = Some(width);
        self
    }

    /// Sets the outline color.
    pub fn outline_color(mut self, color: impl Into<ColorWithAlpha>) -> Self {
        self.outline_color = Some(color.into());
        self
    }

    /// Toggles the outline with default settings.
    pub fn outline(mut self, enabled: bool) -> Self {
        if enabled {
            if self.outline_width.is_none() {
                self.outline_width = Some(1.0);
            }
            if self.outline_color.is_none() {
                self.outline_color = Some(Color::BLACK.into());
            }
        } else {
            self.outline_width = None;
            self.outline_color = None;
        }
        self
    }

    // Shadow builders
    /// Sets the shadow offset.
    pub fn shadow_offset(mut self, offset: impl Into<Vec2>) -> Self {
        self.shadow_offset = Some(offset.into());
        self
    }

    /// Sets the shadow color.
    pub fn shadow_color(mut self, color: impl Into<ColorWithAlpha>) -> Self {
        self.shadow_color = Some(color.into());
        self
    }

    /// Toggles the shadow with default settings.
    pub fn shadow(mut self, enabled: bool) -> Self {
        if enabled {
            if self.shadow_offset.is_none() {
                self.shadow_offset = Some(vec2(1.0, 1.0));
            }
            if self.shadow_color.is_none() {
                self.shadow_color = Some(Color::BLACK.into());
            }
        } else {
            self.shadow_offset = None;
            self.shadow_color = None;
        }
        self
    }

    /// Returns true if the specified screen coordinate is inside this `Rect` under the given transformation.
    pub fn contains_point(
        &self,
        transform: &crate::primback::types::rect_transform::RectTransform,
        point: Vec2,
    ) -> bool {
        // 1. Convert to local coordinates where scale and rotation are already handled
        let p_local = transform.to_local_point(point);

        // 2. Center coordinates based on anchor
        let b = self.size * 0.5;
        let anchor_shift_x = match self.anchor_x {
            Anchor::Start => b.x,
            Anchor::Center => 0.0,
            Anchor::End => -b.x,
        };
        let anchor_shift_y = match self.anchor_y {
            Anchor::Start => b.y,
            Anchor::Center => 0.0,
            Anchor::End => -b.y,
        };

        let p = p_local - Vec2::new(anchor_shift_x, anchor_shift_y);

        // 3. Evaluate Signed Distance Function for the rounded box
        let r = self.radius;
        let r_corner = if p.x > 0.0 {
            if p.y > 0.0 {
                r[2] // bottom_right
            } else {
                r[1] // top_right
            }
        } else {
            if p.y > 0.0 {
                r[3] // bottom_left
            } else {
                r[0] // top_left
            }
        };

        let max_radius = b.x.min(b.y);
        let rad = r_corner.min(max_radius);

        let q = p.abs() - b + Vec2::splat(rad);
        let d = q.x.max(q.y).min(0.0) + q.max(Vec2::ZERO).length() - rad;

        d <= 0.0
    }
}

impl<V: Into<Vec2>> From<V> for Rect {
    fn from(v: V) -> Self {
        Rect::new(v)
    }
}
