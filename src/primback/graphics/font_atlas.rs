use crate::primback::types::text::Text;
use fontdue::{Font, FontSettings};
use glam::Vec2;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub width: f32,
    pub height: f32,
    pub xmin: f32,
    pub ymin: f32,
    pub advance: f32,
}

#[derive(Clone)]
pub struct FontConfig {
    /// Raw bytes of a TrueType (`.ttf`) or OpenType (`.otf`) font file.
    ///
    /// Use [`include_bytes!`] to embed the file at compile time:
    /// ```ignore
    /// data: include_bytes!("assets/my_font.ttf"),
    /// ```
    pub data: &'static [u8],
    pub native_height: f32,
    /// Additional characters to rasterize beyond printable ASCII (U+0020–U+007E).
    /// If `None`, only ASCII is included in the atlas.
    ///
    /// # Examples
    ///
    /// Include a fixed set of extra characters:
    /// ```ignore
    /// chars: Some(vec!['α', 'β', 'γ', '©', '™']),
    /// ```
    ///
    /// Include an entire Unicode range:
    /// ```ignore
    /// chars: Some(
    ///     ('ぁ'..='ん').chain('ァ'..='ン').collect()
    /// ),
    /// ```
    pub chars: Option<Vec<char>>,
}

impl std::fmt::Debug for FontConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontConfig")
            .field("data_len", &self.data.len())
            .field("native_height", &self.native_height)
            .field("chars_count", &self.chars.as_ref().map(|v| v.len()))
            .finish()
    }
}

fn simulate_packing_height(glyphs_metrics: &[(usize, usize)], probe_width: usize) -> usize {
    let mut cur_x = 1usize;
    let mut cur_y = 1usize;
    let mut row_height = 0usize;

    for &(g_width, g_height) in glyphs_metrics {
        if cur_x + g_width + 1 > probe_width {
            cur_x = 1;
            cur_y += row_height + 1;
            row_height = 0;
        }
        cur_x += g_width + 1;
        if g_height > row_height {
            row_height = g_height;
        }
    }
    cur_y + row_height + 1
}

fn calc_atlas_size(glyphs_metrics: &[(usize, usize)]) -> usize {
    let mut probe_width = 256usize;
    loop {
        let required_height = simulate_packing_height(glyphs_metrics, probe_width);
        let dim = probe_width.max(required_height);
        // 2のべき乗に切り上げ
        let mut size = 64usize;
        while size < dim {
            size *= 2;
        }
        // probe_width と atlas_size が一致していれば収束
        if size <= probe_width {
            return size;
        }
        probe_width = size;
    }
}

pub struct FontAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub native_height: f32,
    pub ascent: f32,
}

impl FontAtlas {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        config: &FontConfig,
    ) -> Result<Self, &'static str> {
        let font = Font::from_bytes(config.data, FontSettings::default())?;
        let line_metrics = font.horizontal_line_metrics(config.native_height);
        let ascent = line_metrics
            .map(|m| m.ascent)
            .unwrap_or(config.native_height);

        // Build the character set to rasterize.
        // Start with printable ASCII (U+0020–U+007E).
        let mut char_set: Vec<char> = (0x20u32..=0x7Eu32).filter_map(char::from_u32).collect();
        // Append any extra characters from FontConfig::chars, skipping duplicates.
        if let Some(extra) = &config.chars {
            for &ch in extra {
                if !char_set.contains(&ch) {
                    char_set.push(ch);
                }
            }
        }

        // Collect glyph metrics without rasterizing yet, then auto-size the atlas.
        let metrics_list: Vec<(usize, usize)> = char_set
            .iter()
            .map(|&ch| {
                let m = font.metrics(ch, config.native_height);
                (m.width, m.height)
            })
            .collect();

        let atlas_size = calc_atlas_size(&metrics_list);
        let atlas_width = atlas_size;
        let atlas_height = atlas_size;
        let mut atlas_data = vec![0u8; atlas_width * atlas_height];
        let mut glyphs = HashMap::new();

        let mut cur_x = 1usize;
        let mut cur_y = 1usize;
        let mut row_height = 0usize;

        for ch in char_set {
            let (metrics, bitmap) = font.rasterize(ch, config.native_height);

            let g_width = metrics.width;
            let g_height = metrics.height;

            if cur_x + g_width + 1 > atlas_width {
                cur_x = 1;
                cur_y += row_height + 1;
                row_height = 0;
            }

            // The atlas was sized to fit all glyphs, so overflow here indicates a bug.
            debug_assert!(
                cur_y + g_height + 1 <= atlas_height,
                "Font atlas overflowed (size={atlas_size}): this is a bug in calc_atlas_size"
            );

            if g_width > 0 && g_height > 0 {
                for dy in 0..g_height {
                    for dx in 0..g_width {
                        let src_idx = dy * g_width + dx;
                        let dest_idx = (cur_y + dy) * atlas_width + (cur_x + dx);
                        atlas_data[dest_idx] = bitmap[src_idx];
                    }
                }
            }

            let uv_min = Vec2::new(
                cur_x as f32 / atlas_width as f32,
                cur_y as f32 / atlas_height as f32,
            );
            let uv_max = Vec2::new(
                (cur_x + g_width) as f32 / atlas_width as f32,
                (cur_y + g_height) as f32 / atlas_height as f32,
            );

            glyphs.insert(
                ch,
                GlyphInfo {
                    uv_min,
                    uv_max,
                    width: g_width as f32,
                    height: g_height as f32,
                    xmin: metrics.xmin as f32,
                    ymin: metrics.ymin as f32,
                    advance: metrics.advance_width,
                },
            );

            cur_x += g_width + 1;
            if g_height > row_height {
                row_height = g_height;
            }
        }

        let extent = wgpu::Extent3d {
            width: atlas_width as u32,
            height: atlas_height as u32,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Font Atlas Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_width as u32),
                rows_per_image: Some(atlas_height as u32),
            },
            extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font Atlas Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Ok(Self {
            texture,
            view,
            bind_group,
            glyphs,
            native_height: config.native_height,
            ascent,
        })
    }

    pub fn glyph(&self, c: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&c)
    }

    pub fn measure_text(&self, text: &Text) -> Vec2 {
        let mut max_w_accum = 0.0;
        let mut line_count = 0;

        for original_line in text.content.split('\n') {
            let wrap_width = text.max_width.and_then(|max_w| {
                if max_w <= 0.0 {
                    log::warn!("text.max_width in measure_text is <= 0.0 ({}). Skipping wrapping.", max_w);
                    None
                } else {
                    Some(max_w)
                }
            });

            if let Some(max_w) = wrap_width {
                // Wrapping enabled
                let mut current_sub_w = 0.0;
                let mut has_chars = false;
                for c in original_line.chars() {
                    let char_w = if let Some(glyph) = self.glyph(c) {
                        glyph.advance * text.scale
                    } else {
                        self.native_height * text.scale
                    };
                    if current_sub_w + char_w > max_w && has_chars {
                        if current_sub_w > max_w_accum {
                            max_w_accum = current_sub_w;
                        }
                        line_count += 1;
                        current_sub_w = 0.0;
                    }
                    current_sub_w += char_w;
                    has_chars = true;
                }
                if current_sub_w > max_w_accum {
                    max_w_accum = current_sub_w;
                }
                line_count += 1;
            } else {
                // Wrapping disabled
                let mut line_w = 0.0;
                for c in original_line.chars() {
                    let char_w = if let Some(glyph) = self.glyph(c) {
                        glyph.advance * text.scale
                    } else {
                        self.native_height * text.scale
                    };
                    line_w += char_w;
                }
                if line_w > max_w_accum {
                    max_w_accum = line_w;
                }
                line_count += 1;
            }
        }

        let char_h = self.native_height * text.scale;
        let total_h = line_count as f32 * char_h;

        glam::vec2(max_w_accum, total_h)
    }
}
