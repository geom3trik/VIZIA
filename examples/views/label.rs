use vizia::prelude::*;

#[derive(Lens)]
pub struct AppData {
    text: String,
    value: f32,
    checked: bool,
}

#[derive(Debug)]
pub enum AppEvent {
    Toggle,
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::Toggle => {
                self.checked ^= true;
            }
        });
    }
}

#[allow(dead_code)]
const DARK_THEME: &str = "crates/vizia_core/resources/themes/dark_theme.css";
#[allow(dead_code)]
const LIGHT_THEME: &str = "crates/vizia_core/resources/themes/light_theme.css";

fn main() {
    Application::new(|cx| {
        cx.add_stylesheet(DARK_THEME).expect("Failed to find stylesheet");

        AppData {
            text: String::from("As well as model data which implements ToString:"),
            value: 3.141592,
            checked: false,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                //

                Label::new(cx, "A label can display a static string of text.");

                Label::new(cx, AppData::text);

                Label::new(cx, AppData::value);

                Label::new(cx, "Text which is too long for the label will be wrapped.")
                    .width(Pixels(200.0));

                Label::new(cx, "Unless text wrapping is disabled.")
                    .width(Pixels(200.0))
                    .text_wrap(false);

                HStack::new(cx, |cx| {
                    Checkbox::new(cx, AppData::checked)
                        .on_toggle(|cx| cx.emit(AppEvent::Toggle))
                        .id("checkbox_1")
                        .top(Units::Pixels(2.0))
                        .bottom(Units::Pixels(2.0));

                    Label::new(
                        cx,
                        "A label that is describing a form element also acts as a trigger",
                    )
                    .describing("checkbox_1");
                })
                .col_between(Units::Pixels(4.0));

                //
            })
            .size(Auto)
            .col_between(Pixels(10.0))
            .space(Stretch(1.0));
        })
        .class("main")
        .width(Units::Stretch(1.0))
        .height(Units::Stretch(1.0));
    })
    .ignore_default_theme()
    .title("Label")
    .run();
}
