use vulkano::{DeviceAddress, buffer::BufferContents};

#[derive(BufferContents, Copy, Clone)]
#[repr(C)]
pub struct PushConstants {
    pub(crate) generation: u64,
    pub(crate) word_count: u32,
    pub(crate) nonce_index: u32,
    pub(crate) k: u32,
    pub(crate) words: DeviceAddress,
    pub(crate) nonce: DeviceAddress,
}
