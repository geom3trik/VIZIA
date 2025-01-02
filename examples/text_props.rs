use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    line_height: LineHeight,
}

pub enum AppEvent {
    SetLineHeight(f32),
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SetLineHeight(val) => {
                self.line_height = match &self.line_height {
                    LineHeight::Normal => LineHeight::Normal,
                    LineHeight::Number(_) => LineHeight::Number(*val),
                    LineHeight::Length(l) => match l {
                        LengthOrPercentage::Percentage(_) => LineHeight::Length(LengthOrPercentage::Percentage(*val)),
                        LengthOrPercentage::Length(_) => LineHeight::Length(Length::px(*val).into()),
                    }
                };
            }
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx|{
        AppData {
            line_height: LineHeight::Number(1.0),
        }.build(cx);

        

        VStack::new(cx, |cx|{

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

            Textbox::new(cx, AppData::line_height.map(|lh| match lh {
                LineHeight::Normal => 1.2,
                LineHeight::Number(num) => *num,
                LineHeight::Length(l) => match l {
                    LengthOrPercentage::Length(ll) => ll.to_px().unwrap(),
                    LengthOrPercentage::Percentage(p) => *p,
                }
            })).width(Pixels(100.0));
            
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
                .line_height(AppData::line_height);
        }).child_space(Stretch(1.0));
        
    }).run()
}
