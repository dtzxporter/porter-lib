use std::fmt::Debug;
use std::sync::OnceLock;

use wgpu::Backends;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Features;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::InstanceFlags;
use wgpu::PowerPreference;
use wgpu::Queue;
use wgpu::RequestAdapterOptionsBase;
use wgpu::ShaderModule;

/// Stores an active GPU device, queue, and compiled shaders.
#[derive(Clone)]
pub struct GPUInstance {
    instance: Instance,
    device: Device,
    queue: Queue,
    gpu_converter_shader: ShaderModule,
    gpu_preview_shader: ShaderModule,
}

impl GPUInstance {
    /// Creates a new instance of the GPU instance.
    pub fn new(
        instance: Instance,
        device: Device,
        queue: Queue,
        gpu_converter_shader: ShaderModule,
        gpu_preview_shader: ShaderModule,
    ) -> Self {
        Self {
            instance,
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

    /// Generates a memory report for this instance.
    pub fn memory_report(&self) -> Option<impl Debug> {
        self.instance.generate_report()
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
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::all() & !Backends::GL,
        flags: if cfg!(debug_assertions) {
            InstanceFlags::debugging()
        } else {
            InstanceFlags::empty()
        },
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&RequestAdapterOptionsBase {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .unwrap();

    let descriptor = DeviceDescriptor {
        required_features: Features::TEXTURE_COMPRESSION_BC
            | Features::TEXTURE_FORMAT_16BIT_NORM
            | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | Features::POLYGON_MODE_LINE
            | Features::FLOAT32_FILTERABLE,
        required_limits: adapter.limits(),
        ..Default::default()
    };

    let (device, queue) = adapter.request_device(&descriptor).await.unwrap();

    let gpu_converter_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/gpu_converter.wgsl"));

    let gpu_preview_shader =
        device.create_shader_module(wgpu::include_wgsl!("../shaders/gpu_preview.wgsl"));

    GPUInstance::new(
        instance,
        device,
        queue,
        gpu_converter_shader,
        gpu_preview_shader,
    )
}

/// Global GPU instance, device, queue, and shaders.
static GPU_INSTANCE: OnceLock<GPUInstance> = OnceLock::new();

/// Gets or initializes the current GPU instance.
pub fn gpu_instance() -> &'static GPUInstance {
    GPU_INSTANCE.get_or_init(|| pollster::block_on(initialize()))
}
