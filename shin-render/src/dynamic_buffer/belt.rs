use core::fmt;
use std::sync::mpsc;

use shin_primitives::exclusive::Exclusive;
use shin_render_shader_types::buffer::{
    BufferRef, BufferUsage, BytesAddress, OwnedBuffer, types::RawMarker,
};
use tracing::info;

/// Efficiently performs many buffer writes by sharing and reusing temporary buffers.
///
/// Internally it uses a ring-buffer of staging buffers that are sub-allocated.
/// Its advantage over [`Queue::write_buffer_with()`] is that the individual allocations
/// are cheaper; `StagingBelt` is most useful when you are writing very many small pieces
/// of data. It can be understood as a sort of arena allocator.
///
/// Using a staging belt is slightly complicated, and generally goes as follows:
/// 1. Use [`StagingBelt::allocate()`] to allocate buffer slices, then write your data to them.
/// 2. Call [`StagingBelt::finish()`].
/// 3. Submit all command encoders that were used in step 2.
/// 4. Call [`StagingBelt::recall()`].
///
/// [`Queue::write_buffer_with()`]: wgpu::Queue::write_buffer_with
pub struct StagingBelt {
    chunk_size: BytesAddress,
    alloc_buffer_counter: u32,
    /// Chunks into which we are accumulating data to be transferred.
    active_chunks: Vec<Chunk>,
    /// Chunks that have scheduled transfers already; they are unmapped and some
    /// command encoder has one or more commands with them as source.
    closed_chunks: Vec<Chunk>,
    /// Chunks that are back from the GPU and ready to be mapped for write and put
    /// into `active_chunks`.
    free_chunks: Vec<Chunk>,
    /// When closed chunks are mapped again, the map callback sends them here.
    sender: Exclusive<mpsc::Sender<Chunk>>,
    /// Free chunks are received here to be put on `self.free_chunks`.
    receiver: Exclusive<mpsc::Receiver<Chunk>>,
}

impl StagingBelt {
    /// Create a new staging belt.
    ///
    /// The `chunk_size` is the unit of internal buffer allocation; writes will be
    /// sub-allocated within each chunk. Therefore, for optimal use of memory, the
    /// chunk size should be:
    ///
    /// * larger than the largest single [`StagingBelt::allocate()`] operation;
    /// * 1-4 times less than the total amount of data uploaded per submission
    ///   (per [`StagingBelt::finish()`]); and
    /// * bigger is better, within these bounds.
    pub fn new(chunk_size: BytesAddress) -> Self {
        let (sender, receiver) = mpsc::channel();
        StagingBelt {
            chunk_size,
            alloc_buffer_counter: 0,
            active_chunks: Vec::new(),
            closed_chunks: Vec::new(),
            free_chunks: Vec::new(),
            sender: Exclusive::new(sender),
            receiver: Exclusive::new(receiver),
        }
    }

    /// Allocate a staging belt slice with the given `size` and `alignment` and return it.
    ///
    /// This returns two buffer slices: a staging buffer slice and actual buffer slice.
    ///
    /// First, upload your data to the staging buffer by calling [`BufferSlice::get_mapped_range_mut()`]
    /// and writing your data into that [`BufferViewMut`].
    /// (The view must be dropped before [`StagingBelt::finish()`] is called.)
    ///
    /// The actual buffer will contain the data written after executing commands encoded by [`StagingBelt::finish()`].
    /// Therefore, any commands involving the actual buffer should be submitted after those commands.
    /// They should also be submitted before [`StagingBelt::recall()`] is called.
    ///
    /// If the `size` is greater than the space available in any free internal buffer, a new buffer
    /// will be allocated for it. Therefore, the `chunk_size` passed to [`StagingBelt::new()`]
    /// should ideally be larger than every such size.
    ///
    /// The chosen slice will be positioned within the buffer at a multiple of `alignment`,
    /// which may be used to meet alignment requirements for the operation you wish to perform
    /// with the slice. This does not necessarily affect the alignment of the [`BufferViewMut`].
    ///
    /// NOTE: staging buffer slice can be larger than requested to satisfy `wgpu::MAP_ALIGNMENT`.
    pub fn allocate(
        &mut self,
        size: BytesAddress,
        alignment: BytesAddress,
        device: &wgpu::Device,
    ) -> (BufferRef<RawMarker>, BufferRef<RawMarker>) {
        assert!(
            alignment.get().is_power_of_two(),
            "alignment must be a power of two, not {alignment}"
        );
        // At minimum, we must have alignment sufficient to map the buffer.
        let alignment = alignment.max(BytesAddress::new(wgpu::MAP_ALIGNMENT));

        let mut chunk = if let Some(index) = self
            .active_chunks
            .iter()
            .position(|chunk| chunk.can_allocate(size, alignment))
        {
            self.active_chunks.swap_remove(index)
        } else {
            self.receive_chunks(); // ensure self.free_chunks is up to date

            if let Some(index) = self
                .free_chunks
                .iter()
                .position(|chunk| chunk.can_allocate(size, alignment))
            {
                self.free_chunks.swap_remove(index)
            } else {
                info!("Allocating a new staging belt chunk!");

                let size = self.chunk_size.max(size);
                let index = self.alloc_buffer_counter;
                self.alloc_buffer_counter += 1;

                if device
                    .features()
                    .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
                {
                    // do not allocate a separate staging buffer if `MAPPABLE_PRIMARY_BUFFERS` is supported
                    Chunk {
                        staging: OwnedBuffer::allocate_raw(
                            device,
                            size,
                            BufferUsage::DynamicMappable,
                            true,
                            Some(&format!("StagingBelt/buffer #{index}")),
                        ),
                        actual: None,
                        offset: BytesAddress::ZERO,
                    }
                } else {
                    Chunk {
                        staging: OwnedBuffer::allocate_raw(
                            device,
                            size,
                            BufferUsage::StagingWrite,
                            true,
                            Some(&format!("StagingBelt/staging #{index}")),
                        ),
                        actual: Some(OwnedBuffer::allocate_raw(
                            device,
                            size,
                            BufferUsage::Dynamic,
                            false,
                            Some(&format!("StagingBelt/actual #{index}")),
                        )),
                        offset: BytesAddress::ZERO,
                    }
                }
            }
        };

        let allocation_offset = chunk.allocate(size, alignment);

        self.active_chunks.push(chunk);
        let chunk = self.active_chunks.last().unwrap();

        let (staging, actual) = chunk.get_buffers();

        (
            staging.slice_bytes(
                allocation_offset,
                size.align_to(BytesAddress::new(wgpu::MAP_ALIGNMENT)),
            ),
            actual.slice_bytes(allocation_offset, size),
        )
    }

    /// Prepare currently mapped buffers for use in a submission & schedule copies to the actual buffers.
    ///
    /// At this point, all the partially used staging buffers are closed (cannot be used for
    /// further writes) until after [`StagingBelt::recall()`] is called *and* the GPU is done
    /// copying the data from them.
    ///
    /// The actual buffers will contain the written data and can be used to do GPU work if submitted until [`StagingBelt::recall()`] is called.
    pub fn finish(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for chunk in self.active_chunks.drain(..) {
            chunk.staging.unmap();
            // copy from staging buffer to actual buffer if we are using separate buffers
            if let Some(actual) = &chunk.actual {
                let (staging, staging_offset, staging_size) =
                    chunk.staging.as_buffer_ref().into_parts();
                let (actual, actual_offset, actual_size) = actual.as_buffer_ref().into_parts();
                assert_eq!(staging_size, actual_size);
                assert_eq!(staging_offset, BytesAddress::ZERO);
                assert_eq!(actual_offset, BytesAddress::ZERO);

                let size = chunk
                    .offset
                    .align_to(BytesAddress::new(wgpu::COPY_BUFFER_ALIGNMENT));

                encoder.copy_buffer_to_buffer(staging, 0, actual, 0, size.get());
            }
            self.closed_chunks.push(chunk);
        }
    }

    /// Recall all of the closed buffers back to be reused.
    ///
    /// This must only be called after the command encoder(s) provided to
    /// [`StagingBelt::finish()`] are submitted. Additional calls are harmless.
    /// Not calling this as soon as possible may result in increased buffer memory usage.
    pub fn recall(&mut self) {
        self.receive_chunks();

        for Chunk {
            staging,
            actual,
            offset,
        } in self.closed_chunks.drain(..)
        {
            let sender = self.sender.get_mut().clone();
            staging.map_async(wgpu::MapMode::Write, move |staging, result| {
                result.unwrap();
                let _ = sender.send(Chunk {
                    staging,
                    actual,
                    offset,
                });
            });
        }
    }

    /// Move all chunks that the GPU is done with (and are now mapped again)
    /// from `self.receiver` to `self.free_chunks`.
    fn receive_chunks(&mut self) {
        while let Ok(mut chunk) = self.receiver.get_mut().try_recv() {
            chunk.offset = BytesAddress::ZERO;
            self.free_chunks.push(chunk);
        }
    }
}

impl fmt::Debug for StagingBelt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StagingBelt")
            .field("chunk_size", &self.chunk_size)
            .field("active_chunks", &self.active_chunks.len())
            .field("closed_chunks", &self.closed_chunks.len())
            .field("free_chunks", &self.free_chunks.len())
            .finish_non_exhaustive()
    }
}

struct Chunk {
    staging: OwnedBuffer<RawMarker>,
    actual: Option<OwnedBuffer<RawMarker>>,
    offset: BytesAddress,
}

impl Chunk {
    pub fn get_buffers(&self) -> (&OwnedBuffer<RawMarker>, &OwnedBuffer<RawMarker>) {
        (&self.staging, self.actual.as_ref().unwrap_or(&self.staging))
    }

    fn can_allocate(&self, size: BytesAddress, alignment: BytesAddress) -> bool {
        let alloc_start = self.offset.align_to(alignment);
        let alloc_end = alloc_start + size;

        alloc_end <= self.staging.raw_bytes_size()
    }

    fn allocate(&mut self, size: BytesAddress, alignment: BytesAddress) -> BytesAddress {
        let alloc_start = self.offset.align_to(alignment);
        let alloc_end = alloc_start + size;

        assert!(alloc_end <= self.staging.raw_bytes_size());
        self.offset = alloc_end;
        alloc_start
    }
}
