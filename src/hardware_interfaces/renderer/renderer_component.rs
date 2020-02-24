use anyhow::Error;
use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use gfx_hal::{
    adapter::{Adapter, Gpu, PhysicalDevice},
    buffer, command,
    device::Device,
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{
        AttributeDesc, BakedStates, BasePipeline, BlendDesc, BlendState, ColorBlendDesc, ColorMask,
        DepthStencilDesc, DescriptorRangeDesc, DescriptorSetLayoutBinding, DescriptorType, ElemStride,
        Element, EntryPoint, Face, FrontFace, GraphicsPipelineDesc, GraphicsShaderSet, InputAssemblerDesc,
        LogicOp, PipelineCreationFlags, PolygonMode, Primitive, Rasterizer, Rect, ShaderStageFlags,
        Specialization, VertexBufferDesc, VertexInputRate, Viewport,
    },
    queue::family::{QueueFamily, QueueGroup},
    window::{Extent2D, PresentMode, Surface, SwapchainConfig},
    Backend, Features, Instance,
};
use imgui::DrawVert;
use memoffset::offset_of;
use std::mem;
use winit::window::Window as WinitWindow;

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use super::{
    BufferBundle, Color, ImguiPushConstants, LoadedImage, MemoryWritingError, PipelineBundle,
    PipelineCreationError, RendererCreationError, StandardPushConstants, Vertex, VertexIndexPairBufferBundle,
    QUAD_INDICES, QUAD_VERTICES,
};

pub struct Renderer<B: Backend> {
    // Top
    pub instance: ManuallyDrop<B::Instance>,
    pub surface: B::Surface,
    pub adapter: Adapter<B>,
    pub queue_group: ManuallyDrop<QueueGroup<B>>,
    pub device: ManuallyDrop<B::Device>,
    pub format: Format,

    // Pipeline nonsense
    pub pipeline_bundles: ArrayVec<[PipelineBundle<B>; RendererComponent::PIPELINE_SIZE]>,
    pub iconic_quad_vert_index_buffer_bundle: VertexIndexPairBufferBundle<B>,
    pub imgui_vertex_index_buffer_bundles: Vec<VertexIndexPairBufferBundle<B>>,
    pub textures: Vec<LoadedImage<B>>,

    // GPU Swapchain
    pub swapchain: ManuallyDrop<B::Swapchain>,
    pub viewport: Rect,
    pub in_flight_fences: Vec<B::Fence>,
    pub image_available_semaphores: Vec<B::Semaphore>,
    pub render_finished_semaphores: Vec<B::Semaphore>,

    // Render Pass
    pub render_pass: ManuallyDrop<B::RenderPass>,
    pub clear_color: Color,

    // Render Targets
    pub image_views: Vec<B::ImageView>,
    pub framebuffers: Vec<B::Framebuffer>,

    // Command Issues
    pub command_pool: ManuallyDrop<B::CommandPool>,
    pub command_buffers: Vec<B::CommandBuffer>,

    // Misc
    pub frames_in_flight: usize,
    pub current_frame: usize,
}

pub type RendererComponent = Renderer<back::Backend>;
impl<B: Backend> Renderer<B> {
    pub fn typed_new(window: &WinitWindow) -> Result<RendererComponent, Error> {
        // Create An Instance
        let instance = back::Instance::create("Clockwork Renderer", 1)
            .map_err(|e| RendererCreationError::InstanceCreation(e))?;

        // Create A Surface
        let surface = unsafe { instance.create_surface(window)? };

        // Create A Renderer
        RendererComponent::new(&window, instance, surface)
    }

    pub const STANDARD_PIPELINE: usize = 0;
    pub const IMGUI_PIPELINE: usize = 1;
    pub const PIPELINE_SIZE: usize = 2;

    fn new(window: &WinitWindow, instance: B::Instance, mut surface: B::Surface) -> Result<Self, Error> {
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families.iter().any(|queue_family| {
                    queue_family.queue_type().supports_graphics()
                        && surface.supports_queue_family(queue_family)
                })
            })
            .ok_or(RendererCreationError::GraphicalAdapter)?;

        // open it up!
        let (mut device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|queue_family| {
                    queue_family.queue_type().supports_graphics()
                        && surface.supports_queue_family(queue_family)
                })
                .ok_or(RendererCreationError::FindQueueFamily)?;

            let Gpu {
                device,
                mut queue_groups,
            } = unsafe {
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0; 1])], Features::empty())
                    .map_err(|e| RendererCreationError::OpenPhysicalAdapter(e))?
            };

            let queue_group = queue_groups
                .pop()
                .ok_or(RendererCreationError::OwnershipQueueGroup)?;

            if queue_group.queues.len() == 0 {
                return Err(RendererCreationError::FindCommandQueue.into());
            }
            (device, queue_group)
        };

        let (swapchain, extent, backbuffer, format, frames_in_flight) = {
            let caps = surface.capabilities(&adapter.physical_device);
            let formats = surface.supported_formats(&adapter.physical_device);

            let format = formats.map_or(Format::Rgba8Srgb, |formats| {
                formats
                    .iter()
                    .find(|format| format.base_format().1 == ChannelType::Srgb)
                    .map(|format| *format)
                    .unwrap_or(formats[0])
            });

            let extent = {
                let window_client_area = window.inner_size();

                Extent2D {
                    width: caps.extents.end().width.min(window_client_area.width),
                    height: caps.extents.end().height.min(window_client_area.height),
                }
            };
            let swap_config = SwapchainConfig::from_caps(&caps, format, extent);

            let image_count = if swap_config.present_mode.contains(PresentMode::MAILBOX) {
                (caps.image_count.end() - 1).min(*caps.image_count.start().max(&3))
            } else {
                (caps.image_count.end() - 1).min(*caps.image_count.start().max(&2))
            };

            // Final pop out. PHEW!
            let (swapchain, backbuffer) = unsafe {
                device
                    .create_swapchain(&mut surface, swap_config, None)
                    .map_err(|e| RendererCreationError::Swapchain(e))?
            };

            (swapchain, extent, backbuffer, format, image_count as usize)
        };

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
            let mut image_available_semaphores = vec![];
            let mut render_finished_semaphores = vec![];
            let mut in_flight_fences = vec![];
            for _ in 0..frames_in_flight {
                in_flight_fences.push(
                    device
                        .create_fence(true)
                        .map_err(|e| RendererCreationError::Fence(e))?,
                );
                image_available_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|e| RendererCreationError::ImageAvailableSemaphore(e))?,
                );
                render_finished_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|e| RendererCreationError::RenderFinishedSemaphore(e))?,
                );
            }
            (
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
            )
        };

        let render_pass = {
            let color_attachment = Attachment {
                format: Some(format),
                samples: 1,
                ops: AttachmentOps {
                    load: AttachmentLoadOp::Clear,
                    store: AttachmentStoreOp::Store,
                },
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };

            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            unsafe {
                device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .map_err(|e| RendererCreationError::RenderPassCreation(e))?
            }
        };

        let image_views = {
            backbuffer
                .into_iter()
                .map(|image| unsafe {
                    device
                        .create_image_view(
                            &image,
                            ViewKind::D2,
                            format,
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
                .map(|image_view| unsafe {
                    device
                        .create_framebuffer(
                            &render_pass,
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

        let mut command_pool = unsafe {
            device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::RESET_INDIVIDUAL)
                .map_err(|e| RendererCreationError::CommandPool(e))?
        };

        let command_buffers: Vec<_> = unsafe {
            framebuffers
                .iter()
                .map(|_| command_pool.allocate_one(command::Level::Primary))
                .collect()
        };

        // CREATE PIPELINES
        let mut pipeline_bundles = ArrayVec::new();
        assert_eq!(pipeline_bundles.len(), Self::STANDARD_PIPELINE);
        pipeline_bundles.push(Self::create_quad_pipeline(&mut device, &extent, &render_pass)?);

        // CREATE VERT-INDEX BUFFERS
        let mut vertex_buffer = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_VERTICES) as u64,
            buffer::Usage::VERTEX,
            false,
        )?;
        Self::bind_to_memory(&mut device, &mut vertex_buffer, &QUAD_VERTICES)?;

        let mut index_buffer = BufferBundle::new(
            &adapter,
            &device,
            mem::size_of_val(&QUAD_INDICES) as u64,
            buffer::Usage::INDEX,
            false,
        )?;
        Self::bind_to_memory(&mut device, &mut index_buffer, &QUAD_INDICES)?;

        let iconic_quad_vert_index_buffer_bundle = VertexIndexPairBufferBundle {
            vertex_buffer,
            index_buffer,
            num_vert: 4, // cause of quads
            num_idx: 6,  // cause of indexing quads
        };

        Ok(Self {
            instance: manual_new!(instance),
            surface,
            adapter,
            format,
            device: manual_new!(device),
            queue_group: manual_new!(queue_group),
            swapchain: manual_new!(swapchain),
            viewport: extent.to_extent().rect(),
            render_pass: manual_new!(render_pass),
            image_views,
            framebuffers,
            command_pool: manual_new!(command_pool),
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            frames_in_flight,
            current_frame: 0,
            iconic_quad_vert_index_buffer_bundle,
            imgui_vertex_index_buffer_bundles: Vec::new(),

            pipeline_bundles,
            textures: Vec::new(),
            clear_color: Color::with_u8(31, 29, 29, 255),
        })
    }

    pub fn create_quad_pipeline(
        device: &mut B::Device,
        extent: &Extent2D,
        render_pass: &B::RenderPass,
    ) -> Result<PipelineBundle<B>, PipelineCreationError> {
        let vertex_shader_module = Self::load_shader_module(
            device,
            include_bytes!("../../../assets/gen/shaders/default_vert.vert.spv"),
        );

        let fragment_shader_module = Self::load_shader_module(
            device,
            include_bytes!("../../../assets/gen/shaders/default_frag.frag.spv"),
        );

        let input_assembler = InputAssemblerDesc::new(Primitive::TriangleList);
        let shaders = GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &vertex_shader_module,
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Specialization::default(),
            }),
            domain: None,
            geometry: None,
            hull: None,
        };

        let vertex_buffers = vec![VertexBufferDesc {
            binding: 0,
            stride: mem::size_of::<Vertex>() as ElemStride,
            rate: VertexInputRate::Vertex,
        }];

        let attributes = Vertex::attributes();

        let rasterizer = Rasterizer {
            depth_clamping: false,
            polygon_mode: PolygonMode::Fill,
            cull_face: Face::empty(),
            front_face: FrontFace::Clockwise,
            depth_bias: None,
            conservative: false,
        };

        let depth_stencil = DepthStencilDesc {
            depth: None,
            depth_bounds: false,
            stencil: None,
        };

        let blender = BlendDesc {
            logic_op: Some(LogicOp::Copy),
            targets: vec![ColorBlendDesc {
                mask: ColorMask::ALL,
                blend: Some(BlendState::ALPHA),
            }],
        };

        let baked_states = BakedStates {
            viewport: Some(Viewport {
                rect: extent.to_extent().rect(),
                depth: (0.0..1.0),
            }),
            scissor: Some(extent.to_extent().rect()),
            blend_color: None,
            depth_bounds: None,
        };

        let descriptor_set_layout = Some(unsafe {
            device
                .create_descriptor_set_layout(
                    &[
                        DescriptorSetLayoutBinding {
                            binding: 0,
                            ty: DescriptorType::SampledImage,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                        DescriptorSetLayoutBinding {
                            binding: 1,
                            ty: DescriptorType::Sampler,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                    ],
                    &[],
                )
                .map_err(|e| PipelineCreationError::DescriptorSetLayout(e))?
        });

        // @techdebt wtf am I doing
        let descriptor_pool = Some(unsafe {
            device
                .create_descriptor_pool(
                    100,
                    &[
                        DescriptorRangeDesc {
                            ty: DescriptorType::SampledImage,
                            count: 100,
                        },
                        DescriptorRangeDesc {
                            ty: DescriptorType::Sampler,
                            count: 100,
                        },
                    ],
                    gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
                )
                .map_err(|e| PipelineCreationError::DescriptorPool(e))?
        });

        let push_constants = vec![(
            ShaderStageFlags::VERTEX,
            0u32..std::mem::size_of::<StandardPushConstants>() as u32,
        )];

        let layout = unsafe {
            device
                .create_pipeline_layout(&descriptor_set_layout, push_constants)
                .map_err(|e| PipelineCreationError::PipelineLayout(e))?
        };

        let gfx_pipeline = {
            let desc = GraphicsPipelineDesc {
                shaders,
                rasterizer,
                vertex_buffers,
                attributes,
                input_assembler,
                blender,
                depth_stencil,
                multisampling: None,
                baked_states,
                layout: &layout,
                subpass: Subpass {
                    index: 0,
                    main_pass: render_pass,
                },
                flags: PipelineCreationFlags::empty(),
                parent: BasePipeline::None,
            };

            unsafe {
                device
                    .create_graphics_pipeline(&desc, None)
                    .map_err(|e| PipelineCreationError::PipelineCreation(e, "Standard"))?
            }
        };

        Ok(PipelineBundle {
            descriptor_set_layout,
            descriptor_pool,
            pipeline_layout: manual_new!(layout),
            graphics_pipeline: manual_new!(gfx_pipeline),
        })
    }

    pub fn create_imgui_pipeline(
        device: &B::Device,
        render_pass: &B::RenderPass,
    ) -> Result<PipelineBundle<B>, PipelineCreationError> {
        let vertex_shader_module = Self::load_shader_module(
            device,
            include_bytes!("../../../assets/gen/shaders/imgui_vert.vert.spv"),
        );

        let fragment_shader_module = Self::load_shader_module(
            device,
            include_bytes!("../../../assets/gen/shaders/imgui_frag.frag.spv"),
        );

        let shaders = GraphicsShaderSet {
            vertex: EntryPoint {
                entry: "main",
                module: &vertex_shader_module,
                specialization: Specialization::default(),
            },
            fragment: Some(EntryPoint {
                entry: "main",
                module: &fragment_shader_module,
                specialization: Specialization::default(),
            }),
            domain: None,
            geometry: None,
            hull: None,
        };

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(
                    &[
                        DescriptorSetLayoutBinding {
                            binding: 0,
                            ty: DescriptorType::SampledImage,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                        DescriptorSetLayoutBinding {
                            binding: 1,
                            ty: DescriptorType::Sampler,
                            count: 1,
                            stage_flags: ShaderStageFlags::FRAGMENT,
                            immutable_samplers: false,
                        },
                    ],
                    &[],
                )
                .map_err(|e| PipelineCreationError::DescriptorSetLayout(e))?
        };

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(
                    1,
                    &[
                        DescriptorRangeDesc {
                            ty: DescriptorType::SampledImage,
                            count: 1,
                        },
                        DescriptorRangeDesc {
                            ty: DescriptorType::Sampler,
                            count: 1,
                        },
                    ],
                    gfx_hal::pso::DescriptorPoolCreateFlags::empty(),
                )
                .map_err(|e| PipelineCreationError::DescriptorPool(e))?
        };

        let push_constants = vec![(
            ShaderStageFlags::VERTEX,
            0u32..std::mem::size_of::<ImguiPushConstants>() as u32,
        )];
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(Some(&descriptor_set_layout), push_constants)
                .map_err(|e| PipelineCreationError::PipelineLayout(e))?
        };

        let imgui_pipeline = {
            let mut desc = GraphicsPipelineDesc::new(
                shaders,
                Primitive::TriangleList,
                Rasterizer::FILL,
                &pipeline_layout,
                Subpass {
                    index: 0,
                    main_pass: render_pass,
                },
            );

            desc.vertex_buffers.push(VertexBufferDesc {
                binding: 0,
                stride: mem::size_of::<DrawVert>() as ElemStride,
                rate: VertexInputRate::Vertex,
            });

            desc.attributes.push(
                // Position
                AttributeDesc {
                    location: 0,
                    binding: 0,
                    element: Element {
                        format: Format::Rg32Sfloat,
                        offset: offset_of!(DrawVert, pos) as u32,
                    },
                },
            );

            desc.attributes.push(
                // UV
                AttributeDesc {
                    location: 1,
                    binding: 0,
                    element: Element {
                        format: Format::Rg32Sfloat,
                        offset: offset_of!(DrawVert, uv) as u32,
                    },
                },
            );

            desc.attributes.push(
                // Color
                AttributeDesc {
                    location: 2,
                    binding: 0,
                    element: Element {
                        format: Format::Rgba8Unorm,
                        offset: offset_of!(DrawVert, col) as u32,
                    },
                },
            );

            desc.blender.targets.push(ColorBlendDesc {
                mask: ColorMask::ALL,
                blend: Some(BlendState::ALPHA),
            });

            unsafe {
                device
                    .create_graphics_pipeline(&desc, None)
                    .map_err(|e| PipelineCreationError::PipelineCreation(e, "Imgui"))?
            }
        };

        let pipeline_bundle = PipelineBundle::new(
            descriptor_set_layout,
            Some(descriptor_pool),
            pipeline_layout,
            imgui_pipeline,
        );

        Ok(pipeline_bundle)
    }

    fn bind_to_memory<T: Copy>(
        device: &mut B::Device,
        buffer_bundle: &mut BufferBundle<B>,
        data: &'static [T],
    ) -> Result<(), MemoryWritingError> {
        unsafe {
            let mapping = device.map_memory(&buffer_bundle.memory, 0..buffer_bundle.requirements.size)?;
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                mapping,
                data.len() * std::mem::size_of::<T>(),
            );
            buffer_bundle.mapped = Some(mapping);
            device.unmap_memory(&buffer_bundle.memory);
        };

        Ok(())
    }

    fn load_shader_module(device: &B::Device, shader_file: &'static [u8]) -> B::ShaderModule {
        let spirv = gfx_hal::pso::read_spirv(std::io::Cursor::new(shader_file)).unwrap();
        unsafe { device.create_shader_module(&spirv) }.unwrap()
    }
}

impl<B: Backend> core::ops::Drop for Renderer<B> {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();

        unsafe {
            for fence in self.in_flight_fences.drain(..) {
                self.device.destroy_fence(fence);
            }
            for semaphore in self.render_finished_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore)
            }
            for semaphore in self.image_available_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore)
            }
            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer);
            }
            for image_view in self.image_views.drain(..) {
                self.device.destroy_image_view(image_view);
            }
            for this_pipeline in self.pipeline_bundles.drain(..) {
                this_pipeline.manually_drop(&self.device);
            }

            for this_bundled_bundle in self.imgui_vertex_index_buffer_bundles.drain(..) {
                this_bundled_bundle.manually_drop_parts(&self.device);
            }

            self.iconic_quad_vert_index_buffer_bundle
                .manually_drop_parts(&self.device);

            // LAST RESORT STYLE CODE, NOT TO BE IMITATED LIGHTLY
            use core::ptr::read;
            self.device.destroy_command_pool(manual_drop!(self.command_pool));
            self.device.destroy_render_pass(manual_drop!(self.render_pass));
            self.device.destroy_swapchain(manual_drop!(self.swapchain));

            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }
    }
}
