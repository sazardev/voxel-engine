//! The view of the world. For now a fixed overview of the streamed area; the first and third
//! person rig with a view switch (FR-16) replaces it.

use bevy::prelude::*;

use crate::world::StreamFocus;

/// Where the overview camera sits relative to the stream focus, in metres.
const OVERVIEW_OFFSET: Vec3 = Vec3::new(-150.0, 170.0, 150.0);
/// The height it looks at above the focus: roughly the middle of the terrain.
const LOOK_HEIGHT: f32 = 50.0;

/// Marks the camera the player sees the world through.
#[derive(Debug, Component)]
pub struct MainCamera;

#[derive(Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StreamFocus>()
            .add_systems(Startup, spawn_overview_camera);
    }
}

fn spawn_overview_camera(mut commands: Commands, focus: Res<StreamFocus>) {
    let target = focus.0 + Vec3::Y * LOOK_HEIGHT;
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(focus.0 + OVERVIEW_OFFSET).looking_at(target, Vec3::Y),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_camera_looks_down_at_the_stream_focus() {
        let mut app = App::new();
        app.add_plugins(CameraPlugin);

        app.update();

        let mut cameras = app.world_mut().query::<(&Transform, &MainCamera)>();
        let (transform, _) = cameras.single(app.world()).expect("one main camera");
        let toward_focus = (Vec3::Y * LOOK_HEIGHT - transform.translation).normalize();
        assert!(transform.forward().dot(toward_focus) > 0.999);
        assert!(transform.forward().y < 0.0);
    }
}
