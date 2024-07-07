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

    let style = TextStyle {
        font_size: 10.0,
        ..Default::default()
    };

    let mut parent = text!(&mut commands, [
        ("Hello, "),
        ("World!\n", { color: Color::srgb(1.0, 0.0, 0.0) }),
        ("Hello, Bevy!\n", {}, A),
        ("and so on and so forth...", style, (A, B))
    ]);

    parent.insert(A);

    let mut parent = text!(&mut commands, []);

    parent.insert(B);
}

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;
