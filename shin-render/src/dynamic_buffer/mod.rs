mod belt;

use std::fmt::Debug;

use shin_render_shader_types::buffer::{
    BufferRef, BytesAddress, DynamicBufferBackend,
    types::{BufferType, RawMarker},
};
use sketches_ddsketch::DDSketch;

use crate::dynamic_buffer::belt::StagingBelt;

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
    device: wgpu::Device,
    belt: StagingBelt,
    stats: DynamicBufferStats,
}

impl DynamicBuffer {
    pub fn new(device: wgpu::Device, chunk_size: BytesAddress) -> Self {
        Self {
            device,
            belt: StagingBelt::new(chunk_size),
            stats: DynamicBufferStats::new(),
        }
    }

    pub fn finish(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.belt.finish(encoder)
    }

    pub fn recall(&mut self) {
        self.belt.recall()
    }
}

impl DynamicBufferBackend for DynamicBuffer {
    fn get_with_raw_data(&mut self, alignment: BytesAddress, data: &[u8]) -> BufferRef<RawMarker> {
        let logical_size = BytesAddress::new(data.len() as _);

        let (staging, actual) = self.belt.allocate(logical_size, alignment, &self.device);

        assert!(RawMarker::is_valid_logical_size(logical_size));

        self.stats.allocated += logical_size.get();
        self.stats.alignment_histogram.add(alignment.get() as f64);
        self.stats.size_histogram.add(logical_size.get() as f64);

        staging.get_mapped_range_mut()[..data.len()].copy_from_slice(data);

        actual
    }
}
