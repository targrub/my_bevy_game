use bevy::{
    asset::HandleId,
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    utils::HashMap,
};

#[derive(Component, Default)]
pub struct RenderToTexturePass;

use super::circles::Circles1;
use super::circles::Circles2;

#[derive(Default)]
pub struct AddDynamicTextureEvent {
    pub description: Option<RenderToTextureDescriptor>,
}

pub struct DynamicTexturesPlugin;

impl Plugin for DynamicTexturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DynamicTextures>()
            .add_event::<AddDynamicTextureEvent>()
            .add_system(add_dynamic_texture_event_handler)
            .add_system(crate::systems::circles::circles1_add_circles_to_layer)
            .add_system(crate::systems::circles::circles2_add_circles_to_layer)
            .add_system(crate::systems::circles::circles1_update_colors)
            .add_system(crate::systems::circles::circles2_update);
    }
}

fn add_dynamic_texture_event_handler(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut events: EventReader<AddDynamicTextureEvent>,
    mut dyntex: ResMut<DynamicTextures>,
) {
    for e in events.iter() {
        if let Some(desc) = e.description {
            if let Some(layer) = dyntex.get_available_render_layer() {
                let handle_id = set_up_dynamic_texture(&mut commands, &mut images, &desc, layer);
                dyntex.add_dynamic_texture(&desc, layer, Handle::weak(handle_id));
                match desc.functype {
                    "Circles1" => {
                        commands.spawn().insert(Circles1::new(layer, &desc));
                    }
                    "Circles2" => {
                        commands.spawn().insert(Circles2::new(layer, &desc));
                    }
                    _ => {
                        unimplemented!("{}", desc.functype);
                    }
                }
            } else {
                // Ran out of render layers
            }
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct StartColor {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl std::fmt::Display for StartColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.hue, self.saturation, self.lightness)
    }
}

#[derive(Component, Clone, Copy)]
pub struct RenderToTextureDescriptor {
    pub name: &'static str,
    pub functype: &'static str,
    pub size: u32,
    pub start_color: StartColor,
    pub background_color: Color,
}

#[derive(Default)]
pub struct DynamicTextures {
    list: Vec<(u8, (Handle<Image>, RenderToTextureDescriptor))>,
    map: HashMap<String, (Handle<Image>, u8)>,
    highest_render_layer: u8,
}

impl DynamicTextures {
    pub fn get_texture_handle(&self, name: &str) -> Option<&(Handle<Image>, u8)> {
        self.map.get(name)
    }

    fn add_dynamic_texture(
        &mut self,
        descriptor: &RenderToTextureDescriptor,
        layer: u8,
        h: Handle<Image>,
    ) {
        self.list.push((layer, (Handle::weak(h.id), *descriptor)));
        self.map
            .insert(descriptor.name.to_string(), (Handle::weak(h.id), layer));
    }

    fn get_available_render_layer(&mut self) -> Option<u8> {
        if self.highest_render_layer <= 32 {
            self.highest_render_layer += 1;
            Some(self.highest_render_layer)
        } else {
            None
        }
    }
}

// takes RenderToTextureDescriptor and uses its info to add camera and image it will render to to a render layer of its own
fn set_up_dynamic_texture(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    texture_descriptor: &RenderToTextureDescriptor,
    layer: u8,
) -> HandleId {
    let size = Extent3d {
        width: texture_descriptor.size,
        height: texture_descriptor.size,
        ..default()
    };

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

    let first_pass_layer = RenderLayers::layer(layer);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    let render_target = RenderTarget::Image(image_handle);

    // First pass camera
    commands
        .spawn_bundle(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(texture_descriptor.background_color),
            },
            camera: Camera {
                priority: -(layer as isize),
                target: render_target,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..default()
        })
        .insert(RenderToTexturePass)
        .insert(first_pass_layer);

    image_handle_id
}
