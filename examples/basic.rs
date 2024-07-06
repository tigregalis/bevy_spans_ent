use bevy::prelude::*;
use bevy_text_span_entities::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn((TextBundle::default(), TextSpans))
        .with_children(|parent| {
            parent.spawn(TextSpan(TextSection {
                value: "Hello, World!".to_string(),
                style: TextStyle {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    ..Default::default()
                },
            }));
        });
}
