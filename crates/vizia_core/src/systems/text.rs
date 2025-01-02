use skia_safe::{
    font_arguments::VariationPosition,
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, RectHeightStyle,
        RectWidthStyle, StrutStyle, TextStyle,
    },
    BlendMode, FontArguments, FontStyle, Paint,
};
use vizia_storage::{LayoutChildIterator, LayoutTreeIterator};

use crate::{cache::CachedData, prelude::*};

pub(crate) fn text_system(cx: &mut Context) {
    let iterator = LayoutTreeIterator::full(&cx.tree);
    for entity in iterator {
        if !cx.style.text_construction.contains(entity) {
            continue;
        }

        if cx.style.text.contains(entity)
            && cx.style.display.get(entity).copied().unwrap_or_default() != Display::None
        {
            if let Some(paragraph) =
                build_paragraph(entity, &mut cx.style, &cx.tree, cx.text_context.font_collection())
            {
                cx.text_context.text_paragraphs.insert(entity, paragraph);
                cx.style.needs_relayout();
                cx.style.needs_text_layout(entity);
            }
        }
    }

    cx.style.text_construction.clear();
}

pub(crate) fn text_layout_system(cx: &mut Context) {
    let iterator = LayoutTreeIterator::full(&cx.tree);
    let mut redraw_entities = Vec::new();
    for entity in iterator {
        if !cx.style.text_layout.contains(entity) {
            continue;
        }

        if let Some(paragraph) = cx.text_context.text_paragraphs.get_mut(entity) {
            let bounds = cx.cache.get_bounds(entity);
            let padding_left = cx
                .style
                .child_left
                .get(entity)
                .copied()
                .unwrap_or_default()
                .to_px(bounds.width(), 0.0)
                * cx.style.scale_factor();
            let padding_right = cx
                .style
                .child_right
                .get(entity)
                .copied()
                .unwrap_or_default()
                .to_px(bounds.width(), 0.0)
                * cx.style.scale_factor();
            let text_bounds = cx
                .text_context
                .text_bounds
                .get(entity)
                .copied()
                .unwrap_or(bounds.shrink_sides(padding_left, 0.0, padding_right, 0.0));

            if !cx.style.width.get(entity).copied().unwrap_or_default().is_auto()
                && !cx.style.height.get(entity).copied().unwrap_or_default().is_auto()
            {
                if cx.style.text_overflow.get(entity).copied().unwrap_or_default()
                    == TextOverflow::Clip
                {
                    paragraph.layout(f32::MAX);
                    paragraph
                        .layout(text_bounds.width().max(paragraph.min_intrinsic_width() + 1.0));
                } else {
                    paragraph.layout(text_bounds.width());
                }
            }

            layout_span(&cx.style, &mut cx.cache, &cx.tree, entity, paragraph, bounds);

            redraw_entities.push(entity);
        }
    }
    for entity in redraw_entities {
        cx.needs_redraw(entity);
    }
    cx.style.text_layout.clear();
}

pub fn layout_span(
    style: &Style,
    cache: &mut CachedData,
    tree: &Tree<Entity>,
    entity: Entity,
    paragraph: &Paragraph,
    paragraph_bounds: BoundingBox,
) -> BoundingBox {
    let mut bounds = BoundingBox::default();
    if style.text_span.get(entity).copied().unwrap_or_default() {
        if let Some(range) = style.text_range.get(entity) {
            let rects = paragraph.get_rects_for_range(
                range.clone(),
                RectHeightStyle::Tight,
                RectWidthStyle::Tight,
            );

            let min_x = rects.iter().fold(1000000.0f32, |min, item| min.min(item.rect.x()));
            let min_y = rects.iter().fold(1000000.0f32, |min, item| min.min(item.rect.y()));
            let max_x = rects.iter().fold(0.0f32, |max, item| max.max(item.rect.right()));
            let max_y = rects.iter().fold(0.0f32, |max, item| max.max(item.rect.bottom()));

            bounds = BoundingBox::from_min_max(min_x, min_y, max_x, max_y);
        }
    }

    let iter = LayoutChildIterator::new(tree, entity);
    for child in iter {
        if bounds.width() == 0.0 && bounds.height() == 0.0 {
            bounds = layout_span(style, cache, tree, child, paragraph, paragraph_bounds);
        } else {
            bounds =
                bounds.union(&layout_span(style, cache, tree, child, paragraph, paragraph_bounds));
        }
    }

    if style.text_span.get(entity).copied().unwrap_or_default() {
        cache.bounds.insert(
            entity,
            BoundingBox::from_min_max(
                paragraph_bounds.x + bounds.x,
                paragraph_bounds.y + bounds.y,
                paragraph_bounds.x + bounds.right(),
                paragraph_bounds.y + bounds.bottom(),
            ),
        );
    }

    bounds
}

pub fn build_paragraph(
    entity: Entity,
    style: &mut Style,
    tree: &Tree<Entity>,
    font_collection: &FontCollection,
) -> Option<Paragraph> {
    let mut paragraph_style = ParagraphStyle::default();
    // paragraph_style.turn_hinting_off();

    // Overflow
    match style.text_overflow.get(entity) {
        Some(&TextOverflow::Ellipsis) => {
            paragraph_style.set_ellipsis("…");
        }

        Some(&TextOverflow::Clip) => {
            paragraph_style.set_ellipsis("");
        }

        _ => {
            paragraph_style.set_ellipsis("");
        }
    }

    // Line Clamp
    if let Some(line_clamp) = style.line_clamp.get(entity) {
        paragraph_style.set_max_lines(line_clamp.0 as usize);
    }

    if let Some(text_wrap) = style.text_wrap.get(entity).copied() {
        if !text_wrap {
            paragraph_style.set_max_lines(1);
        }
    }

    // Text Align
    paragraph_style.set_text_align(
        if let Some(text_align) = style.text_align.get(entity) {
            *text_align
        } else if let Some(Units::Stretch(_)) = style.child_left.get(entity) {
            if let Some(Units::Stretch(_)) = style.child_right.get(entity) {
                TextAlign::Center
            } else {
                TextAlign::Right
            }
        } else {
            TextAlign::Left
        }
        .into(),
    );

    // Line Height
    if let Some(line_height) = style.line_height.get(entity) {
        let font_size = style.font_size.get(entity).map_or(16.0, |f| f.0);

        let lh = match line_height {
            LineHeight::Normal => 1.2,
            LineHeight::Number(num) => *num,
            LineHeight::Length(l) => l.to_pixels(font_size, 1.0) / font_size,
        };

        if lh != 1.2 {
            let mut strut_style = StrutStyle::new();
            strut_style.set_strut_enabled(true);
            strut_style.set_force_strut_height(true);
            strut_style.set_height(lh);
            strut_style.set_height_override(true);

            // Font Families
            strut_style.set_font_families(
                style
                    .font_family
                    .get(entity)
                    .map(Vec::as_slice)
                    .unwrap_or(&[FamilyOwned::Generic(GenericFontFamily::SansSerif)]),
            );

            // Font Size
            let font_size = style.font_size.get(entity).map_or(16.0, |f| f.0);
            strut_style.set_font_size(font_size * style.scale_factor());

            // Font Style
            match (
                style.font_weight.get(entity),
                style.font_width.get(entity),
                style.font_slant.get(entity),
            ) {
                (None, None, None) => {}
                (weight, width, slant) => {
                    strut_style.set_font_style(FontStyle::new(
                        weight.copied().unwrap_or_default().into(),
                        width.copied().unwrap_or_default().into(),
                        slant.copied().unwrap_or_default().into(),
                    ));
                }
            }

            paragraph_style.set_strut_style(strut_style);
        }
    }

    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);

    add_block(style, tree, entity, &mut paragraph_builder, &mut 0);

    paragraph_builder.add_text("\u{200B}");
    paragraph_builder.build().into()
}

fn add_block(
    style: &mut Style,
    tree: &Tree<Entity>,
    entity: Entity,
    paragraph_builder: &mut ParagraphBuilder,
    current: &mut usize,
) {
    // let mut new_current = current;

    if let Some(text) = style.text.get(entity) {
        if !text.is_empty() {
            // Text Style

            let mut text_style = TextStyle::new();

            let font_color = style.font_color.get(entity).cloned().unwrap_or_default();

            if let Some(text_decoration_line) = style.text_decoration_line.get(entity).copied() {
                text_style.set_decoration_type(text_decoration_line.into());
                text_style.set_decoration_color(font_color);
            }

            // Font Families
            text_style.set_font_families(
                style
                    .font_family
                    .get(entity)
                    .map(Vec::as_slice)
                    .unwrap_or(&[FamilyOwned::Generic(GenericFontFamily::SansSerif)]),
            );

            let mut paint = Paint::default();

            // Font Color
            if let Some(font_color) = style.font_color.get(entity) {
                paint.set_color(*font_color);
                paint.set_anti_alias(false);
                paint.set_blend_mode(BlendMode::SrcOver);
            }

            if let Some(text_stroke) = style.text_stroke_width.get(entity) {
                paint.set_stroke_width(text_stroke.to_px().unwrap_or(0.0));
                paint.set_style(
                    (*style.text_stroke_style.get(entity).unwrap_or(&TextStrokeStyle::default()))
                        .into(),
                );
            }

            text_style.set_foreground_paint(&paint);

            // Background Color
            if let Some(background_color) = style.background_color.get(entity) {
                if style.text_span.get(entity).is_some() {
                    let mut paint = Paint::default();
                    paint.set_color(*background_color);
                    paint.set_anti_alias(false);
                    paint.set_blend_mode(BlendMode::SrcOver);
                    text_style.set_background_paint(&paint);
                }
            }

            // Font Size
            let font_size = style.font_size.get(entity).map_or(16.0, |f| f.0);
            text_style.set_font_size(font_size * style.scale_factor());

            // Font Style
            match (
                style.font_weight.get(entity),
                style.font_width.get(entity),
                style.font_slant.get(entity),
            ) {
                (None, None, None) => {}
                (weight, width, slant) => {
                    text_style.set_font_style(FontStyle::new(
                        weight.copied().unwrap_or_default().into(),
                        width.copied().unwrap_or_default().into(),
                        slant.copied().unwrap_or_default().into(),
                    ));
                }
            }

            // Font Variations
            if let Some(coordinates) = style.font_variation_settings.get(entity) {
                let coordinates = coordinates.iter().map(|c| c.0).collect::<Vec<_>>();
                text_style.set_font_arguments(&FontArguments::new().set_variation_design_position(
                    VariationPosition { coordinates: &coordinates },
                ));
            }

            // Letter Spacing
            if let Some(letter_spacing) = style.letter_spacing.get(entity) {
                text_style
                    .set_letter_spacing(letter_spacing.to_px().unwrap() * style.scale_factor());
            }

            // Word Spacing
            if let Some(word_spacing) = style.word_spacing.get(entity) {
                text_style.set_word_spacing(word_spacing.to_px().unwrap() * style.scale_factor());
            }

            paragraph_builder.push_style(&text_style);
            style.text_range.insert(entity, *current..*current + text.len());
            paragraph_builder.add_text(text.as_str());
            *current += text.len();
        }
    }

    let iter = LayoutChildIterator::new(tree, entity);
    for child in iter {
        if style.text_span.get(child).copied().unwrap_or_default() {
            add_block(style, tree, child, paragraph_builder, current);
        }
    }
}
