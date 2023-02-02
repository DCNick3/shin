use glam::{vec3, Mat4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraParams {
    pub projection_matrix: Mat4,
}

pub const VIRTUAL_WIDTH: f32 = 1920.0;
pub const VIRTUAL_HEIGHT: f32 = 1080.0;

pub struct Camera {
    /// Projection matrix to draw onto the screen
    screen_projection_matrix: Mat4,
    /// Projection matrix to draw onto the render buffer
    projection_matrix: Mat4,
    render_buffer_size: (u32, u32),
}

impl Camera {
    pub fn new(window_size: (u32, u32)) -> Self {
        let (window_width, window_height) = window_size;

        let w = window_width as f32 / VIRTUAL_WIDTH;
        let h = window_height as f32 / VIRTUAL_HEIGHT;

        let scale = w.min(h);

        let (viewport_width, viewport_height) = if w < h {
            (VIRTUAL_WIDTH, VIRTUAL_HEIGHT * h / w)
        } else {
            (VIRTUAL_WIDTH * w / h, VIRTUAL_HEIGHT)
        };

        // It seems that we are basically one traslation away from matching the game output
        // TODO: figure out a proper way to move the coordinate space of smth
        // because this creates a strip of black pixels on the right and bottom
        let translation = Mat4::from_translation(vec3(-1.0, -1.0, 0.0));

        let mut screen_projection = Mat4::IDENTITY;
        screen_projection.x_axis.x = 2.0 / viewport_width;
        screen_projection.y_axis.y = -2.0 / viewport_height;
        screen_projection.z_axis.z = 1.0 / 1000.0;
        screen_projection.w_axis.w = 1.0;
        let screen_projection = screen_projection * translation;

        let mut projection = Mat4::IDENTITY;
        projection.x_axis.x = 2.0 / VIRTUAL_WIDTH;
        projection.y_axis.y = -2.0 / VIRTUAL_HEIGHT; // in wgpu y is up, so we need to flip the y axis
        projection.z_axis.z = 1.0 / 1000.0;
        projection.w_axis.w = 1.0;

        let render_buffer_size = (
            (VIRTUAL_WIDTH * scale) as u32,
            (VIRTUAL_HEIGHT * scale) as u32,
        );

        Self {
            screen_projection_matrix: screen_projection,
            projection_matrix: projection,
            render_buffer_size,
        }
    }

    // pub fn new(
    //     // device: &wgpu::Device,
    //     // bind_group_layouts: &BindGroupLayouts,
    //     window_size: (u32, u32),
    // ) -> Self {
    //     let (render_buffer_size, projection_matrix) = Self::compute_camera(window_size);
    //
    //     // let camera_params = CameraParams { projection_matrix };
    //     //
    //     // let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //     //     label: Some("camera_buffer"),
    //     //     contents: bytemuck::cast_slice(&[camera_params]),
    //     //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    //     // });
    //     //
    //     // let bind_group = CameraBindGroup(device.create_bind_group(&wgpu::BindGroupDescriptor {
    //     //     label: Some("camera_bind_group"),
    //     //     layout: &bind_group_layouts.camera,
    //     //     entries: &[wgpu::BindGroupEntry {
    //     //         binding: 0,
    //     //         resource: buffer.as_entire_binding(),
    //     //     }],
    //     // }));
    //
    //     // Self {
    //     //     projection_matrix,
    //     //     buffer,
    //     //     bind_group: Arc::new(bind_group),
    //     //     render_buffer_size,
    //     // }
    // }

    pub fn resize(
        &mut self,
        //_device: &wgpu::Device, queue: &mut wgpu::Queue,
        size: (u32, u32),
    ) {
        *self = Self::new(size);
        // (self.render_buffer_size, self.projection_matrix) = Self::compute_camera(size);
        // let mtx = [self.projection_matrix];
        // let contents = bytemuck::cast_slice(&mtx);
        // queue.write_buffer(&self.buffer, 0, contents);
    }

    // pub fn bind_group(&self) -> &Arc<CameraBindGroup> {
    //     &self.bind_group
    // }

    pub fn render_buffer_size(&self) -> (u32, u32) {
        self.render_buffer_size
    }

    pub fn projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }

    pub fn screen_projection_matrix(&self) -> Mat4 {
        self.screen_projection_matrix
    }
}
