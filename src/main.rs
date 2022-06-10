use bevy::app::App;
use bevy::app::CoreStage;

use bevy::DefaultPlugins;

use bevy::ecs::schedule::StageLabel;
use bevy::ecs::schedule::SystemStage;

use bevy::ecs::system::Local;
use bevy::ecs::system::Res;

use bevy::core::Time;
use bevy::core::FixedTimestep;
use bevy::core::FixedTimesteps;

use bevy::core_pipeline::ClearColor;

use bevy::render::view::Msaa;
use bevy::render::color::Color;

use bevy_prototype_lyon::prelude::ShapePlugin;

mod systems;

const LABEL: &str = "my_fixed_timestep";

//use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};


#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

pub const SCREEN_WIDTH:u32 = 1024;
pub const SCREEN_HEIGHT:u32 = 1024;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        //.add_plugin(systems::screenshot::CapturePlugin)
        .add_startup_system(systems::circles::setup_shape_rendering)
        //.add_startup_system(systems::screenshot::setup_capture)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(frame_update)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(
                    FixedTimestep::step(1.0 / 60.0)
                        .with_label(LABEL),
                )
                .with_system(fixed_update)
                .with_system(systems::circles::rotate_colors)
        )
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
//        .add_plugin(LogDiagnosticsPlugin::default())
//        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}


fn frame_update(mut last_time: Local<f64>, time: Res<Time>)
{
    // time.seconds_since_startup() - *last_time
    *last_time = time.seconds_since_startup();
}

fn fixed_update(mut last_time: Local<f64>, time: Res<Time>, fixed_timesteps: Res<FixedTimesteps>)
{
    // time.seconds_since_startup() - *last_time

    let _ = fixed_timesteps.get(LABEL).unwrap();
    // overstep_percentage = fixed_timestep.overstep_percentage();

    *last_time = time.seconds_since_startup();
}
