use once_cell::sync::OnceCell;

use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Features;
use wgpu::Instance;
use wgpu::Queue;
use wgpu::RequestAdapterOptionsBase;
use wgpu::ShaderModule;

/// Stores an active GPU device, queue, and compiled shaders.
pub struct GPUInstance {
    device: Device,
    queue: Queue,
    gpu_converter_shader: ShaderModule,
    gpu_preview_shader: ShaderModule,
}

impl GPUInstance {
    /// Creates a new instance of the GPU instance.
    pub fn new(
        device: Device,
        queue: Queue,
        gpu_converter_shader: ShaderModule,
        gpu_preview_shader: ShaderModule,
    ) -> Self {
        Self {
            device,
            queue,
            gpu_converter_shader,
            gpu_preview_shader,
        }
    }

    /// Returns the device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns the device queue.
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Returns the gpu converter shader module.
    pub fn gpu_converter_shader(&self) -> &ShaderModule {
        &self.gpu_converter_shader
    }

    /// Returns the gpu preview shader module.
    pub fn gpu_preview_shader(&self) -> &ShaderModule {
        &self.gpu_preview_shader
    }
}

/// Async initialization routine required for `wgpu`.
async fn initialize() -> GPUInstance {
    let instance = Instance::new(Default::default());

    let adapter = instance
        .request_adapter(&RequestAdapterOptionsBase {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                features: Features::TEXTURE_COMPRESSION_BC
                    | Features::TEXTURE_FORMAT_16BIT_NORM
                    | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | Features::POLYGON_MODE_LINE,
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    let gpu_converter_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/gpu_converter.wgsl"));

    let gpu_preview_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/gpu_preview.wgsl"));

    GPUInstance::new(device, queue, gpu_converter_shader, gpu_preview_shader)
}

/// Global GPU instance, device, queue, and shaders.
static GPU_INSTANCE: OnceCell<GPUInstance> = OnceCell::new();

/// Gets or initializes the current GPU instance.
pub fn gpu_instance() -> &'static GPUInstance {
    GPU_INSTANCE.get_or_init(|| pollster::block_on(initialize()))
}
