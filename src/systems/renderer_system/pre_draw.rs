use super::{
    ComponentDatabase, ComponentList, DrawingError, LoadedImage, PipelineBundle, RendererComponent,
    ResourcesDatabase, StandardQuad, StandardTexture, TextSource, TextureDescription, Vec2, Vec2Int,
};
use anyhow::Error;
use glyph_brush::{rusttype::Scale, BrushAction, BrushError, Layout, Section};

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use gfx_hal::{adapter::Adapter, Backend};

pub fn pre_draw(
    component_database: &mut ComponentDatabase,
    resource_database: &mut ResourcesDatabase,
    renderer_component: &mut RendererComponent,
) -> Result<(), Error> {
    build_text(
        &mut component_database.text_sources,
        resource_database,
        &mut renderer_component.textures,
        &mut renderer_component.pipeline_bundles[RendererComponent::STANDARD_PIPELINE],
        &renderer_component.adapter,
        &renderer_component.device,
        &mut renderer_component.command_pool,
        &mut renderer_component.queue_group.queues[0],
    )
}

#[allow(dead_code)]
fn build_text(
    text_sources: &mut ComponentList<TextSource>,
    resource_database: &mut ResourcesDatabase,
    textures: &mut Vec<LoadedImage<back::Backend>>,
    quad_pipeline: &mut PipelineBundle<back::Backend>,
    adapter: &Adapter<back::Backend>,
    device: &<back::Backend as Backend>::Device,
    command_pool: &mut <back::Backend as Backend>::CommandPool,
    command_queue: &mut <back::Backend as Backend>::CommandQueue,
) -> Result<(), Error> {
    for text_source_component in text_sources.iter_mut() {
        if let Some(font_data) = resource_database
            .fonts
            .get_mut(&text_source_component.inner().font)
        {
            let text_source: &mut TextSource = text_source_component.inner_mut();

            // Queue it...
            font_data.glyph.queue(Section {
                text: &text_source.text,
                scale: Scale {
                    x: text_source.scale.x,
                    y: text_source.scale.y,
                },
                layout: Layout::default()
                    .v_align(text_source.vertical_align.into())
                    .h_align(text_source.horizontal_align.into()),
                z: text_source.draw_order.to_f32(),
                color: text_source.color.into(),
                ..Section::default()
            });

            // And immediately render it out!
            if let None = font_data.texture_page {
                let tex_dims: (usize, usize) = (
                    font_data.glyph.texture_dimensions().0 as usize,
                    font_data.glyph.texture_dimensions().1 as usize,
                );
                font_data.texture_page = Some(textures.len());

                let our_vec = vec![0; tex_dims.0 * tex_dims.1 * std::mem::size_of::<u32>()];
                let image = LoadedImage::allocate_and_create(
                    adapter,
                    device,
                    command_pool,
                    command_queue,
                    quad_pipeline,
                    &our_vec,
                    tex_dims.0,
                    tex_dims.1,
                    gfx_hal::image::Filter::Nearest,
                )
                .map_err(|e| DrawingError::DynamicTextureCreation(e))?;

                textures.push(image);
            }

            // This is borrow checker silliness
            let texture_page: usize = font_data.texture_page.clone().unwrap();

            // We're figuring this out!
            loop {
                let result = font_data.glyph.process_queued(
                    |rect, tex_data| {
                        let mut pass_through_vec =
                            Vec::with_capacity(tex_data.iter().count() * std::mem::size_of::<u32>());

                        for opacity_val in tex_data {
                            for _ in 0..4 {
                                pass_through_vec.push(*opacity_val);
                            }
                        }

                        if let Err(e) = textures[texture_page].edit_image(
                            rect.width(),
                            rect.height(),
                            Vec2Int::new(rect.min.x as i32, rect.min.y as i32),
                            &pass_through_vec,
                            adapter,
                            device,
                            command_pool,
                            command_queue,
                        ) {
                            error!("Error on editing font data {}", e);
                        }
                    },
                    |glyph_vert| {
                        let standard_tex = StandardTexture {
                            norm_image_coordinate: Vec2::new(
                                glyph_vert.tex_coords.min.x,
                                glyph_vert.tex_coords.min.y,
                            ),
                            norm_image_size: Vec2::new(
                                glyph_vert.tex_coords.width(),
                                glyph_vert.tex_coords.height(),
                            ),
                            texture_page,
                        };

                        let pos = Vec2::new(
                            glyph_vert.pixel_coords.min.x as f32,
                            -glyph_vert.pixel_coords.max.y as f32,
                        );

                        let image_size = Vec2::new(
                            glyph_vert.pixel_coords.width() as f32,
                            glyph_vert.pixel_coords.height() as f32,
                        );

                        StandardQuad {
                            pos,
                            image_size,
                            texture_info: TextureDescription::Standard(standard_tex),

                            color: super::Color::default(),
                            draw_order: super::DrawOrder::default(),
                        }
                    },
                );

                match result {
                    Ok(br) => {
                        match br {
                            BrushAction::Draw(vert_data) => text_source.cached_quads = vert_data,
                            BrushAction::ReDraw => {}
                        };
                        break;
                    }

                    Err(e) => {
                        error!("Expanding texture page..");
                        match e {
                            BrushError::TextureTooSmall { suggested } => {
                                let old_page = {
                                    let width = suggested.0 as usize;
                                    let height = suggested.1 as usize;

                                    error!("New size is ({},{})", width, height);

                                    // @techdebt this is silly to allocate here, but it's what we do!
                                    let our_vec = vec![0; width * height * std::mem::size_of::<u32>()];
                                    let image = LoadedImage::allocate_and_create(
                                        adapter,
                                        device,
                                        command_pool,
                                        command_queue,
                                        quad_pipeline,
                                        &our_vec,
                                        width,
                                        height,
                                        gfx_hal::image::Filter::Nearest,
                                    )
                                    .map_err(|e| DrawingError::DynamicTextureCreation(e))?;

                                    font_data.glyph.resize_texture(width as u32, height as u32);
                                    // @techdebt this will fail if we go over the max size of a sheet,
                                    // ie -- we can't really go over about 1000 font size.

                                    std::mem::replace(&mut textures[texture_page], image)
                                };
                                unsafe {
                                    old_page.manually_drop(device);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
