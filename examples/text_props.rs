use vizia::{icons::ICON_CHEVRON_DOWN, prelude::*};

#[derive(Lens)]
pub struct AppData {
    line_height: LineHeight,
    list: Vec<String>,
    selected: usize,

    letter_spacing: Length,
    word_spacing: Length,

    font_size: f32,

    pub fonts: Vec<String>,
    pub selected_font: usize,
    pub font: String,
}

pub enum AppEvent {
    SetLineHeight(f32),
    SetLineHeight2(LineHeight),
    SetLineHeightType(usize),
    SetLetterSpacing(f32),
    SetWordSpacing(f32),
    SetFontSize(f32),
    SetFont(usize),
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetLineHeight(val) => {
                self.line_height = match &self.line_height {
                    LineHeight::Normal => LineHeight::Normal,
                    LineHeight::Number(_) => LineHeight::Number(*val),
                    LineHeight::Length(l) => match l {
                        LengthOrPercentage::Percentage(_) => {
                            LineHeight::Length(LengthOrPercentage::Percentage(*val))
                        }
                        LengthOrPercentage::Length(_) => {
                            LineHeight::Length(Length::px(*val).into())
                        }
                    },
                };
            }

            AppEvent::SetLineHeight2(val) => {
                self.line_height = val.clone();
            }

            AppEvent::SetLineHeightType(ty) => {
                self.selected = *ty;
                self.line_height = match ty {
                    0 => LineHeight::Normal,
                    1 => LineHeight::Number(1.2),
                    2 => LineHeight::Length(Length::px(16.0).into()),
                    3 => LineHeight::Length(Percentage(120.0).into()),
                    _ => LineHeight::Normal,
                }
            }

            AppEvent::SetLetterSpacing(val) => {
                self.letter_spacing = Length::px(*val);
            }

            AppEvent::SetWordSpacing(val) => {
                self.word_spacing = Length::px(*val);
            }

            AppEvent::SetFontSize(val) => {
                self.font_size = *val;
            }

            AppEvent::SetFont(val) => {
                self.selected_font = *val;
                self.font = self.fonts[self.selected_font].clone();
            }
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx|{
        AppData {
            line_height: LineHeight::Number(1.2),
            list: vec!["Normal".to_string(), "Number".to_string(), "Pixels".to_string(), "Percentage".to_string()],
            selected: 0,

            letter_spacing: Length::px(0.0),
            word_spacing: Length::px(0.0),
            font_size: 16.0,

            fonts: cx.text_context.default_font_manager.family_names().collect(),
            selected_font: 0,
            font: String::new(),
        }.build(cx);

        HStack::new(cx, |cx|{
            VStack::new(cx, |cx|{
                Label::new(cx, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.")
                    .border_color(Color::black())
                    .border_width(Pixels(1.0))
                    .text_wrap(true)
                    .width(Pixels(200.0))
                    .child_space(Pixels(10.0));

                Label::new(cx, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.")
                    .border_color(Color::black())
                    .border_width(Pixels(1.0))
                    .text_wrap(true)
                    .width(Pixels(200.0))
                    .child_space(Pixels(10.0))
                    .font_family(AppData::font.map(|fam| vec![FamilyOwned::Named(fam.clone())]))
                    .font_size(AppData::font_size)
                    .letter_spacing(AppData::letter_spacing)
                    .word_spacing(AppData::word_spacing)
                    .line_height(AppData::line_height);
            })
            .row_between(Pixels(8.0))
            .child_space(Stretch(1.0));

            Divider::new(cx);

            VStack::new(cx, |cx|{

                ComboBox::new(cx, AppData::fonts, AppData::selected_font)
                    .width(Stretch(1.0))
                    .on_select(|cx, val| cx.emit(AppEvent::SetFont(val)));

                HStack::new(cx, |cx|{

                    Slider::new(cx, AppData::font_size)
                        .range(0.0..40.0)
                        .on_changing(|cx, val| cx.emit(AppEvent::SetFontSize(val)));
                    

                    Textbox::new(cx, AppData::font_size.map(|ls| format!("{:.1}px", *ls))).width(Pixels(100.0))
                        .on_submit(|cx, val, _| cx.emit(AppEvent::SetFontSize(val.strip_suffix("px").unwrap().parse::<f32>().unwrap())))
                        .width(Pixels(100.0));
                })
                .height(Auto)
                .col_between(Pixels(8.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));

                HStack::new(cx, |cx|{

                    Slider::new(cx, AppData::letter_spacing.map(|lh| lh.to_px().unwrap()))
                        .range(0.0..20.0)
                        .on_changing(|cx, val| cx.emit(AppEvent::SetLetterSpacing(val)));
                    

                    Textbox::new(cx, AppData::letter_spacing.map(|ls| format!("{:.1}px", ls.to_px().unwrap()))).width(Pixels(100.0))
                        .on_submit(|cx, val, _| cx.emit(AppEvent::SetLetterSpacing(val.strip_suffix("px").unwrap().parse::<f32>().unwrap())))
                        .width(Pixels(100.0));
                })
                .height(Auto)
                .col_between(Pixels(8.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));

                HStack::new(cx, |cx|{

                    Slider::new(cx, AppData::word_spacing.map(|lh| lh.to_px().unwrap()))
                        .range(0.0..20.0)
                        .on_changing(|cx, val| cx.emit(AppEvent::SetWordSpacing(val)));
                    

                    Textbox::new(cx, AppData::word_spacing.map(|ls| format!("{:.1}px", ls.to_px().unwrap()))).width(Pixels(100.0))
                        .on_submit(|cx, val, _| cx.emit(AppEvent::SetWordSpacing(val.strip_suffix("px").unwrap().parse::<f32>().unwrap())))
                        .width(Pixels(100.0));
                })
                .height(Auto)
                .col_between(Pixels(8.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));

                HStack::new(cx, |cx|{
                    Slider::new(cx, AppData::line_height.map(|lh| match lh {
                        LineHeight::Normal => 1.2,
                        LineHeight::Number(num) => *num,
                        LineHeight::Length(l) => match l {
                            LengthOrPercentage::Length(ll) => ll.to_px().unwrap(),
                            LengthOrPercentage::Percentage(p) => *p,
                        }
                    }))
                    .range(AppData::line_height.map(|lh| match lh {
                        LineHeight::Normal => 0.0..2.0,
                        LineHeight::Number(_) => 0.0..2.0,
                        LineHeight::Length(l) => match l {
                            LengthOrPercentage::Length(_) => 0.0..40.0,
                            LengthOrPercentage::Percentage(_) => 0.0..200.0,
                        }
                    }))
                    .on_changing(|cx, val| cx.emit(AppEvent::SetLineHeight(val)));

                    Dropdown::new(
                        cx,
                        move |cx| {
                            ButtonGroup::new(cx, |cx| {
                                Textbox::new(cx, AppData::line_height).width(Pixels(100.0))
                                .on_submit(|cx, val, _| cx.emit(AppEvent::SetLineHeight2(val)))
                                .width(Stretch(1.0));

                                Button::new(cx, |cx| {
                                    Svg::new(cx, ICON_CHEVRON_DOWN)
                                        .class("icon")
                                        .size(Pixels(16.0))
                                        .hoverable(false)
                                })
                                .on_press(|cx| cx.emit(PopupEvent::Switch));
                            }).width(Stretch(1.0));
                        },
                        move |cx| {
                            ScrollView::new(cx, 0.0, 0.0, false, true, move |cx| {
                                List::new(cx, AppData::list, move |cx, index, item| {
                                    Label::new(cx, item)
                                        .child_top(Stretch(1.0))
                                        .child_bottom(Stretch(1.0))
                                        .checked(AppData::selected.map(move |selected| *selected == index))
                                        .navigable(true)
                                        .on_press(move |cx| {
                                            cx.emit(AppEvent::SetLineHeightType(index));
                                            cx.emit(PopupEvent::Close);
                                        });
                                });
                            })
                            .height(Auto);
                        },
                    )
                    .width(Pixels(100.0));
                })
                .height(Auto)
                .col_between(Pixels(8.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));

                
            })
            .width(Pixels(250.0))
            .row_between(Pixels(8.0))
            .child_space(Pixels(8.0));
        });
    }).run()
}
