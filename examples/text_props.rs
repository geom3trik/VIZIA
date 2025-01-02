use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {}

pub enum AppEvent {}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {}
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx|{
        AppData {}.build(cx);
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
                .word_spacing(Pixels(10.0));
        }).child_space(Stretch(1.0));
        
    }).run()
}
