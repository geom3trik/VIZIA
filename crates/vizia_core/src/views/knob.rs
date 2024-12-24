#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::skia;
use morphorm::Units;

use crate::prelude::*;

static DEFAULT_DRAG_SCALAR: f32 = 0.0042;
static DEFAULT_WHEEL_SCALAR: f32 = 0.005;
static DEFAULT_ARROW_SCALAR: f32 = 0.1;
static DEFAULT_MODIFIER_SCALAR: f32 = 0.04;

use std::{default, f32::consts::PI};

pub struct Knob<L> {
    lens: L,
    default_normal: f32,

    is_dragging: bool,
    prev_drag_y: f32,
    continuous_normal: f32,

    drag_scalar: f32,
    wheel_scalar: f32,
    arrow_scalar: f32,
    modifier_scalar: f32,

    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,
}

impl<L: Lens<Target = f32>> Knob<L> {
    pub fn new(
        cx: &mut Context,
        normalized_default: impl Res<f32>,
        lens: L,
        centered: bool,
    ) -> Handle<Self> {
        Self {
            lens,
            default_normal: normalized_default.get(cx),

            is_dragging: false,
            prev_drag_y: 0.0,
            continuous_normal: lens.get(cx),

            drag_scalar: DEFAULT_DRAG_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            arrow_scalar: DEFAULT_ARROW_SCALAR,
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,

            on_changing: None,
        }
        .build(cx, move |cx| {
            ZStack::new(cx, move |cx| {
                ArcTrack::new(
                    cx,
                    centered,
                    Percentage(100.0),
                    Percentage(15.0),
                    -240.,
                    60.,
                    KnobMode::Continuous,
                )
                .value(lens)
                .class("knob-track");

                HStack::new(cx, |cx| {
                    Element::new(cx).class("knob-tick");
                })
                .rotate(lens.map(|v| Angle::Deg(*v * 300.0 - 150.0)))
                .class("knob-head");
            });
        })
        .navigable(true)
    }

    pub fn custom<F, V: View>(
        cx: &mut Context,
        default_normal: f32,
        lens: L,
        content: F,
    ) -> Handle<'_, Self>
    where
        F: 'static + Fn(&mut Context, L) -> Handle<V>,
    {
        Self {
            lens,
            default_normal,

            is_dragging: false,
            prev_drag_y: 0.0,
            continuous_normal: lens.get(cx),

            drag_scalar: DEFAULT_DRAG_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            arrow_scalar: DEFAULT_ARROW_SCALAR,
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,

            on_changing: None,
        }
        .build(cx, move |cx| {
            ZStack::new(cx, move |cx| {
                (content)(cx, lens).width(Percentage(100.0)).height(Percentage(100.0));
            });
        })
    }
}

impl<L: Lens<Target = f32>> Handle<'_, Knob<L>> {
    pub fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut EventContext, f32),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(knob) = view.downcast_mut::<Knob<L>>() {
                knob.on_changing = Some(Box::new(callback));
            }
        }

        self
    }
}

impl<L: Lens<Target = f32>> View for Knob<L> {
    fn element(&self) -> Option<&'static str> {
        Some("knob")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        let move_virtual_slider = |self_ref: &mut Self, cx: &mut EventContext, new_normal: f32| {
            self_ref.continuous_normal = new_normal.clamp(0.0, 1.0);

            if let Some(callback) = &self_ref.on_changing {
                (callback)(cx, self_ref.continuous_normal);
            }
        };

        event.map(|window_event, _| match window_event {
            WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                self.is_dragging = true;
                self.prev_drag_y = cx.mouse.left.pos_down.1;

                cx.capture();
                cx.focus_with_visibility(false);

                self.continuous_normal = self.lens.get(cx);
            }

            WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                self.is_dragging = false;

                self.continuous_normal = self.lens.get(cx);

                cx.release();
            }

            WindowEvent::MouseMove(_, y) => {
                if self.is_dragging && !cx.is_disabled() {
                    let mut delta_normal = (*y - self.prev_drag_y) * self.drag_scalar;

                    self.prev_drag_y = *y;

                    if cx.modifiers.shift() {
                        delta_normal *= self.modifier_scalar;
                    }

                    let new_normal = self.continuous_normal - delta_normal;

                    move_virtual_slider(self, cx, new_normal);
                }
            }

            WindowEvent::MouseScroll(_, y) => {
                if *y != 0.0 {
                    let delta_normal = -*y * self.wheel_scalar;

                    let new_normal = self.continuous_normal - delta_normal;

                    move_virtual_slider(self, cx, new_normal);
                }
            }

            WindowEvent::MouseDoubleClick(button) if *button == MouseButton::Left => {
                self.is_dragging = false;

                move_virtual_slider(self, cx, self.default_normal);
            }

            WindowEvent::KeyDown(Code::ArrowUp | Code::ArrowRight, _) => {
                self.continuous_normal = self.lens.get(cx);
                move_virtual_slider(self, cx, self.continuous_normal + self.arrow_scalar);
            }

            WindowEvent::KeyDown(Code::ArrowDown | Code::ArrowLeft, _) => {
                self.continuous_normal = self.lens.get(cx);
                move_virtual_slider(self, cx, self.continuous_normal - self.arrow_scalar);
            }

            _ => {}
        });
    }
}

/// Makes a knob that represents the current value with an arc
pub struct ArcTrack {
    angle_start: f32,
    angle_end: f32,
    radius: Units,
    span: Units,
    normalized_value: f32,

    center: bool,
    mode: KnobMode,
}

impl ArcTrack {
    pub fn new(
        cx: &mut Context,
        center: bool,
        radius: Units,
        span: Units,
        angle_start: f32,
        angle_end: f32,
        mode: KnobMode,
    ) -> Handle<Self> {
        Self {
            // angle_start: -150.0,
            // angle_end: 150.0,
            angle_start,
            angle_end,
            radius,
            span,

            normalized_value: 0.5,

            center,
            mode,
        }
        .build(cx, |_| {})
    }
}

impl View for ArcTrack {
    fn element(&self) -> Option<&'static str> {
        Some("arctrack")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let opacity = cx.opacity();

        let foreground_color = cx.font_color();

        let background_color = cx.background_color();

        let bounds = cx.bounds();

        // Calculate arc center
        let centerx = bounds.x + 0.5 * bounds.w;
        let centery = bounds.y + 0.5 * bounds.h;

        // Convert start and end angles to radians and rotate origin direction to be upwards instead of to the right
        let start = self.angle_start;
        let end = self.angle_end;

        let parent = cx.tree.get_parent(cx.current).unwrap();

        let parent_width = cx.cache.get_width(parent);

        // Convert radius and span into screen coordinates
        let radius = self.radius.to_px(parent_width / 2.0, 0.0);
        // default value of span is 15 % of radius. Original span value was 16.667%
        let span = self.span.to_px(radius, 0.0);

        // Draw the track arc
        let path = skia::Path::new();
        // path.arc(centerx, centery, radius - span / 2.0, end, start, Solidity::Solid);
        let oval = skia::Rect::new(bounds.left(), bounds.top(), bounds.right(), bounds.bottom());

        let mut paint = skia::Paint::default();
        paint.set_color(background_color);
        paint.set_stroke_width(span);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_style(skia::PaintStyle::Stroke);
        // canvas.draw_path(&path, &paint);
        canvas.draw_arc(oval, start, end - start, true, &paint);

        // Draw the active arc
        let mut path = skia::Path::new();

        let value = match self.mode {
            KnobMode::Continuous => self.normalized_value,
            // snapping
            KnobMode::Discrete(steps) => {
                (self.normalized_value * (steps - 1) as f32).floor() / (steps - 1) as f32
            }
        };

        if self.center {
            let center = -90.0;

            if value <= 0.5 {
                let current = value * 2.0 * (center - start) + start;
                path.arc_to(oval.with_inset((span / 2.0, span / 2.0)), start, current, false);
            } else {
                let current = (value * 2.0 - 1.0) * (end - center);
                path.arc_to(oval.with_inset((span / 2.0, span / 2.0)), center, current, false);
            }
        } else {
            let current = value * (end - start) + start;
            path.arc_to(oval.with_inset((span / 2.0, span / 2.0)), start, current - start, false);
        }

        let mut paint = skia::Paint::default();
        paint.set_color(foreground_color);
        paint.set_stroke_width(span);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_style(skia::PaintStyle::Stroke);
        paint.set_anti_alias(true);
        canvas.draw_path(&path, &paint);
    }
}

impl Handle<'_, ArcTrack> {
    pub fn value<L: Lens<Target = f32>>(self, lens: L) -> Self {
        let entity = self.entity;
        Binding::new(self.cx, lens, move |cx, value| {
            let value = value.get(cx);
            if let Some(view) = cx.views.get_mut(&entity) {
                if let Some(knob) = view.downcast_mut::<ArcTrack>() {
                    knob.normalized_value = value;
                    cx.needs_redraw(entity);
                }
            }
        });

        self
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum KnobMode {
    Discrete(usize),
    #[default]
    Continuous,
}

/// Adds tickmarks to a knob to show the steps that a knob can be set to.
/// When added to a knob, the knob should be made smaller (depending on span),
/// so the knob doesn't overlap with the tick marks
pub struct Ticks {
    angle_start: f32,
    angle_end: f32,
    radius: Units,
    // TODO: should this be renamed to inner_radius?
    tick_len: Units,
    tick_width: Units,
    // steps: u32,
    mode: KnobMode,
}
impl Ticks {
    pub fn new(
        cx: &mut Context,
        radius: Units,
        tick_len: Units,
        tick_width: Units,
        arc_len: f32,
        mode: KnobMode,
    ) -> Handle<Self> {
        Self {
            // angle_start: -150.0,
            // angle_end: 150.0,
            angle_start: -arc_len / 2.0,
            angle_end: arc_len / 2.0,
            radius,
            tick_len,
            tick_width,
            mode,
        }
        .build(cx, |_| {})
    }
}

impl View for Ticks {
    fn element(&self) -> Option<&'static str> {
        Some("ticks")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let opacity = cx.opacity();
        //let mut background_color: femtoskia::Color = cx.current.get_background_color(cx).into();
        // background_color.set_alphaf(background_color.a * opacity);
        let foreground_color = cx.background_color();
        // let background_color = femtoskia::Color::rgb(54, 54, 54);
        //et mut foreground_color = femtoskia::Color::rgb(50, 50, 200);
        let bounds = cx.bounds();
        // Clalculate arc center
        let centerx = bounds.x + 0.5 * bounds.w;
        let centery = bounds.y + 0.5 * bounds.h;
        // Convert start and end angles to radians and rotate origin direction to be upwards instead of to the right
        let start = self.angle_start.to_radians() - PI / 2.0;
        let end = self.angle_end.to_radians() - PI / 2.0;
        let parent = cx.tree.get_parent(cx.current).unwrap();
        let parent_width = cx.cache.get_width(parent);
        // Convert radius and span into screen coordinates
        let radius = self.radius.to_px(parent_width / 2.0, 0.0);
        // default value of span is 15 % of radius. Original span value was 16.667%
        let tick_len = self.tick_len.to_px(radius, 0.0);
        let line_width = self.tick_width.to_px(radius, 0.0);
        // Draw ticks
        let mut path = skia::Path::new();
        match self.mode {
            // can't really make ticks for a continuous knob
            KnobMode::Continuous => return,
            KnobMode::Discrete(steps) => {
                for n in 0..steps {
                    let a = n as f32 / (steps - 1) as f32;
                    let angle = start + (end - start) * a;
                    path.move_to((
                        centerx + angle.cos() * (radius - tick_len),
                        centery + angle.sin() * (radius - tick_len),
                    ));
                    path.line_to((
                        centerx + angle.cos() * (radius - line_width / 2.0),
                        centery + angle.sin() * (radius - line_width / 2.0),
                    ));
                }
            }
        }
        let mut paint = skia::Paint::default();
        paint.set_color(foreground_color);
        paint.set_stroke_width(line_width);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_style(skia::PaintStyle::Stroke);
        canvas.draw_path(&path, &paint);
    }
}

/// Makes a round knob with a tick to show the current value
pub struct TickKnob {
    angle_start: f32,
    angle_end: f32,
    radius: Units,
    tick_width: Units,
    tick_len: Units,
    normalized_value: f32,
    mode: KnobMode,
}
impl TickKnob {
    pub fn new(
        cx: &mut Context,
        radius: Units,
        tick_width: Units,
        tick_len: Units,
        arc_len: f32,
        // steps: u32,
        mode: KnobMode,
    ) -> Handle<Self> {
        Self {
            // angle_start: -150.0,
            // angle_end: 150.0,
            angle_start: -arc_len / 2.0,
            angle_end: arc_len / 2.0,
            radius,
            tick_width,
            tick_len,
            normalized_value: 0.5,
            mode,
        }
        .build(cx, |_| {})
    }
}

impl View for TickKnob {
    fn element(&self) -> Option<&'static str> {
        Some("tickknob")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let opacity = cx.opacity();
        //let mut background_color: femtoskia::Color = cx.current.get_background_color(cx).into();
        // background_color.set_alphaf(background_color.a * opacity);
        let foreground_color = cx.font_color();
        let background_color = cx.background_color();
        //et mut foreground_color = femtoskia::Color::rgb(50, 50, 200);
        let bounds = cx.bounds();
        // Calculate arc center
        let centerx = bounds.x + 0.5 * bounds.w;
        let centery = bounds.y + 0.5 * bounds.h;
        // Convert start and end angles to radians and rotate origin direction to be upwards instead of to the right
        let start = self.angle_start.to_radians() - PI / 2.0;
        let end = self.angle_end.to_radians() - PI / 2.0;
        let parent = cx.tree.get_parent(cx.current).unwrap();
        let parent_width = cx.cache.get_width(parent);
        // Convert radius and span into screen coordinates
        let radius = self.radius.to_px(parent_width / 2.0, 0.0);
        let tick_width = self.tick_width.to_px(radius, 0.0);
        let tick_len = self.tick_len.to_px(radius, 0.0);
        // Draw the circle
        let mut path = skia::Path::new();
        path.add_circle((centerx, centery), radius, None);
        // path.arc(centerx, centery, radius - span / 2.0, end, start, Solidity::Solid);
        let mut paint = skia::Paint::default();
        paint.set_color(background_color);
        paint.set_stroke_width(tick_width);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_style(skia::PaintStyle::Stroke);
        canvas.draw_path(&path, &paint);
        // Draw the tick
        let mut path = skia::Path::new();
        let angle = match self.mode {
            KnobMode::Continuous => start + (end - start) * self.normalized_value,
            // snapping
            KnobMode::Discrete(steps) => {
                start
                    + (end - start) * (self.normalized_value * (steps - 1) as f32).floor()
                        / (steps - 1) as f32
            }
        };
        path.move_to(
            // centerx + angle.cos() * (radius * 0.70),
            (
                centerx + angle.cos() * (radius - tick_len),
                centery + angle.sin() * (radius - tick_len),
            ),
        );
        path.line_to((
            centerx + angle.cos() * (radius - tick_width / 2.0),
            centery + angle.sin() * (radius - tick_width / 2.0),
        ));
        let mut paint = skia::Paint::default();
        paint.set_color(foreground_color);
        paint.set_stroke_width(tick_width);
        paint.set_stroke_cap(skia::PaintCap::Round);
        paint.set_style(skia::PaintStyle::Stroke);
        canvas.draw_path(&path, &paint);
    }
}

impl Handle<'_, TickKnob> {
    pub fn value<L: Lens<Target = f32>>(self, lens: L) -> Self {
        let entity = self.entity;
        Binding::new(self.cx, lens, move |cx, value| {
            let value = value.get(cx);
            if let Some(view) = cx.views.get_mut(&entity) {
                if let Some(knob) = view.downcast_mut::<TickKnob>() {
                    knob.normalized_value = value;
                    cx.needs_redraw(entity);
                }
            }
        });
        self
    }
}
