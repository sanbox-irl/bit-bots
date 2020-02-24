use anyhow::Error;
use gfx_hal::{
    adapter::Adapter,
    buffer,
    command::*,
    pso::{Rect, ShaderStageFlags, Viewport},
    Backend, IndexType,
};
use std::ops::Deref;

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use super::{
    BufferBundle, DrawingError, ImGui, ImGuiDrawCommands, ImguiPushConstants, LoadedImage, PipelineBundle,
    Vec2, VertexIndexPairBufferBundle, RC,
};

pub fn initialize_imgui(renderer: &mut RC, imgui: &mut ImGui) -> Result<(), Error> {
    allocate_imgui_pipeline(renderer)?;
    allocate_imgui_textures(renderer, &mut imgui.imgui)?;
    allocate_imgui_buffers(renderer)?;

    Ok(())
}

fn allocate_imgui_pipeline(renderer: &mut RC) -> Result<(), Error> {
    renderer.pipeline_bundles.push(RC::create_imgui_pipeline(
        &renderer.device,
        &renderer.render_pass,
    )?);
    Ok(())
}

use imgui::Context as ImGuiContext;
fn allocate_imgui_textures(renderer: &mut RC, imgui: &mut ImGuiContext) -> Result<(), Error> {
    let mut fonts = imgui.fonts();
    let imgui::FontAtlasTexture {
        width: font_width,
        height: font_height,
        data: font_data,
    } = fonts.build_rgba32_texture();

    let imgui_image = LoadedImage::allocate_and_create(
        &renderer.adapter,
        &renderer.device,
        &mut renderer.command_pool,
        &mut renderer.queue_group.queues[0],
        &mut renderer.pipeline_bundles[RC::IMGUI_PIPELINE],
        font_data,
        font_width as usize,
        font_height as usize,
        gfx_hal::image::Filter::Linear,
    )?;

    let ret = renderer.textures.len();
    renderer.textures.push(imgui_image);

    use imgui::TextureId;
    fonts.tex_id = TextureId::from(ret);

    Ok(())
}

use imgui::{DrawIdx, DrawVert};
use std::mem;

fn allocate_imgui_buffers(renderer: &mut RC) -> Result<(), Error> {
    const VERT_SIZE: usize = 2000;
    const IDX_SIZE: usize = 5000;

    for _ in 0..renderer.frames_in_flight {
        let vertex_buffer = BufferBundle::new(
            &renderer.adapter,
            &renderer.device,
            (VERT_SIZE * mem::size_of::<DrawVert>()) as u64,
            buffer::Usage::VERTEX,
            true,
        )?;

        let index_buffer = BufferBundle::new(
            &renderer.adapter,
            &renderer.device,
            (IDX_SIZE * mem::size_of::<DrawIdx>()) as u64,
            buffer::Usage::INDEX,
            true,
        )?;

        renderer
            .imgui_vertex_index_buffer_bundles
            .push(VertexIndexPairBufferBundle {
                vertex_buffer,
                index_buffer,
                num_vert: VERT_SIZE,
                num_idx: IDX_SIZE,
            });
    }

    Ok(())
}

#[allow(dead_code)]
pub(super) unsafe fn draw_imgui<'a>(
    encoder: &mut <back::Backend as Backend>::CommandBuffer,
    imgui_data: ImGuiDrawCommands<'_>,
    imgui_pipeline: &'a PipelineBundle<back::Backend>,
    imgui_buffer_bundle: &'a mut VertexIndexPairBufferBundle<back::Backend>,
    device: &<back::Backend as Backend>::Device,
    adapter: &Adapter<back::Backend>,
    textures: &Vec<LoadedImage<back::Backend>>,
) -> Result<(), DrawingError> {
    imgui_buffer_bundle
        .update_size(
            mem::size_of::<DrawVert>(),
            mem::size_of::<DrawIdx>(),
            imgui_data.draw_data.total_vtx_count as usize,
            imgui_data.draw_data.total_idx_count as usize,
            &device,
            &adapter,
        )
        .map_err(|_| DrawingError::BufferCreationError)?;

    // Check our Buffers
    let VertexIndexPairBufferBundle {
        vertex_buffer: imgui_vertex_buffer,
        index_buffer: imgui_index_buffer,
        ..
    } = imgui_buffer_bundle;

    // Bind pipeline
    encoder.bind_graphics_pipeline(&imgui_pipeline.graphics_pipeline);

    // Bind vertex and index buffers
    encoder.bind_vertex_buffers(0, Some((imgui_vertex_buffer.buffer.deref(), 0)));
    encoder.bind_index_buffer(buffer::IndexBufferView {
        buffer: &imgui_index_buffer.buffer,
        offset: 0,
        index_type: IndexType::U16,
    });

    // Set push constants
    let push_constants = ImguiPushConstants {
        scale: Vec2::new(
            2.0 / imgui_data.imgui_dimensions.x,
            2.0 / imgui_data.imgui_dimensions.y,
        ),
        translate: Vec2::new(-1.0, -1.0),
    };

    let viewport = Viewport {
        rect: Rect {
            x: 0,
            y: 0,
            w: imgui_data.imgui_dimensions.x as i16,
            h: imgui_data.imgui_dimensions.y as i16,
        },
        depth: 0.0..1.0,
    };

    encoder.set_viewports(0, &[viewport]);
    encoder.push_graphics_constants(
        &imgui_pipeline.pipeline_layout,
        ShaderStageFlags::VERTEX,
        0,
        &push_constants.to_bits(),
    );

    let mut vertex_offset = 0;
    let mut index_offset = 0;
    let mut current_texture_page = 0;

    // Iterate over drawlists
    for list in imgui_data.draw_data.draw_lists() {
        // Update vertex and index buffers
        imgui_vertex_buffer.update_buffer(list.vtx_buffer(), vertex_offset);
        imgui_index_buffer.update_buffer(list.idx_buffer(), index_offset);

        for cmd in list.commands() {
            if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                // Calculate the scissor
                let scissor = Rect {
                    x: cmd_params.clip_rect[0] as i16,
                    y: cmd_params.clip_rect[1] as i16,
                    w: (cmd_params.clip_rect[2] - cmd_params.clip_rect[0]) as i16,
                    h: (cmd_params.clip_rect[3] - cmd_params.clip_rect[1]) as i16,
                };
                encoder.set_scissors(0, &[scissor]);

                // Check textures
                let texture_id = cmd_params.texture_id.id();
                if current_texture_page != texture_id {
                    current_texture_page = texture_id;

                    encoder.bind_graphics_descriptor_sets(
                        &imgui_pipeline.pipeline_layout,
                        0,
                        Some(textures[texture_id].descriptor_set.deref()),
                        &[],
                    );
                }

                // Actually draw things
                encoder.draw_indexed(
                    index_offset as u32..(index_offset + count) as u32,
                    vertex_offset as i32,
                    0..1,
                );

                index_offset += count as usize;
            }
        }

        // Increment offsets
        vertex_offset += list.vtx_buffer().len();
    }

    // flush em down the toilet
    imgui_vertex_buffer
        .flush(device)
        .map_err(|_| DrawingError::BufferError)?;
    imgui_index_buffer
        .flush(device)
        .map_err(|_| DrawingError::BufferError)?;

    Ok(())
}
