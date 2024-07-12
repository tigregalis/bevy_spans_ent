//! VERY WIP
//!
//! Basic idea:
//! 1. Construct an ephemeral `TempEditor` (wraps `cosmic_text::Editor`) around the existing `cosmic_text::Buffer`,
//! 2. Apply any changes to the `TempEditor` and it will be reflected in the `Buffer`
//! 3. Extract and store any state that needs to be persisted between frames in EditorState (Cursor, Selection) and drop the `TempEditor`
//! 4. Working backwards, update the `Text` component from the updated `Buffer`
//!
//! TODO:
//! ~~1. when cursor is at 0 on any line, it doesn't insert anything... why?~~
//! ~~2. when I empty the buffer all hell breaks loose~~
//! ~~3. when I try to backspace from the start of a line, sometimes everything blows up~~
//! 4. store and selections
//! 5. the cursor should be its own entity! (and there should be the possibility of multiple cursors)
//! 6. multiple windows
//! 7. "Focused" Editor, not every editor
//! 8. "external"/programmatic changes to the text/spans should update the cursor/selection safely
//! 9. currently text spans have been cut out of this implementation
//! 10. with spans-as-entities (not yet implemented) it should be possible to restrict editing (e.g. only edit a span)

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::{Extract, ExtractSchedule, RenderApp};
use bevy::text::cosmic_text::{Action, Buffer, Cursor, Edit, Editor, LayoutRun, Motion};
use bevy::text::CosmicBuffer;
use bevy::ui::{ExtractedUiNode, ExtractedUiNodes, NodeType, RenderUiSystem};
use bevy::window::PrimaryWindow;
use bevy_text_span_entities::prelude::*;
use unicode_segmentation::UnicodeSegmentation as _;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (hit.pipe(handle_click), listen_keyboard_input_events),
        )
        .add_systems(Update, animate_cursor);
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.add_systems(
        ExtractSchedule,
        extract_cursor.in_set(RenderUiSystem::ExtractText),
    );
    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let style = TextStyle {
        font_size: 70.0,
        ..Default::default()
    };

    let mut parent = text!(&mut commands, [
        ("Hello, ", { font_size: 40.0 }),
        ("World!\n", { font_size: 60.0, color: Color::srgb(1.0, 0.0, 0.0) }),
        ("Hello, Bevy!\n", { font_size: 50.0 }, A),
        ("and so on and so forth...", style, (A, B))
    ]);

    parent.insert((EditorState::default(), CursorConfig::default()));
}

/// Piped from [`hit`]
///
/// TODO: The child spans aren't actually used in this case, we just care about the parent Text node
/// TODO: This should respect UI stack indexes / Z ordering
#[allow(clippy::type_complexity)]
fn handle_click(
    In(hit): In<Option<HitOutput>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut buffer: Query<(&mut CosmicBuffer, &mut EditorState), With<Text>>,
    mut text_pipeline: ResMut<bevy::text::TextPipeline>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(HitOutput {
        parent,
        span: _,
        position,
    }) = hit
    else {
        return;
    };
    let Ok((mut buf, mut state)) = buffer.get_mut(parent) else {
        return;
    };
    let new_state = {
        let mut editor = TempEditor::new(&mut buf.0, *state);
        let font_system = text_pipeline.font_system_mut();
        editor.action(
            font_system,
            Action::Click {
                x: position.x as i32,
                y: position.y as i32,
            },
        );
        editor.state()
    };
    *state = new_state;
}

fn listen_keyboard_input_events(
    mut events: EventReader<KeyboardInput>,
    mut buffer: Query<(&mut CosmicBuffer, &mut Text, &mut EditorState)>,
    mut text_pipeline: ResMut<bevy::text::TextPipeline>,
    mut scratch_spans_for_deletion: Local<Vec<usize>>,
    mut scratch_spans_for_update: Local<HashMap<usize, String>>,
) {
    for event in events.read() {
        // Only trigger changes when the key is first pressed.
        if event.state == ButtonState::Released {
            continue;
        }

        for (mut buf, mut text, mut state) in &mut buffer {
            let new_state = {
                let mut editor = TempEditor::new(&mut buf.0, *state);
                let font_system = text_pipeline.font_system_mut();
                // info!("Before: {:?}", editor.cursor());
                match &event.logical_key {
                    Key::Character(character) => {
                        for c in character.chars() {
                            editor.action(font_system, Action::Insert(c));
                        }
                    }
                    Key::Enter => editor.action(font_system, Action::Enter),
                    Key::Space => editor.action(font_system, Action::Insert(' ')),
                    Key::Backspace => editor.action(font_system, Action::Backspace),
                    Key::Delete => editor.action(font_system, Action::Delete),
                    Key::Control => {
                        info!("TODO: Control");
                        continue;
                    }
                    Key::Shift => {
                        info!("TODO: Shift");
                        continue;
                    }
                    Key::Tab => {
                        info!("TODO: Tab");
                        continue;
                    }
                    Key::ArrowDown => editor.action(font_system, Action::Motion(Motion::Down)),
                    Key::ArrowLeft => editor.action(font_system, Action::Motion(Motion::Left)),
                    Key::ArrowRight => editor.action(font_system, Action::Motion(Motion::Right)),
                    Key::ArrowUp => editor.action(font_system, Action::Motion(Motion::Up)),
                    Key::End => editor.action(font_system, Action::Motion(Motion::End)),
                    Key::Home => editor.action(font_system, Action::Motion(Motion::Home)),
                    Key::PageDown => editor.action(font_system, Action::Motion(Motion::PageDown)),
                    Key::PageUp => editor.action(font_system, Action::Motion(Motion::PageUp)),
                    _ => continue,
                }
                // info!("After:  {:?}", editor.cursor());
                editor.state()
            };

            // rebuild the text from scratch
            for line in &buf.lines {
                let line_text = dbg!(line.text());
                let len = line_text.len();
                let ending = line.ending().as_str();
                let spans = line.attrs_list().spans();
                // NOTE: cosmic-text allows for "unstyled" (default-styled) spans/ranges
                //       this means not all `spans` actually have styles
                //       so imagine a line with 21 characters (full range 0..21)
                //       the `spans` iterator can yield for example 2..7, 9..12, 12..13, 13..16, 17..19
                //       so 0..2, 7..9, 16..17, 19..21 are unstyled, and we have to specially handle these
                //       in this case, we will style
                //       0..2 like 2..7 (unstyled span will be styled like next styled span)
                //       7..9 like 9..12 (unstyled span will be styled like next styled span)
                //       16..17 like 17..19 (unstyled span will be styled like next styled span)
                //       19..21 like 17..19 (final part of line, unstyled span will be styled like previous styled span)
                let mut current_pos = 0;
                let mut bevy_span_index = 0;
                for (range, attrs) in spans.into_iter() {
                    bevy_span_index = attrs.metadata;
                    let s = scratch_spans_for_update.entry(bevy_span_index).or_default();
                    // "unstyled" spans will take the following range's attrs
                    if current_pos < range.start {
                        s.push_str(&line_text[current_pos..range.start]);
                    }
                    // push the styled span
                    s.push_str(&line_text[range.clone()]);
                    current_pos = range.end;
                    // push the line ending if we've reached the end of the line
                    if current_pos == len {
                        s.push_str(ending);
                    }
                }
                // final part of the line
                if current_pos < len {
                    let s = scratch_spans_for_update.entry(bevy_span_index).or_default();
                    // push the styled span
                    s.push_str(&line_text[current_pos..len]);
                    // push the line ending since we've reached the end of the line
                    s.push_str(ending);
                }
            }

            // apply the changes (well, everything) to the text component
            for i in 0..text.sections.len() {
                match scratch_spans_for_update.remove(&i) {
                    // TODO: should be forwarded to the TextSpan component for child spans instead
                    // TODO: could be more efficient (don't update the whole string if no changes were made)
                    Some(s) => text.sections[i].value = s,
                    None => scratch_spans_for_deletion.push(i),
                }
            }
            scratch_spans_for_deletion.reverse();
            for i in scratch_spans_for_deletion.drain(..) {
                if text.sections.len() > 1 {
                    text.sections.remove(i);
                } else {
                    text.sections[0].value = String::new();
                }
            }

            *state = new_state;
        }
    }
}

/// Adapted from `bevy_ui::extract_uinode_text` and `bevy_ui::extract_uinode_background_colors`
#[allow(clippy::type_complexity)]
fn extract_cursor(
    mut commands: Commands,
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    camera_query: Extract<Query<(Entity, &Camera)>>,
    default_ui_camera: Extract<DefaultUiCamera>,
    ui_scale: Extract<Res<UiScale>>,
    // TODO: the cursor should be its own entity!
    uinode_query: Extract<
        Query<
            (
                &Node,
                &GlobalTransform,
                &ViewVisibility,
                Option<&CalculatedClip>,
                Option<&TargetCamera>,
                Option<&CursorConfig>,
                &CosmicBuffer,
                &EditorState,
            ),
            With<Text>,
        >,
    >,
) {
    for (
        uinode,
        global_transform,
        view_visibility,
        clip,
        camera,
        cursor_config,
        buffer,
        editor_state,
    ) in &uinode_query
    {
        let Some(cursor) = editor_state.cursor else {
            continue;
        };

        let Some(camera_entity) = camera.map(TargetCamera::entity).or(default_ui_camera.get())
        else {
            continue;
        };

        // Skip if not visible or if size is set to zero (e.g. when a parent is set to `Display::None`)
        if !view_visibility.get() || uinode.size().x == 0. || uinode.size().y == 0. {
            continue;
        }

        let scale_factor = camera_query
            .get(camera_entity)
            .ok()
            .and_then(|(_, c)| c.target_scaling_factor())
            .unwrap_or(1.0)
            * ui_scale.0;
        let inverse_scale_factor = scale_factor.recip();

        // Align the text to the nearest physical pixel:
        // * Translate by minus the text node's half-size
        //      (The transform translates to the center of the node but the text coordinates are relative to the node's top left corner)
        // * Multiply the logical coordinates by the scale factor to get its position in physical coordinates
        // * Round the physical position to the nearest physical pixel
        // * Multiply by the rounded physical position by the inverse scale factor to return to logical coordinates

        let logical_top_left = -0.5 * uinode.size();

        let mut transform = global_transform.affine()
            * bevy::math::Affine3A::from_translation(logical_top_left.extend(0.));

        transform.translation *= scale_factor;
        transform.translation = transform.translation.round();
        transform.translation *= inverse_scale_factor;

        let cursor_config = match cursor_config {
            Some(c) => *c,
            None => Default::default(),
        };
        let color = cursor_config.color.into();
        let width = cursor_config.width;

        // TODO: we can locate the exact layout_run by the cursor position
        for run in buffer.layout_runs() {
            // TODO: this should happen in the main world so that we do as little work as possible here
            if let Some((x, y)) = cursor_position(&cursor, &run) {
                let position = Vec2::new(x as f32, y as f32 + run.line_height / 2.0);
                extracted_uinodes.uinodes.insert(
                    commands.spawn_empty().id(),
                    ExtractedUiNode {
                        stack_index: uinode.stack_index(),
                        transform: transform
                            * Mat4::from_translation(position.extend(0.) * inverse_scale_factor),
                        color,
                        rect: Rect {
                            min: Vec2::ZERO,
                            // TODO: size?
                            max: Vec2::new(width, run.line_height),
                        },
                        image: AssetId::default(),
                        atlas_size: None,
                        clip: clip.map(|clip| clip.clip),
                        flip_x: false,
                        flip_y: false,
                        camera_entity,
                        border: [0.; 4],
                        border_radius: [0.; 4],
                        node_type: NodeType::Rect,
                    },
                );
            }
        }
    }
}

// from cosmic-text/src/edit/editor.rs:66
fn cursor_position(cursor: &Cursor, run: &LayoutRun) -> Option<(i32, i32)> {
    let (cursor_glyph, cursor_glyph_offset) = cursor_glyph_opt(cursor, run)?;
    let x = match run.glyphs.get(cursor_glyph) {
        Some(glyph) => {
            // Start of detected glyph
            if glyph.level.is_rtl() {
                (glyph.x + glyph.w - cursor_glyph_offset) as i32
            } else {
                (glyph.x + cursor_glyph_offset) as i32
            }
        }
        None => match run.glyphs.last() {
            Some(glyph) => {
                // End of last glyph
                if glyph.level.is_rtl() {
                    glyph.x as i32
                } else {
                    (glyph.x + glyph.w) as i32
                }
            }
            None => {
                // Start of empty line
                0
            }
        },
    };

    Some((x, run.line_top as i32))
}

// from cosmic-text/src/edit/editor.rs:30
fn cursor_glyph_opt(cursor: &Cursor, run: &LayoutRun) -> Option<(usize, f32)> {
    if cursor.line == run.line_i {
        for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
            if cursor.index == glyph.start {
                return Some((glyph_i, 0.0));
            } else if cursor.index > glyph.start && cursor.index < glyph.end {
                // Guess x offset based on characters
                let mut before = 0;
                let mut total = 0;

                let cluster = &run.text[glyph.start..glyph.end];
                for (i, _) in cluster.grapheme_indices(true) {
                    if glyph.start + i < cursor.index {
                        before += 1;
                    }
                    total += 1;
                }

                let offset = glyph.w * (before as f32) / (total as f32);
                return Some((glyph_i, offset));
            }
        }
        match run.glyphs.last() {
            Some(glyph) => {
                if cursor.index == glyph.end {
                    return Some((run.glyphs.len(), 0.0));
                }
            }
            None => {
                return Some((0, 0.0));
            }
        }
    }
    None
}

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;

#[derive(Deref, DerefMut)]
struct TempEditor<'buffer>(Editor<'buffer>);

impl<'buffer> TempEditor<'buffer> {
    fn new(buffer: &'buffer mut Buffer, state: EditorState) -> Self {
        let mut editor = Editor::new(buffer);
        if let Some(cursor) = state.cursor {
            editor.set_cursor(cursor);
        }
        Self(editor)
    }

    fn state(&self) -> EditorState {
        EditorState {
            cursor: Some(self.cursor()),
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
struct EditorState {
    cursor: Option<Cursor>,
}

#[derive(Component, Clone, Copy)]
struct CursorConfig {
    color: Color,
    width: f32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            color: Color::LinearRgba(LinearRgba::WHITE),
            width: 1.0,
        }
    }
}
// TODO: does not support multiple windows
#[derive(SystemParam)]
struct HitSystemParams<'w, 's> {
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    buffers: Query<
        'w,
        's,
        (
            Entity,
            &'static CosmicBuffer,
            &'static GlobalTransform,
            &'static Children,
        ),
        With<Node>,
    >,
}

struct HitOutput {
    parent: Entity,
    span: Entity,
    position: Vec2,
}

/// Assumes only one entity gets hit (early returns)
#[allow(clippy::type_complexity)]
fn hit(params: HitSystemParams) -> Option<HitOutput> {
    let window = params.window.single();

    let cursor_window_position = window.cursor_position()?;

    for (entity, buffer, transform, children) in &params.buffers {
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
                return Some(HitOutput {
                    parent: entity,
                    span: children[span_index],
                    position,
                });
            }
        }
    }

    None
}

fn animate_cursor(mut query: Query<&mut CursorConfig>, time: Res<Time>) {
    let seconds = time.elapsed_seconds();

    for mut config in &mut query {
        config.color = Color::srgb(
            cycle(seconds, 0.5) * 0.5 + 0.5, // varies between 0.5 and 1
            cycle(seconds, 1.1) * 0.5 + 0.5, // varies between 0.5 and 1
            cycle(seconds, 1.7) * 0.5 + 0.5, // varies between 0.5 and 1
        );
        config.width = 2.0 + 8.0 * cycle(seconds, 8.0); // varies between 2 and 10
    }
}

/// varies between 0 and 1
fn cycle(seconds: f32, frequency: f32) -> f32 {
    (seconds * frequency).sin() * 0.5 + 0.5
}
