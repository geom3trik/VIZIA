use crate::prelude::*;

pub(crate) fn animation_system(cx: &mut Context) -> bool {
    cx.style.play_pending_animations();

    // Tick all animations

    let time = Instant::now();

    let mut redraw_entities = Vec::new();
    let mut reflow_entities = Vec::new();
    let mut relayout_entities = Vec::new();

    // Properties which affect rendering
    // Opacity
    redraw_entities.extend(cx.style.opacity.tick(time));
    // Border Colour
    redraw_entities.extend(cx.style.border_color.tick(time));
    // Corner Radius
    redraw_entities.extend(cx.style.corner_top_left_radius.tick(time));
    redraw_entities.extend(cx.style.corner_top_right_radius.tick(time));
    redraw_entities.extend(cx.style.corner_bottom_left_radius.tick(time));
    redraw_entities.extend(cx.style.corner_bottom_right_radius.tick(time));
    // Background
    redraw_entities.extend(cx.style.background_color.tick(time));
    redraw_entities.extend(cx.style.background_image.tick(time));
    redraw_entities.extend(cx.style.background_size.tick(time));
    // Shadows
    redraw_entities.extend(cx.style.shadow.tick(time));
    reflow_entities.extend(cx.style.text_shadow.tick(time));
    // Transform
    redraw_entities.extend(cx.style.transform.tick(time));
    redraw_entities.extend(cx.style.transform_origin.tick(time));
    redraw_entities.extend(cx.style.translate.tick(time));
    redraw_entities.extend(cx.style.rotate.tick(time));
    redraw_entities.extend(cx.style.scale.tick(time));
    // Outline
    redraw_entities.extend(cx.style.outline_color.tick(time));
    redraw_entities.extend(cx.style.outline_offset.tick(time));
    redraw_entities.extend(cx.style.outline_width.tick(time));
    // Clip Path
    redraw_entities.extend(cx.style.clip_path.tick(time));

    redraw_entities.extend(cx.style.fill.tick(time));

    // Font Color
    reflow_entities.extend(cx.style.font_color.tick(time));
    // Font Size
    reflow_entities.extend(cx.style.font_size.tick(time));

    // Properties which affect layout
    relayout_entities.extend(cx.style.display.tick(time));
    // Border Width
    relayout_entities.extend(cx.style.border_width.tick(time));
    // Space
    relayout_entities.extend(cx.style.left.tick(time));
    relayout_entities.extend(cx.style.right.tick(time));
    relayout_entities.extend(cx.style.top.tick(time));
    relayout_entities.extend(cx.style.bottom.tick(time));
    // Size
    relayout_entities.extend(cx.style.width.tick(time));
    relayout_entities.extend(cx.style.height.tick(time));
    // Min/Max Size
    relayout_entities.extend(cx.style.max_width.tick(time));
    relayout_entities.extend(cx.style.max_height.tick(time));
    relayout_entities.extend(cx.style.min_width.tick(time));
    relayout_entities.extend(cx.style.min_height.tick(time));
    // Min/Max Space
    relayout_entities.extend(cx.style.min_left.tick(time));
    relayout_entities.extend(cx.style.max_left.tick(time));
    relayout_entities.extend(cx.style.min_right.tick(time));
    relayout_entities.extend(cx.style.max_right.tick(time));
    relayout_entities.extend(cx.style.min_top.tick(time));
    relayout_entities.extend(cx.style.max_top.tick(time));
    relayout_entities.extend(cx.style.min_bottom.tick(time));
    relayout_entities.extend(cx.style.max_bottom.tick(time));
    // Row/Col Between
    relayout_entities.extend(cx.style.row_between.tick(time));
    relayout_entities.extend(cx.style.col_between.tick(time));
    // Child Space
    relayout_entities.extend(cx.style.child_left.tick(time));
    relayout_entities.extend(cx.style.child_right.tick(time));
    relayout_entities.extend(cx.style.child_top.tick(time));
    relayout_entities.extend(cx.style.child_bottom.tick(time));

    reflow_entities.extend(cx.style.letter_spacing.tick(time));
    reflow_entities.extend(cx.style.word_spacing.tick(time));
    reflow_entities.extend(cx.style.line_height.tick(time));

    if !relayout_entities.is_empty() {
        cx.style.system_flags.set(SystemFlags::RELAYOUT, true);
    }

    for entity in redraw_entities.iter() {
        cx.needs_redraw(*entity);
    }

    for entity in reflow_entities.iter() {
        cx.style.text_construction.insert(*entity).unwrap();
    }

    !redraw_entities.is_empty() | !relayout_entities.is_empty() | !reflow_entities.is_empty()
}
