use vizia::*;
const STYLE: &str = r#"
    label {
        font-size: 20;
        color: #C2C2C2;
    }
    
    knob {
        width: 100px;
        height: 100px;
    }
    
    knob .track {
        background-color: #ffb74d;
    }
    .label_knob {
        border-width: 2px;
        border-color: #28282b;
        background-color: #000000;
        col-between: 10px;
        child-space: 1s;
    }
"#;
fn main() {
    Application::new(
        WindowDescription::new().with_title("More Knobs").with_inner_size(1000, 200),
        |cx| {
            cx.add_theme(STYLE);
            KnobData { knobs: vec![0.5; 5] }.build(cx);

            HStack::new(cx, |cx| {
                Binding::new(cx, KnobData::knobs, move |cx, knobs| {
                    let radius = 50.0;
                    // default knob
                    VStack::new(cx, move |cx| {
                        Knob::new(cx, 0.5, knobs.get(cx)[0], false)
                            .on_changing(move |knob, cx| {
                                cx.emit(KnobChangeEvent::SetKnob(0, knob.normalized_value))
                            })
                            .color(Color::red());
                        Label::new(cx, "Default knob");
                    })
                    .child_space(Stretch(1.));
                    // simple tick knob
                    VStack::new(cx, move |cx| {
                        Knob::custom(cx, 0.5, knobs.get(cx)[1], move |cx, val| {
                            // FIXME: Using this for radius resulted in a memory leak??
                            // let height = cx.cache.get_height(cx.current);
                            // let width = cx.cache.get_width(cx.current);
                            // let radius = height.min(width) / 2.;
                            TickKnob::new(cx, val, Pixels(radius), Percentage(25.), 300., 0)
                                .class("track")
                        })
                        .on_changing(move |knob, cx| {
                            cx.emit(KnobChangeEvent::SetKnob(1, knob.normalized_value))
                        });
                        Label::new(cx, "Tick knob");
                    })
                    .child_space(Stretch(1.));
                    // steppy knob
                    VStack::new(cx, move |cx| {
                        Knob::custom(cx, 0.5, knobs.get(cx)[2], move |cx, val| {
                            // FIXME: Using this for radius resulted in a memory leak??
                            // let height = cx.cache.get_height(cx.current);
                            // let width = cx.cache.get_width(cx.current);
                            // let radius = height.min(width) / 2.;
                            let steps = 5;
                            TickKnob::new(
                                cx,
                                val,
                                Pixels(radius * 0.60),
                                Percentage(15.0),
                                300.,
                                steps,
                            )
                            .class("track");
                            Ticks::new(cx, Pixels(radius), Percentage(25.), 300., steps)
                                .class("track")
                            // TODO: cyan is yellow?
                            // .background_color(Color::cyan())
                        })
                        .on_changing(move |knob, cx| {
                            cx.emit(KnobChangeEvent::SetKnob(2, knob.normalized_value))
                        });
                        Label::new(cx, "Steppy knob");
                    })
                    .child_space(Stretch(1.));
                    // Arc+tick knob knob
                    VStack::new(cx, move |cx| {
                        Knob::custom(cx, 0.5, knobs.get(cx)[3], move |cx, val| {
                            // FIXME: Using this for radius resulted in a memory leak??
                            // let height = cx.cache.get_height(cx.current);
                            // let width = cx.cache.get_width(cx.current);
                            // let radius = height.min(width) / 2.;
                            TickKnob::new(
                                cx,
                                val,
                                Pixels(radius * 0.9),
                                Percentage(0.0),
                                300.,
                                0,
                            )
                            .class("track");
                            Slider::new(cx, 0.5, Orientation::Horizontal).height(Pixels(50.)).width(Pixels(50.));
                            ArcTrack::new(
                                cx,
                                val,
                                false,
                                Pixels(radius),
                                Percentage(10.),
                                300.,
                            )
                            .class("track")
                        })
                        .on_changing(move |knob, cx| {
                            cx.emit(KnobChangeEvent::SetKnob(3, knob.normalized_value))
                        });
                        Label::new(cx, "Arc knob");
                    })
                    .child_space(Stretch(1.));
                    // drag-able label
                    VStack::new(cx, move |cx| {
                        Knob::custom(cx, 0.5, knobs.get(cx)[4], move |cx, val| {
                            HStack::new(cx, move |cx| {
                                Label::new(cx, "val:")
                                    .width(Pixels(radius - 10.))
                                    .right(Stretch(1.));
                                //     .class("blue_label");
                                Label::new(cx, &format!("{:.2}", val)).width(Pixels(radius - 10.));
                            })
                            .class("label_knob")
                        })
                        .on_changing(move |knob, cx| {
                            cx.emit(KnobChangeEvent::SetKnob(4, knob.normalized_value))
                        });
                        Label::new(cx, "Label \"knob\"");
                    })
                    .child_space(Stretch(1.));
                    // TODO: Add a hidden value knob?
                });
            })
            .col_between(Pixels(10.))
            .width(Pixels(500.))
            .height(Pixels(200.))
            .background_color(Color::from("#191919"));
        },
    )
    .run();
}

#[derive(Lens)]
pub struct KnobData {
    knobs: Vec<f32>,
}

#[derive(Debug)]
pub enum KnobChangeEvent {
    SetKnob(usize, f32),
}
impl Model for KnobData {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        if let Some(param_change_event) = event.message.downcast() {
            match param_change_event {
                KnobChangeEvent::SetKnob(idx, new_val) => {
                    self.knobs[*idx] = *new_val;
                }
            }
        }
    }
}
