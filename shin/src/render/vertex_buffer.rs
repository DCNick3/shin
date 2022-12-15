use crate::render;
use crate::render::{GpuCommonResources, PosColTexVertex, PosVertex, TextVertex, VertexSource};
use cgmath::{Vector2, Vector3, Vector4};
use wgpu::util::DeviceExt;

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

// TODO: derive this
impl Vertex for PosColTexVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        PosColTexVertex::desc()
    }
}
impl Vertex for PosVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        PosVertex::desc()
    }
}
impl Vertex for TextVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        TextVertex::desc()
    }
}

pub struct VertexBuffer<T: Vertex> {
    buffer: wgpu::Buffer,
    num_vertices: u32,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn new(resources: &GpuCommonResources, vertices: &[T], label: Option<&str>) -> Self {
        let buffer = resources
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let num_vertices = vertices.len() as u32;
        Self {
            buffer,
            num_vertices,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn vertex_source(&self) -> VertexSource<T> {
        VertexSource::VertexBuffer {
            vertex_buffer: &self.buffer,
            vertices: 0..self.num_vertices,
            instances: 0..1,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    num_indices: u32,
}

impl IndexBuffer {
    pub fn new(resources: &GpuCommonResources, indices: &[u16], label: Option<&str>) -> Self {
        let buffer = resources
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let num_indices = indices.len() as u32;
        Self {
            buffer,
            num_indices,
        }
    }

    pub fn num_indices(&self) -> u32 {
        self.num_indices
    }
}

pub struct SpriteVertexBuffer {
    vertex_buffer: VertexBuffer<PosColTexVertex>,
    index_buffer: IndexBuffer,
}

impl SpriteVertexBuffer {
    pub fn new(
        resources: &GpuCommonResources,
        (l, t, r, b): (f32, f32, f32, f32),
        color: Vector4<f32>,
    ) -> Self {
        let vertices = [
            // 0
            PosColTexVertex {
                position: Vector3::new(l, b, 0.0),
                color,
                texture_coordinate: Vector2::new(0.0, 0.0),
            },
            // 1
            PosColTexVertex {
                position: Vector3::new(l, t, 0.0),
                color,
                texture_coordinate: Vector2::new(0.0, 1.0),
            },
            // 2
            PosColTexVertex {
                position: Vector3::new(r, b, 0.0),
                color,
                texture_coordinate: Vector2::new(1.0, 0.0),
            },
            // 3
            PosColTexVertex {
                position: Vector3::new(r, t, 0.0),
                color,
                texture_coordinate: Vector2::new(1.0, 1.0),
            },
        ];

        let indices = [0, 1, 2, 2, 1, 3];

        Self {
            vertex_buffer: VertexBuffer::new(
                resources,
                &vertices,
                Some(&format!("SpriteVertexBuffer({}, {}, {}, {})", l, t, r, b)),
            ),
            index_buffer: IndexBuffer::new(
                resources,
                &indices,
                Some("SpriteVertexBuffer.index_buffer"),
            ),
        }
    }

    pub fn new_fullscreen(resources: &GpuCommonResources) -> Self {
        let w = render::VIRTUAL_WIDTH as f32 / 2.0;
        let h = render::VIRTUAL_HEIGHT as f32 / 2.0;

        Self::new(resources, (-w, -h, w, h), Vector4::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn vertex_source(&self) -> VertexSource<PosColTexVertex> {
        self.vertex_buffer
            .vertex_source()
            .with_index_buffer(&self.index_buffer.buffer, 0..self.index_buffer.num_indices)
    }
}
