use crate::context::TreeProps;
use crate::skia;
use crate::{modifiers::ModalModel, prelude::*};

/// A tooltip view.
///
/// Should be used with the [tooltip](crate::modifiers::ActionModifiers::tooltip) modifier.
///
/// # Example
/// ```
/// # use vizia_core::prelude::*;
/// #
/// # enum AppEvent {
/// #     Action,
/// # }
/// #
/// # let cx = &mut Context::default();
/// #
/// Button::new(cx, |cx| Label::new(cx, "Text"))
///     .tooltip(|cx|{
///         Tooltip::new(cx, |cx|{
///             Label::new(cx, "Tooltip Text");
///         })
///     })

#[derive(Lens)]
pub struct Tooltip {
    placement: Placement,
    shift: Placement,
    show_arrow: bool,
    arrow_size: Length,
}

impl Tooltip {
    /// Creates a new Tooltip view with the given content.
    ///
    /// Should be used with the [tooltip](crate::modifiers::ActionModifiers::tooltip) modifier.
    ///
    /// # Example
    /// ```
    /// # use vizia_core::prelude::*;
    /// #
    /// # enum AppEvent {
    /// #     Action,
    /// # }
    /// #
    /// # let cx = &mut Context::default();
    /// #
    /// Button::new(cx, |cx| Label::new(cx, "Text"))
    ///     .tooltip(|cx|{
    ///         Tooltip::new(cx, |cx|{
    ///             Label::new(cx, "Tooltip Text");
    ///         })
    ///     })
    /// ```
    pub fn new(cx: &mut Context, content: impl FnOnce(&mut Context)) -> Handle<Self> {
        Self {
            placement: Placement::Bottom,
            shift: Placement::Bottom,
            show_arrow: true,
            arrow_size: Length::Value(LengthValue::Px(8.0)),
        }
        .build(cx, |cx| {
            Binding::new(cx, Tooltip::show_arrow, |cx, show_arrow| {
                if show_arrow.get(cx) {
                    Arrow::new(cx);
                }
            });
            (content)(cx);
        })
        .z_index(110)
        .hoverable(false)
        .position_type(PositionType::SelfDirected)
        .space(Pixels(0.0))
        .on_build(|ex| {
            ex.add_listener(move |tooltip: &mut Tooltip, ex, event| {
                let flag = ModalModel::tooltip_visible.get(ex);
                event.map(|window_event, meta| match window_event {
                    WindowEvent::MouseDown(_) => {
                        if flag && meta.origin != ex.current() {
                            ex.toggle_class("vis", false);
                        }
                    }

                    WindowEvent::MouseMove(x, y) => {
                        if tooltip.placement == Placement::Cursor && !x.is_nan() && !y.is_nan() {
                            let scale = ex.scale_factor();
                            let parent = ex.parent();
                            let parent_bounds = ex.cache.get_bounds(parent);
                            if parent_bounds.contains_point(*x, *y) {
                                ex.set_left(Pixels(
                                    ((*x - parent_bounds.x) - ex.bounds().width() / 2.0) / scale,
                                ));
                                ex.set_top(Pixels((*y - parent_bounds.y) / scale));
                            }
                        }
                    }

                    _ => {}
                });
            });
        })
    }
}

impl View for Tooltip {
    fn element(&self) -> Option<&'static str> {
        Some("tooltip")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            // Reposition popup if there isn't enough room for it.
            WindowEvent::GeometryChanged(_) => {
                let parent = cx.parent();
                let parent_bounds = cx.cache.get_bounds(parent);
                let bounds = cx.bounds();
                let window_bounds =
                    cx.cache.get_bounds(cx.parent_window().unwrap_or(Entity::root()));

                let arrow_size = self.arrow_size.to_px().unwrap() * cx.scale_factor();

                let mut available = AvailablePlacement::all();

                let top_start_bounds = BoundingBox::from_min_max(
                    parent_bounds.left(),
                    parent_bounds.top() - bounds.height() - arrow_size,
                    parent_bounds.left() + bounds.width(),
                    parent_bounds.top(),
                );

                available
                    .set(AvailablePlacement::TOP_START, window_bounds.contains(&top_start_bounds));

                let top_bounds = BoundingBox::from_min_max(
                    parent_bounds.center().0 - bounds.width() / 2.0,
                    parent_bounds.top() - bounds.height() - arrow_size,
                    parent_bounds.center().0 + bounds.width() / 2.0,
                    parent_bounds.top(),
                );

                available.set(AvailablePlacement::TOP, window_bounds.contains(&top_bounds));

                let top_end_bounds = BoundingBox::from_min_max(
                    parent_bounds.right() - bounds.width(),
                    parent_bounds.top() - bounds.height() - arrow_size,
                    parent_bounds.right(),
                    parent_bounds.top(),
                );

                available.set(AvailablePlacement::TOP_END, window_bounds.contains(&top_end_bounds));

                let bottom_start_bounds = BoundingBox::from_min_max(
                    parent_bounds.left(),
                    parent_bounds.bottom(),
                    parent_bounds.left() + bounds.width(),
                    parent_bounds.bottom() + bounds.height() + arrow_size,
                );

                available.set(
                    AvailablePlacement::BOTTOM_START,
                    window_bounds.contains(&bottom_start_bounds),
                );

                let bottom_bounds = BoundingBox::from_min_max(
                    parent_bounds.center().0 - bounds.width() / 2.0,
                    parent_bounds.bottom(),
                    parent_bounds.center().0 + bounds.width() / 2.0,
                    parent_bounds.bottom() + bounds.height() + arrow_size,
                );

                available.set(AvailablePlacement::BOTTOM, window_bounds.contains(&bottom_bounds));

                let bottom_end_bounds = BoundingBox::from_min_max(
                    parent_bounds.right() - bounds.width(),
                    parent_bounds.bottom(),
                    parent_bounds.right(),
                    parent_bounds.bottom() + bounds.height() + arrow_size,
                );

                available.set(
                    AvailablePlacement::BOTTOM_END,
                    window_bounds.contains(&bottom_end_bounds),
                );

                let left_start_bounds = BoundingBox::from_min_max(
                    parent_bounds.left() - bounds.width() - arrow_size,
                    parent_bounds.top(),
                    parent_bounds.left(),
                    parent_bounds.top() + bounds.height(),
                );

                available.set(
                    AvailablePlacement::LEFT_START,
                    window_bounds.contains(&left_start_bounds),
                );

                let left_bounds = BoundingBox::from_min_max(
                    parent_bounds.left() - bounds.width() - arrow_size,
                    parent_bounds.center().1 - bounds.height() / 2.0,
                    parent_bounds.left(),
                    parent_bounds.center().1 + bounds.height() / 2.0,
                );

                available.set(AvailablePlacement::LEFT, window_bounds.contains(&left_bounds));

                let left_end_bounds = BoundingBox::from_min_max(
                    parent_bounds.left() - bounds.width() - arrow_size,
                    parent_bounds.bottom() - bounds.height(),
                    parent_bounds.left(),
                    parent_bounds.bottom(),
                );

                available
                    .set(AvailablePlacement::LEFT_END, window_bounds.contains(&left_end_bounds));

                let right_start_bounds = BoundingBox::from_min_max(
                    parent_bounds.right(),
                    parent_bounds.top(),
                    parent_bounds.right() + bounds.width() + arrow_size,
                    parent_bounds.top() + bounds.height(),
                );

                available.set(
                    AvailablePlacement::RIGHT_START,
                    window_bounds.contains(&right_start_bounds),
                );

                let right_bounds = BoundingBox::from_min_max(
                    parent_bounds.right(),
                    parent_bounds.center().1 - bounds.height() / 2.0,
                    parent_bounds.right() + bounds.width() + arrow_size,
                    parent_bounds.center().1 + bounds.height() / 2.0,
                );

                available.set(AvailablePlacement::RIGHT, window_bounds.contains(&right_bounds));

                let right_end_bounds = BoundingBox::from_min_max(
                    parent_bounds.right(),
                    parent_bounds.bottom() - bounds.height(),
                    parent_bounds.right() + bounds.width() + arrow_size,
                    parent_bounds.bottom(),
                );

                available
                    .set(AvailablePlacement::RIGHT_END, window_bounds.contains(&right_end_bounds));

                let scale = cx.scale_factor();

                self.shift = self.placement.place(available);

                let arrow_size = self.arrow_size.to_px().unwrap();

                let translate = match self.shift {
                    Placement::Top => (
                        -(bounds.width() - parent_bounds.width()) / (2.0 * scale),
                        -bounds.height() / scale - arrow_size,
                    ),
                    Placement::TopStart => (0.0, -bounds.height() / scale - arrow_size),
                    Placement::TopEnd => (
                        -(bounds.width() - parent_bounds.width()) / scale,
                        -bounds.height() / scale - arrow_size,
                    ),
                    Placement::Bottom => (
                        -(bounds.width() - parent_bounds.width()) / (2.0 * scale),
                        parent_bounds.height() / scale + arrow_size,
                    ),
                    Placement::BottomStart => (0.0, parent_bounds.height() / scale + arrow_size),
                    Placement::BottomEnd => (
                        -(bounds.width() - parent_bounds.width()) / scale,
                        parent_bounds.height() / scale + arrow_size,
                    ),
                    Placement::LeftStart => (-(bounds.width() / scale) - arrow_size, 0.0),
                    Placement::Left => (
                        -(bounds.width() / scale) - arrow_size,
                        -(bounds.height() - parent_bounds.height()) / (2.0 * scale),
                    ),
                    Placement::LeftEnd => (
                        -(bounds.width() / scale) - arrow_size,
                        -(bounds.height() - parent_bounds.height()) / scale,
                    ),
                    Placement::RightStart => ((parent_bounds.width() / scale) + arrow_size, 0.0),
                    Placement::Right => (
                        (parent_bounds.width() / scale) + arrow_size,
                        -(bounds.height() - parent_bounds.height()) / (2.0 * scale),
                    ),
                    Placement::RightEnd => (
                        (parent_bounds.width() / scale) + arrow_size,
                        -(bounds.height() - parent_bounds.height()) / scale,
                    ),

                    _ => (0.0, 0.0),
                };

                cx.set_translate((Pixels(translate.0.round()), Pixels(translate.1.round())));
            }

            _ => {}
        });
    }
}

impl Handle<'_, Tooltip> {
    // TODO: Change this to use Res when lens value PR is merged
    /// Sets the position where the tooltip should appear relative to its parent element.
    /// Defaults to `Placement::Bottom`.
    pub fn placement(self, placement: Placement) -> Self {
        self.modify(|tooltip| {
            tooltip.placement = placement;
            tooltip.shift = placement;
        })
    }

    // TODO: Change this to use Res when lens value PR is merged
    /// Sets whether the tooltip should include an arrow. Defaults to true.
    pub fn arrow(self, show_arrow: bool) -> Self {
        self.modify(|tooltip| tooltip.show_arrow = show_arrow)
    }

    pub fn arrow_size(self, size: impl Into<Length>) -> Self {
        self.modify(|tooltip| tooltip.arrow_size = size.into())
    }
}

/// An arrow view used by the Tooltip view.
pub(crate) struct Arrow {}

impl Arrow {
    pub(crate) fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |_| {}).bind(Tooltip::shift, |mut handle, placement| {
            let (t, b) = match placement.get(&handle) {
                Placement::TopStart | Placement::Top | Placement::TopEnd => {
                    (Percentage(100.0), Stretch(1.0))
                }
                Placement::BottomStart | Placement::Bottom | Placement::BottomEnd => {
                    (Stretch(1.0), Percentage(100.0))
                }
                _ => (Stretch(1.0), Stretch(1.0)),
            };

            let (l, r) = match placement.get(&handle) {
                Placement::LeftStart | Placement::Left | Placement::LeftEnd => {
                    (Percentage(100.0), Stretch(1.0))
                }
                Placement::RightStart | Placement::Right | Placement::RightEnd => {
                    (Stretch(1.0), Percentage(100.0))
                }
                Placement::TopStart | Placement::BottomStart => {
                    // TODO: Use border radius
                    (Pixels(8.0), Stretch(1.0))
                }
                Placement::TopEnd | Placement::BottomEnd => {
                    // TODO: Use border radius
                    (Stretch(1.0), Pixels(8.0))
                }
                _ => (Stretch(1.0), Stretch(1.0)),
            };

            handle = handle.top(t).bottom(b).left(l).right(r);

            handle.bind(Tooltip::arrow_size, move |handle, arrow_size| {
                let arrow_size = arrow_size.get(&handle).to_px().unwrap_or(8.0);
                let (w, h) = match placement.get(&handle) {
                    Placement::Top
                    | Placement::Bottom
                    | Placement::TopStart
                    | Placement::BottomStart
                    | Placement::TopEnd
                    | Placement::BottomEnd => (Pixels(arrow_size * 2.0), Pixels(arrow_size)),

                    _ => (Pixels(arrow_size), Pixels(arrow_size * 2.0)),
                };

                handle.width(w).height(h);
            });
        })
    }
}

impl View for Arrow {
    fn element(&self) -> Option<&'static str> {
        Some("arrow")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let mut path = skia::Path::new();
        match Tooltip::shift.get(cx) {
            Placement::Bottom | Placement::BottomStart | Placement::BottomEnd => {
                path.move_to(bounds.bottom_left());
                path.line_to(bounds.center_top());
                path.line_to(bounds.bottom_right());
                path.line_to(bounds.bottom_left());
            }

            Placement::Top | Placement::TopStart | Placement::TopEnd => {
                path.move_to(bounds.top_left());
                path.line_to(bounds.center_bottom());
                path.line_to(bounds.top_right());
                path.line_to(bounds.top_left());
            }

            Placement::Left | Placement::LeftStart | Placement::LeftEnd => {
                path.move_to(bounds.top_left());
                path.line_to(bounds.center_right());
                path.line_to(bounds.bottom_left());
                path.line_to(bounds.top_left());
            }

            Placement::Right | Placement::RightStart | Placement::RightEnd => {
                path.move_to(bounds.top_right());
                path.line_to(bounds.center_left());
                path.line_to(bounds.bottom_right());
                path.line_to(bounds.top_right());
            }

            _ => {}
        }
        path.close();

        let bg = cx.background_color();

        let mut paint = skia::Paint::default();
        paint.set_color(bg);
        canvas.draw_path(&path, &paint);
    }
}
