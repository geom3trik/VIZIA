use std::{collections::HashMap, error::Error, sync::Arc};

use crate::{
    convert::{winit_key_code_to_code, winit_key_to_key},
    window::{WinState, Window},
    window_modifiers::WindowModifiers,
};

// #[cfg(feature = "accesskit")]
// use accesskit::{Action, NodeBuilder, NodeId, TreeUpdate};
// #[cfg(feature = "accesskit")]
// use accesskit_winit;
// use std::cell::RefCell;
use vizia_core::context::EventProxy;
use vizia_core::prelude::*;
use vizia_core::{backend::*, events::EventManager};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    error::EventLoopError,
    event::ElementState,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{NativeKeyCode, PhysicalKey},
    platform::windows::WindowAttributesExtWindows,
    raw_window_handle::HasWindowHandle,
    window::{WindowAttributes, WindowId, WindowLevel},
};
// #[cfg(all(
//     feature = "clipboard",
//     feature = "wayland",
//     any(
//         target_os = "linux",
//         target_os = "dragonfly",
//         target_os = "freebsd",
//         target_os = "netbsd",
//         target_os = "openbsd"
//     )
// ))]
// use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle};
use vizia_window::Position;

#[derive(Debug)]
pub enum UserEvent {
    Event(Event),
    #[cfg(feature = "accesskit")]
    AccessKitActionRequest(accesskit_winit::ActionRequestEvent),
}

#[cfg(feature = "accesskit")]
impl From<accesskit_winit::ActionRequestEvent> for UserEvent {
    fn from(action_request_event: accesskit_winit::ActionRequestEvent) -> Self {
        UserEvent::AccessKitActionRequest(action_request_event)
    }
}

impl From<vizia_core::events::Event> for UserEvent {
    fn from(event: vizia_core::events::Event) -> Self {
        UserEvent::Event(event)
    }
}

type IdleCallback = Option<Box<dyn Fn(&mut Context)>>;

#[derive(Debug)]
pub enum ApplicationError {
    EventLoopError(EventLoopError),
    LogError,
}

///Creating a new application creates a root `Window` and a `Context`. Views declared within the closure passed to `Application::new()` are added to the context and rendered into the root window.
///
/// # Example
/// ```no_run
/// # use vizia_core::prelude::*;
/// # use vizia_winit::application::Application;
/// Application::new(|cx|{
///    // Content goes here
/// })
/// .run();
///```
/// Calling `run()` on the `Application` causes the program to enter the event loop and for the main window to display.
pub struct Application {
    cx: BackendContext,
    event_manager: EventManager,
    pub(crate) event_loop: Option<EventLoop<UserEvent>>,
    on_idle: IdleCallback,
    window_description: WindowDescription,
    control_flow: ControlFlow,
    event_loop_proxy: EventLoopProxy<UserEvent>,
    windows: HashMap<WindowId, WinState>,
    window_ids: HashMap<Entity, WindowId>,
}

pub struct WinitEventProxy(EventLoopProxy<UserEvent>);

impl EventProxy for WinitEventProxy {
    fn send(&self, event: Event) -> Result<(), ()> {
        self.0.send_event(UserEvent::Event(event)).map_err(|_| ())
    }

    fn make_clone(&self) -> Box<dyn EventProxy> {
        Box::new(WinitEventProxy(self.0.clone()))
    }
}

impl Application {
    pub fn new<F>(content: F) -> Self
    where
        F: 'static + FnOnce(&mut Context),
    {
        let context = Context::new();

        let event_loop =
            EventLoop::<UserEvent>::with_user_event().build().expect("Failed to create event loop");

        let mut cx = BackendContext::new(context);
        let event_proxy_obj = event_loop.create_proxy();
        cx.set_event_proxy(Box::new(WinitEventProxy(event_proxy_obj)));

        cx.renegotiate_language();
        cx.0.remove_user_themes();
        (content)(cx.context());

        let proxy = event_loop.create_proxy();

        Self {
            cx,
            event_manager: EventManager::new(),
            event_loop: Some(event_loop),
            on_idle: None,
            window_description: WindowDescription::new(),
            control_flow: ControlFlow::Wait,
            event_loop_proxy: proxy,
            windows: HashMap::new(),
            window_ids: HashMap::new(),
        }
    }

    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_entity: Entity,
        window_description: &WindowDescription,
        owner: Option<Arc<winit::window::Window>>,
    ) -> Result<Arc<winit::window::Window>, Box<dyn Error>> {
        let mut window_attributes = apply_window_description(window_description);

        if let Some(owner) = owner {
            use winit::raw_window_handle::RawWindowHandle::Win32;
            let Win32(handle) = owner.window_handle().unwrap().as_raw() else {
                unreachable!();
            };
            window_attributes =
                window_attributes.with_owner_window(handle.hwnd.get()).with_decorations(false);
        }

        let window = event_loop.create_window(window_attributes)?;

        let window = Arc::new(window);

        let mut window_state = WinState::new(event_loop, window.clone(), window_entity)?;

        // // On windows cloak (hide) the window initially, we later reveal it after the first draw.
        // // This is a workaround to hide the "white flash" that occurs during application startup.
        // #[cfg(target_os = "windows")]
        // {
        //     window_state.is_initially_cloaked = window_state.set_cloak(true);
        // }

        let window_id = window_state.window.id();
        self.windows.insert(window_id, window_state);
        self.window_ids.insert(window_entity, window_id);
        Ok(window)
    }

    /// Sets the default built-in theming to be ignored.
    pub fn ignore_default_theme(mut self) -> Self {
        self.cx.context().ignore_default_theme = true;
        self
    }

    pub fn should_poll(mut self) -> Self {
        self.control_flow = ControlFlow::Poll;

        self
    }

    /// Takes a closure which will be called at the end of every loop of the application.
    ///
    /// The callback provides a place to run 'idle' processing and happens at the end of each loop but before drawing.
    /// If the callback pushes events into the queue in state then the event loop will re-run. Care must be taken not to
    /// push events into the queue every time the callback runs unless this is intended.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use vizia_core::prelude::*;
    /// # use vizia_winit::application::Application;
    /// #
    /// Application::new(|cx| {
    ///     // Build application here
    /// })
    /// .on_idle(|cx| {
    ///     // Code here runs at the end of every event loop after OS and vizia events have been handled
    /// })
    /// .run();
    /// ```
    pub fn on_idle<F: 'static + Fn(&mut Context)>(mut self, callback: F) -> Self {
        self.on_idle = Some(Box::new(callback));

        self
    }

    /// Returns a `ContextProxy` which can be used to send events from another thread.
    pub fn get_proxy(&self) -> ContextProxy {
        self.cx.0.get_proxy()
    }

    pub fn run(mut self) -> Result<(), ApplicationError> {
        self.event_loop.take().unwrap().run_app(&mut self).map_err(ApplicationError::EventLoopError)
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, user_event: UserEvent) {
        match user_event {
            UserEvent::Event(event) => {
                self.cx.send_event(event);
            }

            #[cfg(feature = "accesskit")]
            UserEvent::AccessKitActionRequest(action_request_event) => {
                let node_id = action_request_event.request.target;

                if action_request_event.request.action != Action::ScrollIntoView {
                    let entity = Entity::new(node_id.0 as u64, 0);

                    // Handle focus action from screen reader
                    if action_request_event.request.action == Action::Focus {
                        cx.0.with_current(entity, |cx| {
                            cx.focus();
                        });
                    }

                    cx.send_event(
                        Event::new(WindowEvent::ActionRequest(action_request_event.request))
                            .direct(entity),
                    );
                }
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let main_window: Arc<winit::window::Window> = self
            .create_window(event_loop, Entity::root(), &self.window_description.clone(), None)
            .expect("failed to create initial window");
        self.cx.add_main_window(Entity::root(), &self.window_description, 1.0);
        self.cx.add_window(Window { window: Some(main_window.clone()) });

        self.cx.0.windows.insert(
            Entity::root(),
            WindowState {
                window_description: self.window_description.clone(),
                ..Default::default()
            },
        );

        self.cx.0.remove_user_themes();

        for (window_entity, window_state) in self.cx.0.windows.clone().into_iter() {
            if window_entity == Entity::root() {
                continue;
            }
            let window = self
                .create_window(
                    event_loop,
                    window_entity,
                    &window_state.window_description,
                    Some(main_window.clone()),
                )
                .expect("Failed to create window");
            self.cx.add_main_window(window_entity, &window_state.window_description, 1.0);
            self.cx.mutate_window(window_entity, |_, win: &mut Window| {
                win.window = Some(window.clone())
            });
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        let window = match self.windows.get_mut(&window_id) {
            Some(window) => window,
            None => return,
        };

        match event {
            winit::event::WindowEvent::Resized(size) => {
                window.resize(size);
                self.cx.set_window_size(window.entity, size.width as f32, size.height as f32);
                self.cx.needs_refresh();
                window.window().request_redraw();

                // #[cfg(target_os = "windows")]
                // {
                //     while self.event_manager.flush_events(self.cx.context()) {}

                //     self.cx.process_style_updates();

                //     if self.cx.process_animations() {
                //         // self.control_flow = ControlFlow::Poll;

                //         // self.event_loop_proxy
                //         //     .send_event(UserEvent::Event(Event::new(WindowEvent::Redraw)))
                //         //     .expect("Failed to send redraw event");

                //         // window.window().request_redraw();
                //     }

                //     self.cx.process_visual_updates();

                //     #[cfg(feature = "accesskit")]
                //     self.cx.process_tree_updates(|tree_updates| {
                //         for update in tree_updates.iter_mut() {
                //             accesskit.update_if_active(|| update.take().unwrap());
                //         }
                //     });

                //     // window.window().request_redraw();
                // }
            }

            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                self.cx.context().remove(window.entity);
                self.cx.context().windows.remove(&window.entity);
                window.swap_buffers();
                self.windows.remove(&window_id);

                self.windows.retain(|_, win| self.cx.0.windows.contains_key(&win.entity));
                self.window_ids.retain(|e, _| self.cx.0.windows.contains_key(e));
            }
            winit::event::WindowEvent::DroppedFile(path) => {
                self.cx.emit_origin(WindowEvent::Drop(DropData::File(path)));
            }
            winit::event::WindowEvent::HoveredFile(_) => {}
            winit::event::WindowEvent::HoveredFileCancelled => {}
            winit::event::WindowEvent::Focused(is_focused) => {
                self.cx.0.window_has_focus = is_focused;
                // #[cfg(feature = "accesskit")]
                // accesskit.update_if_active(|| TreeUpdate {
                //     nodes: vec![],
                //     tree: None,
                //     focus: is_focused.then_some(self.cx.focused().accesskit_id()).unwrap_or(NodeId(0)),
                // });
            }
            winit::event::WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                let code = match event.physical_key {
                    PhysicalKey::Code(code) => winit_key_code_to_code(code),
                    PhysicalKey::Unidentified(native) => match native {
                        NativeKeyCode::Windows(_scancode) => return,
                        _ => return,
                    },
                };

                let key = match event.logical_key {
                    winit::keyboard::Key::Named(named_key) => winit_key_to_key(named_key),
                    _ => None,
                };

                if let winit::keyboard::Key::Character(character) = event.logical_key {
                    if event.state == ElementState::Pressed {
                        self.cx.emit_window_event(
                            window.entity,
                            WindowEvent::CharInput(character.as_str().chars().next().unwrap()),
                        );
                    }
                }

                let event = match event.state {
                    winit::event::ElementState::Pressed => WindowEvent::KeyDown(code, key),
                    winit::event::ElementState::Released => WindowEvent::KeyUp(code, key),
                };

                self.cx.emit_window_event(window.entity, event);
                window.window().request_redraw();
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.cx.modifiers().set(Modifiers::SHIFT, modifiers.state().shift_key());

                self.cx.modifiers().set(Modifiers::ALT, modifiers.state().alt_key());

                self.cx.modifiers().set(Modifiers::CTRL, modifiers.state().control_key());

                self.cx.modifiers().set(Modifiers::SUPER, modifiers.state().super_key());

                window.window().request_redraw();
            }
            winit::event::WindowEvent::Ime(_) => {}
            winit::event::WindowEvent::CursorMoved { device_id: _, position } => {
                self.cx.context().mouse.cursorx = position.x as f32;
                self.cx.context().mouse.cursory = position.y as f32;
                // hover_system(self.cx.context(), window.entity);

                self.cx.emit_window_event(
                    window.entity,
                    WindowEvent::MouseMove(position.x as f32, position.y as f32),
                );
                window.window().request_redraw();
            }
            winit::event::WindowEvent::CursorEntered { device_id: _ } => {
                self.cx.emit_window_event(window.entity, WindowEvent::MouseEnter);
                window.window().request_redraw();
            }
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.cx.emit_window_event(window.entity, WindowEvent::MouseLeave);
                window.window().request_redraw();
            }
            winit::event::WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                let out_event = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        WindowEvent::MouseScroll(x, y)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        WindowEvent::MouseScroll(
                            pos.x as f32 / 20.0,
                            pos.y as f32 / 20.0, // this number calibrated for wayland
                        )
                    }
                };

                self.cx.emit_window_event(window.entity, out_event);
                window.window().request_redraw();
            }
            winit::event::WindowEvent::MouseInput { device_id: _, state, button } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Other(val) => MouseButton::Other(val),
                    winit::event::MouseButton::Back => MouseButton::Back,
                    winit::event::MouseButton::Forward => MouseButton::Forward,
                };

                let event = match state {
                    winit::event::ElementState::Pressed => WindowEvent::MouseDown(button),
                    winit::event::ElementState::Released => WindowEvent::MouseUp(button),
                };

                self.cx.emit_window_event(window.entity, event);
                window.window().request_redraw();
            }

            winit::event::WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                self.cx.set_scale_factor(scale_factor);
                self.cx.needs_refresh();
            }
            winit::event::WindowEvent::ThemeChanged(theme) => {
                let theme = match theme {
                    winit::window::Theme::Light => ThemeMode::LightMode,
                    winit::window::Theme::Dark => ThemeMode::DarkMode,
                };
                self.cx.emit_window_event(window.entity, WindowEvent::ThemeChanged(theme));
            }
            winit::event::WindowEvent::Occluded(_) => {}
            winit::event::WindowEvent::RedrawRequested => {
                self.cx.needs_refresh();
                self.cx.draw(window.entity, &mut window.surface, &mut window.dirty_surface);
                window.swap_buffers();

                // // Un-cloak
                // #[cfg(target_os = "windows")]
                // if window.is_initially_cloaked {
                //     window.is_initially_cloaked = false;
                //     self.cx.draw(window.entity, &mut window.surface, &mut window.dirty_surface);
                //     window.swap_buffers();
                //     window.set_cloak(false);
                // }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            event_loop.exit();
            return;
        }

        event_loop.set_control_flow(self.control_flow);

        while self.event_manager.flush_events(self.cx.context()) {}

        self.cx.process_style_updates();

        if self.cx.process_animations() {
            for window in self.windows.values() {
                window.window().request_redraw();
            }
        }

        self.cx.process_visual_updates();

        #[cfg(feature = "accesskit")]
        cx.process_tree_updates(|tree_updates| {
            for update in tree_updates.iter_mut() {
                accesskit.update_if_active(|| update.take().unwrap());
            }
        });

        if let Some(idle_callback) = &self.on_idle {
            self.cx.set_current(Entity::root());
            (idle_callback)(self.cx.context());
        }

        if self.cx.has_queued_events() {
            self.event_loop_proxy
                .send_event(UserEvent::Event(Event::new(())))
                .expect("Failed to send event");
        }

        self.cx.style().should_redraw(|| {
            for window in self.windows.values() {
                window.window().request_redraw();
            }
        });

        if self.control_flow != ControlFlow::Poll {
            if let Some(timer_time) = self.cx.get_next_timer_time() {
                event_loop.set_control_flow(ControlFlow::WaitUntil(timer_time));
            } else {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
        }

        // Sync window state with context
        self.windows.retain(|_, win| self.cx.0.windows.contains_key(&win.entity));
        self.window_ids.retain(|e, _| self.cx.0.windows.contains_key(e));

        if self.windows.len() != self.cx.0.windows.len() {
            for (window_entity, window_state) in self.cx.0.windows.clone().iter() {
                if !self.window_ids.contains_key(window_entity) {
                    self.cx.add_main_window(*window_entity, &window_state.window_description, 1.0);
                    let window = self
                        .create_window(
                            event_loop,
                            *window_entity,
                            &window_state.window_description,
                            None,
                        )
                        .expect("Failed to create window");

                    self.cx.mutate_window(*window_entity, |_, win: &mut Window| {
                        win.window = Some(window.clone())
                    });
                }
            }
        }

        if self.windows.is_empty() {
            event_loop.exit();
            return;
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.cx.process_timers();
        self.cx.emit_scheduled_events();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Exiting");
    }
}

impl WindowModifiers for Application {
    fn title<T: ToString>(mut self, title: impl Res<T>) -> Self {
        self.window_description.title = title.get(&self.cx.0).to_string();

        title.set_or_bind(&mut self.cx.0, Entity::root(), |cx, title| {
            cx.emit(WindowEvent::SetTitle(title.get(cx).to_string()));
        });

        self
    }

    fn inner_size<S: Into<WindowSize>>(mut self, size: impl Res<S>) -> Self {
        self.window_description.inner_size = size.get(&self.cx.0).into();

        size.set_or_bind(&mut self.cx.0, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetSize(size.get(cx).into()));
        });

        self
    }

    fn min_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.min_inner_size = size.get(&self.cx.0).map(|s| s.into());

        size.set_or_bind(&mut self.cx.0, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetMinSize(size.get(cx).map(|s| s.into())));
        });

        self
    }

    fn max_inner_size<S: Into<WindowSize>>(mut self, size: impl Res<Option<S>>) -> Self {
        self.window_description.max_inner_size = size.get(&self.cx.0).map(|s| s.into());

        size.set_or_bind(&mut self.cx.0, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetMaxSize(size.get(cx).map(|s| s.into())));
        });
        self
    }

    fn position<P: Into<Position>>(mut self, position: impl Res<P>) -> Self {
        self.window_description.position = Some(position.get(&self.cx.0).into());

        position.set_or_bind(&mut self.cx.0, Entity::root(), |cx, size| {
            cx.emit(WindowEvent::SetPosition(size.get(cx).into()));
        });

        self
    }

    fn resizable(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.resizable = flag.get(&self.cx.0);

        flag.set_or_bind(&mut self.cx.0, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetResizable(flag.get(cx)));
        });

        self
    }

    fn minimized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.minimized = flag.get(&self.cx.0);

        flag.set_or_bind(&mut self.cx.0, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetMinimized(flag.get(cx)));
        });
        self
    }

    fn maximized(mut self, flag: impl Res<bool>) -> Self {
        self.window_description.maximized = flag.get(&self.cx.0);

        flag.set_or_bind(&mut self.cx.0, Entity::root(), |cx, flag| {
            cx.emit(WindowEvent::SetMaximized(flag.get(cx)));
        });

        self
    }

    fn visible(mut self, flag: bool) -> Self {
        self.window_description.visible = flag;

        self
    }

    fn transparent(mut self, flag: bool) -> Self {
        self.window_description.transparent = flag;

        self
    }

    fn decorations(mut self, flag: bool) -> Self {
        self.window_description.decorations = flag;

        self
    }

    fn always_on_top(mut self, flag: bool) -> Self {
        self.window_description.always_on_top = flag;
        self
    }

    fn vsync(mut self, flag: bool) -> Self {
        self.window_description.vsync = flag;

        self
    }

    fn icon(mut self, width: u32, height: u32, image: Vec<u8>) -> Self {
        self.window_description.icon = Some(image);
        self.window_description.icon_width = width;
        self.window_description.icon_height = height;

        self
    }
}

fn apply_window_description(description: &WindowDescription) -> WindowAttributes {
    let mut window_attributes = winit::window::Window::default_attributes();

    window_attributes = window_attributes.with_title(&description.title).with_inner_size(
        LogicalSize::new(description.inner_size.width, description.inner_size.height),
    );

    if let Some(min_inner_size) = description.min_inner_size {
        window_attributes = window_attributes
            .with_min_inner_size(LogicalSize::new(min_inner_size.width, min_inner_size.height));
    }

    if let Some(max_inner_size) = description.max_inner_size {
        window_attributes = window_attributes
            .with_max_inner_size(LogicalSize::new(max_inner_size.width, max_inner_size.height));
    }

    if let Some(position) = description.position {
        window_attributes =
            window_attributes.with_position(LogicalPosition::new(position.x, position.y));
    }

    window_attributes
        .with_resizable(description.resizable)
        .with_maximized(description.maximized)
        // Accesskit requires that the window start invisible until accesskit is initialized.
        .with_visible(false)
        .with_window_level(if description.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        })
        .with_transparent(description.transparent)
        .with_decorations(description.decorations)
        .with_window_icon(description.icon.as_ref().map(|icon| {
            winit::window::Icon::from_rgba(
                icon.clone(),
                description.icon_width,
                description.icon_height,
            )
            .unwrap()
        }))
}
