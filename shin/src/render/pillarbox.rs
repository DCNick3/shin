use crate::render::{
    GpuCommonResources, PosVertex, Renderable, VertexSource, VIRTUAL_HEIGHT, VIRTUAL_WIDTH,
};
use cgmath::{Matrix4, Vector3, Vector4};
use wgpu::util::DeviceExt;

pub struct Pillarbox {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Pillarbox {
    pub fn new(resources: &GpuCommonResources) -> Self {
        let letterbox_size = 10000000.0;
        let left = -VIRTUAL_WIDTH / 2.0;
        let ultra_left = left - letterbox_size;
        let right = VIRTUAL_WIDTH / 2.0;
        let ultra_right = right + letterbox_size;
        let top = VIRTUAL_HEIGHT / 2.0;
        let ultra_top = top + letterbox_size;
        let bottom = -VIRTUAL_HEIGHT / 2.0;
        let ultra_bottom = bottom - letterbox_size;

        // we want to draw 4 rectangles to the sides
        // those will paint over with black everything that should not be seen
        let vertices = [
            // 0
            PosVertex {
                position: Vector3::new(left, top, 0.0),
            },
            // 1
            PosVertex {
                position: Vector3::new(left, bottom, 0.0),
            },
            // 2
            PosVertex {
                position: Vector3::new(right, top, 0.0),
            },
            // 3
            PosVertex {
                position: Vector3::new(right, bottom, 0.0),
            },
            // ====
            // 4
            PosVertex {
                position: Vector3::new(ultra_left, top, 0.0),
            },
            // 5
            PosVertex {
                position: Vector3::new(ultra_left, bottom, 0.0),
            },
            // 6
            PosVertex {
                position: Vector3::new(ultra_right, top, 0.0),
            },
            // 7
            PosVertex {
                position: Vector3::new(ultra_right, bottom, 0.0),
            },
            // 8
            PosVertex {
                position: Vector3::new(left, ultra_top, 0.0),
            },
            // 9
            PosVertex {
                position: Vector3::new(right, ultra_top, 0.0),
            },
            // 10
            PosVertex {
                position: Vector3::new(left, ultra_bottom, 0.0),
            },
            // 11
            PosVertex {
                position: Vector3::new(right, ultra_bottom, 0.0),
            },
        ];

        let indices = [
            0u16, 1, 5, 0, 4, 5, // left
            2, 3, 7, 2, 6, 7, // right
            0, 2, 9, 0, 8, 9, // top
            1, 3, 11, 1, 10, 11, // bottom
        ];

        let vertex_buffer =
            resources
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("pillarbox_vertex_buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer = resources
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("pillarbox_index_buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

impl Renderable for Pillarbox {
    fn render<'enc>(
        &'enc self,
        resources: &'enc GpuCommonResources,
        render_pass: &mut wgpu::RenderPass<'enc>,
        transform: Matrix4<f32>,
    ) {
        render_pass.push_debug_group("Pillarbox");
        resources.pipelines.fill_screen.draw(
            render_pass,
            VertexSource::VertexIndexBuffer {
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                indices: 0..self.num_indices,
                instances: 0..1,
            },
            transform,
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        );
        render_pass.pop_debug_group();
    }

    fn resize(&mut self, _resources: &GpuCommonResources) {
        // No internal state to resize
    }
}
