use super::internal;
use crate::prelude::*;

/// Modifiers for changing the text properties of a view.
pub trait TextModifiers: internal::Modifiable {
    /// Sets the text content of the view.
    fn text<T: ToStringLocalized>(mut self, value: impl Res<T>) -> Self {
        let entity = self.entity();
        let current = self.current();
        self.context().with_current(current, |cx| {
            value.set_or_bind(cx, entity, move |cx, val| {
                let cx: &mut EventContext<'_> = &mut EventContext::new_with_current(cx, entity);
                let text_data = val.get(cx).to_string_local(cx);
                // cx.text_context.set_text(entity, &text_data);
                cx.style.text.insert(entity, text_data);

                cx.style.needs_text_update(entity);
                cx.needs_relayout();
                cx.needs_redraw();
            });
        });

        self
    }

    modifier!(
        /// Sets the font that should be used by the view.
        ///
        /// The font name refers to the name assigned when the font is added to context.
        font_family,
        Vec<FamilyOwned>,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the font weight that should be used by the view.
        font_weight,
        FontWeight,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the font style that should be used by the view.
        font_slant,
        FontSlant,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the font stretch that should be used by the view if the font supports it.
        font_width,
        FontWidth,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the font variation settings that should be used by the view.
        font_variation_settings,
        Vec<FontVariation>,
        SystemFlags::REFLOW
    );

    /// Sets the text color of the view.
    fn color<U: Clone + Into<Color>>(mut self, value: impl Res<U>) -> Self {
        let entity = self.entity();
        let current = self.current();
        self.context().with_current(current, move |cx| {
            value.set_or_bind(cx, entity, move |cx, v| {
                cx.style.font_color.insert(entity, v.get(cx).into());
                cx.style.needs_text_update(entity);
                cx.needs_redraw(entity);
            });
        });
        self
    }

    /// Sets the font size of the view.
    fn font_size<U: Into<FontSize>>(mut self, value: impl Res<U>) -> Self {
        let entity = self.entity();
        let current = self.current();
        self.context().with_current(current, move |cx| {
            value.set_or_bind(cx, entity, move |cx, v| {
                cx.style.font_size.insert(cx.current, v.get(cx).into());
                cx.style.needs_text_update(entity);
            });
        });
        self
    }

    modifier!(
        /// Sets the ext caret color of the view.
        caret_color,
        Color,
        SystemFlags::REDRAW
    );

    modifier!(
        /// Sets the color used to highlight selected text within the view.
        selection_color,
        Color,
        SystemFlags::REDRAW
    );

    modifier!(
        /// Sets whether the text of the view should be allowed to wrap.
        text_wrap,
        bool,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the horizontal alignment of text within the view.
        text_align,
        TextAlign,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the text overflow.
        text_overflow,
        TextOverflow,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the max number of .
        line_clamp,
        LineClamp,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the max number of .
        text_decoration_line,
        TextDecorationLine,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the width of the text stroke.
        /// This sets Skia's [`skia_safe::textlayout::TextStyle`]'s foreground [`skia_safe::Paint`] to
        /// draw a stroke on the text.
        ///
        /// See also [`Self::text_stroke_style`].
        text_stroke_width,
        Length,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the letter spacing.
        letter_spacing,
        Length,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the paint style of the text stroke.
        /// You can either draw text with a stroke, or just the stroke outline.
        ///
        /// If you desire to paint the text stroke in a separate colour to the
        /// fill property, Skia does not seem to support this (see the
        /// [discussion on the PR to add text stroke](https://github.com/vizia/vizia/pull/528)).
        ///
        /// A workaround would involve a [`ZStack`] to overlay two copies of the
        /// same text, one with a stroke and one without. These can have two
        /// different colours. See also the [Flutter documentation](
        /// https://api.flutter.dev/flutter/painting/TextStyle-class.html#painting.TextStyle.6)
        /// on achieving this.
        text_stroke_style,
        TextStrokeStyle,
        SystemFlags::REFLOW
    );

    modifier!(
        /// Sets the word spacing.
        word_spacing,
        Length,
        SystemFlags::REFLOW
    );
}

impl<V> TextModifiers for Handle<'_, V> {}
