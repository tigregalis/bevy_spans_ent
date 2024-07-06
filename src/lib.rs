pub mod prelude {
    pub use crate::main::{TextSpan, TextSpans, TsePlugin};
}

mod main {

    use bevy::prelude::*;

    pub struct TsePlugin;

    impl Plugin for TsePlugin {
        fn build(&self, app: &mut App) {
            app.register_type::<TextSpans>();
            app.register_type::<TextSpan>();
            app.add_systems(
                Update,
                update_parent
                    .before(bevy::ui::widget::measure_text_system)
                    .before(bevy::text::update_text2d_layout),
            );
        }
    }

    /// The parent
    #[derive(Component, Debug, Clone, Default, Reflect)]
    #[reflect(Component, Default)]
    pub struct TextSpans;

    /// The children
    #[derive(Component, Debug, Clone, Default, Reflect)]
    #[reflect(Component, Default)]
    pub struct TextSpan(pub TextSection);

    #[allow(clippy::type_complexity)]
    fn update_parent(
        mut changed: Local<std::collections::HashSet<Entity>>,
        changed_parents: Query<Entity, (With<TextSpans>, Changed<Children>, Without<TextSpan>)>,
        mut parents: Query<&mut Text, (With<TextSpans>, With<Children>, Without<TextSpan>)>,
        changed_children: Query<&Parent, Changed<TextSpan>>,
        all_children: Query<&TextSpan, With<Parent>>,
        children: Query<&Children>,
    ) {
        for parent in &changed_children {
            changed.insert(parent.get());
        }
        for parent in &changed_parents {
            changed.insert(parent);
        }

        for parent in changed.drain() {
            if let Ok(mut text) = parents.get_mut(parent) {
                text.sections.clear();
                for child in children.iter_descendants(parent) {
                    if let Ok(span) = all_children.get(child) {
                        text.sections.push(span.0.clone());
                    } else {
                        error!("Missing `TextSpan` for child {child:?} for parent {parent:?}");
                    }
                }
            } else {
                error!("Missing `Text` for parent {parent:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
