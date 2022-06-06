use rand::Rng;
use bevy::app::App;
use bevy::app::Plugin;
use bevy::math::Quat;
use bevy::math::Vec2;
use bevy::math::Vec3;
use bevy::render::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::asset::Assets;
use bevy::asset::HandleUntyped;
use bevy::render::texture::Image;
use bevy::render::camera::Camera;
use bevy::render::camera::OrthographicCameraBundle;
use bevy::render::camera::OrthographicProjection;
use bevy::render::view::Msaa;
use bevy::transform::components::Transform;
use bevy::utils::default;
use bevy::DefaultPlugins;
use bevy::core_pipeline::ClearColor;

use bevy_prototype_lyon::prelude::*;

use bevy::core_pipeline::{
    draw_2d_graph, node, AlphaMask3d, Opaque3d, RenderTargetClearColors, Transparent2d,
};
use bevy::reflect::TypeUuid;
use bevy::render::camera::{ActiveCamera, CameraProjection, CameraTypePlugin, DepthCalculation, RenderTarget};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_phase::RenderPhase;
use bevy::render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer,
    ImageDataLayout, MapMode, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::{RenderApp, RenderStage};
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;
//use bevy_prototype_lyon::prelude::tess::geom::Translation;

use palette::{FromColor, Hsl/*, Srgb */};

const SCREEN_WIDTH:u32 = 1024;
const SCREEN_HEIGHT:u32 = 1024;

const TEXTURE_WIDTH:u32 = 1024;
const TEXTURE_HEIGHT:u32 = 1024;

const MIN_RADIUS:f32 = 4.0;
const MAX_CIRCLES_PER_RADIUS:u32 = 100;

#[derive(Component, Default)]
pub struct CaptureCamera;

pub const CAPTURE_IMAGE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 13373934772014884929);

// The name of the final node of the first pass.
pub const CAPTURE_DRIVER: &str = "capture_driver";

pub fn setup_capture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    render_device: Res<RenderDevice>,
) {
    let size = Extent3d {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
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

    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(TEXTURE_WIDTH.try_into().unwrap()) * 4;

    let size = padded_bytes_per_row as u64 * TEXTURE_HEIGHT as u64;

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
    clear_colors.insert(render_target.clone(), Color::rgba(0.0,0.0,0.0,0.0));//Color::BLACK);
    commands
        .spawn_bundle(OrthographicCameraBundle {
            camera: Camera {
                target: render_target,
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            marker: CaptureCamera,
        })
        .insert(Capture {
            buf: output_cpu_buffer,
        });
}

// Add 3D render phases for CAPTURE_CAMERA.
pub fn extract_camera_phases(
    mut commands: Commands,
    cap: Query<&Capture>,
    active: Res<ActiveCamera<CaptureCamera>>,
) {
    if let Some(entity) = active.get() {
        if let Some(cap) = cap.iter().next() {
            commands
                .get_or_spawn(entity)
                .insert_bundle((
                    RenderPhase::<Opaque3d>::default(),
                    RenderPhase::<AlphaMask3d>::default(),
                    RenderPhase::<Transparent2d>::default(),
                ))
                .insert(Capture {
                    buf: cap.buf.clone(),
                });
        }
    }
}

// A node for the first pass camera that runs draw_3d_graph with this camera.
pub struct CaptureCameraDriver {
    pub buf: Option<Buffer>,
}

impl bevy::render::render_graph::Node for CaptureCameraDriver {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();

        if let Some(camera_3d) = world.resource::<ActiveCamera<CaptureCamera>>().get() {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera_3d)])?;

            let gpu_image = gpu_images.get(&CAPTURE_IMAGE_HANDLE.typed()).unwrap();
            let mut encoder = render_context
                .render_device
                .create_command_encoder(&CommandEncoderDescriptor::default());
            let padded_bytes_per_row =
                RenderDevice::align_copy_bytes_per_row((gpu_image.size.width) as usize) * 4;

            let texture_extent = Extent3d {
                width: gpu_image.size.width as u32,
                height: gpu_image.size.height as u32,
                depth_or_array_layers: 1,
            };

            if let Some(buf) = &self.buf {
                encoder.copy_texture_to_buffer(
                    gpu_image.texture.as_image_copy(),
                    ImageCopyBuffer {
                        buffer: buf,
                        layout: ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                std::num::NonZeroU32::new(padded_bytes_per_row as u32).unwrap(),
                            ),
                            rows_per_image: None,
                        },
                    },
                    texture_extent,
                );
                let render_queue = world.get_resource::<RenderQueue>().unwrap();
                render_queue.submit(std::iter::once(encoder.finish()));
            }
        }

        Ok(())
    }
    fn update(&mut self, world: &mut World) {
        for cap in world.query::<&mut Capture>().iter_mut(world) {
            self.buf = Some(cap.buf.clone());
        }
    }
}

pub fn save_img(cap: Query<&Capture>, render_device: Res<RenderDevice>) {
    if let Some(cap) = cap.iter().next() {
        let large_buffer_slice = cap.buf.slice(..);
        render_device.map_buffer(&large_buffer_slice, MapMode::Read);
        {
            let large_padded_buffer = large_buffer_slice.get_mapped_range();

            image::save_buffer(
                "test.png",
                &large_padded_buffer,
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
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
        app.add_plugin(CameraTypePlugin::<CaptureCamera>::default())
            .add_startup_system(setup_capture)
            .add_system(save_img);

        let render_app = app.sub_app_mut(RenderApp);

        // This will add 3D render phases for the capture camera.
        render_app.add_system_to_stage(RenderStage::Extract, extract_camera_phases);

        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        // Add a node for the capture.
        graph.add_node(CAPTURE_DRIVER, CaptureCameraDriver { buf: None });

        // The capture's dependencies include those of the main pass.
        graph
            .add_node_edge(node::MAIN_PASS_DEPENDENCIES, CAPTURE_DRIVER)
            .unwrap();

        // Insert the capture node: CLEAR_PASS_DRIVER -> CAPTURE_DRIVER -> MAIN_PASS_DRIVER
        graph
            .add_node_edge(node::CLEAR_PASS_DRIVER, CAPTURE_DRIVER)
            .unwrap();
        graph
            .add_node_edge(CAPTURE_DRIVER, node::MAIN_PASS_DRIVER)
            .unwrap();
    }
}


fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(CapturePlugin)
        .add_startup_system(setup_shape_rendering)
        .add_startup_system(setup_capture)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Debug)]
struct MyCircle {
    pos: Vec2,
    r: f32,
    c: Color
}

fn intersects_any(c:&MyCircle, cv:&Vec<MyCircle>) -> bool {
    for tc in cv {
        let distsq: f32 = (c.pos.x - tc.pos.x) * (c.pos.x - tc.pos.x) + (c.pos.y - tc.pos.y) * (c.pos.y - tc.pos.y);
        let radsumsq:f32 = (c.r + tc.r) * (c.r + tc.r);
        if (radsumsq + 50.0) > distsq {
            return true
        }
    }
    false
}

fn rand_circle_color(circle_hsl: &mut Hsl, rng:  & mut rand::prelude::ThreadRng) -> Color {
    //let clamped_hue: f32 = num::clamp(circle_hsl.hue.to_degrees() + rng.gen_range(-30.0..30.0), 0.0, 360.0);
    let clamped_hue = rng.gen_range(0.0..30.0);
    circle_hsl.hue = palette::RgbHue::from_degrees(clamped_hue);
    let c_srgb = palette::Srgb::from_color(*circle_hsl);
    Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
}

fn rand_circle_color_variation(circle_hsl: &mut Hsl, rng: &mut rand::prelude::ThreadRng) -> Color {
    circle_hsl.saturation = num::clamp(circle_hsl.saturation + rng.gen_range(-0.01..0.01), 0.0, 1.0);
    circle_hsl.lightness = num::clamp(circle_hsl.lightness + rng.gen_range(-0.05..0.05), 0.3, 0.9);
    let c_srgb = palette::Srgb::from_color(*circle_hsl);
    Color::rgba(c_srgb.red, c_srgb.green, c_srgb.blue, 1.0)
}

fn setup_shape_rendering(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    let mut circs:Vec<MyCircle> = Vec::new();
    let mut r = 20.0;

    let mut rng = rand::thread_rng();

    let mut color_change_count = 0;
    let mut circle_hsl: Hsl = Hsl::new(0.1, 0.8, 0.7);
    let mut current_color = rand_circle_color(&mut circle_hsl, &mut rng);

    let mut circles_of_this_radius: u32 = 0;

    loop {
        let mut success:bool = false;
        for _ in 1..=100 {  // take many chances to fit this circle in
            let npos: Vec2 = Vec2::new(
                rng.gen::<f32>() * (SCREEN_WIDTH as f32 - r * 2.0) + r - SCREEN_WIDTH as f32 / 2.0,
                rng.gen::<f32>() * (SCREEN_WIDTH as f32 - r * 2.0) + r - SCREEN_WIDTH as f32 / 2.0);
            let nc = MyCircle {
              pos : npos,
                r : r,
                c : current_color
            };
            if !intersects_any(&nc, &circs) {
                circs.push(nc);
                success = true;
                circles_of_this_radius += 1;
                break
            }
        }
        // if failure, decrease radius and loop if not <= min_radius
        if !success || circles_of_this_radius >= MAX_CIRCLES_PER_RADIUS {
            circles_of_this_radius = 0;
            r -= 1.0;
            if r <= MIN_RADIUS {
                break
            }
        } else {
            // if success, might change color's hue
            color_change_count += 1;
            if color_change_count >= 30 {    // every 30 circles, change color
                color_change_count = 0;
                current_color = rand_circle_color(&mut circle_hsl, &mut rng);
            } else {
                current_color = rand_circle_color_variation(&mut circle_hsl, &mut rng);
            }
        }
    }

    for c in circs.iter() {
        let circ = shapes::Circle {
            radius: c.r,
            center: Vec2::ZERO,
        };

        commands.spawn_bundle(GeometryBuilder::build_as(
            &circ,
            DrawMode::Fill(FillMode::color(c.c)),
            Transform {
                translation: Vec3::new(c.pos.x, c.pos.y, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        ));
    }
}
