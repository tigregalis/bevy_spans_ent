use bevy::{prelude::*, window::PrimaryWindow};
use bevy_text_span_entities::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                damage,
                sync_current_health.after(damage),
                sync_max_health.after(damage),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let half_size = Vec2::new(50.0, 80.0);
    let font_size = 16.0;

    let monster = Monster { half_size };
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(half_size * 2.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            monster,
            CurrentHealth(10),
            MaxHealth(10),
        ))
        .with_children(|monster| {
            let m = monster.parent_entity();
            monster
                .spawn((
                    Text2dBundle {
                        transform: Transform::from_xyz(0.0, half_size.y + font_size / 2.0, 0.0),
                        ..Default::default()
                    },
                    TextSpans,
                ))
                .with_children(|text| {
                    text.spawn(TextSpan(TextSection {
                        value: "HP: ".to_string(),
                        style: TextStyle {
                            color: Color::srgb(1.0, 1.0, 1.0),
                            font_size,
                            ..Default::default()
                        },
                    }));
                    text.spawn((
                        TextSpan(TextSection {
                            value: "PLACEHOLDER".to_string(),
                            style: TextStyle {
                                color: Color::srgb(1.0, 1.0, 1.0),
                                font_size,
                                ..Default::default()
                            },
                        }),
                        CurrentHealthDisplay(m),
                    ));
                    text.spawn(TextSpan(TextSection {
                        value: "/".to_string(),
                        style: TextStyle {
                            color: Color::srgb(1.0, 1.0, 1.0),
                            font_size,
                            ..Default::default()
                        },
                    }));
                    text.spawn((
                        TextSpan(TextSection {
                            value: "PLACEHOLDER".to_string(),
                            style: TextStyle {
                                color: Color::srgb(0.0, 0.0, 1.0),
                                font_size,
                                ..Default::default()
                            },
                        }),
                        MaxHealthDisplay(m),
                    ));
                });
        });
}

#[derive(Component)]
struct Monster {
    half_size: Vec2,
}

#[derive(Component)]
struct CurrentHealth(u32);
#[derive(Component)]
struct MaxHealth(u32);

#[derive(Component)]
struct CurrentHealthDisplay(Entity);
#[derive(Component)]
struct MaxHealthDisplay(Entity);

fn damage(
    mut commands: Commands,
    mut query: Query<(Entity, &Monster, &Transform, &mut CurrentHealth)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera.single();

        let window = window.single();

        if let Some(cursor_world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (entity, monster, transform, mut health) in &mut query {
                let hitbox = Rect::from_center_half_size(
                    transform.translation.truncate(),
                    monster.half_size,
                );

                if hitbox.contains(cursor_world_position) {
                    if health.0 > 0 {
                        health.0 -= 1;
                    }
                    if health.0 == 0 {
                        info!("Monster died!");
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn sync_current_health(
    monster: Query<(&CurrentHealth, &MaxHealth), (With<Monster>, Changed<CurrentHealth>)>,
    // TODO: should be lazier
    mut display: Query<(&mut TextSpan, &CurrentHealthDisplay), Without<MaxHealthDisplay>>,
) {
    for (mut text, health) in &mut display {
        if let Ok((health, max)) = monster.get(health.0) {
            text.0.value = health.0.to_string();
            let cmp = health.0 as f32 / max.0 as f32;
            text.0.style.color = if cmp < 0.5 {
                Color::srgb(1.0, 0.0, 0.0)
            } else if cmp < 1.0 {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::srgb(0.0, 1.0, 0.0)
            };
        }
    }
}

fn sync_max_health(
    monster: Query<&MaxHealth, (With<Monster>, Changed<MaxHealth>)>,
    // TODO: should be lazier
    mut display: Query<(&mut TextSpan, &MaxHealthDisplay), Without<CurrentHealthDisplay>>,
) {
    for (mut text, health) in &mut display {
        if let Ok(health) = monster.get(health.0) {
            text.0.value = health.0.to_string();
        }
    }
}
