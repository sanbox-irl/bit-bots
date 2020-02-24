use arrayvec::ArrayVec;
use anyhow::Error;
use gfx_hal::{
    command::{ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents},
    device::Device,
    pso::PipelineStage,
    queue::{CommandQueue, Submission},
    window::{Suboptimal, Swapchain},
};

use super::{draw_game_world, draw_imgui, utilities, DrawCommand, DrawingError, Window, RC};

pub fn render<'a>(
    renderer: &mut RC,
    window: &Window,
    draw_commands: &mut DrawCommand<'a>,
) -> Result<(), Error> {
    let result = draw(renderer, draw_commands);

    match result {
        Ok(sub_optimal) => {
            if let Some(_) = sub_optimal {
                utilities::recreate_swapchain(renderer, window)
            } else {
                Ok(())
            }
        }

        Err(e) => match e {
            DrawingError::AcquireAnImageFromSwapchain(_) | DrawingError::PresentIntoSwapchain(_) => {
                utilities::recreate_swapchain(renderer, window)
            }

            _ => Err(e)?,
        },
    }
}

fn draw<'a>(
    renderer: &mut RC,
    draw_commands: &mut DrawCommand<'a>,
) -> Result<Option<Suboptimal>, DrawingError> {
    // SETUP FOR THIS FRAME
    let image_available = &renderer.image_available_semaphores[renderer.current_frame];
    let render_finished = &renderer.render_finished_semaphores[renderer.current_frame];
    // Advance the frame *before* we start using the `?` operator
    renderer.current_frame = (renderer.current_frame + 1) % renderer.frames_in_flight;

    let (i_u32, i_usize) = unsafe {
        let image_index = renderer
            .swapchain
            .acquire_image(core::u64::MAX, Some(image_available), None)
            .map_err(|e| DrawingError::AcquireAnImageFromSwapchain(e))?;

        (image_index.0, image_index.0 as usize)
    };

    // Get the fence, and wait for the fence
    let flight_fence = &renderer.in_flight_fences[i_usize];
    unsafe {
        renderer
            .device
            .wait_for_fence(flight_fence, core::u64::MAX)
            .map_err(|e| DrawingError::WaitOnFence(e))?;
        renderer
            .device
            .reset_fence(flight_fence)
            .map_err(|e| DrawingError::ResetFence(e))?;
    }

    // RECORD COMMANDS
    unsafe {
        let cmd_buffer = &mut renderer.command_buffers[i_usize];
        cmd_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
        {
            cmd_buffer.begin_render_pass(
                &renderer.render_pass,
                &renderer.framebuffers[i_usize],
                renderer.viewport,
                &[ClearValue {
                    color: ClearColor {
                        float32: renderer.clear_color.into(),
                    },
                }],
                SubpassContents::Inline,
            );

            // DRAW THE GAME
            if let Some(game_world_commands) = draw_commands.game_world.take() {
                draw_game_world::draw_game_world(
                    cmd_buffer,
                    game_world_commands,
                    &mut renderer.textures,
                    &mut renderer.pipeline_bundles[RC::STANDARD_PIPELINE],
                    &renderer.iconic_quad_vert_index_buffer_bundle,
                )?;
            }

            // DRAW THE IMGUI
            if let Some(imgui_data) = draw_commands.imgui.take() {
                draw_imgui::draw_imgui(
                    cmd_buffer,
                    imgui_data,
                    &renderer.pipeline_bundles[RC::IMGUI_PIPELINE],
                    &mut renderer.imgui_vertex_index_buffer_bundles[i_usize],
                    &renderer.device,
                    &renderer.adapter,
                    &renderer.textures,
                )?;
            }
            cmd_buffer.end_render_pass();
        }
        cmd_buffer.finish();
    }

    // SUBMISSION AND PRESENT
    let command_buffers = &renderer.command_buffers[i_usize..=i_usize];
    let wait_semaphores: ArrayVec<[_; 1]> =
        [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
    let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    // yes, you have to write it twice like this. yes, it's silly.
    let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let submission = Submission {
        command_buffers,
        wait_semaphores,
        signal_semaphores,
    };
    let the_command_queue = &mut renderer.queue_group.queues[0];
    unsafe {
        the_command_queue.submit(submission, Some(flight_fence));
        Ok(renderer
            .swapchain
            .present(the_command_queue, i_u32, present_wait_semaphores)
            .map_err(|e| DrawingError::PresentIntoSwapchain(e))?)
    }
}
