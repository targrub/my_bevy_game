/*
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::app::App;
use bevy::app::Plugin;
use bevy::asset::Assets;
use bevy::asset::HandleUntyped;
use bevy::ecs::component::Component;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::reflect::TypeUuid;
use bevy::render::camera::Camera;
use bevy::render::camera::OrthographicProjection;
use bevy::render::camera::{
    CameraProjection, DepthCalculation, RenderTarget,
};
use bevy::render::primitives::Frustum;
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, Extent3d,
    MapMode, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderDevice};
use bevy::render::texture::Image;
use bevy::transform::components::Transform;
use bevy::utils::default;

#[derive(Component, Default)]
pub struct CaptureCamera;

pub const CAPTURE_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 13373934772014884929);

// The name of the final node of the first pass.
pub const CAPTURE_DRIVER: &str = "capture_driver";

pub fn setup_capture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<ClearColorConfig>,
    render_device: Res<RenderDevice>,
) {
    let texture_size = 256;
    let size = Extent3d {
        width: texture_size,
        height: texture_size,
        ..Default::default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..Default::default()
    };
    image.resize(size);

    let image_handle = images.set(CAPTURE_IMAGE_HANDLE, image);

    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(256) * 4;

    let size = padded_bytes_per_row as u64 * 256u64;

    let output_cpu_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let far = 1000.0;
    let orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        ..Default::default()
    };
    let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
    let view_projection =
        orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &transform.translation,
        &transform.back(),
        orthographic_projection.far(),
    );
    let render_target = RenderTarget::Image(image_handle);
 //   clear_colors.insert(render_target.clone(), Color::rgba(0.0, 0.0, 0.0, 0.0)); //Color::BLACK);
    commands
        .spawn_bundle(Camera2dBundle {
            camera: Camera {
                target: RenderTarget::Image(image_handle),
                ..default()
            },
                ..default()
        })
        .insert(Capture {
            buf: output_cpu_buffer,
        });
}


pub fn save_img(cap: Query<&Capture>, render_device: Res<RenderDevice>) {
    if let Some(cap) = cap.iter().next() {
        let large_buffer_slice = cap.buf.slice(..);
        render_device.map_buffer(&large_buffer_slice, MapMode::Read, { if let Err(buffer_async_error) =  {
        } {
        // can't complete gfx op successfully, so do nothing
        } else {
            // gfx mapping successful; save image
            let large_padded_buffer = large_buffer_slice.get_mapped_range();

            image::save_buffer(
                "test.png",
                &large_padded_buffer,
                256,
                256,
                image::ColorType::Rgba8,
            );
        }
    );
        cap.buf.unmap();
    }
}

#[derive(Component)]
pub struct Capture {
    pub buf: Buffer,
}

pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app
       // .add_plugin(CameraTypePlugin::<CaptureCamera>::default())
            .add_startup_system(setup_capture)
            .add_system(save_img);
    }
}
*/
