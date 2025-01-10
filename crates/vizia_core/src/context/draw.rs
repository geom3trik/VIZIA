use skia_safe::canvas::SaveLayerRec;
use skia_safe::gradient_shader::GradientShaderColors;
use skia_safe::path::ArcSize;
use skia_safe::rrect::Corner;
use skia_safe::wrapper::PointerWrapper;
use skia_safe::{
    BlurStyle, ClipOp, MaskFilter, Matrix, Paint, PaintStyle, Path, PathDirection, PathEffect,
    Point, RRect, Rect, SamplingOptions, Shader, TileMode,
};
use std::any::{Any, TypeId};
use std::f32::consts::SQRT_2;
use vizia_style::LengthPercentageOrAuto;

use hashbrown::HashMap;

use crate::animation::Interpolator;
use crate::cache::CachedData;
use crate::events::ViewHandler;
use crate::model::ModelDataStore;
use crate::prelude::*;
use crate::resource::{ImageOrSvg, ResourceManager};
use crate::text::TextContext;
use vizia_input::MouseState;

/// A context used when drawing a view.
///
/// The `DrawContext` is provided by the [`draw`](crate::view::View::draw) method in [`View`] and can be used to immutably access the
/// computed style and layout properties of the current view.
///
/// # Example
/// ```
/// # use vizia_core::prelude::*;
/// # use vizia_core::vg;
/// # let cx = &mut Context::default();
///
/// pub struct CustomView {}
///
/// impl CustomView {
///     pub fn new(cx: &mut Context) -> Handle<Self> {
///         Self{}.build(cx, |_|{})
///     }
/// }
///
/// impl View for CustomView {
///     fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
///         // Get the computed bounds after layout of the current view
///         let bounds = cx.bounds();
///         // Draw to the canvas using the bounds of the current view
///         let path = vg::Path::new();
///         path.rect(bounds.x, bounds.y, bounds.w, bounds.h);
///         let mut paint = vg::Paint::default();
///         paint.set_color(Color::rgb(200, 100, 100));
///         canvas.draw_path(&path, &paint);
///     }
/// }
/// ```
pub struct DrawContext<'a> {
    pub(crate) current: Entity,
    pub(crate) style: &'a Style,
    pub(crate) cache: &'a CachedData,
    pub(crate) tree: &'a Tree<Entity>,
    pub(crate) data: &'a HashMap<Entity, ModelDataStore>,
    pub(crate) views: &'a mut HashMap<Entity, Box<dyn ViewHandler>>,
    pub(crate) resource_manager: &'a ResourceManager,
    pub(crate) text_context: &'a mut TextContext,
    pub(crate) modifiers: &'a Modifiers,
    pub(crate) mouse: &'a MouseState<Entity>,
    pub(crate) windows: &'a mut HashMap<Entity, WindowState>,
}

macro_rules! get_units_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> Units {
            let result = self.style.$name.get(self.current);
            if let Some(Units::Pixels(p)) = result {
                Units::Pixels(self.logical_to_physical(*p))
            } else {
                result.copied().unwrap_or_default()
            }
        }
    };
}

macro_rules! get_color_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> Color {
            if let Some(col) = self.style.$name.get(self.current) {
                Color::rgba(col.r(), col.g(), col.b(), col.a())
            } else {
                Color::rgba(0, 0, 0, 0)
            }
        }
    };
}

macro_rules! get_length_property {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> f32 {
            if let Some(length) = self.style.$name.get(self.current) {
                let bounds = self.bounds();

                let px = length.to_pixels(bounds.w.min(bounds.h), self.scale_factor());
                return px.round();
            }

            0.0
        }
    };
}

impl DrawContext<'_> {
    /// Returns the bounds of the current view.
    pub fn bounds(&self) -> BoundingBox {
        self.cache.get_bounds(self.current)
    }

    /// Marks the current view as needing to be redrawn.
    pub fn needs_redraw(&mut self) {
        let parent_window = self.tree.get_parent_window(self.current).unwrap_or(Entity::root());
        if let Some(window_state) = self.windows.get_mut(&parent_window) {
            window_state.redraw_list.insert(self.current);
        }
    }

    /// Returns the z-index of the current view.
    pub fn z_index(&self) -> i32 {
        self.style.z_index.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.style.dpi_factor as f32
    }

    /// Returns a reference to the keyboard modifiers state.
    pub fn modifiers(&self) -> &Modifiers {
        self.modifiers
    }

    /// Returns a reference to the mouse state.
    pub fn mouse(&self) -> &MouseState<Entity> {
        self.mouse
    }

    /// Returns the clip path of the current view.
    pub fn clip_path(&self) -> Option<skia_safe::Path> {
        let bounds = self.bounds();
        let overflowx = self.style.overflowx.get(self.current).copied().unwrap_or_default();
        let overflowy = self.style.overflowy.get(self.current).copied().unwrap_or_default();

        let scale = self.scale_factor();

        let clip_bounds = self
            .style
            .clip_path
            .get(self.current)
            .map(|clip| match clip {
                ClipPath::Auto => bounds,
                ClipPath::Shape(rect) => bounds.shrink_sides(
                    rect.3.to_pixels(bounds.w, scale),
                    rect.0.to_pixels(bounds.h, scale),
                    rect.1.to_pixels(bounds.w, scale),
                    rect.2.to_pixels(bounds.h, scale),
                ),
            })
            .unwrap_or(bounds);

        let root_bounds = self.cache.get_bounds(Entity::root());

        let clip_bounds = match (overflowx, overflowy) {
            (Overflow::Visible, Overflow::Visible) => return None,
            (Overflow::Hidden, Overflow::Visible) => {
                let left = clip_bounds.left();
                let right = clip_bounds.right();
                let top = root_bounds.top();
                let bottom = root_bounds.bottom();
                BoundingBox::from_min_max(left, top, right, bottom)
            }
            (Overflow::Visible, Overflow::Hidden) => {
                let left = root_bounds.left();
                let right = root_bounds.right();
                let top = clip_bounds.top();
                let bottom = clip_bounds.bottom();
                BoundingBox::from_min_max(left, top, right, bottom)
            }
            (Overflow::Hidden, Overflow::Hidden) => clip_bounds,
        };

        Some(self.build_path(clip_bounds, (0.0, 0.0)))
    }

    /// Returns the 2D transform of the current view.
    pub fn transform(&self) -> Matrix {
        let bounds = self.bounds();
        let scale_factor = self.scale_factor();

        // Apply transform origin.
        let mut origin = self
            .style
            .transform_origin
            .get(self.current)
            .map(|transform_origin| {
                let mut origin = Matrix::translate(bounds.top_left());
                let offset = transform_origin.as_transform(bounds, scale_factor);
                origin = offset * origin;
                origin
            })
            .unwrap_or(Matrix::translate(bounds.center()));

        let mut transform = origin;
        origin = origin.invert().unwrap();

        // Apply translation.
        if let Some(translate) = self.style.translate.get(self.current) {
            transform = transform * translate.as_transform(bounds, scale_factor);
        }

        // Apply rotation.
        if let Some(rotate) = self.style.rotate.get(self.current) {
            transform = transform * rotate.as_transform(bounds, scale_factor);
        }

        // Apply scaling.
        if let Some(scale) = self.style.scale.get(self.current) {
            transform = transform * scale.as_transform(bounds, scale_factor);
        }

        // Apply transform functions.
        if let Some(transforms) = self.style.transform.get(self.current) {
            // Check if the transform is currently animating
            // Get the animation state
            // Manually interpolate the value to get the overall transform for the current frame
            if let Some(animation_state) = self.style.transform.get_active_animation(self.current) {
                if let Some(start) = animation_state.keyframes.first() {
                    if let Some(end) = animation_state.keyframes.last() {
                        let start_transform = start.value.as_transform(bounds, scale_factor);
                        let end_transform = end.value.as_transform(bounds, scale_factor);
                        let t = animation_state.t;
                        let animated_transform =
                            Matrix::interpolate(&start_transform, &end_transform, t);
                        transform = transform * animated_transform;
                    }
                }
            } else {
                transform = transform * transforms.as_transform(bounds, scale_factor);
            }
        }

        transform = transform * origin;

        transform
    }

    /// Returns the visibility of the current view.
    pub fn visibility(&self) -> Option<Visibility> {
        self.style.visibility.get(self.current).copied()
    }

    /// Returns the display of the current view.
    pub fn display(&self) -> Display {
        self.style.display.get(self.current).copied().unwrap_or(Display::Flex)
    }

    /// Returns the opacity of the current view.
    pub fn opacity(&self) -> f32 {
        self.style.opacity.get(self.current).copied().unwrap_or(Opacity(1.0)).0
    }

    /// Returns the lookup pattern to pick the default font.
    pub fn default_font(&self) -> &[FamilyOwned] {
        &self.style.default_font
    }

    /// Returns the font-size of the current view in physical pixels.
    pub fn font_size(&self) -> f32 {
        self.logical_to_physical(
            self.style.font_size.get(self.current).copied().map(|f| f.0).unwrap_or(16.0),
        )
    }

    /// Returns the font-weight of the current view.
    pub fn font_weight(&self) -> FontWeight {
        self.style.font_weight.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the font-width of the current view.
    pub fn font_width(&self) -> FontWidth {
        self.style.font_width.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the font-slant of the current view.
    pub fn font_slant(&self) -> FontSlant {
        self.style.font_slant.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the font variation settings of the current view.
    pub fn font_variation_settings(&self) -> &[FontVariation] {
        self.style.font_variation_settings.get(self.current).map(Vec::as_slice).unwrap_or_default()
    }

    /// Function to convert logical points to physical pixels.
    pub fn logical_to_physical(&self, logical: f32) -> f32 {
        self.style.logical_to_physical(logical)
    }

    /// Function to convert physical pixels to logical points.
    pub fn physical_to_logical(&self, physical: f32) -> f32 {
        self.style.physical_to_logical(physical)
    }

    get_length_property!(
        /// Returns the border width of the current view in physical pixels.
        border_width
    );

    get_color_property!(
        /// Returns the outline color of the current view.
        outline_color
    );

    get_length_property!(
        /// Returns the outline width of the current view in physical pixels.
        outline_width
    );

    get_length_property!(
        /// Returns the outline offset of the current view in physcial pixels.
        outline_offset
    );

    get_length_property!(
        /// Returns the corner radius for the top-left corner of the current view.
        corner_top_left_radius
    );

    get_length_property!(
        /// Returns the corner radius for the top-right corner of the current view.
        corner_top_right_radius
    );

    get_length_property!(
        /// Returns the corner radius for the bottom-left corner of the current view.
        corner_bottom_left_radius
    );

    get_length_property!(
        /// Returns the corner radius for the bottom-right corner of the current view.
        corner_bottom_right_radius
    );

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_top_left_shape(&self) -> CornerShape {
        self.style.corner_top_left_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_top_right_shape(&self) -> CornerShape {
        self.style.corner_top_right_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_bottom_left_shape(&self) -> CornerShape {
        self.style.corner_bottom_left_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_bottom_right_shape(&self) -> CornerShape {
        self.style.corner_bottom_right_shape.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner smoothing for the top-left corner of the current view.
    pub fn corner_top_left_smoothing(&self) -> f32 {
        self.style.corner_top_left_smoothing.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_top_right_smoothing(&self) -> f32 {
        self.style.corner_top_right_smoothing.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_bottom_left_smoothing(&self) -> f32 {
        self.style.corner_bottom_left_smoothing.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the corner shape for the top-left corner of the current view.
    pub fn corner_bottom_right_smoothing(&self) -> f32 {
        self.style.corner_bottom_right_smoothing.get(self.current).copied().unwrap_or_default()
    }

    get_units_property!(
        /// Returns the child-left space of the current view.
        child_left
    );

    get_units_property!(
        /// Returns the child-right space of the current view.
        child_right
    );

    get_units_property!(
        /// Returns the child-top space of the current view.
        child_top
    );

    get_units_property!(
        /// Returns the child-bottom space of the current view.
        child_bottom
    );

    get_color_property!(background_color);
    get_color_property!(border_color);

    /// Returns the border style of the current view.
    pub fn border_style(&self) -> BorderStyleKeyword {
        self.style.border_style.get(self.current).copied().unwrap_or_default()
    }

    get_color_property!(selection_color);
    get_color_property!(caret_color);
    get_color_property!(font_color);

    /// Returns whether the current view should have its text wrapped.
    pub fn text_wrap(&self) -> bool {
        self.style.text_wrap.get(self.current).copied().unwrap_or(true)
    }

    /// Returns the text alignment of the current view.
    pub fn text_align(&self) -> TextAlign {
        self.style.text_align.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the text overflow preference of the current view.
    pub fn text_overflow(&self) -> TextOverflow {
        self.style.text_overflow.get(self.current).copied().unwrap_or_default()
    }

    /// Returns the line clamp Of the current view.
    pub fn line_clamp(&self) -> Option<usize> {
        self.style.line_clamp.get(self.current).copied().map(|lc| lc.0 as usize)
    }

    /// Returns a reference to the shadows of the current view.
    pub fn shadows(&self) -> Option<&Vec<Shadow>> {
        self.style.shadow.get(self.current)
    }

    pub fn backdrop_filter(&self) -> Option<&Filter> {
        self.style.backdrop_filter.get(self.current)
    }

    pub fn background_images(&self) -> Option<&Vec<ImageOrGradient>> {
        self.style.background_image.get(self.current)
    }

    pub fn background_size(&self) -> Vec<BackgroundSize> {
        self.style.background_size.get(self.current).cloned().unwrap_or_default()
    }

    /// Get the vector path of the current view.
    pub fn build_path(&self, bounds: BoundingBox, outset: (f32, f32)) -> Path {
        let corner_top_left_radius = self.corner_top_left_radius();
        let corner_top_right_radius = self.corner_top_right_radius();
        let corner_bottom_right_radius = self.corner_bottom_right_radius();
        let corner_bottom_left_radius = self.corner_bottom_left_radius();

        let corner_top_left_shape = self.corner_top_left_shape();
        let corner_top_right_shape = self.corner_top_right_shape();
        let corner_bottom_right_shape = self.corner_bottom_right_shape();
        let corner_bottom_left_shape = self.corner_bottom_left_shape();

        let corner_top_left_smoothing = self.corner_top_left_smoothing();
        let corner_top_right_smoothing = self.corner_top_right_smoothing();
        let corner_bottom_right_smoothing = self.corner_bottom_right_smoothing();
        let corner_bottom_left_smoothing = self.corner_bottom_left_smoothing();

        let rect: Rect = bounds.into();

        let mut rr = RRect::new_rect_radii(
            rect,
            &[
                Point::new(corner_top_left_radius, corner_top_left_radius),
                Point::new(corner_top_right_radius, corner_top_right_radius),
                Point::new(corner_bottom_right_radius, corner_bottom_right_radius),
                Point::new(corner_bottom_left_radius, corner_bottom_left_radius),
            ],
        );

        rr = rr.with_outset(outset);

        let x = rr.bounds().x();
        let y = rr.bounds().y();
        let width = rr.width();
        let height = rr.height();

        //TODO: Cache the path and regenerate if the bounds change
        let mut path = Path::new();

        if width == height
            && corner_bottom_left_radius == width / 2.0
            && corner_bottom_right_radius == width / 2.0
            && corner_top_left_radius == height / 2.0
            && corner_top_right_radius == height / 2.0
        {
            path.add_circle((width / 2.0, bounds.h / 2.0), width / 2.0, PathDirection::CW);
        } else {
            let top_right = rr.radii(Corner::UpperRight).x;

            if top_right > 0.0 {
                let (a, b, c, d, l, p, radius) = compute_smooth_corner(
                    top_right,
                    corner_top_right_smoothing,
                    bounds.width(),
                    bounds.height(),
                );

                path.move_to((f32::max(width / 2.0, width - p), 0.0));
                if corner_top_right_shape == CornerShape::Round {
                    path.cubic_to(
                        (width - (p - a), 0.0),
                        (width - (p - a - b), 0.0),
                        (width - (p - a - b - c), d),
                    )
                    .r_arc_to_rotated(
                        (radius, radius),
                        0.0,
                        ArcSize::Small,
                        PathDirection::CW,
                        (l, l),
                    )
                    .cubic_to(
                        (width, p - a - b),
                        (width, p - a),
                        (width, f32::min(height / 2.0, p)),
                    );
                } else {
                    path.line_to((width, f32::min(height / 2.0, p)));
                }
            } else {
                path.move_to((width / 2.0, 0.0))
                    .line_to((width, 0.0))
                    .line_to((width, height / 2.0));
            }

            let bottom_right = rr.radii(Corner::LowerRight).x;
            if bottom_right > 0.0 {
                let (a, b, c, d, l, p, radius) = compute_smooth_corner(
                    bottom_right,
                    corner_bottom_right_smoothing,
                    width,
                    height,
                );

                path.line_to((width, f32::max(height / 2.0, height - p)));
                if corner_bottom_right_shape == CornerShape::Round {
                    path.cubic_to(
                        (width, height - (p - a)),
                        (width, height - (p - a - b)),
                        (width - d, height - (p - a - b - c)),
                    )
                    .r_arc_to_rotated(
                        (radius, radius),
                        0.0,
                        ArcSize::Small,
                        PathDirection::CW,
                        (-l, l),
                    )
                    .cubic_to(
                        (width - (p - a - b), height),
                        (width - (p - a), height),
                        (f32::max(width / 2.0, width - p), height),
                    );
                } else {
                    path.line_to((f32::max(width / 2.0, width - p), height));
                }
            } else {
                path.line_to((width, height)).line_to((width / 2.0, height));
            }

            let bottom_left = rr.radii(Corner::LowerLeft).x;
            if bottom_left > 0.0 {
                let (a, b, c, d, l, p, radius) =
                    compute_smooth_corner(bottom_left, corner_bottom_left_smoothing, width, height);

                path.line_to((f32::min(width / 2.0, p), height));
                if corner_bottom_left_shape == CornerShape::Round {
                    path.cubic_to(
                        (p - a, height),
                        (p - a - b, height),
                        (p - a - b - c, height - d),
                    )
                    .r_arc_to_rotated(
                        (radius, radius),
                        0.0,
                        ArcSize::Small,
                        PathDirection::CW,
                        (-l, -l),
                    )
                    .cubic_to(
                        (0.0, height - (p - a - b)),
                        (0.0, height - (p - a)),
                        (0.0, f32::max(height / 2.0, height - p)),
                    );
                } else {
                    path.line_to((0.0, f32::max(height / 2.0, height - p)));
                }
            } else {
                path.line_to((0.0, height)).line_to((0.0, height / 2.0));
            }

            let top_left = rr.radii(Corner::UpperLeft).x;
            if top_left > 0.0 {
                let (a, b, c, d, l, p, radius) =
                    compute_smooth_corner(top_left, corner_top_left_smoothing, width, height);

                path.line_to((0.0, f32::min(height / 2.0, p)));
                if corner_top_left_shape == CornerShape::Round {
                    path.cubic_to((0.0, p - a), (0.0, p - a - b), (d, p - a - b - c))
                        .r_arc_to_rotated(
                            (radius, radius),
                            0.0,
                            ArcSize::Small,
                            PathDirection::CW,
                            (l, -l),
                        )
                        .cubic_to((p - a - b, 0.0), (p - a, 0.0), (f32::min(width / 2.0, p), 0.0));
                } else {
                    path.line_to((f32::min(width / 2.0, p), 0.0));
                }
            } else {
                path.line_to((0.0, 0.0));
            }

            path.close();
        }

        path.offset((x, y));

        path
    }

    /// Draw background color or background image (including gradients) for the current view.
    pub fn draw_background(&mut self, canvas: &Canvas) {
        let path = self.build_path(self.bounds(), (0.0, 0.0));
        let background_color = self.background_color();
        if background_color.a() != 0 {
            let mut paint = Paint::default();
            paint.set_color(skia_safe::Color::from_argb(
                background_color.a(),
                background_color.r(),
                background_color.g(),
                background_color.b(),
            ));
            paint.set_anti_alias(true);
            canvas.draw_path(&path, &paint);
        }

        self.draw_background_images(canvas, &path);
    }

    /// Draw the border of the current view.
    pub fn draw_border(&mut self, canvas: &Canvas) {
        let border_color = self.border_color();
        let border_width = self.border_width();
        let border_style = self.border_style();

        if border_width > 0.0 && border_color.a() > 0 && border_style != BorderStyleKeyword::None {
            let bounds = self.bounds();
            let path = self.build_path(bounds, (-border_width / 2.0, -border_width / 2.0));
            let mut paint = Paint::default();
            paint.set_style(PaintStyle::Stroke);
            paint.set_color(border_color);
            paint.set_stroke_width(border_width);
            match border_style {
                BorderStyleKeyword::Dashed => {
                    paint.set_path_effect(PathEffect::dash(
                        &[border_width * 2.0, border_width],
                        0.0,
                    ));
                }

                BorderStyleKeyword::Dotted => {
                    paint.set_path_effect(PathEffect::dash(&[0.0, border_width * 2.0], 0.0));
                    paint.set_stroke_cap(skia_safe::PaintCap::Round);
                }

                _ => {}
            }

            paint.set_anti_alias(true);
            canvas.draw_path(&path, &paint);
        }
    }

    /// Draw the outline of the current view.
    pub fn draw_outline(&mut self, canvas: &Canvas) {
        let outline_width = self.outline_width();
        let outline_color = self.outline_color();

        if outline_width > 0.0 && outline_color.a() != 0 {
            let outline_offset = self.outline_offset();

            let bounds = self.bounds();

            let half_outline_width = outline_width / 2.0;
            let outline_path = self.build_path(
                bounds,
                (half_outline_width + outline_offset, half_outline_width + outline_offset),
            );

            let mut outline_paint = Paint::default();
            outline_paint.set_color(outline_color);
            outline_paint.set_stroke_width(outline_width);
            outline_paint.set_style(PaintStyle::Stroke);
            outline_paint.set_anti_alias(true);
            canvas.draw_path(&outline_path, &outline_paint);
        }
    }

    /// Draw shadows for the current view.
    pub fn draw_shadows(&mut self, canvas: &Canvas) {
        if let Some(shadows) = self.shadows() {
            if shadows.is_empty() {
                return;
            }

            let bounds = self.bounds();

            let path = self.build_path(bounds, (0.0, 0.0));

            for shadow in shadows.iter().rev() {
                let shadow_color = shadow.color.unwrap_or_default();

                let shadow_x_offset = shadow.x_offset.to_px().unwrap_or(0.0) * self.scale_factor();
                let shadow_y_offset = shadow.y_offset.to_px().unwrap_or(0.0) * self.scale_factor();
                let spread_radius =
                    shadow.spread_radius.as_ref().and_then(|l| l.to_px()).unwrap_or(0.0)
                        * self.scale_factor();

                let blur_radius =
                    shadow.blur_radius.as_ref().and_then(|br| br.to_px()).unwrap_or(0.0);

                if shadow_color.a() == 0
                    || (shadow_x_offset == 0.0
                        && shadow_y_offset == 0.0
                        && spread_radius == 0.0
                        && blur_radius == 0.0)
                {
                    continue;
                }

                let mut shadow_paint = Paint::default();

                let outset = if shadow.inset { -spread_radius } else { spread_radius };

                shadow_paint.set_style(PaintStyle::Fill);

                let mut shadow_path = self.build_path(bounds, (outset, outset));

                shadow_paint.set_color(shadow_color);

                if blur_radius > 0.0 {
                    shadow_paint.set_mask_filter(MaskFilter::blur(
                        BlurStyle::Normal,
                        blur_radius / 2.0,
                        false,
                    ));
                }

                shadow_path.offset((shadow_x_offset, shadow_y_offset));

                if shadow.inset {
                    shadow_path = path.op(&shadow_path, skia_safe::PathOp::Difference).unwrap();
                }

                canvas.save();
                canvas.clip_path(
                    &path,
                    if shadow.inset { ClipOp::Intersect } else { ClipOp::Difference },
                    true,
                );
                canvas.draw_path(&shadow_path, &shadow_paint);
                canvas.restore();
            }
        }
    }

    /// Draw background images (including gradients) for the current view.
    fn draw_background_images(&mut self, canvas: &Canvas, path: &Path) {
        let bounds = self.bounds();

        if let Some(images) = self.background_images() {
            let image_sizes = self.background_size();

            for (index, image) in images.iter().enumerate() {
                match image {
                    ImageOrGradient::None => return,
                    ImageOrGradient::Gradient(gradient) => match gradient {
                        Gradient::Linear(linear_gradient) => {
                            let (start, end, parent_length) = match linear_gradient.direction {
                                LineDirection::Horizontal(horizontal_keyword) => {
                                    match horizontal_keyword {
                                        HorizontalPositionKeyword::Left => (
                                            bounds.center_right(),
                                            bounds.center_left(),
                                            bounds.width(),
                                        ),

                                        HorizontalPositionKeyword::Right => (
                                            bounds.center_left(),
                                            bounds.center_right(),
                                            bounds.width(),
                                        ),
                                    }
                                }

                                LineDirection::Vertical(vertical_keyword) => match vertical_keyword
                                {
                                    VerticalPositionKeyword::Top => (
                                        bounds.center_bottom(),
                                        bounds.center_top(),
                                        bounds.height(),
                                    ),

                                    VerticalPositionKeyword::Bottom => (
                                        bounds.center_top(),
                                        bounds.center_bottom(),
                                        bounds.height(),
                                    ),
                                },

                                LineDirection::Corner { horizontal, vertical } => {
                                    match (horizontal, vertical) {
                                        (
                                            HorizontalPositionKeyword::Right,
                                            VerticalPositionKeyword::Bottom,
                                        ) => (
                                            bounds.top_left(),
                                            bounds.bottom_right(),
                                            bounds.diagonal(),
                                        ),

                                        (
                                            HorizontalPositionKeyword::Right,
                                            VerticalPositionKeyword::Top,
                                        ) => (
                                            bounds.bottom_left(),
                                            bounds.top_right(),
                                            bounds.diagonal(),
                                        ),

                                        _ => (bounds.top_left(), bounds.bottom_right(), 0.0),
                                    }
                                }

                                LineDirection::Angle(angle) => {
                                    let angle_rad = angle.to_radians();
                                    let start_x =
                                        bounds.x + ((angle_rad.sin() * bounds.w) - bounds.w) / -2.0;
                                    let end_x =
                                        bounds.x + ((angle_rad.sin() * bounds.w) + bounds.w) / 2.0;
                                    let start_y =
                                        bounds.y + ((angle_rad.cos() * bounds.h) + bounds.h) / 2.0;
                                    let end_y =
                                        bounds.y + ((angle_rad.cos() * bounds.h) - bounds.h) / -2.0;

                                    let x = (end_x - start_x).abs();
                                    let y = (end_y - start_y).abs();

                                    let dist = (x * x + y * y).sqrt();

                                    ((start_x, start_y), (end_x, end_y), dist)
                                }
                            };

                            let num_stops = linear_gradient.stops.len();

                            let mut stops = linear_gradient
                                .stops
                                .iter()
                                .enumerate()
                                .map(|(index, stop)| {
                                    let pos = if let Some(pos) = &stop.position {
                                        pos.to_pixels(parent_length, self.scale_factor())
                                            / parent_length
                                    } else {
                                        index as f32 / (num_stops - 1) as f32
                                    };
                                    (pos, skia_safe::Color::from(stop.color))
                                })
                                .collect::<Vec<_>>();

                            // Insert a stop at the front if the first stop is not at 0.
                            if let Some(first) = stops.first() {
                                if first.0 != 0.0 {
                                    stops.insert(0, (0.0, first.1));
                                }
                            }

                            // Insert a stop at the end if the last stop is not at 1.0.
                            if let Some(last) = stops.last() {
                                if last.0 != 1.0 {
                                    stops.push((1.0, last.1));
                                }
                            }

                            let (offsets, colors): (Vec<f32>, Vec<skia_safe::Color>) =
                                stops.into_iter().unzip();

                            let shader = Shader::linear_gradient(
                                (Point::from(start), Point::from(end)),
                                GradientShaderColors::Colors(&colors[..]),
                                Some(&offsets[..]),
                                TileMode::Clamp,
                                None,
                                None,
                            );

                            let mut paint = Paint::default();
                            paint.set_shader(shader);

                            canvas.draw_path(path, &paint);
                        }

                        Gradient::Radial(radial_gradient) => {
                            let num_stops = radial_gradient.stops.len();

                            let mut stops = radial_gradient
                                .stops
                                .iter()
                                .enumerate()
                                .map(|(index, stop)| {
                                    let pos = if let Some(pos) = &stop.position {
                                        pos.to_pixels(bounds.width(), self.scale_factor())
                                            / bounds.width()
                                    } else {
                                        index as f32 / (num_stops - 1) as f32
                                    };

                                    (pos, skia_safe::Color::from(stop.color))
                                })
                                .collect::<Vec<_>>();

                            // Insert a stop at the front if the first stop is not at 0.
                            if let Some(first) = stops.first() {
                                if first.0 != 0.0 {
                                    stops.insert(0, (0.0, first.1));
                                }
                            }

                            // Insert a stop at the end if the last stop is not at 1.0.
                            if let Some(last) = stops.last() {
                                if last.0 != 1.0 {
                                    stops.push((1.0, last.1));
                                }
                            }

                            let (offsets, colors): (Vec<f32>, Vec<skia_safe::Color>) =
                                stops.into_iter().unzip();

                            let shader = Shader::radial_gradient(
                                Point::from(bounds.center()),
                                bounds.w.max(bounds.h),
                                GradientShaderColors::Colors(&colors[..]),
                                Some(&offsets[..]),
                                TileMode::Clamp,
                                None,
                                None,
                            );

                            let mut paint = Paint::default();
                            paint.set_shader(shader);
                            canvas.draw_path(path, &paint);
                        }

                        _ => {}
                    },

                    ImageOrGradient::Image(image_name) => {
                        if let Some(image_id) = self.resource_manager.image_ids.get(image_name) {
                            if let Some(image) = self.resource_manager.images.get(image_id) {
                                match &image.image {
                                    ImageOrSvg::Image(image) => {
                                        let image_width = image.width();
                                        let image_height = image.height();
                                        let (width, height) = if let Some(background_size) =
                                            image_sizes.get(index)
                                        {
                                            match background_size {
                                                BackgroundSize::Explicit { width, height } => {
                                                    let w = match width {
                                                LengthPercentageOrAuto::LengthPercentage(
                                                    length,
                                                ) => {
                                                    length.to_pixels(bounds.w, self.scale_factor())
                                                }
                                                LengthPercentageOrAuto::Auto => image_width as f32,
                                            };

                                                    let h = match height {
                                                LengthPercentageOrAuto::LengthPercentage(
                                                    length,
                                                ) => {
                                                    length.to_pixels(bounds.h, self.scale_factor())
                                                }
                                                LengthPercentageOrAuto::Auto => image_height as f32,
                                            };

                                                    (w, h)
                                                }

                                                BackgroundSize::Contain => {
                                                    let image_ratio =
                                                        image_width as f32 / image_height as f32;
                                                    let container_ratio = bounds.w / bounds.h;

                                                    let (w, h) = if image_ratio > container_ratio {
                                                        (bounds.w, bounds.w / image_ratio)
                                                    } else {
                                                        (bounds.h * image_ratio, bounds.h)
                                                    };

                                                    (w, h)
                                                }

                                                BackgroundSize::Cover => {
                                                    let image_ratio =
                                                        image_width as f32 / image_height as f32;
                                                    let container_ratio = bounds.w / bounds.h;

                                                    let (w, h) = if image_ratio < container_ratio {
                                                        (bounds.w, bounds.w / image_ratio)
                                                    } else {
                                                        (bounds.h * image_ratio, bounds.h)
                                                    };

                                                    (w, h)
                                                }
                                            }
                                        } else {
                                            (image_width as f32, image_height as f32)
                                        };

                                        let matrix = Matrix::rect_to_rect(
                                            Rect::new(
                                                0.0,
                                                0.0,
                                                image.width() as f32,
                                                image.height() as f32,
                                            ),
                                            Rect::new(
                                                bounds.left(),
                                                bounds.top(),
                                                bounds.left() + width,
                                                bounds.top() + height,
                                            ),
                                            None,
                                        );

                                        let mut paint = Paint::default();
                                        paint.set_anti_alias(true);
                                        paint.set_shader(image.to_shader(
                                            (TileMode::Repeat, TileMode::Repeat),
                                            SamplingOptions::default(),
                                            &matrix,
                                        ));

                                        canvas.draw_path(path, &paint);
                                    }

                                    ImageOrSvg::Svg(svg) => {
                                        canvas.save_layer(&SaveLayerRec::default());
                                        canvas.translate((bounds.x, bounds.y));
                                        let (scale_x, scale_y) = (
                                            bounds.width() / svg.inner().fContainerSize.fWidth,
                                            bounds.height() / svg.inner().fContainerSize.fHeight,
                                        );

                                        if scale_x.is_finite() && scale_y.is_finite() {
                                            canvas.scale((scale_x, scale_y));
                                        } else {
                                            svg.clone().set_container_size((
                                                bounds.width(),
                                                bounds.height(),
                                            ));
                                        }

                                        svg.render(canvas);

                                        if let Some(color) =
                                            self.style.fill.get(self.current).copied()
                                        {
                                            let mut paint = Paint::default();

                                            paint.set_anti_alias(true);
                                            paint.set_blend_mode(skia_safe::BlendMode::SrcIn);
                                            paint.set_color(color);
                                            canvas.draw_paint(&paint);
                                        }
                                        canvas.restore();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw any text for the current view.
    pub fn draw_text(&mut self, canvas: &Canvas) {
        if let Some(paragraph) = self.text_context.text_paragraphs.get(self.current) {
            let bounds = self.bounds();

            let mut vertical_flex_sum = 0.0;
            let mut horizontal_flex_sum = 0.0;

            let mut padding_top = match self.child_top() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    vertical_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let padding_bottom = match self.child_bottom() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    vertical_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let vertical_free_space =
                bounds.height() - paragraph.height() - padding_top - padding_bottom;

            if let Units::Stretch(val) = self.child_top() {
                padding_top = (vertical_free_space * val / vertical_flex_sum).round()
            }

            let mut padding_left = match self.child_left() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    horizontal_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let padding_right = match self.child_right() {
                Units::Pixels(val) => val,
                Units::Stretch(val) => {
                    horizontal_flex_sum += val;
                    0.0
                }
                _ => 0.0,
            };

            let horizontal_free_space =
                bounds.width() - paragraph.max_width() - padding_left - padding_right;

            if let Units::Stretch(val) = self.child_left() {
                padding_left = (horizontal_free_space * val / horizontal_flex_sum).round()
            }

            paragraph.paint(
                canvas,
                ((bounds.x + padding_left).round(), (bounds.y + padding_top).round()),
            );
        }
    }
}

impl DataContext for DrawContext<'_> {
    fn data<T: 'static>(&self) -> Option<&T> {
        // Return data for the static model.
        if let Some(t) = <dyn Any>::downcast_ref::<T>(&()) {
            return Some(t);
        }

        for entity in self.current.parent_iter(self.tree) {
            // Return model data.
            if let Some(model_data_store) = self.data.get(&entity) {
                if let Some(model) = model_data_store.models.get(&TypeId::of::<T>()) {
                    return model.downcast_ref::<T>();
                }
            }

            // Return view data.
            if let Some(view_handler) = self.views.get(&entity) {
                if let Some(data) = view_handler.downcast_ref::<T>() {
                    return Some(data);
                }
            }
        }

        None
    }
}

// Helper function for computing a rounded corner with variable smoothing
fn compute_smooth_corner(
    corner_radius: f32,
    smoothing: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32, f32, f32, f32) {
    let max_p = f32::min(width, height) / 2.0;
    let corner_radius = f32::min(corner_radius, max_p);

    let p = f32::min((1.0 + smoothing) * corner_radius, max_p);

    let angle_alpha: f32;
    let angle_beta: f32;

    if corner_radius <= max_p / 2.0 {
        angle_alpha = 45.0 * smoothing;
        angle_beta = 90.0 * (1.0 - smoothing);
    } else {
        let diff_ratio = (corner_radius - max_p / 2.0) / (max_p / 2.0);

        angle_alpha = 45.0 * smoothing * (1.0 - diff_ratio);
        angle_beta = 90.0 * (1.0 - smoothing * (1.0 - diff_ratio));
    }

    let angle_theta = (90.0 - angle_beta) / 2.0;
    let dist_p3_p4 = corner_radius * (angle_theta / 2.0).to_radians().tan();

    let l = (angle_beta / 2.0).to_radians().sin() * corner_radius * SQRT_2;
    let c = dist_p3_p4 * angle_alpha.to_radians().cos();
    let d = c * angle_alpha.to_radians().tan();
    let b = (p - l - c - d) / 3.0;
    let a = 2.0 * b;

    (a, b, c, d, l, p, corner_radius)
}
