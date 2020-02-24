use gfx_hal::{
    buffer::*,
    command::*,
    pso::{Rect, ShaderStageFlags, Viewport},
    Backend, IndexType,
};

use super::{
    sprite_system, BasicTextures, DrawingError, GameWorldDrawCommands, LoadedImage, PipelineBundle,
    RenderingUtility, StandardPushConstants, StandardQuadFactory, StandardTexture, TextureDescription,
    VertexIndexPairBufferBundle,
};

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use std::ops::Deref;
pub(super) unsafe fn draw_game_world<'a>(
    encoder: &mut <back::Backend as Backend>::CommandBuffer,
    gameworld_draw_commands: GameWorldDrawCommands<'_>,
    textures: &mut Vec<LoadedImage<back::Backend>>,
    quad_pipeline: &mut PipelineBundle<back::Backend>,
    standard_render_bundle: &VertexIndexPairBufferBundle<back::Backend>,
) -> Result<(), DrawingError> {
    // Deconstruct our DrawCommands
    let GameWorldDrawCommands {
        text_sources,
        sprites,
        rects,
        transforms,
        // tilemaps,
        camera,
        camera_entity,
        rendering_utility,
        resources,
    } = gameworld_draw_commands;

    // Bind pipeline and Verts
    encoder.bind_graphics_pipeline(&quad_pipeline.graphics_pipeline);
    encoder.bind_vertex_buffers(0, Some((standard_render_bundle.vertex_buffer.buffer.deref(), 0)));
    encoder.bind_index_buffer(IndexBufferView {
        buffer: &standard_render_bundle.index_buffer.buffer,
        offset: 0,
        index_type: IndexType::U16,
    });

    let display_size = camera.display_size();
    let viewport = Viewport {
        rect: Rect {
            x: (display_size.x - 480.0) as i16 / 2,
            y: (display_size.y - 840.0) as i16 / 2,
            w: 480,
            h: 840,
        },
        depth: 0.0..1.0,
    };

    encoder.set_viewports(0, &[viewport]);

    // Deconstruct for ease of use...
    let RenderingUtility {
        quad_buffer,
        basic_textures,
    } = rendering_utility;

    quad_buffer.clear();
    for this_sprite in sprites.iter() {
        if let Some(transform) = transforms.get(&this_sprite.entity_id()) {
            quad_buffer.push(sprite_system::to_standard_quad(
                this_sprite.inner(),
                transform.inner().world_position(),
                resources,
            ));
        }
    }

    for text_source in text_sources.iter() {
        if let Some(transform) = transforms.get(&text_source.entity_id()) {
            for quad in &text_source.inner().cached_quads {
                quad_buffer.push(
                    text_source
                        .inner()
                        .prepare_standard_quad(transform.inner().world_position(), quad),
                );
            }
        }
    }

    for this_draw_rect in rects.iter() {
        if let Some(transform) = transforms.get(&this_draw_rect.entity_id()) {
            quad_buffer.push(
                this_draw_rect
                    .inner()
                    .to_standard_quad(transform.inner().world_position()),
            );
        }
    }
    // for this_tilemap in tilemaps.iter() {
    //     if let Some(transform) = transforms.get(&this_tilemap.entity_id()) {
    //         this_tilemap
    //             .inner()
    //             .create_tile_quads(transform.inner().world_position(), quad_buffer);
    //     }
    // }

    quad_buffer.sort();

    let mut current_texture_page = 0;
    encoder.bind_graphics_descriptor_sets(
        &quad_pipeline.pipeline_layout,
        0,
        Some(textures[current_texture_page].descriptor_set.deref()),
        &[],
    );

    let camera_position = camera_entity
        .map(|camera_entity| {
            transforms
                .get(camera_entity)
                .map(|tc| tc.inner().world_position())
                .unwrap_or_else(|| camera.default_position)
        })
        .unwrap_or_else(|| camera.default_position);

    for quad in quad_buffer {
        let mut push_constants = StandardPushConstants::with_camera_data(camera_position, camera);
        let texture_info: &StandardTexture = match &quad.texture_info {
            TextureDescription::Standard(s) => s,
            TextureDescription::White => &basic_textures[BasicTextures::White as usize],
        };

        push_constants.update(quad, texture_info);

        if current_texture_page != texture_info.texture_page {
            current_texture_page = texture_info.texture_page;

            encoder.bind_graphics_descriptor_sets(
                &quad_pipeline.pipeline_layout,
                0,
                Some(textures[current_texture_page].descriptor_set.deref()),
                &[],
            );
        }

        encoder.push_graphics_constants(
            &quad_pipeline.pipeline_layout,
            ShaderStageFlags::VERTEX,
            0,
            &push_constants.to_bits(),
        );

        encoder.draw_indexed(0..6, 0, 0..1);
    }

    Ok(())
}
