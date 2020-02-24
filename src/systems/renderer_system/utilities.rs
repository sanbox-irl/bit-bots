use super::{LoadedImage, RendererCreationError, Window, RC};
use core::mem::ManuallyDrop;
use anyhow::Error;
use gfx_hal::{
    command,
    device::Device,
    format::{Aspects, Swizzle},
    image::{Extent, SubresourceRange, ViewKind},
    pool::{CommandPool, CommandPoolCreateFlags},
    window::{Extent2D, Surface},
};
use image::RgbaImage;

pub fn register_texture(renderer: &mut RC, image: &RgbaImage) -> Result<usize, Error> {
    let texture = {
        let mut pipeline_bundle = &mut renderer.pipeline_bundles[RC::STANDARD_PIPELINE];

        LoadedImage::allocate_and_create(
            &renderer.adapter,
            &renderer.device,
            &mut renderer.command_pool,
            &mut renderer.queue_group.queues[0],
            &mut pipeline_bundle,
            &*image,
            image.width() as usize,
            image.height() as usize,
            gfx_hal::image::Filter::Nearest,
        )?
    };

    let ret = renderer.textures.len();
    renderer.textures.push(texture);

    Ok(ret)
}

pub(super) fn recreate_swapchain(renderer: &mut RC, window: &Window) -> Result<(), Error> {
    let caps = renderer.surface.capabilities(&renderer.adapter.physical_device);
    let formats = renderer
        .surface
        .supported_formats(&renderer.adapter.physical_device);

    assert!(formats.iter().any(|fs| fs.contains(&renderer.format)));

    let extent = {
        let window_client_area = window.window.inner_size();

        Extent2D {
            width: caps.extents.end().width.min(window_client_area.width),
            height: caps.extents.end().height.min(window_client_area.height),
        }
    };

    renderer.viewport = extent.to_extent().rect();

    let swapchain_config = gfx_hal::window::SwapchainConfig::from_caps(&caps, renderer.format, extent);

    unsafe {
        drop_swapchain(renderer);
        let (swapchain, backbuffer) = renderer
            .device
            .create_swapchain(&mut renderer.surface, swapchain_config, None)
            .map_err(|e| RendererCreationError::Swapchain(e))?;

        let image_views = {
            backbuffer
                .into_iter()
                .map(|image| {
                    renderer
                        .device
                        .create_image_view(
                            &image,
                            ViewKind::D2,
                            renderer.format,
                            Swizzle::NO,
                            SubresourceRange {
                                aspects: Aspects::COLOR,
                                levels: 0..1,
                                layers: 0..1,
                            },
                        )
                        .map_err(|e| RendererCreationError::ImageViews(e))
                })
                .collect::<Result<Vec<_>, RendererCreationError>>()?
        };

        let framebuffers = {
            image_views
                .iter()
                .map(|image_view| {
                    renderer
                        .device
                        .create_framebuffer(
                            &renderer.render_pass,
                            vec![image_view],
                            Extent {
                                width: extent.width as u32,
                                height: extent.height as u32,
                                depth: 1,
                            },
                        )
                        .map_err(|e| RendererCreationError::FrameBuffers(e))
                })
                .collect::<Result<Vec<_>, RendererCreationError>>()?
        };

        let mut command_pool = renderer
            .device
            .create_command_pool(
                renderer.queue_group.family,
                CommandPoolCreateFlags::RESET_INDIVIDUAL,
            )
            .map_err(|e| RendererCreationError::CommandPool(e))?;

        let command_buffers = framebuffers
            .iter()
            .map(|_| command_pool.allocate_one(command::Level::Primary))
            .collect();

        // Recreate the pipelines...
        assert_eq!(renderer.pipeline_bundles.len(), RC::STANDARD_PIPELINE);
        renderer.pipeline_bundles.push(RC::create_quad_pipeline(
            &mut renderer.device,
            &extent,
            &renderer.render_pass,
        )?);

        assert_eq!(renderer.pipeline_bundles.len(), RC::IMGUI_PIPELINE);
        renderer.pipeline_bundles.push(RC::create_imgui_pipeline(
            &mut renderer.device,
            &renderer.render_pass,
        )?);

        // Finally, we got ourselves a nice and shiny new swapchain!
        renderer.swapchain = ManuallyDrop::new(swapchain);

        renderer.framebuffers = framebuffers;
        renderer.command_buffers = command_buffers;
        renderer.command_pool = ManuallyDrop::new(command_pool);
    }
    Ok(())
}

macro_rules! manual_drop {
    ($this_val:expr) => {
        ManuallyDrop::into_inner(read(&$this_val))
    };
}

pub(super) fn drop_swapchain(renderer: &mut RC) {
    renderer.device.wait_idle().unwrap();

    use core::ptr::read;
    unsafe {
        for framebuffer in renderer.framebuffers.drain(..) {
            renderer.device.destroy_framebuffer(framebuffer);
        }
        renderer
            .device
            .destroy_command_pool(manual_drop!(renderer.command_pool));

        for pipeline_bundle in renderer.pipeline_bundles.drain(..) {
            pipeline_bundle.manually_drop(&renderer.device);
        }

        renderer
            .device
            .destroy_swapchain(manual_drop!(renderer.swapchain));
    }
}
