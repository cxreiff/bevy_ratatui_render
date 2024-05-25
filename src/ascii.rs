use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*, render::renderer::RenderDevice};

use crate::headless::{setup_render_target, ImageCopyPlugin, SceneController};

const TEMP_DIMENSIONS: (u32, u32) = (256, 256);

pub struct AsciiPlugin;

impl Plugin for AsciiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SceneController::new(TEMP_DIMENSIONS.0, TEMP_DIMENSIONS.1))
            .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
            .add_plugins(ImageCopyPlugin)
            .init_resource::<SceneController>()
            .add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<SceneController>,
    render_device: Res<RenderDevice>,
) {
    let render_target = setup_render_target(
        &mut commands,
        &mut images,
        &render_device,
        &mut scene_controller,
        2,
        "main_scene".into(),
    );
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 3., 3.0).looking_at(Vec3::ZERO, Vec3::Z),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: render_target,
            ..default()
        },
        ..default()
    });
}
