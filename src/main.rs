#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::multiple_crate_versions,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
#![allow(dead_code, unused)]

use bevy::prelude::*;

mod systems;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use systems::dynamic_textures::{AddDynamicTextureEvent, RenderToTextureDescriptor, StartColor};
use systems::dynamic_textures::{DynamicTextures, DynamicTexturesPlugin};

#[derive(Component, Default)]
pub struct RenderToTexturePass;

//-----------------------

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
    functype: "Circles2",
    size: 256,
    start_color: RED_MONSTER_START_COLOR,
    background_color: Color::MAROON,
};

const GREEN_MONSTER_DESCRIPTOR: RenderToTextureDescriptor = RenderToTextureDescriptor {
    name: "green_512",
    functype: "Circles2",
    size: 512,
    start_color: GREEN_MONSTER_START_COLOR,
    background_color: Color::LIME_GREEN,
};

//------------------------------------------------------------

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            title: "My Gamename!".to_string(),
            width: 1280.,
            height: 1024.,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK));

    app.add_plugin(DynamicTexturesPlugin);

    app.add_system(draw_textured_rect_setup)
        .add_system(move_textured_rect);

    app.add_startup_system(add_game_camera);

    app.add_system(bevy::window::close_on_esc).run();
}

fn draw_textured_rect_setup(
    mut commands: Commands,
    dyntex: Res<DynamicTextures>,
    mut ew: EventWriter<AddDynamicTextureEvent>,
    mut added_textures: Local<bool>,
    mut added_sprite: Local<bool>,
) {
    if let Some(texture_handle) = dyntex.get_texture_handle("red_256") {
        if !*added_sprite {
            commands
                .spawn_bundle(SpriteBundle {
                    texture: Handle::weak(texture_handle.0.id),
                    ..default()
                })
                .insert(Direction::Up);
            *added_sprite = true;
        }
    } else if !*added_textures {
        ew.send(AddDynamicTextureEvent {
            description: Some(RED_MONSTER_DESCRIPTOR),
        });
        //        ew.send(AddDynamicTextureEvent {
        //            description: Some(GREEN_MONSTER_DESCRIPTOR),
        //        });
        *added_textures = true;
    }
}

fn move_textured_rect(
    time: Res<Time>,
    mut sprite_position: Query<(&mut Direction, &mut Transform)>,
) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 150. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        }

        if transform.translation.y > 200. {
            *logo = Direction::Down;
        } else if transform.translation.y < -200. {
            *logo = Direction::Up;
        }
    }
}
fn add_game_camera(mut commands: Commands) {
    // we have a handle that's been created, so we can draw with it
    commands.spawn_bundle(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Default,
        },
        camera: Camera { ..default() },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..default()
    });
}
