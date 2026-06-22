use std::marker::PhantomData;

/// A wrapper around `wgpu::Buffer` that automatically resizes when data size exceeds its capacity.
pub struct DynamicBuffer<T> {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) capacity: usize,
    usage: wgpu::BufferUsages,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    /// Creates a new `DynamicBuffer` with the specified initial capacity and usage.
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, initial_capacity: usize) -> Self {
        let size = (initial_capacity * std::mem::size_of::<T>()) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DynamicBuffer"),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity: initial_capacity,
            usage,
            _marker: PhantomData,
        }
    }

    /// Writes data to the buffer. If the data size exceeds the current capacity, the buffer will be reallocated with a larger capacity.
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        if data.is_empty() {
            return;
        }

        if data.len() > self.capacity {
            self.capacity = data.len().next_power_of_two();
            let size = (self.capacity * std::mem::size_of::<T>()) as u64;
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Buffer (resized)"),
                size,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}
