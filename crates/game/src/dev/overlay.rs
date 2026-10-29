use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use super::settings::DevSettings;
use crate::environment::Environment;

const OVERLAY_MARGIN_PX: f32 = 8.0;
const OVERLAY_FONT_SIZE: f32 = 14.0;

#[derive(Debug, Component)]
pub struct OverlayText;

pub fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        OverlayText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(OVERLAY_FONT_SIZE),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(OVERLAY_MARGIN_PX),
            left: px(OVERLAY_MARGIN_PX),
            ..default()
        },
    ));
}

pub fn update_overlay(
    settings: Res<DevSettings>,
    diagnostics: Res<DiagnosticsStore>,
    environment: Res<Environment>,
    mut overlay: Single<(&mut Text, &mut Visibility), With<OverlayText>>,
) {
    let (text, visibility) = &mut *overlay;
    **visibility = if settings.overlay_visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if settings.overlay_visible {
        let measure = |path| diagnostics.get(path).and_then(Diagnostic::smoothed);
        text.0 = format_overlay(
            measure(&FrameTimeDiagnosticsPlugin::FPS),
            measure(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
            *environment,
        );
    }
}

fn format_overlay(fps: Option<f64>, frame_ms: Option<f64>, environment: Environment) -> String {
    let fps = fps.map_or_else(|| "--".to_owned(), |value| format!("{value:.0}"));
    let frame_ms = frame_ms.map_or_else(|| "--".to_owned(), |value| format!("{value:.1}"));
    format!(
        "{fps} fps | {frame_ms} ms\ntime {:.2} | {:?}\nF3 overlay  F6 weather  ] fast time",
        environment.time_of_day, environment.weather
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_dashes_until_the_first_measurement() {
        let text = format_overlay(None, None, Environment::default());

        assert!(text.starts_with("-- fps | -- ms"));
    }

    #[test]
    fn shows_rounded_frame_stats_and_the_environment() {
        let text = format_overlay(Some(59.7), Some(16.74), Environment::default());

        assert!(text.starts_with("60 fps | 16.7 ms"));
        assert!(text.contains("Clear"));
    }
}
