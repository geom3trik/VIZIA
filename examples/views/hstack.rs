use vizia::*;

const COLORS: [Color; 3] = [Color::red(), Color::green(), Color::blue()];

fn main() {
    Application::new(WindowDescription::new().with_title("ZStack"), |cx| {
        Label::new(cx, "A HStack arranges its children horizontally.")
            .width(Stretch(1.0))
            .position_type(PositionType::SelfDirected)
            .space(Pixels(10.0));

        HStack::new(cx, |cx| {
            for i in 0..3 {
                Element::new(cx).size(Pixels(100.0)).background_color(COLORS[i]);
            }
        })
        .left(Pixels(10.0))
        .top(Pixels(50.0));
    })
    .run();
}