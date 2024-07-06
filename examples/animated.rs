use bevy::prelude::*;
use bevy_text_span_entities::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (iridescence, oscillate))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((TextBundle::default(), TextSpans))
        .with_children(|parent| {
            parent.spawn(TextSpan(TextSection {
                value: "Hello, ".to_string(),
                style: TextStyle {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    ..Default::default()
                },
            }));
            parent.spawn((
                TextSpan(TextSection {
                    value: "World".to_string(),
                    ..Default::default()
                }),
                Iridescent(2.0, 3.0, 5.0),
            ));
            parent.spawn((
                TextSpan(TextSection {
                    value: "!".to_string(),
                    style: TextStyle {
                        color: Color::srgb(0.0, 0.0, 1.0),
                        ..Default::default()
                    },
                }),
                Iridescent(1.0, 1.5, 2.7),
            ));
        });

    commands
        .spawn((Text2dBundle::default(), TextSpans, Oscillates))
        .with_children(|parent| {
            parent.spawn((
                TextSpan(TextSection {
                    value: "Wow!".to_string(),
                    style: TextStyle {
                        color: Color::srgb(1.0, 0.0, 0.0),
                        font_size: 40.0,
                        ..Default::default()
                    },
                }),
                Iridescent(1.4, 1.9, 2.8),
            ));
        });
}

#[derive(Component)]
struct Iridescent(f32, f32, f32);

fn iridescence(mut query: Query<(&mut TextSpan, &Iridescent)>, time: Res<Time>) {
    let seconds = time.elapsed_seconds();
    for (mut span, &Iridescent(r, g, b)) in &mut query.iter_mut() {
        span.0.style.color = Color::srgb(cycle(seconds, r), cycle(seconds, g), cycle(seconds, b));
    }
}

#[derive(Component)]
struct Oscillates;

fn oscillate(time: Res<Time>, mut query: Query<&mut Transform, With<Oscillates>>) {
    let seconds = time.elapsed_seconds();
    for mut transform in &mut query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(cycle(seconds, 10.0));
    }
}

fn cycle(seconds: f32, frequency: f32) -> f32 {
    (seconds * frequency).sin()
}
