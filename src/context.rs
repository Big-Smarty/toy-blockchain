use std::sync::Arc;

use vulkano::NonNullDeviceAddress;
use vulkano::VulkanLibrary;
use vulkano::buffer::Buffer;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocatorCreateInfo;
use vulkano::device::DeviceFeatures;
use vulkano::device::Queue;
use vulkano::device::QueueFlags;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::PipelineShaderStageCreateInfo;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PushConstantRange;
use vulkano::shader::ShaderStages;
use vulkano::sync;
use vulkano::sync::GpuFuture;

use crate::push_constants::PushConstants;
use crate::shader;

const WORK_GROUP_COUNTS: [u32; 3] = [4096, 1, 1];

pub struct Context {
    pub(crate) _library: Arc<VulkanLibrary>,
    pub(crate) _instance: Arc<Instance>,
    pub(crate) _physical_device: Arc<PhysicalDevice>,
    pub(crate) device: Arc<Device>,
    pub(crate) queue: Arc<Queue>,
    pub(crate) _allocator: Arc<StandardMemoryAllocator>,
    pub(crate) words_buffer: Arc<Subbuffer<[u32]>>,
    pub(crate) nonce_buffer: Arc<Subbuffer<u64>>,
    pub(crate) pipeline_layout: Arc<PipelineLayout>,
    pub(crate) pipeline: Arc<ComputePipeline>,
    pub(crate) command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl Context {
    pub fn new(words: &Vec<u32>) -> Self {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("failed to create instance");

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .next()
            .expect("no devices available");

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family")
            as u32;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_features: DeviceFeatures {
                    buffer_device_address: true,
                    shader_int64: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("failed to create device");

        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let words_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            words.iter().map(|x| *x),
        )
        .expect("Failed to create the words buffer!");

        let nonce = 0u64;
        let nonce_buffer = Buffer::from_data(
            memory_allocator.clone(),
            vulkano::buffer::BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                ..Default::default()
            },
            vulkano::memory::allocator::AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            nonce,
        )
        .expect("Failed to create the nonce buffer!");

        let shader = shader::load(device.clone()).unwrap();
        let cs = shader.entry_point("main").unwrap();
        let stage = PipelineShaderStageCreateInfo::new(cs);

        let mut pipeline_layout_create_info =
            vulkano::pipeline::layout::PipelineLayoutCreateInfo::default();
        pipeline_layout_create_info.push_constant_ranges = vec![PushConstantRange {
            stages: ShaderStages::COMPUTE,
            offset: 0,
            size: core::mem::size_of::<PushConstants>() as u32,
        }];
        let layout = PipelineLayout::new(device.clone(), pipeline_layout_create_info).unwrap();

        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout.clone()),
        )
        .expect("Failed to create pipeline");

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        Self {
            _library: library.clone(),
            _instance: instance.clone(),
            _physical_device: physical_device.clone(),
            device: device.clone(),
            queue: queue.clone(),
            _allocator: memory_allocator.clone(),
            words_buffer: Arc::new(words_buffer.clone()),
            nonce_buffer: Arc::new(nonce_buffer.clone()),
            pipeline_layout: layout.clone(),
            pipeline: pipeline.clone(),
            command_buffer_allocator: command_buffer_allocator.clone(),
        }
    }

    pub fn words_address(&self) -> NonNullDeviceAddress {
        self.words_buffer.device_address().unwrap()
    }

    pub fn nonce_address(&self) -> NonNullDeviceAddress {
        self.nonce_buffer.device_address().unwrap()
    }

    pub fn invoke(&mut self, push_constants: &PushConstants) -> u64 {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        {
            let mut nonce_mapping = self.nonce_buffer.write().unwrap();
            *nonce_mapping = 0;
        }

        unsafe {
            command_buffer_builder
                .bind_pipeline_compute(self.pipeline.clone())
                .unwrap()
                .push_constants(self.pipeline_layout.clone(), 0, *push_constants)
                .unwrap()
                .dispatch(WORK_GROUP_COUNTS)
                .unwrap();
        }
        let command_buffer = command_buffer_builder.build().unwrap().clone();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();

        let nonce = *self.nonce_buffer.read().unwrap();

        nonce
    }

    // TODO: implement this function!!!
    pub fn update_words(&mut self, words: &Vec<u32>) {
        let mut write_words = self.words_buffer.write().unwrap();
        write_words.copy_from_slice(words.as_slice());
    }
}
