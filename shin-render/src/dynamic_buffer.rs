use std::{fmt::Debug, sync::Arc};

use shin_render_shader_types::buffer::{
    types::{BufferType, RawMarker},
    Buffer, BufferUsage, BytesAddress, DynamicBufferBackend, SharedBuffer,
};
use sketches_ddsketch::DDSketch;
use tracing::info;

pub struct DynamicBufferStats {
    pub start_time: std::time::Instant,
    pub allocated: u64,
    pub wasted: u64,
    pub generation: u64,
    pub alignment_histogram: DDSketch,
    pub size_histogram: DDSketch,
}

impl DynamicBufferStats {
    pub fn new() -> Self {
        let config = sketches_ddsketch::Config::new(0.001, 2048, 1.0);

        Self {
            start_time: std::time::Instant::now(),
            allocated: 0,
            wasted: 0,
            generation: 0,
            alignment_histogram: DDSketch::new(config),
            size_histogram: DDSketch::new(config),
        }
    }
}

fn dump_summary(summary: &DDSketch) -> String {
    fn dump_summary_inner(summary: &DDSketch) -> Option<String> {
        Some(format!(
            "q5={:.1}, q25={:.1}, q50={:.1}, q75={:.1}, q95={:.1}",
            summary.quantile(0.05).unwrap()?,
            summary.quantile(0.25).unwrap()?,
            summary.quantile(0.5).unwrap()?,
            summary.quantile(0.75).unwrap()?,
            summary.quantile(0.95).unwrap()?,
        ))
    }

    dump_summary_inner(summary).unwrap_or_else(|| "empty".to_string())
}

fn per_second_summary(value: u64, elapsed: f64) -> String {
    format!("{} ({:.1}/s)", value, (value as f64 / elapsed).round())
}

impl Debug for DynamicBufferStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elapsed = self.start_time.elapsed().as_secs_f64();

        f.debug_struct("DynamicBufferStats")
            .field("allocated", &per_second_summary(self.allocated, elapsed))
            .field("wasted", &per_second_summary(self.wasted, elapsed))
            .field("generation", &self.generation)
            .field(
                "alignment_histogram",
                &dump_summary(&self.alignment_histogram),
            )
            .field("size_histogram", &dump_summary(&self.size_histogram))
            .finish()
    }
}

/// Dynamically allocates space in a gpu buffer, mostly used for submitting uniform data
pub struct DynamicBuffer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    block_size: BytesAddress,
    position: BytesAddress,
    buffer: SharedBuffer<RawMarker>,
    stats: DynamicBufferStats,
}

impl DynamicBuffer {
    const ALIGNMENT: BytesAddress = BytesAddress::new(16);

    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        block_size: BytesAddress,
    ) -> Self {
        let buffer = Buffer::allocate_raw(
            &device,
            block_size,
            BufferUsage::DynamicBuffer,
            Some(&format!("DynamicBuffer generation {}", 0)),
        );

        Self {
            device,
            queue,
            block_size,
            position: BytesAddress::ZERO,
            buffer,
            stats: DynamicBufferStats::new(),
        }
    }
}

impl DynamicBufferBackend for DynamicBuffer {
    fn get_with_raw_data(
        &mut self,
        alignment: BytesAddress,
        data: &[u8],
    ) -> SharedBuffer<RawMarker> {
        let offset = self.position.align_to(alignment);
        let size = BytesAddress::new(data.len() as _);

        let mut waste = offset - self.position;

        if offset + size > self.block_size {
            info!("reallocating the dynamic buffer, stats={:?}", self.stats);

            self.stats.generation += 1;

            self.buffer = Buffer::allocate_raw(
                &self.device,
                self.block_size,
                BufferUsage::DynamicBuffer,
                Some(&format!(
                    "DynamicBuffer generation {}",
                    self.stats.generation
                )),
            );
            self.position = BytesAddress::ZERO;

            return self.get_with_raw_data(alignment, data);
        }

        assert!(RawMarker::is_valid_offset(offset));
        assert!(RawMarker::is_valid_logical_size(size));

        self.position = (offset + size).align_to(Self::ALIGNMENT);

        waste += self.position - (offset + size);

        self.stats.wasted += waste.get();
        self.stats.allocated += size.get();
        self.stats.alignment_histogram.add(alignment.get() as f64);
        self.stats.size_histogram.add(size.get() as f64);

        self.buffer.write(&self.queue, offset, data);

        self.buffer.slice_bytes(offset, size)
    }
}
