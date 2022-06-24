// some uses aren't used when running headless
//#![allow(unused_imports)]
#![allow(dead_code)]
use bevy::{
    asset::HandleId,
    core_pipeline::{
        draw_2d_graph,
        //draw_3d_graph, AlphaMask3d, Opaque3d, Transparent3d,
        node,
        RenderTargetClearColors,
        Transparent2d,
    },
    prelude::*,
    render::{
        camera::{ActiveCamera, Camera, CameraTypePlugin, RenderTarget},
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotValue},
        render_phase::RenderPhase,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderContext,
        view::RenderLayers,
        RenderApp, RenderStage,
    },
    utils::HashMap,
};

//use bevy::core::{FixedTimestep, FixedTimesteps, Time};
use bevy_prototype_lyon::prelude::ShapePlugin;

mod systems;

//const LABEL: &str = "my_fixed_timestep";

//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

#[derive(Component, Default)]
pub struct FirstPassCamera;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

// The name of the final node of the first pass.
pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

#[derive(Default, Copy, Clone, Debug)]
pub struct StartColor {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl std::fmt::Display for StartColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.hue, self.saturation, self.lightness)
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct RenderToTextureDescriptor {
    name: &'static str,
    size: u32,
    start_color: StartColor,
    background_color: Color,
    //    scene_startup_system_set: SystemSet,
    //    scene_system_set: SystemSet,
}

#[derive(Component, Clone, Copy, Debug)]
struct TextureListItem {
    name: &'static str,
    texture_handle_id: HandleId,
}

#[derive(Component)]
struct RenderToTextureDescriptorList {
    list: Vec<RenderToTextureDescriptor>,
}

impl RenderToTextureDescriptorList {
    pub fn new() -> RenderToTextureDescriptorList {
        let list = Vec::new();
        RenderToTextureDescriptorList { list }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RenderingState {
    Normal,
    GeneratingTextures,
}

const RED_MONSTER_START_COLOR: StartColor = StartColor {
    hue: 0.1,
    saturation: 0.8,
    lightness: 0.7,
};
const GREEN_MONSTER_START_COLOR: StartColor = StartColor {
    hue: 0.4,
    saturation: 0.8,
    lightness: 0.6,
};

const RED_MONSTER_DESCRIPTOR: RenderToTextureDescriptor = RenderToTextureDescriptor {
    name: "red_256",
    size: 256,
    start_color: RED_MONSTER_START_COLOR,
    background_color: Color::MAROON,
    // scene name to load
};

const GREEN_MONSTER_DESCRIPTOR: RenderToTextureDescriptor = RenderToTextureDescriptor {
    name: "green_512",
    size: 512,
    start_color: GREEN_MONSTER_START_COLOR,
    background_color: Color::LIME_GREEN,
    // scene name to load
};

struct GameTextureMap {
    map: HashMap<String, HandleId>,
}

impl FromWorld for GameTextureMap {
    fn from_world(_world: &mut World) -> Self {
        let m = HashMap::<String, HandleId>::new();

        GameTextureMap { map: m }
    }
}

fn main() {
    let mut descriptor_list = RenderToTextureDescriptorList::new();
    descriptor_list.list.push(RED_MONSTER_DESCRIPTOR);
    descriptor_list.list.push(GREEN_MONSTER_DESCRIPTOR);

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .init_resource::<GameTextureMap>()
        .add_plugin(CameraTypePlugin::<FirstPassCamera>::default())
        .insert_resource(WindowDescriptor {
            title: "My Gamename!".to_string(),
            width: 1280.,
            height: 1024.,
            ..default()
        })
        // if running headless, add this resource
        // .insert_resource(WgpuSettings { backends: None, ..default()} )
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(ShapePlugin);

    let render_app = app.sub_app_mut(RenderApp);

    let driver = FirstPassCameraDriver::new(&mut render_app.world);

    // This will add render phases for the new camera.
    render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);

    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    // Add a node for the first pass.
    graph.add_node(FIRST_PASS_DRIVER, driver);

    // The first pass's dependencies include those of the main pass.
    graph
        .add_node_edge(node::MAIN_PASS_DEPENDENCIES, FIRST_PASS_DRIVER)
        .unwrap();

    // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
    graph
        .add_node_edge(node::CLEAR_PASS_DRIVER, FIRST_PASS_DRIVER)
        .unwrap();
    graph
        .add_node_edge(FIRST_PASS_DRIVER, node::MAIN_PASS_DRIVER)
        .unwrap();

    app.add_system(get_next_descriptor.before(create_game_texture))
        .add_system(
            //
            create_game_texture
                .after(get_next_descriptor)
                .before(systems::circles::add_circles_to_render_layer_1),
        )
        .add_system(
            systems::circles::add_circles_to_render_layer_1
                .after(create_game_texture)
                .before(save_texture_to_list),
        )
        .add_system(
            save_texture_to_list
                .after(create_game_texture)
                .before(check_if_completed_textures),
        )
        .add_system(check_if_completed_textures.after(save_texture_to_list))
        .insert_resource(descriptor_list);

    app.add_system(draw_textured_rect_setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        /*
        .add_system(frame_update)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.0).with_label(LABEL))
                .with_system(fixed_update)
                .with_system(systems::circles::rotate_colors),
        )
        */
        .run();
}

/*
fn frame_update(mut last_time: Local<f64>, time: Res<Time>) {
    // time.seconds_since_startup() - *last_time
    *last_time = time.seconds_since_startup();
}

fn fixed_update(mut last_time: Local<f64>, time: Res<Time>, fixed_timesteps: Res<FixedTimesteps>) {
    // time.seconds_since_startup() - *last_time

    let _ = fixed_timesteps.get(LABEL).unwrap();
    // overstep_percentage = fixed_timestep.overstep_percentage();

    *last_time = time.seconds_since_startup();
}
*/

// Add render phases for FIRST_PASS_CAMERA.
fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<FirstPassCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Transparent2d>::default(),
            //            RenderPhase::<Opaque3d>::default(),
            //            RenderPhase::<AlphaMask3d>::default(),
            //            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

// A node for the first pass camera that runs draw_2d_graph/*draw_3d_graph*/ with this camera.
struct FirstPassCameraDriver {
    query: QueryState<Entity, With<FirstPassCamera>>,
}

impl FirstPassCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl Node for FirstPassCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(
                //draw_3d_graph::NAME,
                draw_2d_graph::NAME,
                vec![SlotValue::Entity(camera)],
            )?;
        }
        Ok(())
    }
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// We only want to write to the texture when we want to, not necessarily every frame
// Our texture's entities are set to only be rendered on RenderLayer(1).  So we don't *have* to
// delete them, just set that RenderLayer invisible.
// BUT, there are only 31 such layers, so we can only create 31 textures in this simple manner.
// And, we don't want to do any work on those entities when they're invisible.
// And it'd be nice to free up the memory those entities are using, and avoid any overhead
// their existence causes.  So they ought to get deleted after the last generated texture is drawn.
// Maybe use iyes_loopless states to control when.

// That gives us 32 textures (and as many Materials) we can create procedurally.
// And they can be updated in real time as the app is running.

// create_game_texture
// takes RenderToTextureDescriptor and uses its info to add Image and TextureListItem resource to world
fn create_game_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut render_target_clear_colors: ResMut<RenderTargetClearColors>,
    texture_descriptor_res_opt: Option<Res<RenderToTextureDescriptor>>,
) {
    if let Some(texture_descriptor_res) = texture_descriptor_res_opt {
        let texture_descriptor = *texture_descriptor_res;
        let size = Extent3d {
            width: texture_descriptor.size,
            height: texture_descriptor.size,
            ..default()
        };

        commands.remove_resource::<RenderToTextureDescriptor>();

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);
        let image_handle = images.add(image);
        let image_handle_id = image_handle.id;

        // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
        let first_pass_layer = RenderLayers::layer(1);

        // Now add stuff to the scene we're going to render to that texture
        // The key is to .insert(first_pass_layer) onto Meshes that we're adding

        //      We're doing this in insert_scene()

        // Done adding entities to that scene;
        // add light and camera now.

        // Light
        // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
        commands.spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..default()
        });

        // The Camera will have a RenderTarget of a clone of the image_handle created before.
        // So once the scene renders, the texture will be all set to be used.
        let render_target = RenderTarget::Image(image_handle);

        // Note that we clear the background for the texture's destination by adding
        // a color to its clear_colors.
        render_target_clear_colors
            .insert(render_target.clone(), texture_descriptor.background_color);

        // First pass camera
        commands
            .spawn_bundle({
                let mut cam = OrthographicCameraBundle::new_2d();

                cam.camera = Camera {
                    target: render_target,
                    ..default()
                };
                /*PerspectiveCameraBundle::<FirstPassCamera> {
                camera: Camera {
                    target: render_target,
                    ..default()
                },
                //transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                //    .looking_at(Vec3::default(), Vec3::Y),
                //..PerspectiveCameraBundle::new()
                */
                //                ..OrthographicCameraBundle::new_2d()
                cam
            })
            // This .insert(first_pass_layer) for the camera bundle is key
            .insert(first_pass_layer);
        // NOTE: omitting the RenderLayers component for this camera may cause a validation error:
        //
        // thread 'main' panicked at 'wgpu error: Validation Error
        //
        //    Caused by:
        //        In a RenderPass
        //          note: encoder = `<CommandBuffer-(0, 1, Metal)>`
        //        In a pass parameter
        //          note: command buffer = `<CommandBuffer-(0, 1, Metal)>`
        //        Attempted to use texture (5, 1, Metal) mips 0..1 layers 0..1 as a combination of COLOR_TARGET within a usage scope.
        //
        // This happens because the texture would be written and read in the same frame, which is not allowed.
        // So either render layers must be used to avoid this, or the texture must be double buffered.

        // now have Handle<Image>, which is my about-to-be-rendered-to texture
        // save its HandleId in a TextureListItem resource
        commands.insert_resource(TextureListItem {
            name: texture_descriptor.name,
            texture_handle_id: image_handle_id,
        });
    }
}

fn save_texture_to_list(
    mut commands: Commands,
    texture_list_item_res_opt: Option<Res<TextureListItem>>,
    mut texture_map: ResMut<GameTextureMap>,
) {
    if let Some(texture_list_item_res) = texture_list_item_res_opt {
        let texture_list_item = *texture_list_item_res;
        texture_map.map.insert(
            texture_list_item.name.to_string(),
            texture_list_item.texture_handle_id,
        );
        // now can remove the TextureListItem resource to provoke obtaining the subsequent one
        commands.remove_resource::<TextureListItem>();
    }
}

fn has_texture_list_item(query: Query<Option<&TextureListItem>>) -> bool {
    for i in query.iter() {
        if i.is_some() {
            return true;
        }
    }
    false
}

#[derive(Default)]
struct CompletedCreatingRenderTargetTextures {
    times_through: u8,
    did_it: bool,
}

fn check_if_completed_textures(
    texture_list_item_res_opt: Option<Res<TextureListItem>>,
    descriptor_list: Res<RenderToTextureDescriptorList>,
    mut did_it: Local<CompletedCreatingRenderTargetTextures>,
) {
    if did_it.did_it {
        return;
    }
    if descriptor_list.list.is_empty() {
        if let Some(texture_list_item_res) = texture_list_item_res_opt {
            let _texture_list_item = *texture_list_item_res;
            did_it.times_through = 0;
        } else {
            did_it.times_through += 1;
            if did_it.times_through > 3 {
                did_it.did_it = true;
            }
        }
    }
}

#[derive(Default)]
struct CompletedDescriptorList {
    did_it: bool,
}

fn get_next_descriptor(
    mut commands: Commands,
    descriptor_res_opt: Option<Res<RenderToTextureDescriptor>>,
    mut descriptor_list: ResMut<RenderToTextureDescriptorList>,
    mut did_it: Local<CompletedDescriptorList>,
) {
    if did_it.did_it {
        return;
    }
    if !descriptor_list.list.is_empty() {
        // only want to insert_resource if there isn't one in the World
        if let Some(_descriptor_res) = descriptor_res_opt {
        } else {
            let descriptor_option = descriptor_list.list.pop();
            match descriptor_option {
                Some(descriptor) => {
                    commands.insert_resource(descriptor.start_color);
                    commands.insert_resource(descriptor);
                    if descriptor_list.list.is_empty() {
                        did_it.did_it = true;
                    }
                }
                None => {}
            }
        }
    }
}

#[derive(Default)]
struct UsedTexture {
    did_it: bool,
}

fn draw_textured_rect_setup(
    mut commands: Commands,
    texture_map: Res<GameTextureMap>,
    mut did_it: Local<UsedTexture>,
) {
    if did_it.did_it {
        return;
    }
    if let Some(texture_handle_id) = texture_map.map.get("green_512") {
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
        commands.spawn_bundle(SpriteBundle {
            texture: Handle::weak(*texture_handle_id),
            ..default()
        });
        did_it.did_it = true;
    }
}
