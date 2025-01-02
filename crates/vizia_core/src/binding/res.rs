use std::ops::Range;

use crate::prelude::*;

#[macro_export]
macro_rules! impl_res_simple {
    ($t:ty) => {
        impl ResGet<$t> for $t {
            fn get_ref<'a>(&self, _: &impl DataContext) -> Option<LensValue<$t>> {
                Some(LensValue::Borrowed(self))
            }

            fn get(&self, _: &impl DataContext) -> $t {
                *self
            }
        }

        impl Res<$t> for $t {}
    };
}

macro_rules! impl_res_clone {
    ($t:ty) => {
        impl ResGet<$t> for $t {
            fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, $t>> {
                Some(LensValue::Borrowed(self))
            }

            fn get(&self, _: &impl DataContext) -> $t {
                self.clone()
            }
        }

        impl Res<$t> for $t {}
    };
}

pub trait ResGet<T> {
    fn get_ref<'a>(&'a self, cx: &'a impl DataContext) -> Option<LensValue<'a, T>>;

    fn get(&self, _: &impl DataContext) -> T;
}

/// A trait which allows passing a value or a lens to a view or modifier.
///
/// For example, the `Label` view constructor takes a type which implements `Res<T>` where
/// `T` implements `ToString`. This allows the user to pass a type which implements `ToString`,
/// such as `String` or `&str`, or a lens to a type which implements `ToString`.
pub trait Res<T>: ResGet<T> {
    fn set_or_bind<F>(self, cx: &mut Context, entity: Entity, closure: F)
    where
        Self: Sized,
        F: 'static + Clone + Fn(&mut Context, Self),
    {
        cx.with_current(entity, |cx| {
            (closure)(cx, self);
        });
    }
}

impl<L> ResGet<L::Target> for L
where
    L: Lens<Target: Clone>,
{
    fn get_ref<'a>(&'a self, cx: &'a impl DataContext) -> Option<LensValue<'a, L::Target>> {
        self.view(
            cx.data()
                .unwrap_or_else(|| panic!("Failed to get data from context for lens: {:?}", self)),
        )
    }

    fn get(&self, cx: &impl DataContext) -> L::Target {
        self.get_ref(cx).unwrap().into_owned()
    }
}

impl<L> Res<L::Target> for L
where
    L: Lens<Target: Data>,
{
    fn set_or_bind<F>(self, cx: &mut Context, entity: Entity, closure: F)
    where
        F: 'static + Fn(&mut Context, Self),
    {
        // cx.with_current(entity, |cx| {
        Binding::new(cx, self, move |cx, val| {
            cx.with_current(entity, |cx| {
                (closure)(cx, val);
            });
        });
        // });
        // });
    }
}

impl_res_simple!(i8);
impl_res_simple!(i16);
impl_res_simple!(i32);
impl_res_simple!(i64);
impl_res_simple!(i128);
impl_res_simple!(isize);
impl_res_simple!(u8);
impl_res_simple!(u16);
impl_res_simple!(u32);
impl_res_simple!(u64);
impl_res_simple!(u128);
impl_res_simple!(usize);
impl_res_simple!(char);
impl_res_simple!(bool);
impl_res_simple!(f32);
impl_res_simple!(f64);
impl_res_simple!(CursorIcon);
impl_res_simple!(Overflow);
impl_res_simple!(LengthValue);
impl_res_simple!(FontWeight);
impl_res_simple!(FontWeightKeyword);
impl_res_simple!(FontSlant);
impl_res_simple!(CornerShape);
impl_res_simple!(Angle);
impl_res_simple!(TextAlign);
impl_res_simple!(TextOverflow);
impl_res_simple!(LineClamp);
impl_res_clone!(Shadow);
impl_res_clone!(LinearGradientBuilder);
impl_res_clone!(ShadowBuilder);
impl_res_simple!(FontVariation);
impl_res_clone!(Filter);
impl_res_simple!(Opacity);
impl_res_simple!(FontWidth);
impl_res_clone!(Translate);
impl_res_clone!(Scale);
impl_res_clone!(Position);
impl_res_simple!(PointerEvents);
impl_res_simple!(ButtonVariant);
impl_res_simple!(AvatarVariant);
impl_res_clone!(FamilyOwned);
impl_res_simple!(TextDecorationLine);
impl_res_clone!(TextStroke);
impl_res_clone!(TextStrokeStyle);
impl_res_clone!(LineHeight);
impl_res_clone!(Range<f32>);

impl<'i> ResGet<FontFamily<'i>> for FontFamily<'i> {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl<'i> Res<FontFamily<'i>> for FontFamily<'i> {}

impl<'i> ResGet<BackgroundImage<'i>> for BackgroundImage<'i> {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl<'i> Res<BackgroundImage<'i>> for BackgroundImage<'i> {}

impl<'s> ResGet<&'s str> for &'s str {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> &'s str {
        self
    }
}

impl<'s> Res<&'s str> for &'s str {}

impl<'s> ResGet<&'s String> for &'s String {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self
    }
}

impl<'s> Res<&'s String> for &'s String {}

impl ResGet<String> for String {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl Res<String> for String {}

impl ResGet<Transform> for Transform {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Transform {
        self.clone()
    }
}

impl Res<Transform> for Transform {}

impl ResGet<Color> for Color {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Color {
        *self
    }
}

impl Res<Color> for Color {}

impl ResGet<LinearGradient> for LinearGradient {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> LinearGradient {
        self.clone()
    }
}

impl Res<LinearGradient> for LinearGradient {}

impl ResGet<Units> for Units {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Units {
        *self
    }
}

impl Res<Units> for Units {}

impl ResGet<Visibility> for Visibility {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Visibility {
        *self
    }
}

impl Res<Visibility> for Visibility {}

impl ResGet<Display> for Display {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Display {
        *self
    }
}

impl Res<Display> for Display {}

impl ResGet<LayoutType> for LayoutType {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> LayoutType {
        *self
    }
}
impl Res<LayoutType> for LayoutType {}

impl ResGet<PositionType> for PositionType {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> PositionType {
        *self
    }
}

impl Res<PositionType> for PositionType {}

impl<T: Clone + ResGet<T>> ResGet<Option<T>> for Option<T> {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Option<T> {
        self.clone()
    }
}

impl<T: Clone + ResGet<T>> Res<Option<T>> for Option<T> {}

impl ResGet<Length> for Length {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl Res<Length> for Length {}

impl ResGet<LengthOrPercentage> for LengthOrPercentage {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl Res<LengthOrPercentage> for LengthOrPercentage {}

impl ResGet<RGBA> for RGBA {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        *self
    }
}

impl Res<RGBA> for RGBA {}

impl<T: Clone + ResGet<T>> ResGet<Vec<T>> for Vec<T> {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Vec<T> {
        self.clone()
    }
}

impl<T: Clone + ResGet<T>> Res<Vec<T>> for Vec<T> {}

impl<T: Clone + ResGet<T>, const N: usize> ResGet<[T; N]> for [T; N] {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _: &impl DataContext) -> Self {
        self.clone()
    }
}

impl<T: Clone + ResGet<T>, const N: usize> Res<[T; N]> for [T; N] {}

impl<T1: Clone, T2: Clone> ResGet<(T1, T2)> for (T1, T2) {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _cx: &impl DataContext) -> (T1, T2) {
        self.clone()
    }
}

impl<T1: Clone, T2: Clone> Res<(T1, T2)> for (T1, T2) {}

impl<T1: Clone, T2: Clone, T3: Clone> ResGet<(T1, T2, T3)> for (T1, T2, T3) {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _cx: &impl DataContext) -> (T1, T2, T3) {
        self.clone()
    }
}

impl<T1: Clone, T2: Clone, T3: Clone> Res<(T1, T2, T3)> for (T1, T2, T3) {}

impl<T1: Clone, T2: Clone, T3: Clone, T4: Clone> ResGet<(T1, T2, T3, T4)> for (T1, T2, T3, T4) {
    fn get_ref<'a>(&'a self, _: &'a impl DataContext) -> Option<LensValue<'a, Self>> {
        Some(LensValue::Borrowed(self))
    }

    fn get(&self, _cx: &impl DataContext) -> (T1, T2, T3, T4) {
        self.clone()
    }
}

impl<T1: Clone, T2: Clone, T3: Clone, T4: Clone> Res<(T1, T2, T3, T4)> for (T1, T2, T3, T4) {}
