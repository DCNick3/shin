use bevy::prelude::*;
use bevy::render::camera::{
    CameraProjection, CameraRenderGraph, DepthCalculation, ScalingMode, WindowOrigin,
};
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub window_origin: WindowOrigin,
    pub scale: f32,
    pub depth_calculation: DepthCalculation,
    pub virtual_width: f32,
    pub virtual_height: f32,
}

impl CameraProjection for OrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left * self.scale,
            self.right * self.scale,
            self.bottom * self.scale,
            self.top * self.scale,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        let w = width / self.virtual_width;
        let h = height / self.virtual_height;

        let (viewport_width, viewport_height) = if w < h {
            (self.virtual_width, self.virtual_height * h / w)
        } else {
            (self.virtual_width * w / h, self.virtual_height)
        };

        trace!(?width, ?height, ?w, ?h, ?viewport_width, ?viewport_height);

        match self.window_origin {
            WindowOrigin::Center => {
                let half_width = viewport_width / 2.0;
                let half_height = viewport_height / 2.0;
                self.left = -half_width;
                self.bottom = -half_height;
                self.right = half_width;
                self.top = half_height;
            }
            WindowOrigin::BottomLeft => {
                self.left = 0.0;
                self.bottom = 0.0;
                self.right = viewport_width;
                self.top = viewport_height;
            }
        }
    }

    fn depth_calculation(&self) -> DepthCalculation {
        self.depth_calculation
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        OrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::Center,
            scale: 1.0,
            depth_calculation: DepthCalculation::Distance,
            virtual_width: 1920.0,
            virtual_height: 1080.0,
        }
    }
}

#[derive(Bundle)]
pub struct Camera2dBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: OrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_2d: Camera2d,
}

impl Camera2dBundle {
    /// Create an orthographic projection camera with a custom `Z` position.
    ///
    /// The camera is placed at `Z=far-0.1`, looking toward the world origin `(0,0,0)`.
    /// Its orthographic projection extends from `0.0` to `-far` in camera view space,
    /// corresponding to `Z=far-0.1` (closest to camera) to `Z=-0.1` (furthest away from
    /// camera) in world space.
    pub fn new_with_far(far: f32) -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let projection = OrthographicProjection {
            far,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        };
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection =
            projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            projection.far(),
        );
        Self {
            camera_render_graph: CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::NAME),
            projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            camera: Camera::default(),
            camera_2d: Camera2d::default(),
        }
    }
}

impl Default for Camera2dBundle {
    fn default() -> Self {
        Self::new_with_far(1000.0)
    }
}
