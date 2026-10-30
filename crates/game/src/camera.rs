//! The view of the world: one rig for first and third person (FR-16), placed from the player's
//! body and look every frame, interpolated between fixed steps.

mod cursor;
mod look;
mod rig;

use bevy::input::InputSystems;
use bevy::prelude::*;

pub use rig::{CameraRig, View, arm_length};

/// Marks the camera the player sees the world through.
#[derive(Debug, Component)]
pub struct MainCamera;

#[derive(Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, cursor::capture_on_start))
            .add_systems(
                Update,
                (
                    look::apply_look,
                    rig::switch_view,
                    rig::place_camera,
                    rig::show_model_in_third_person,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                cursor::toggle_capture
                    .after(InputSystems)
                    .run_if(resource_exists::<ButtonInput<KeyCode>>)
                    .run_if(resource_exists::<ButtonInput<MouseButton>>),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        CameraRig::default(),
        Camera3d::default(),
        Transform::default(),
    ));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::asset::AssetPlugin;
    use bevy::time::TimeUpdateStrategy;
    use leafwing_input_manager::prelude::ActionState;

    use super::*;
    use crate::input::PlayerAction;
    use crate::player::{Body, EYE_HEIGHT, Player, PlayerPlugin};
    use crate::world::{ChunkWorkers, WorldPlugin, WorldSettings};

    fn game_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 64.0,
            )))
            .insert_resource(ChunkWorkers::Inline)
            .insert_resource(WorldSettings {
                view_radius: 1,
                ..WorldSettings::default()
            })
            .add_plugins((WorldPlugin, PlayerPlugin, CameraPlugin));
        app
    }

    fn body(app: &mut App) -> Body {
        let mut query = app.world_mut().query_filtered::<&Body, With<Player>>();
        *query.single(app.world()).expect("one player")
    }

    fn camera(app: &mut App) -> (Transform, CameraRig) {
        let mut query = app
            .world_mut()
            .query_filtered::<(&Transform, &CameraRig), With<MainCamera>>();
        let (transform, rig) = query.single(app.world()).expect("one camera");
        (*transform, *rig)
    }

    fn set_switch_view(app: &mut App, pressed: bool) {
        let mut query = app
            .world_mut()
            .query_filtered::<&mut ActionState<PlayerAction>, With<Player>>();
        let mut actions = query.single_mut(app.world_mut()).expect("one player");
        if pressed {
            actions.press(&PlayerAction::SwitchView);
        } else {
            actions.release(&PlayerAction::SwitchView);
        }
    }

    /// Streams the world and lets the player fall until they stand on the ground.
    fn land(app: &mut App) {
        for _ in 0..2_000 {
            app.update();
            let mut query = app.world_mut().query_filtered::<&Body, With<Player>>();
            if query.iter(app.world()).any(|body| body.state.on_ground) {
                return;
            }
        }
        panic!("the player never landed");
    }

    #[test]
    fn the_player_lands_and_the_first_person_camera_sits_at_the_eye() {
        let mut app = game_app();

        land(&mut app);
        app.update();

        let feet = body(&mut app).state.feet;
        let (camera, rig) = camera(&mut app);
        assert_eq!(rig.view, View::FirstPerson);
        assert!((camera.translation - (feet + Vec3::Y * EYE_HEIGHT)).length() < 0.05);
    }

    #[test]
    fn the_view_switch_moves_the_same_rig_behind_the_shoulder_and_back() {
        let mut app = game_app();
        land(&mut app);

        set_switch_view(&mut app, true);
        app.update();
        set_switch_view(&mut app, false);
        app.update();

        let feet = body(&mut app).state.feet;
        let (third, rig) = camera(&mut app);
        assert_eq!(rig.view, View::ThirdPerson);
        let eye = feet + Vec3::Y * EYE_HEIGHT;
        assert!(
            third.translation.distance(eye) > 1.0,
            "camera at {third:?}, eye {eye}"
        );

        set_switch_view(&mut app, true);
        app.update();

        assert_eq!(camera(&mut app).1.view, View::FirstPerson);
    }
}
