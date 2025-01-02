use std::ops::Range;

use accesskit::ActionData;

use crate::prelude::*;

#[derive(Debug)]
enum SliderEventInternal {
    SetThumbSize(f32, f32),
    SetRange(Range<f32>),
    SetKeyboardFraction(f32),
}

#[derive(Clone, Debug, Default, Data)]
pub struct SliderDataInternal {
    pub orientation: Orientation,
    pub size: f32,
    pub thumb_size: f32,
    pub range: Range<f32>,
    pub step: f32,
    pub keyboard_fraction: f32,
}

/// The slider control can be used to select from a continuous set of values.
///
/// The slider control consists of three main parts, a **thumb** element which can be moved between the extremes of a linear **track**,
/// and an **active** element which fills the slider to indicate the current value.
///
/// The slider orientation is determined by its dimensions. If the slider width is greater than the height then the thumb
/// moves horizontally, whereas if the slider height is greater than the width the thumb moves vertically.
///
/// # Examples
///
/// ## Basic Slider
/// In the following example, a slider is bound to a value. The `on_changing` callback is used to send an event to mutate the
/// bound value when the slider thumb is moved, or if the track is clicked on.
/// ```
/// # use vizia_core::prelude::*;
/// # use vizia_derive::*;
/// # let mut cx = &mut Context::default();
/// # #[derive(Lens, Default)]
/// # pub struct AppData {
/// #     value: f32,
/// # }
/// # impl Model for AppData {}
/// # AppData::default().build(cx);
/// Slider::new(cx, AppData::value)
///     .on_changing(|cx, value| {
///         debug!("Slider on_changing: {}", value);
///     });
/// ```
///
/// ## Slider with Label
/// ```
/// # use vizia_core::prelude::*;
/// # use vizia_derive::*;
/// # let mut cx = &mut Context::default();
/// # #[derive(Lens, Default)]
/// # pub struct AppData {
/// #     value: f32,
/// # }
/// # impl Model for AppData {}
/// # AppData::default().build(cx);
/// HStack::new(cx, |cx|{
///     Slider::new(cx, AppData::value)
///         .on_changing(|cx, value| {
///             debug!("Slider on_changing: {}", value);
///         });
///     Label::new(cx, AppData::value.map(|val| format!("{:.2}", val)));
/// });
/// ```
#[derive(Lens)]
pub struct Slider<L: Lens> {
    lens: L,
    is_dragging: bool,
    internal: SliderDataInternal,
    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,
}

impl<L> Slider<L>
where
    L: Lens<Target = f32>,
{
    /// Creates a new slider bound to the value targeted by the lens.
    ///
    /// # Example
    /// ```
    /// # use vizia_core::prelude::*;
    /// # use vizia_derive::*;
    /// # let mut cx = &mut Context::default();
    /// # #[derive(Lens, Default)]
    /// # pub struct AppData {
    /// #     value: f32,
    /// # }
    /// # impl Model for AppData {}
    /// # AppData::default().build(cx);
    /// Slider::new(cx, AppData::value)
    ///     .on_changing(|cx, value| {
    ///         debug!("Slider on_changing: {}", value);
    ///     });
    /// ```
    pub fn new(cx: &mut Context, lens: L) -> Handle<Self> {
        Self {
            lens,
            is_dragging: false,

            internal: SliderDataInternal {
                orientation: Orientation::Horizontal,
                thumb_size: 0.0,
                size: 0.0,
                range: 0.0..1.0,
                step: 0.01,
                keyboard_fraction: 0.1,
            },

            on_changing: None,
        }
        .build(cx, move |cx| {
            Binding::new(cx, Slider::<L>::internal, move |cx, slider_data| {
                ZStack::new(cx, move |cx| {
                    let slider_data = slider_data.get(cx);
                    let thumb_size = slider_data.thumb_size;
                    let orientation = slider_data.orientation;
                    let size = slider_data.size;
                    let range = slider_data.range;

                    // Active track
                    Element::new(cx).class("active").bind(lens, move |handle, value| {
                        let val = value.get(&handle);

                        let normal_val = (val - range.start) / (range.end - range.start);
                        let min = thumb_size / size;
                        let max = 1.0;
                        let dx = min + normal_val * (max - min);

                        if orientation == Orientation::Horizontal {
                            handle
                                .height(Stretch(1.0))
                                .left(Pixels(0.0))
                                .right(Stretch(1.0))
                                .width(Percentage(dx * 100.0));
                        } else {
                            handle
                                .width(Stretch(1.0))
                                .top(Stretch(1.0))
                                .bottom(Pixels(0.0))
                                .height(Percentage(dx * 100.0));
                        }
                    });

                    // Thumb
                    Element::new(cx)
                        .class("thumb")
                        .on_geo_changed(|cx, geo| {
                            if geo.contains(GeoChanged::WIDTH_CHANGED)
                                || geo.contains(GeoChanged::HEIGHT_CHANGED)
                            {
                                let bounds = cx.bounds();
                                cx.emit(SliderEventInternal::SetThumbSize(bounds.w, bounds.h));
                            }
                        })
                        .bind(lens, move |handle, value| {
                            let val = value.get(&handle);
                            let normal_val = (val - range.start) / (range.end - range.start);
                            let px = normal_val * (1.0 - (thumb_size / size));
                            if orientation == Orientation::Horizontal {
                                handle
                                    .right(Stretch(1.0))
                                    .top(Stretch(1.0))
                                    .bottom(Stretch(1.0))
                                    .left(Percentage(100.0 * px));
                            } else {
                                handle
                                    .top(Stretch(1.0))
                                    .left(Stretch(1.0))
                                    .right(Stretch(1.0))
                                    .bottom(Percentage(100.0 * px));
                            }
                        });
                });
            });
        })
        .role(Role::Slider)
        .numeric_value(lens.map(|val| (*val as f64 * 100.0).round() / 100.0))
        .text_value(lens.map(|val| {
            let v = (*val as f64 * 100.0).round() / 100.0;
            format!("{}", v)
        }))
        .navigable(true)
    }

    pub fn custom<F>(cx: &mut Context, lens: L, content: F) -> Handle<Self>
    where
        F: FnOnce(&mut Context),
    {
        Self {
            lens,
            is_dragging: false,

            internal: SliderDataInternal {
                orientation: Orientation::Horizontal,
                thumb_size: 0.0,
                size: 0.0,
                range: 0.0..1.0,
                step: 0.01,
                keyboard_fraction: 0.1,
            },

            on_changing: None,
        }
        .build(cx, move |cx| {
            (content)(cx);
        })
        .navigable(true)
    }
}

impl<L: Lens<Target = f32>> View for Slider<L> {
    fn element(&self) -> Option<&'static str> {
        Some("slider")
    }

    fn accessibility(&self, _cx: &mut AccessContext, node: &mut AccessNode) {
        node.set_numeric_value_step(self.internal.step as f64);
        node.set_min_numeric_value(self.internal.range.start as f64);
        node.set_max_numeric_value(self.internal.range.end as f64);
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|slider_event_internal, _| match slider_event_internal {
            SliderEventInternal::SetThumbSize(width, height) => match self.internal.orientation {
                Orientation::Horizontal => {
                    self.internal.thumb_size = *width;
                }

                Orientation::Vertical => {
                    self.internal.thumb_size = *height;
                }
            },

            SliderEventInternal::SetRange(range) => {
                self.internal.range = range.clone();
            }

            SliderEventInternal::SetKeyboardFraction(keyboard_fraction) => {
                self.internal.keyboard_fraction = *keyboard_fraction;
            }
        });

        event.map(|window_event, _| match window_event {
            WindowEvent::GeometryChanged(_) => {
                let current = cx.current();
                let width = cx.cache.get_width(current);
                let height = cx.cache.get_height(current);

                if width >= height {
                    self.internal.orientation = Orientation::Horizontal;
                    self.internal.size = width;
                } else {
                    self.internal.orientation = Orientation::Vertical;
                    self.internal.size = height;
                }
            }

            WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                if !cx.is_disabled() {
                    self.is_dragging = true;
                    cx.capture();
                    cx.focus_with_visibility(false);
                    cx.with_current(Entity::root(), |cx| {
                        cx.set_pointer_events(false);
                    });

                    let thumb_size = self.internal.thumb_size;
                    let min = self.internal.range.start;
                    let max = self.internal.range.end;
                    let step = self.internal.step;

                    let current = cx.current();
                    let width = cx.cache.get_width(current);
                    let height = cx.cache.get_height(current);
                    let posx = cx.cache.get_posx(current);
                    let posy = cx.cache.get_posy(current);

                    let mut dx = match self.internal.orientation {
                        Orientation::Horizontal => {
                            (cx.mouse.left.pos_down.0 - posx - thumb_size / 2.0)
                                / (width - thumb_size)
                        }

                        Orientation::Vertical => {
                            (height - (cx.mouse.left.pos_down.1 - posy) - thumb_size / 2.0)
                                / (height - thumb_size)
                        }
                    };

                    dx = dx.clamp(0.0, 1.0);

                    let mut val = min + dx * (max - min);

                    val = step * (val / step).ceil();
                    val = val.clamp(min, max);

                    if let Some(callback) = self.on_changing.take() {
                        (callback)(cx, val);

                        self.on_changing = Some(callback);
                    }
                }
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                self.is_dragging = false;
                cx.focus_with_visibility(false);
                cx.release();
                cx.with_current(Entity::root(), |cx| {
                    cx.set_pointer_events(true);
                });
            }

            WindowEvent::MouseMove(x, y) => {
                if self.is_dragging {
                    let thumb_size = self.internal.thumb_size;

                    let min = self.internal.range.start;
                    let max = self.internal.range.end;
                    let step = self.internal.step;

                    let current = cx.current();
                    let width = cx.cache.get_width(current);
                    let height = cx.cache.get_height(current);
                    let posx = cx.cache.get_posx(current);
                    let posy = cx.cache.get_posy(current);

                    let mut dx = match self.internal.orientation {
                        Orientation::Horizontal => {
                            (*x - posx - thumb_size / 2.0) / (width - thumb_size)
                        }

                        Orientation::Vertical => {
                            (height - (*y - posy) - thumb_size / 2.0) / (height - thumb_size)
                        }
                    };

                    dx = dx.clamp(0.0, 1.0);

                    let mut val = min + dx * (max - min);

                    val = step * (val / step).ceil();
                    val = val.clamp(min, max);

                    if let Some(callback) = &self.on_changing {
                        (callback)(cx, val);
                    }
                }
            }

            WindowEvent::KeyDown(Code::ArrowUp | Code::ArrowRight, _) => {
                let min = self.internal.range.start;
                let max = self.internal.range.end;
                let step = self.internal.step;
                let mut val = self.lens.get(cx) + step;
                // val = step * (val / step).ceil();
                val = val.clamp(min, max);
                if let Some(callback) = &self.on_changing {
                    (callback)(cx, val);
                }
            }

            WindowEvent::KeyDown(Code::ArrowDown | Code::ArrowLeft, _) => {
                let min = self.internal.range.start;
                let max = self.internal.range.end;
                let step = self.internal.step;
                let mut val = self.lens.get(cx) - step;
                // val = step * (val / step).ceil();
                val = val.clamp(min, max);
                if let Some(callback) = &self.on_changing {
                    (callback)(cx, val);
                }
            }

            WindowEvent::ActionRequest(action) => match action.action {
                Action::Increment => {
                    let min = self.internal.range.start;
                    let max = self.internal.range.end;
                    let step = self.internal.step;
                    let mut val = self.lens.get(cx) + step;
                    val = step * (val / step).ceil();
                    val = val.clamp(min, max);
                    if let Some(callback) = &self.on_changing {
                        (callback)(cx, val);
                    }
                }

                Action::Decrement => {
                    let min = self.internal.range.start;
                    let max = self.internal.range.end;
                    let step = self.internal.step;
                    let mut val = self.lens.get(cx) - step;
                    val = step * (val / step).ceil();
                    val = val.clamp(min, max);
                    if let Some(callback) = &self.on_changing {
                        (callback)(cx, val);
                    }
                }

                Action::SetValue => {
                    if let Some(ActionData::NumericValue(val)) = action.data {
                        let min = self.internal.range.start;
                        let max = self.internal.range.end;
                        let mut v = val as f32;
                        v = v.clamp(min, max);
                        if let Some(callback) = &self.on_changing {
                            (callback)(cx, v);
                        }
                    }
                }

                _ => {}
            },

            _ => {}
        });
    }
}

impl<L: Lens> Handle<'_, Slider<L>> {
    /// Sets the callback triggered when the slider value is changing (dragging).
    ///
    /// Takes a closure which triggers when the slider value is changing,
    /// either by pressing the track or dragging the thumb along the track.
    ///
    /// # Example
    ///
    /// ```
    /// # use vizia_core::prelude::*;
    /// # use vizia_derive::*;
    /// # let mut cx = &mut Context::default();
    /// # #[derive(Lens, Default)]
    /// # pub struct AppData {
    /// #     value: f32,
    /// # }
    /// # impl Model for AppData {}
    /// # AppData::default().build(cx);
    /// Slider::new(cx, AppData::value)
    ///     .on_changing(|cx, value| {
    ///         debug!("Slider on_changing: {}", value);
    ///     });
    /// ```
    pub fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, f32),
    {
        self.modify(|slider| slider.on_changing = Some(Box::new(callback)))
    }

    /// Sets the range of the slider.
    ///
    /// If the bound data is outside of the range then the slider will clip to min/max of the range.
    ///
    /// # Example
    /// ```
    /// # use vizia_core::prelude::*;
    /// # use vizia_derive::*;
    /// # let mut cx = &mut Context::default();
    /// # #[derive(Lens, Default)]
    /// # pub struct AppData {
    /// #     value: f32,
    /// # }
    /// # impl Model for AppData {}
    /// # AppData::default().build(cx);
    /// Slider::new(cx, AppData::value)
    ///     .range(-20.0..50.0)
    ///     .on_changing(|cx, value| {
    ///         debug!("Slider on_changing: {}", value);
    ///     });
    /// ```
    pub fn range<U: Into<Range<f32>>>(self, range: impl Res<U>) -> Self {
        self.bind(range, |handle, val| {
            let range = val.get(&handle).into();
            handle.modify(|slider: &mut Slider<L>| slider.internal.range = range);
        })
    }

    pub fn step(self, step: f32) -> Self {
        self.modify(|slider: &mut Slider<L>| slider.internal.step = step)
    }

    /// Sets the fraction of a slider that a press of an arrow key will change.
    ///
    /// # Example
    /// ```
    /// # use vizia_core::prelude::*;
    /// # use vizia_derive::*;
    /// # let mut cx = &mut Context::default();
    /// # #[derive(Lens, Default)]
    /// # pub struct AppData {
    /// #     value: f32,
    /// # }
    /// # impl Model for AppData {}
    /// # AppData::default().build(cx);
    /// Slider::new(cx, AppData::value)
    ///     .keyboard_fraction(0.05)
    ///     .on_changing(|cx, value| {
    ///         debug!("Slider on_changing: {}", value);
    ///     });
    /// ```
    pub fn keyboard_fraction(self, keyboard_fraction: f32) -> Self {
        self.cx.emit_to(self.entity, SliderEventInternal::SetKeyboardFraction(keyboard_fraction));

        self
    }
}

enum NamedSliderEvent {
    Change(f32),
}

#[derive(Lens)]
pub struct NamedSlider {
    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,
}

impl NamedSlider {
    pub fn new<L, T>(cx: &mut Context, lens: L, name: impl Res<T>) -> Handle<Self>
    where
        L: Lens<Target = f32>,
        T: ToString,
    {
        let name = name.get(cx).to_string();
        Self { on_changing: None }
            .build(cx, move |cx| {
                Binding::new(cx, lens, move |cx, lens| {
                    Textbox::new(cx, lens.map(|v| format!("{:.2}", v))).on_submit(|cx, txt, _| {
                        if let Ok(val) = txt.parse() {
                            cx.emit(NamedSliderEvent::Change(val));
                        }
                    });
                });
                Slider::custom(cx, lens, move |cx| {
                    Binding::new(cx, Slider::<L>::internal, move |cx, slider_data| {
                        ZStack::new(cx, |cx| {
                            let slider_data = slider_data.get(cx);
                            let thumb_size = slider_data.thumb_size;
                            let size = slider_data.size;
                            let range = slider_data.range;

                            // Active track
                            Element::new(cx).class("active").bind(lens, move |handle, value| {
                                let val = value.get(&handle);
                                let normal_val = (val - range.start) / (range.end - range.start);
                                let min = thumb_size / size;
                                let max = 1.0;
                                let dx = min + normal_val * (max - min);

                                handle
                                    .height(Stretch(1.0))
                                    .left(Pixels(0.0))
                                    .right(Stretch(1.0))
                                    .width(Percentage(dx * 100.0));
                            });

                            Label::new(cx, &name);
                        });
                    })
                })
                .on_changing(|cx, v| cx.emit(NamedSliderEvent::Change(v)));
            })
            .layout_type(LayoutType::Row)
    }
}

impl View for NamedSlider {
    fn element(&self) -> Option<&'static str> {
        Some("namedslider")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _| match e {
            NamedSliderEvent::Change(v) => {
                if let Some(callback) = self.on_changing.take() {
                    (callback)(cx, *v);

                    self.on_changing = Some(callback);
                }
            }
        })
    }
}

impl Handle<'_, NamedSlider> {
    pub fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, f32),
    {
        self.modify(|slider| slider.on_changing = Some(Box::new(callback)))
    }

    pub fn range(self, range: Range<f32>) -> Self {
        self.cx.emit_custom(
            Event::new(SliderEventInternal::SetRange(range))
                .target(self.entity())
                .origin(self.entity())
                .propagate(Propagation::Subtree),
        );

        self
    }

    pub fn keyboard_fraction(self, keyboard_fraction: f32) -> Self {
        self.cx.emit_custom(
            Event::new(SliderEventInternal::SetKeyboardFraction(keyboard_fraction))
                .origin(self.entity())
                .propagate(Propagation::Subtree),
        );

        self
    }
}
