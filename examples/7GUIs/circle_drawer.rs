use vizia::prelude::*;
use vizia::vg::{Paint, Path};

const STYLE: &str = r#"
    :root {
        child-space: 10px;
    }

    .header {
        left: 1s;
        right: 1s;
        col-between: 5px;
    }

    circle-drawer {
        row-between: 10px;
    }

    circle-drawer-canvas {
        border-color: black;
        border-width: 2px;
    }

    .dialog-box {
        child-space: 1s;
        width: 460px;
        height: 100px;
        space: 1s;
        background-color: rgba(255, 255, 255, 0.7);
        border: 1px black;
        border-radius: 10%;
    }

    .dialog-box vstack {
        child-space: 1s;
        row-between: 15px;
    }

    .dialog-box vstack slider {
        width: 400px;
    }
"#;

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    f32::sqrt(f32::powf(x1 - x2, 2.0) + f32::powf(y1 - y2, 2.0))
}

#[derive(Clone, Copy, Data)]
struct Circle {
    x: f32,
    y: f32,
    r: f32,
}

#[derive(Clone, Default, Data, Lens)]
struct CircleData {
    circles: Vec<Circle>,
    selected: Option<usize>,
}

impl CircleData {
    fn add_circle(&mut self, circle: Circle) {
        self.selected = Some(self.circles.len());
        self.circles.push(circle);
    }

    fn change_radius(&mut self, r: f32) {
        if let Some(idx) = self.selected {
            self.circles[idx].r = r;
        }
    }

    fn update_selected(&mut self, x: f32, y: f32) {
        self.selected = self
            .circles
            .iter()
            .enumerate()
            .rev()
            .find(|(_, c)| distance(c.x, c.y, x, y) < c.r)
            .map(|(i, _)| i);
    }

    fn get_selected_radius(&self) -> Option<f32> {
        self.selected.map(|idx| self.circles[idx].r)
    }
}

enum UndoRedoAction {
    Circle(Circle),
    RadiusChange(usize, f32),
}

#[derive(Default, Lens)]
struct UndoRedo {
    undo_list: Vec<UndoRedoAction>,
    redo_list: Vec<UndoRedoAction>,
}

impl UndoRedo {
    fn add_action(&mut self, action: UndoRedoAction) {
        self.undo_list.push(action);
        self.redo_list.clear(); // empty the redo list
    }

    fn undo(&mut self, circles: &mut Vec<Circle>) {
        let last = self.undo_list.pop().unwrap();

        match last {
            UndoRedoAction::Circle(_) => {
                self.redo_list.push(last);
                circles.pop(); // remove the last circle
            }
            UndoRedoAction::RadiusChange(idx, r) => {
                self.redo_list.push(UndoRedoAction::RadiusChange(
                    idx,
                    circles[idx].r, // store the current radius in redo list
                ));
                circles[idx].r = r; // update the radius to the old one
            }
        }
    }

    fn redo(&mut self, circles: &mut Vec<Circle>) {
        let last = self.redo_list.pop().unwrap();

        match last {
            UndoRedoAction::Circle(c) => {
                self.undo_list.push(last);
                circles.push(c);
            }
            UndoRedoAction::RadiusChange(idx, r) => {
                self.undo_list.push(UndoRedoAction::RadiusChange(
                    idx,
                    circles[idx].r, // store the current radius in undo list
                ));
                circles[idx].r = r; // restore the radius
            }
        }
    }
}

#[derive(Default, Lens)]
struct CircleDrawerData {
    circles_data: CircleData,
    /// Undo redo
    undo_redo: UndoRedo,
    radius_before: f32,
    /// is right click menu open
    menu_open: bool,
    menu_posx: Units,
    menu_posy: Units,
    /// is dialog box open
    dialog_open: bool,
}

enum CircleDrawerEvent {
    AddCircle(f32, f32),
    TrySelectCircle(f32, f32),
    ChangeRadius(f32),
    Undo,
    Redo,
    ToggleRightMenu,
    ToggleDialog,
}

impl Model for CircleDrawerData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.take(|event, _| match event {
            CircleDrawerEvent::AddCircle(x, y) => {
                let circle =
                    Circle { x: cx.physical_to_logical(x), y: cx.physical_to_logical(y), r: 26.0 };
                self.circles_data.add_circle(circle);
                self.undo_redo.add_action(UndoRedoAction::Circle(circle));
            }
            CircleDrawerEvent::TrySelectCircle(x, y) => {
                if !(self.dialog_open || self.menu_open) {
                    self.circles_data
                        .update_selected(cx.physical_to_logical(x), cx.physical_to_logical(y))
                }
            }
            CircleDrawerEvent::ChangeRadius(r) => self.circles_data.change_radius(r),
            CircleDrawerEvent::Undo => self.undo_redo.undo(&mut self.circles_data.circles),
            CircleDrawerEvent::Redo => self.undo_redo.redo(&mut self.circles_data.circles),
            CircleDrawerEvent::ToggleRightMenu => {
                if !self.menu_open && self.circles_data.selected.is_some() {
                    let (x, y) = cx.mouse().right.pos_down;

                    self.menu_open = true;
                    self.menu_posx = Pixels(cx.physical_to_logical(x));
                    self.menu_posy = Pixels(cx.physical_to_logical(y));
                } else {
                    self.menu_open = false;
                }

                if !self.dialog_open {
                    let x = cx.physical_to_logical(cx.mouse().cursorx);
                    let y = cx.physical_to_logical(cx.mouse().cursory);

                    self.circles_data.update_selected(x, y);
                }
            }
            CircleDrawerEvent::ToggleDialog => {
                self.dialog_open ^= true;

                let radius = self.circles_data.get_selected_radius().unwrap();

                if self.dialog_open {
                    // if dialog just opened save the current radius as before radius
                    self.radius_before = radius;
                } else {
                    if self.radius_before != radius {
                        self.undo_redo.add_action(UndoRedoAction::RadiusChange(
                            self.circles_data.selected.unwrap(),
                            self.radius_before,
                        ));
                    }

                    let x = cx.physical_to_logical(cx.mouse().cursorx);
                    let y = cx.physical_to_logical(cx.mouse().cursory);

                    self.circles_data.update_selected(x, y);
                }
            }
        });
    }
}

struct CircleDrawerCanvas;

impl CircleDrawerCanvas {
    fn new(cx: &mut Context, lens: impl Lens<Target = CircleData>) -> Handle<Self> {
        Self.build(cx, |cx| {
            Binding::new(cx, lens, |cx, _| cx.needs_redraw());
        })
        .overflow(Overflow::Hidden)
    }
}

impl View for CircleDrawerCanvas {
    fn element(&self) -> Option<&'static str> {
        Some("circle-drawer-canvas")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event, _| match event {
            WindowEvent::MouseDown(button) => match button {
                MouseButton::Left => {
                    let x = cx.mouse().cursorx;
                    let y = cx.mouse().cursory;

                    cx.emit(CircleDrawerEvent::AddCircle(x, y))
                }
                MouseButton::Right => cx.emit(CircleDrawerEvent::ToggleRightMenu),
                _ => (),
            },
            WindowEvent::MouseMove(x, y) => cx.emit(CircleDrawerEvent::TrySelectCircle(*x, *y)),
            _ => (),
        })
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let mut path = cx.build_path();
        cx.draw_border(canvas, &mut path);

        let paint = Paint::color(Color::black().into()).with_line_width(2.0);

        let circle_data = CircleDrawerData::circles_data.get(cx);
        for (idx, Circle { x, y, r }) in circle_data.circles.iter().copied().enumerate() {
            let mut path = Path::new();
            path.circle(cx.logical_to_physical(x), cx.logical_to_physical(y), r);

            if circle_data.selected.is_some_and(|i| i == idx) {
                let paint = Paint::color(Color::gray().into());
                canvas.fill_path(&path, &paint);
            }

            canvas.stroke_path(&path, &paint);
        }
    }
}

struct CircleDrawer;

impl CircleDrawer {
    fn new(cx: &mut Context) -> Handle<Self> {
        cx.add_stylesheet(STYLE).expect("Failed to add stylesheet");

        CircleDrawerData::default().build(cx);

        Self.build(cx, |cx| {
            Binding::new(cx, CircleDrawerData::menu_open, |cx, is_open| {
                if is_open.get(cx) {
                    Popup::new(cx, |cx| {
                        Button::new(cx, |cx| Label::new(cx, "Adjust diameter..")).on_press(|cx| {
                            cx.emit(CircleDrawerEvent::ToggleDialog);
                            cx.emit(CircleDrawerEvent::ToggleRightMenu);
                        });
                    })
                    .size(Auto)
                    .left(CircleDrawerData::menu_posx)
                    .top(CircleDrawerData::menu_posy)
                    .on_blur(|cx| cx.emit(CircleDrawerEvent::ToggleRightMenu))
                    .lock_focus_to_within();
                }
            });

            Binding::new(cx, CircleDrawerData::dialog_open, |cx, is_open| {
                if is_open.get(cx) {
                    Popup::new(cx, |cx| {
                        let selected = CircleDrawerData::circles_data
                            .then(CircleData::selected)
                            .get(cx)
                            .unwrap();

                        VStack::new(cx, |cx| {
                            Label::new(
                                cx,
                                &format!(
                                    "Adjust diameter of circle at {:?}.",
                                    CircleDrawerData::circles_data
                                        .then(CircleData::circles)
                                        .get(cx)
                                        .get(selected)
                                        .map(|c| (c.x, c.y))
                                        .unwrap()
                                ),
                            );

                            Slider::new(
                                cx,
                                CircleDrawerData::circles_data
                                    .then(CircleData::circles)
                                    .index(selected)
                                    .map(|c| c.r),
                            )
                            .range(4.0..150.0)
                            .on_changing(|cx, value| {
                                cx.emit(CircleDrawerEvent::ChangeRadius(value))
                            });
                        })
                        .size(Auto);
                    })
                    .on_blur(|cx| cx.emit(CircleDrawerEvent::ToggleDialog))
                    .class("dialog-box")
                    .top(Stretch(1.0))
                    .bottom(Stretch(0.01))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0));
                }
            });

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "Undo"))
                    .disabled(
                        CircleDrawerData::undo_redo.then(UndoRedo::undo_list).map(|v| v.is_empty()),
                    )
                    .on_press(|cx| cx.emit(CircleDrawerEvent::Undo));

                Button::new(cx, |cx| Label::new(cx, "Redo"))
                    .disabled(
                        CircleDrawerData::undo_redo.then(UndoRedo::redo_list).map(|v| v.is_empty()),
                    )
                    .on_press(|cx| cx.emit(CircleDrawerEvent::Redo));
            })
            .size(Auto)
            .class("header");

            CircleDrawerCanvas::new(cx, CircleDrawerData::circles_data);
        })
    }
}

impl View for CircleDrawer {
    fn element(&self) -> Option<&'static str> {
        Some("circle-drawer")
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx: &mut Context| {
        CircleDrawer::new(cx);
    })
    .title("Circle drawer")
    .run()
}
