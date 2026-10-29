/// Graphics API the device runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuApi {
    Vulkan,
    Metal,
    Dx12,
    WebGpu,
    Gl,
}

/// What the device can do, probed once at startup. Decides the render tier (never the OS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub gpu_api: GpuApi,
    pub gpu_memory_mb: u32,
    pub system_memory_mb: u32,
    pub max_texture_size: u32,
    pub supports_compute: bool,
    pub has_touch: bool,
    pub has_threads: bool,
}

impl Default for Capabilities {
    /// The weakest device we support; used until a probe reports real numbers.
    fn default() -> Self {
        Self {
            gpu_api: GpuApi::Gl,
            gpu_memory_mb: 1024,
            system_memory_mb: 2048,
            max_texture_size: 4096,
            supports_compute: false,
            has_touch: false,
            has_threads: false,
        }
    }
}
