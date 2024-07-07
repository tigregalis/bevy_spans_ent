use bevy::{ecs::system::SystemParam, prelude::*, text::CosmicBuffer, window::PrimaryWindow};
use bevy_text_span_entities::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                initialise_hover,
                hit.pipe(notify_hovered)
                    .after(initialise_hover)
                    .before(hover_effect),
                hover_effect,
                hit.pipe(notify_navigate).before(navigate_effect),
                navigate_effect,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    let hover_text1 = "Hover me!";
    let hover_text2 = "Hover\nme!";
    let bevy = "https://bevyengine.org";
    let example = "https://example.com";
    let hello = "Hello, world!";

    let style = TextStyle {
        font_size: 30.0,
        color: Color::srgb(0.0, 0.8, 0.1),
        ..Default::default()
    };

    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            border_color: Color::srgb(0.1, 0.1, 0.1).into(),
            ..Default::default()
        })
        .with_children(|parent| {
            let _parent = text!(
                parent, [
                (
                    "Links clicked: ",
                    style.clone()
                ),
                (
                    "0",
                    TextStyle {
                        color: Color::srgb(0.0, 0.8, 0.8),
                        ..style.clone()
                    },
                    Mutate(0)
                ),
                (
                    " times\n",
                    style.clone()
                ),
                (
                    example,
                    {
                        color: Color::srgb(0.0, 0.8, 0.1)
                    },
                    (Link(example.into()), LinkNotifier(false))
                ),
                ("\n"),
                (
                    hover_text1,
                    {},
                    (
                        HoverConfig {
                            base_color: Color::srgb(0.8, 0.0, 0.1),
                            hover_color: Color::srgb(0.5, 0.4, 1.0),
                        },
                        HoverNotifier(false),
                    )
                ),
                ("\n"),
                (hello, { font_size: 60.0 }),
                ("\n"),
            ]);
        })
        .with_children(|parent| {
            let _parent = text!(
                parent, [
                (hello, { font_size: 60.0 }),
                ("\n"),
                (
                    "Links clicked: ",
                    style.clone()
                ),
                (
                    "0",
                    TextStyle {
                        color: Color::srgb(0.5, 0.8, 0.7),
                        ..style.clone()
                    },
                    Mutate(0)
                ),
                (
                    " times\n",
                    style.clone()
                ),
                (
                    bevy,
                    {
                        color: Color::srgb(0.0, 0.8, 0.1)
                    },
                    (Link(bevy.into()), LinkNotifier(false))
                ),
                ("\n"),
                (
                    hover_text2,
                    style,
                    (
                        HoverConfig {
                            base_color: Color::srgb(0.5, 0.4, 0.3),
                            hover_color: Color::srgb(0.8, 0.7, 0.6),
                        },
                        HoverNotifier(false),
                    )
                ),
            ]);
        });
}

fn initialise_hover(mut query: Query<(&mut TextSpan, &HoverConfig), Added<HoverConfig>>) {
    for (mut span, hover) in &mut query {
        span.0.style.color = hover.base_color;
    }
}

/// Piped from [`hit`]
#[allow(clippy::type_complexity)]
fn notify_hovered(
    In(hit): In<Option<Entity>>,
    mut spans: Query<&mut HoverNotifier, (With<Parent>, With<HoverConfig>)>,
) {
    let Some(entity) = hit else {
        return;
    };
    if let Ok(mut notifier) = spans.get_mut(entity) {
        notifier.notify();
    }
}

#[allow(clippy::type_complexity)]
fn hover_effect(
    mut last: Local<Option<Entity>>,
    mut spans: ParamSet<(
        Query<(&mut TextSpan, &HoverConfig), With<HoverNotifier>>,
        Query<(Entity, &mut TextSpan, &HoverConfig, &HoverNotifier), Changed<HoverNotifier>>,
    )>,
) {
    if let Some(last_entity) = last.as_mut() {
        if let Ok((mut span, config)) = spans.p0().get_mut(*last_entity) {
            span.0.style.color = config.base_color;
            *last = None;
        }
    }
    for (entity, mut span, config, notifier) in &mut spans.p1() {
        if !notifier.0 {
            continue;
        }
        span.0.style.color = config.hover_color;
        *last = Some(entity);
    }
}

/// Piped from [`hit`]
#[allow(clippy::type_complexity)]
fn notify_navigate(
    In(hit): In<Option<Entity>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut spans: Query<&mut LinkNotifier, With<Parent>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(entity) = hit else {
        return;
    };
    if let Ok(mut notifier) = spans.get_mut(entity) {
        notifier.notify();
    }
}

#[allow(clippy::type_complexity)]
fn navigate_effect(
    spans: Query<(&Link, &LinkNotifier), Changed<LinkNotifier>>,
    mut mutables: Query<(&mut TextSpan, &mut Mutate)>,
) {
    // TODO: bug that means this is triggered once at app start-up
    for (link, notifier) in &spans {
        if !notifier.0 {
            continue;
        }
        info!("Navigate to {}", link.0);
        for (mut span, mut mutate) in &mut mutables {
            mutate.0 += 1;
            span.0.value = mutate.0.to_string();
        }
    }
}

#[derive(Component)]
struct Mutate(usize);

#[derive(Component)]
struct Link(String);

#[derive(Component)]
struct HoverConfig {
    base_color: Color,
    hover_color: Color,
}

#[derive(Component)]
struct LinkNotifier(bool);

#[derive(Component)]
struct HoverNotifier(bool);

impl Notifier for HoverNotifier {
    fn notify(&mut self) {
        self.0 = true;
    }
}
impl Notifier for LinkNotifier {
    fn notify(&mut self) {
        self.0 = true;
    }
}

trait Notifier {
    /// Just triggers change detection
    fn notify(&mut self);
}

#[derive(SystemParam)]
struct HitSystemParams<'w, 's> {
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    buffers: Query<
        'w,
        's,
        (
            &'static CosmicBuffer,
            &'static GlobalTransform,
            &'static Children,
        ),
        With<Node>,
    >,
}

/// Assumes only one entity gets hit (early returns)

#[allow(clippy::type_complexity)]
fn hit(params: HitSystemParams) -> Option<Entity> {
    let window = params.window.single();

    let cursor_window_position = window.cursor_position()?;

    for (buffer, transform, children) in &params.buffers {
        let size = buffer.size();
        let size = Vec2::new(
            size.0.expect("Buffer has a width"),
            size.1.expect("Buffer has a height"),
        );
        let origin = transform.translation().truncate();
        let rect = Rect::from_center_size(origin, size);
        if rect.contains(cursor_window_position) {
            // top left corner of buffer (where +Y down, +X right)
            // TODO: slightly off for some reason, unsure if cosmic-text or this is wrong
            let offset = origin - size / 2.0;
            // position in buffer
            let position = cursor_window_position - offset;
            // TODO: fix the issue where this always registers a hit on the first span if no other is hit
            if let Some(text_cursor) = buffer.hit(position.x, position.y) {
                // get attrs from cursor
                let line = &buffer.lines[text_cursor.line];
                let attrs = line.attrs_list().get_span(text_cursor.index);
                let span_index = attrs.metadata;
                // notify only the relevant child
                return Some(children[span_index]);
            }
        }
    }

    None
}
