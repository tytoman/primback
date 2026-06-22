use crate::primback::types::{
    color::{Color, ColorWithAlpha},
    rect_transform::{Anchor},
};
use glam::{vec2, Vec2};

/// Represents text configuration and rendering settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    /// The string content to render.
    pub content: String,
    /// The size multiplier for the text.
    ///
    /// Note: Non-integer scale values may cause the text to render blurry or degraded.
    pub scale: f32,
    /// Horizontal anchor alignment.
    pub anchor_x: Anchor,
    /// Vertical anchor alignment.
    pub anchor_y: Anchor,
    /// Drop shadow offset, if enabled.
    pub shadow_offset: Option<Vec2>,
    /// Drop shadow color, if enabled.
    pub shadow_color: Option<ColorWithAlpha>,
    /// Maximum width before wrapping, if enabled.
    pub max_width: Option<f32>,
}

impl Text {
    /// Creates a new `Text` with default settings (scale 1.0, top-left anchor, no shadow).
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            scale: 1.0,
            anchor_x: Anchor::Start,
            anchor_y: Anchor::Start,
            shadow_offset: None,
            shadow_color: None,
            max_width: None,
        }
    }

    /// Sets the scale of the text.
    ///
    /// Note: Non-integer scale values may cause the text to render blurry or degraded.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the horizontal anchor alignment.
    pub fn anchor_x(mut self, anchor_x: Anchor) -> Self {
        self.anchor_x = anchor_x;
        self
    }

    /// Sets the vertical anchor alignment.
    pub fn anchor_y(mut self, anchor_y: Anchor) -> Self {
        self.anchor_y = anchor_y;
        self
    }

    /// Sets both the horizontal and vertical anchor alignments.
    pub fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.anchor_x = anchor_x;
        self.anchor_y = anchor_y;
        self
    }

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

    /// Enables or disables drop shadow.
    pub fn shadow(mut self, enabled: bool) -> Self {
        if enabled {
            if self.shadow_offset.is_none() {
                self.shadow_offset = Some(vec2(1.0, 1.0));
            }
            if self.shadow_color.is_none() {
                self.shadow_color = Some(ColorWithAlpha::from(Color::BLACK));
            }
        } else {
            self.shadow_offset = None;
            self.shadow_color = None;
        }
        self
    }

    /// Sets the maximum width of the text before wrapping.
    pub fn max_width(mut self, max_width: f32) -> Self {
        if max_width <= 0.0 {
            log::warn!("Text max_width must be greater than 0.0. Treating as None (disabled). Got: {}", max_width);
            self.max_width = None;
        } else {
            self.max_width = Some(max_width);
        }
        self
    }

}

impl<S: Into<String>> From<S> for Text {
    fn from(s: S) -> Self {
        Text::new(s)
    }
}

