use std::collections::HashMap;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::text::cosmic_text::{
    Action, Buffer, BufferRef, Cursor, Edit, Editor, FontSystem, LineEnding,
};
use bevy::text::CosmicBuffer;
use bevy_text_span_entities::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TsePlugin)
        .add_event::<CommittedKey>()
        .add_systems(Startup, setup)
        // .add_systems(PreUpdate, keyboard_event_system)
        // .add_systems(Update, print_committed_key)
        .add_systems(PreUpdate, listen_keyboard_input_events)
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

    parent.insert(EditorState::default());
}

#[derive(Event, Debug)]
struct CommittedKey(Key);

impl From<Key> for CommittedKey {
    fn from(key: Key) -> Self {
        CommittedKey(key)
    }
}

impl From<CommittedKey> for Key {
    fn from(key: CommittedKey) -> Self {
        key.0
    }
}

fn listen_keyboard_input_events(
    mut events: EventReader<KeyboardInput>,
    // mut edit_text: Query<&mut TextSpan>,
    mut buffer: Query<(&mut CosmicBuffer, &mut Text, &mut EditorState)>,
    mut text_pipeline: ResMut<bevy::text::TextPipeline>,
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
                match &event.logical_key {
                    Key::Character(character) => {
                        for c in character.chars() {
                            info!("here");
                            editor.action(font_system, Action::Insert(c));
                        }
                    }
                    Key::Enter => editor.action(font_system, Action::Enter),
                    Key::Space => editor.action(font_system, Action::Insert(' ')),
                    Key::Backspace => editor.action(font_system, Action::Backspace),
                    Key::Delete => editor.action(font_system, Action::Delete),
                    _ => continue,
                }
                EditorState {
                    cursor: Some(editor.cursor()),
                }
            };

            let mut map: HashMap<usize, String> = HashMap::new();
            for line in &buf.lines {
                let text = line.text();
                let ending = line.ending().as_str();
                let spans = line.attrs_list().spans();
                let count = spans.len();
                for (i, (range, attrs)) in spans.into_iter().enumerate() {
                    let s = map.entry(attrs.metadata).or_default();

                    s.push_str(&text[range.clone()]);
                    if i + 1 == count {
                        s.push_str(ending);
                    }
                }
            }
            dbg!(&map);

            for (i, s) in map.into_iter() {
                // TODO: should be forwarded to the TextSpan component instead
                text.sections[i].value = s;
            }

            *state = new_state;
        }
    }
}

// /// This system prints out all keyboard events as they come in
// fn keyboard_event_system(
//     mut keyboard_input_events: EventReader<KeyboardInput>,
//     mut pressed_key: Local<Option<Key>>,
//     mut committed_key: EventWriter<CommittedKey>,
//     mut frame: Local<usize>,
// ) {
//     // take the most recently pressed key
//     for event in keyboard_input_events.read() {
//         match event.state {
//             bevy::input::ButtonState::Pressed => {
//                 info!(
//                     "+{frame}:{event:?}",
//                     frame = *frame,
//                     event = event.logical_key
//                 );
//                 *pressed_key = Some(event.logical_key.clone());
//             }
//             bevy::input::ButtonState::Released => {
//                 info!(
//                     "-{frame}:{event:?}",
//                     frame = *frame,
//                     event = event.logical_key
//                 );
//                 if pressed_key.as_ref() == Some(&event.logical_key) {
//                     *pressed_key = None;
//                 }
//             }
//         }
//         if let Some(key) = pressed_key.as_ref() {
//             committed_key.send(key.clone().into());
//         }
//     }
//     *frame += 1;
// }

// fn print_committed_key(mut committed_key: EventReader<CommittedKey>, mut frame: Local<usize>) {
//     for CommittedKey(key) in committed_key.read() {
//         match key {
//             Key::Character(c) => {
//                 info!("{frame}: character '{c}' was pressed", frame = *frame);
//             }
//             _ => {
//                 info!("{key:?}");
//             }
//         }
//     }
//     *frame += 1;
// }

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;

#[derive(Deref, DerefMut)]
struct TempEditor<'buffer>(Editor<'buffer>);

#[derive(Component, Clone, Copy, Default)]
struct EditorState {
    cursor: Option<Cursor>,
}

impl<'buffer> TempEditor<'buffer> {
    fn new(buffer: &'buffer mut Buffer, state: EditorState) -> Self {
        let mut editor = Editor::new(buffer);
        if let Some(cursor) = state.cursor {
            editor.set_cursor(cursor);
        }
        Self(editor)
    }
}
