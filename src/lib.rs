pub mod prelude {
    pub use crate::helper::{text, text2d};
    pub use crate::lib::{TextSpan, TextSpans, TsePlugin};
}

mod lib {

    use bevy::prelude::*;

    pub struct TsePlugin;

    impl Plugin for TsePlugin {
        fn build(&self, app: &mut App) {
            app.register_type::<TextSpans>();
            app.register_type::<TextSpan>();
            app.add_systems(
                PostUpdate,
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

mod helper {

    // pub fn spawn_text_with_children(
    //     commands: &mut EntityCommands,
    //     style: TextStyle,
    //     components: impl Bundle,
    // ) {
    //     let parent = commands
    //         .spawn((TextBundle::default(), TextSpans))
    //         .with_children(|parent| {
    //             parent.spawn((
    //                 TextSpan(TextSection {
    //                     value: "Hello, World!".into(),
    //                     style,
    //                 }),
    //                 components,
    //             ));
    //         });
    // }

    // pub fn spawn_children(commands: &mut Commands, components: impl Bundle) {}

    /// A macro that creates a `TextSpans` `TextBundle` entity with the given `TextSpan` spans as children.
    ///
    /// Returns the parent `EntityCommands`.
    ///
    /// # Usage
    /// ```
    /// use bevy_text_span_entities::text;
    /// # use bevy::prelude::TextStyle;
    /// # use bevy::prelude::Component;
    /// # use bevy::prelude::Color;
    /// # #[derive(Component)] struct Mutate;
    /// # #[derive(Component)] struct Link(&'static str);
    /// # #[derive(Component)] struct Hover(Color);
    /// # let world = Default::default();
    /// # let mut command_queue = Default::default();
    /// # let mut commands = bevy::prelude::Commands::new(&mut command_queue, &world);
    ///
    /// // Spawn no spans
    /// text!(&mut commands, []);
    ///
    /// // Not valid
    /// /*
    /// text!(&mut commands, [()]);
    /// */
    ///
    /// // Valid, spawn default-styled spans with no custom components
    /// text!(&mut commands, [("")]);
    /// text!(&mut commands, [("foo")]);
    /// text!(&mut commands, [("foo"), ("bar")]);
    ///
    /// // Valid, expressions
    /// let foo = "foo";
    /// let bar = "bar";
    /// text!(&mut commands, [(foo), (bar)]);
    ///
    /// // Valid, spawn custom-styled spans with custom components
    /// text!(
    ///     &mut commands, [
    ///     (
    ///         "Hel", // The text to display, required for this span
    ///         {
    ///             color: Color::srgb(0.0, 0.8, 0.1)
    ///         }, // Optional style key-values, or expression
    ///         Mutate // Optional components
    ///     ),
    ///     (
    ///         "lo\nwo",
    ///         {
    ///             color: Color::srgb(0.0, 0.1, 0.8)
    ///         },
    ///         Link("https://example.com/")
    ///     ),
    ///     (
    ///         "rld!",
    ///         {
    ///             color: Color::srgb(0.8, 0.0, 0.1)
    ///         },
    ///         Hover(Color::srgb(0.1, 0.0, 0.8))
    ///     ),
    /// ]);
    /// ```
    #[macro_export]
    macro_rules! text {

        // TODO: Handle more trailing commas

        // handle trailing comma
        ( $commands:expr, [ $( ( $($tt:tt)* ) ),* , ]) => {
            text!( $commands, [ $( ( $($tt)* ) ),* ]);
        };

        // main
        // converts text!(commands, [("text", style, Component), ("text", style, Component)])
        // into text!(@process TextBundle ; commands; [["text", style, Component], ["text", style, Component]])
        ( $commands:expr, [ $( ( $($tt:tt)* ) ),* ]) => {
            {
                text!( @trace $commands, [ $( ( $($tt)* ) ),* ] );
                text!( @process bevy::ui::node_bundles::TextBundle ; $commands ; $( [ $($tt)* ] )* )
            }
        };
        ( @process $bundle:expr ; $commands:expr ; $( [ $($tt:tt)* ] )* ) => {
            {
                use bevy::{
                    hierarchy::{BuildChildren as _, ChildBuild as _},
                    text::TextSection,
                    ui::node_bundles::TextBundle,
                };
                use $crate::prelude::{TextSpan, TextSpans};
                let mut parent = $commands.spawn((TextBundle::default(), TextSpans));
                parent.with_children(|parent| {
                    $(
                        text!(@parse_inputs parent ; [ $($tt)* ]);
                    )*
                });
                parent
            }
        };


        ( @expand_styles $( $key:tt : $value:expr ),* ) => {
            TextStyle {
                $( $key: $value, )*
                ..Default::default()
            }
        };

        // Matches: text!(&mut commands, [ ( "Foo", {} ), ( "Bar", { color: red } ) ] );
        ( @parse_inputs $parent:expr ; [ $text:expr, { $( $key:tt : $value:expr ),* } ] ) => {
            text!( @parse_inputs $parent ; [ $text, text!( @expand_styles $( $key : $value ),* ) ] )
        };
        // Matches: text!(&mut commands, [ ( "Foo", {} ), ( "Bar", { color: red }, A ) ] );
        ( @parse_inputs $parent:expr ; [ $text:expr, { $( $key:tt : $value:expr ),* }, $components:expr ] ) => {
            text!( @parse_inputs $parent ; [ $text, text!( @expand_styles $( $key : $value ),* ), $components ] )
        };
        // Matches: text!(&mut commands, [ ( "Foo" ) ] );
        // Matches: text!(&mut commands, [ ( "Foo" ), ( "Bar" ) ] );
        ( @parse_inputs $parent:expr ; [ $text:expr ]) => {
            text!(@spawn_span $parent ; @text $text ; @style Default::default() ; @components ())
        };
        // Matches: text!(&mut commands, [ ( "Foo", style.clone() ), ( "Bar", style ) ] );
        ( @parse_inputs $parent:expr ; [ $text:expr, $style:expr ] ) => {
            text!(@spawn_span $parent ; @text $text ; @style $style ; @components ())
        };
        // Matches: text!(&mut commands, [ ( "Foo", style.clone(), A ), ( "Bar", style, B ) ] );
        // Matches: text!(&mut commands, [ ( text_expr0, style_expr0, components_expr0 ), /* ... */ ] );
        // Matches: text!(&mut commands, [ ( text_expr0, style_expr0, components_expr0 ), ( text_expr1, style_expr1, components_expr1 ), /* ... */ ] );
        ( @parse_inputs $parent:expr ; [ $text:expr, $style:expr, $components:expr ] ) => {
            text!( @spawn_span $parent ; @text $text ; @style $style ; @components $components )
        };
        ( @parse_inputs $parent:expr ; $($tt:tt)* ) => {
            // should no longer be reachable
            text!(@unhandled $($tt)*);
        };

        ( @spawn_span $parent:expr ; @text $text:expr ; @style $style:expr ; @components $components:expr ) => {
            $parent.spawn((
                TextSpan(TextSection {
                    value: $text.into(),
                    style: $style,
                }),
                $components,
            ));
        };

        // development
        ( @trace $($tt:tt)* ) => {
            // let _ = concat!("TRACE: ", stringify!( $($tt)* ));
        };
        ( @unhandled $($tt:tt)* ) => {
            let _ = concat!("UNHANDLED: ", stringify!( $($tt)* ));
        };

    }

    /// See [`text`] - this is the same but for Text2dBundle.
    #[macro_export]
    macro_rules! text2d {
        // TODO: Handle more trailing commas

        // handle trailing comma
        ( $commands:expr, [ $( ( $($tt:tt)* ) ),* , ]) => {
            text2d!( $commands, [ $( ( $($tt)* ) ),* ]);
        };

        // main
        // converts text2d!(commands, [("text", style, Component), ("text", style, Component)])
        // into text!(@process Text2dBundle ; commands; [["text", style, Component], ["text", style, Component]])
        // into text!(@process commands; [["text", style, Component], ["text", style, Component]])
        ( $commands:expr, [ $( ( $($tt:tt)* ) ),* ]) => {
            {
                text!( @trace $commands, [ $( ( $($tt)* ) ),* ] );
                text!( @process bevy::text::Text2dBundle ; $commands ; $( [ $($tt)* ] )* )
            }
        };
    }

    pub use {text, text2d};
}

#[cfg(test)]
mod test {
    use bevy::{color::Color, prelude::Component, text::TextStyle};

    use super::helper::{text, text2d};

    #[test]
    fn test_text_macro() {
        let world = Default::default();
        let mut command_queue = Default::default();
        let mut commands = bevy::prelude::Commands::new(&mut command_queue, &world);
        let s = "Hel";
        let t = "lo, Wor";
        let u = "ld!";
        let style = TextStyle {
            font_size: 30.0,
            color: Color::srgb(0.0, 0.8, 0.1),
            ..Default::default()
        };
        text!(&mut commands, [(s), (t), (u)]);
        text!(
            &mut commands,
            [(s, style.clone()), (t, style.clone()), (u, style.clone())]
        );
        text!(&mut commands, [(s, {}), (t, {}), (u, {})]);
        text!(
            &mut commands,
            [
                (s.to_string(), { color: Color::srgb(0.0, 0.8, 0.1) }),
                (t, { font_size: 30.0 }),
                (u, { font_size: 30.0, color: Color::srgb(0.0, 0.8, 0.1) }),
            ]
        );
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        text!(
            &mut commands,
            [
                (s, style.clone(), A),
                (t, style.clone(), B),
                (u, style.clone(), (A, B)),
            ]
        );

        text!(
            &mut commands,
            [
                (s, {}, A),
                (t, style.clone(), (A, B)),
                (u, style.clone()),
                ("?"),
                ("foo", { color: Color::srgb(0.0, 0.8, 0.1) }, A),
            ]
        );
        drop(style);
    }

    #[test]
    fn test_text2d_macro() {
        let world = Default::default();
        let mut command_queue = Default::default();
        let mut commands = bevy::prelude::Commands::new(&mut command_queue, &world);
        let s = "Hel";
        let t = "lo, Wor";
        let u = "ld!";
        let style = TextStyle {
            font_size: 30.0,
            color: Color::srgb(0.0, 0.8, 0.1),
            ..Default::default()
        };
        text2d!(&mut commands, [(s), (t), (u)]);
        text2d!(
            &mut commands,
            [(s, style.clone()), (t, style.clone()), (u, style.clone())]
        );
        text2d!(&mut commands, [(s, {}), (t, {}), (u, {})]);
        text2d!(
            &mut commands,
            [
                (s.to_string(), { color: Color::srgb(0.0, 0.8, 0.1) }),
                (t, { font_size: 30.0 }),
                (u, { font_size: 30.0, color: Color::srgb(0.0, 0.8, 0.1) }),
            ]
        );
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        text2d!(
            &mut commands,
            [
                (s, style.clone(), A),
                (t, style.clone(), B),
                (u, style.clone(), (A, B)),
            ]
        );

        text2d!(
            &mut commands,
            [
                (s, {}, A),
                (t, style.clone(), (A, B)),
                (u, style.clone()),
                ("?"),
                ("foo", { color: Color::srgb(0.0, 0.8, 0.1) }, A),
            ]
        );
        drop(style);
    }
}
