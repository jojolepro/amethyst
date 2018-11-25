//! Rasterizes UiText to TextureHandle when UiText changes.

use amethyst_core::{
    specs::{System, ReadStorage, WriteStorage, ReadExpect, Join, BitSet, Read, Entities, ReaderId, Resources, SystemData, storage::ComponentEvent},
};
use amethyst_assets::{
    Loader, Handle, AssetStorage
};
use amethyst_renderer::{
    Texture, ScreenDimensions, TextureData, TextureMetadata,ImageData,
};

use rusttype::Scale;
use unicode_segmentation::UnicodeSegmentation;
use glyph_brush_layout::*;
use image::{DynamicImage, Rgba};

use crate::{UiText, LineMode, FontAsset, TextEditing, UiTransform};

/// System in charge of rasterizing the UiText component into a TextureHandle.
/// Said TextureHandle is then attached to the entity.
#[derive(Default, new)]
pub struct UiTextRasterizerSystem {
    #[new(default)]
    text_reader: Option<ReaderId<ComponentEvent>>,
    #[new(default)]
    inserted: BitSet,
    #[new(default)]
    modified: BitSet,
}

impl<'a> System<'a> for UiTextRasterizerSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, UiTransform>,
        ReadStorage<'a, UiText>,
        ReadStorage<'a, TextEditing>,
        WriteStorage<'a, Handle<Texture>>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, AssetStorage<FontAsset>>,
        ReadExpect<'a, ScreenDimensions>,
        ReadExpect<'a, Loader>,
    );

    fn run(&mut self, (entities, transforms, texts, editings, mut textures, texture_assets, font_assets, screen_dimensions, loader): Self::SystemData) {
        self.inserted.clear();
        self.modified.clear();

        let events = texts.channel().read(&mut self.text_reader.as_mut().unwrap());
        for event in events {
            info!("reee");
            match event {
                ComponentEvent::Modified(id) => { self.modified.add(*id); },
                ComponentEvent::Inserted(id) => { self.inserted.add(*id); },
                ComponentEvent::Removed(id) => {
                    let entity = entities.entity(*id);
                    // TODO: is this true?
                    textures.remove(entity).expect("unreachable: entity valid");
                },
            }
        }

        for (entity, ui_transform, ui_text, _) in (&*entities, &transforms, &texts, &self.inserted | &self.modified).join() {
            let font = match font_assets.get(&ui_text.font) {
                Some(font) => font,
                None => continue,
            };

            // Build text sections.
            let editing = editings.get(entity);

            let password_string = if ui_text.password {
                // Build a string composed of black dot characters.
                let mut ret = String::with_capacity(ui_text.text.len());
                for _grapheme in ui_text.text.graphemes(true) {
                    ret.push('\u{2022}');
                }
                Some(ret)
            } else {
                None
            };
            let rendered_string = password_string.as_ref().unwrap_or(&ui_text.text);

            let hidpi = screen_dimensions.hidpi_factor() as f32;
            let size = ui_text.font_size * hidpi;
            let scale = Scale::uniform(size);

            let sections = editings.get(entity).and_then(|editing| {
                if editing.highlight_vector == 0 {
                    return None;
                }
                let start = editing
                    .cursor_position
                    .min(editing.cursor_position + editing.highlight_vector)
                    as usize;
                let end = editing
                    .cursor_position
                    .max(editing.cursor_position + editing.highlight_vector)
                    as usize;
                let start_byte = rendered_string
                    .grapheme_indices(true)
                    .nth(start)
                    .map(|i| i.0);
                let end_byte = rendered_string
                    .grapheme_indices(true)
                    .nth(end)
                    .map(|i| i.0)
                    .unwrap_or_else(|| rendered_string.len());
                start_byte.map(|start_byte| (editing, (start_byte, end_byte)))
            }).map(|(editing, (start_byte, end_byte))| {
                vec![
                    SectionText {
                        text: &((rendered_string)[0..start_byte]),
                        scale: scale,
                        color: ui_text.color,
                        font_id: FontId(0),
                    },
                    SectionText {
                        text: &((rendered_string)[start_byte..end_byte]),
                        scale: scale,
                        color: editing.selected_text_color,
                        font_id: FontId(0),
                    },
                    SectionText {
                        text: &((rendered_string)[end_byte..]),
                        scale: scale,
                        color: ui_text.color,
                        font_id: FontId(0),
                    },
                ]
            }).unwrap_or_else(|| {
                vec![SectionText {
                    text: rendered_string,
                    scale: scale,
                    color: ui_text.color,
                    font_id: FontId(0),
                }]
            });

            let layout = match ui_text.line_mode {
                LineMode::Single => Layout::SingleLine {
                    line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                    h_align: ui_text.align.horizontal_align(),
                    v_align: ui_text.align.vertical_align(),
                },
                LineMode::Wrap => Layout::Wrap {
                    line_breaker: BuiltInLineBreaker::UnicodeLineBreaker,
                    h_align: ui_text.align.horizontal_align(),
                    v_align: ui_text.align.vertical_align(),
                },
            };

            // Needs a recenter because we are using [-0.5,0.5] for the mesh
            // instead of the expected [0,1]
            let screen_position = (
                (ui_transform.pixel_x
                    + ui_transform.pixel_width * ui_text.align.norm_offset().0)
                    * hidpi,
                // invert y because gfx-glyph inverts it back
                (screen_dimensions.height()
                    - ui_transform.pixel_y
                    - ui_transform.pixel_height * ui_text.align.norm_offset().1)
                    * hidpi,
            );
            let bounds = (
                ui_transform.pixel_width * hidpi,
                ui_transform.pixel_height * hidpi,
            );


            let glyphs = layout.calculate_glyphs(
                &vec![font.0.clone()], // TODO: Remove that clone
                &SectionGeometry {
                    screen_position: (0.0, 0.0),
                    bounds,
                },
                &sections,
            );

            let size = layout.bounds_rect(
                &SectionGeometry {
                    screen_position: (0.0, 0.0),
                    bounds,
                },
            );

            let bounds = (size.width() as u32, size.height() as u32);

            if bounds != (0, 0) {
                let mut image = DynamicImage::new_rgba8(bounds.0, bounds.1).to_rgba();
                for (glyph, color, _font) in glyphs {
                    if let Some(bounding_box) = glyph.pixel_bounding_box() {
                        // Draw the glyph into the image per-pixel by using the draw closure
                        glyph.draw(|x, y, v| {
                            info!("x {} y {} bounding_box.min.x {} bounding_box.min.y {}", x, y, bounding_box.min.x, bounding_box.min.y);
                            //if x_with_offset < bounds.0 as u32 && y_with_offset < bounds.1 as u32 && x_with_offset >= 0 && y_with_offset >= 0 {
                            //if bounding_box.min.x >= 0 && bounding_box.min.y >= 0 {
                                let x_with_offset = (x as i32 + bounding_box.min.x + bounds.0 as i32/2) as u32;
                                let y_with_offset = (y as i32 + bounding_box.min.y + bounds.1 as i32/2) as u32;
                            if x_with_offset <= bounds.0 as u32 && y_with_offset <= bounds.1 as u32 {
                            image.put_pixel(
                                // Offset the position by the glyph bounding box
                                x_with_offset,
                                y_with_offset,
                                // Turn the coverage into an alpha value
                                Rgba {
                                    data: [(color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8, (v * 255.0) as u8],
                                },
                            )
                            }
                        });
                    }
                }
                let handle = loader.load_from_data(TextureData::Image(ImageData{rgba: image}, TextureMetadata::srgb().with_size(bounds.0 as u16, bounds.1 as u16)), (), &texture_assets);
                textures.insert(entity, handle).expect("unreachable: entity is valid");
            } else {
                textures.remove(entity);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let mut text_storage: WriteStorage<UiText> = SystemData::fetch(&res);
        self.text_reader = Some(text_storage.register_reader());
    }
}
