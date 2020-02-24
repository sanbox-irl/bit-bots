use super::SpriteName;
use anyhow::Error as AnyError;
use gfx_hal::{
    device::{CreationError, MapError, OomOrDeviceLost, OutOfMemory, ShaderError},
    UnsupportedBackend,
};
use std::error::Error;

#[allow(unused_macros)]
macro_rules! quick_from {
    ($our_type:ty, $our_member:expr, $target_type:ty) => {
        impl From<$target_type> for $our_type {
            fn from(error: $target_type) -> Self {
                $our_member(error)
            }
        }
    };
}

#[derive(Debug)]
pub enum DrawingError {
    AcquireAnImageFromSwapchain(gfx_hal::window::AcquireError),
    WaitOnFence(OomOrDeviceLost),
    ResetFence(OutOfMemory),
    PresentIntoSwapchain(gfx_hal::window::PresentError),
    BufferCreationError,
    BufferError,
    SpriteWithoutTexturePage(SpriteName),
    ProcessingQueuedData,
    DynamicTextureCreation(AnyError),
}

impl Error for DrawingError {}

impl std::fmt::Display for DrawingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in Drawing {:?}", self)
    }
}

#[derive(Debug)]
pub enum RendererCreationError {
    InstanceCreation(UnsupportedBackend),
    GraphicalAdapter,
    FindQueueFamily,
    OpenPhysicalAdapter(CreationError),
    OwnershipQueueGroup,
    FindCommandQueue,
    PresentMode,
    EmptyFormatList,
    WindowExist,
    SurfaceColor,
    Swapchain(gfx_hal::window::CreationError),
    Fence(OutOfMemory),
    ImageAvailableSemaphore(OutOfMemory),
    RenderFinishedSemaphore(OutOfMemory),
    RenderPassCreation(OutOfMemory),
    ImageViews(gfx_hal::image::ViewError),
    FrameBuffers(OutOfMemory),
    CommandPool(OutOfMemory),
}

impl Error for RendererCreationError {}

impl std::fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write: String = match self {
            RendererCreationError::InstanceCreation(unsupported) => {
                format!("Couldn't create an instance! {:?}", unsupported)
            }
            RendererCreationError::GraphicalAdapter => "Couldn't find a graphical adapter!".to_owned(),
            RendererCreationError::FindQueueFamily => {
                "Couldn't find a queue family with graphics!".to_owned()
            }
            RendererCreationError::OpenPhysicalAdapter(e) => {
                format!("Couldn't open the Physical Device: {}", e)
            }
            RendererCreationError::OwnershipQueueGroup => {
                "Couldn't take ownership of the QueueGroup".to_owned()
            }
            RendererCreationError::FindCommandQueue => {
                "The QueueGroup did not have any CommandQueues available".to_owned()
            }
            RendererCreationError::PresentMode => "No PresentMode values specified!".to_owned(),
            RendererCreationError::EmptyFormatList => "Preferred format list was empty!".to_owned(),
            RendererCreationError::WindowExist => "Window doesn't exist!".to_owned(),
            RendererCreationError::SurfaceColor => {
                "the Surface isn't capable of supporting color!".to_owned()
            }
            RendererCreationError::Swapchain(e) => format!("Failed to create the swapchain! => {}", e),
            RendererCreationError::Fence(e) => format!("Couldn't create a fence! => {}", e),
            RendererCreationError::ImageAvailableSemaphore(e) => {
                format!("Couldn't create the ImageAvailable semaphore! => {}", e)
            }
            RendererCreationError::RenderFinishedSemaphore(e) => {
                format!("Couldn't create the RenderFinished semaphore! => {}", e)
            }
            RendererCreationError::RenderPassCreation(e) => {
                format!("Couldn't create the RenderPass! => {}", e)
            }
            RendererCreationError::ImageViews(e) => {
                format!("Couldn't create the image view for the image! => {}", e)
            }
            RendererCreationError::FrameBuffers(e) => format!("Couldn't create the framebuffer! => {}", e),
            RendererCreationError::CommandPool(e) => {
                format!("Couldn't create the raw command pool! => {}", e)
            }
        };
        write!(f, "{}", write)
    }
}

#[derive(Debug)]
pub enum PipelineCreationError {
    ShaderCompilerFailed,
    VertexModule(ShaderError),
    FragmentModule(ShaderError),
    DescriptorSetLayout(OutOfMemory),
    DescriptorPool(OutOfMemory),
    PipelineLayout(OutOfMemory),
    PipelineCreation(gfx_hal::pso::CreationError, &'static str),
}

impl Error for PipelineCreationError {}

impl std::fmt::Display for PipelineCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write: String = match self {
            PipelineCreationError::ShaderCompilerFailed => format!("Shader compiler was not created!"),
            PipelineCreationError::VertexModule(e) => format!("Vertex module did not compile! => {}", e),
            PipelineCreationError::FragmentModule(e) => format!("Fragment module did not compile! => {}", e),
            PipelineCreationError::DescriptorSetLayout(e) => {
                format!("DescriptorSetLayout could not be allocated! => {}", e)
            }
            PipelineCreationError::DescriptorPool(e) => {
                format!("DescriptorPool could not be allocated! => {}", e)
            }
            PipelineCreationError::PipelineLayout(e) => {
                format!("PipelineLayout could not be allocated! => {}", e)
            }
            PipelineCreationError::PipelineCreation(e, k) => {
                format!("Pipeline {} could not be created! => {}", k, e)
            }
        };
        write!(f, "{}", write)
    }
}

#[derive(Debug)]
pub enum BufferBundleError {
    Creation(gfx_hal::buffer::CreationError),
}

impl Error for BufferBundleError {}

impl std::fmt::Display for BufferBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            BufferBundleError::Creation(e) => format!("Buffer creation error! => {}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug)]
pub enum BufferError {
    MemoryId,
    Allocate(gfx_hal::device::AllocationError),
    Bind(gfx_hal::device::BindError),
}

impl Error for BufferError {}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            BufferError::MemoryId => format!("MemoryID Error"),
            BufferError::Allocate(e) => format!("Buffer allocation error! => {}", e),
            BufferError::Bind(e) => format!("Buffer binding error! => {}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug)]
pub enum LoadedImageError {
    AcquireMappingWriter(MapError),
    CreateImage(gfx_hal::image::CreationError),
    ImageView(gfx_hal::image::ViewError),
    Sampler(gfx_hal::device::AllocationError),
    UploadFence(OutOfMemory),
    WaitForFence(OomOrDeviceLost),
}

impl Error for LoadedImageError {}

impl std::fmt::Display for LoadedImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_this = match self {
            LoadedImageError::AcquireMappingWriter(e) => format!(
                "Couldn't acquire a mapping writer to the staging buffer! => {}",
                e
            ),
            LoadedImageError::CreateImage(e) => format!("Couldn't create the image! => {}", e),
            LoadedImageError::ImageView(e) => format!("Couldn't create the image view! => {}", e),
            LoadedImageError::Sampler(e) => format!("Couldn't create the sampler! => {}", e),
            LoadedImageError::UploadFence(e) => format!("Couldn't create the upload fence! => {}", e),
            LoadedImageError::WaitForFence(e) => format!("Couldn't wait for the fence! => {:?}", e),
        };

        write!(f, "{}", write_this)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryWritingError {
    #[error("Couldn't acquire a mapping writer to the staging buffer! => {}", _0)]
    AcquireMappingWriter(MapError),
    #[error("Couldn't release the mapping writer to the staging buffer! => {}", _0)]
    ReleaseMappingWriter(OutOfMemory),
}

quick_from!(
    MemoryWritingError,
    MemoryWritingError::AcquireMappingWriter,
    MapError
);

quick_from!(
    MemoryWritingError,
    MemoryWritingError::ReleaseMappingWriter,
    OutOfMemory
);
