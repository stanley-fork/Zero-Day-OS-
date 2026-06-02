use crate::drm_be::Backends;
use crate::hdmi;
use crate::input::{InputHandler, FnAction};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::process::Command;
use std::sync::Arc;

use smithay::backend::input::{
    AbsolutePositionEvent, Axis, ButtonState, KeyState,
    Event as InputEventTrait, InputEvent, KeyboardKeyEvent,
    PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::desktop::{Space, Window};
use smithay::input::keyboard::{FilterResult, XkbConfig};
use smithay::input::keyboard::KeyboardHandle;
use smithay::input::pointer::{AxisFrame, ButtonEvent, CursorImageStatus, MotionEvent, PointerHandle};
use smithay::wayland::seat::WaylandFocus;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Output, PhysicalProperties, Subpixel};
use smithay::reexports::calloop;
use smithay::reexports::input::Libinput;
use smithay::reexports::input::DeviceCapability;
use smithay::reexports::input::event::keyboard::KeyboardEventTrait;
use smithay::reexports::wayland_server::{self, Client, Display, DisplayHandle};
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState};
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
};
use smithay::wayland::shell::xdg::decoration::XdgDecorationState;
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::socket::ListeningSocketSource;
use smithay::utils::{Serial, SerialCounter, Transform};
use wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use wayland_server::protocol::wl_buffer;
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::protocol::wl_seat as wl_seat_group;

static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Clone)]
pub struct AppData {
    pub client_cmd: String,
    pub client_args: Option<String>,
    pub no_cursor: bool,
    pub target_fps: u32,
    pub drm_path: String,
    pub hdmi_fps: u32,
    pub hdmi_auto: bool,
    pub input_handler: InputHandler,
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct State {
    pub display_handle: DisplayHandle,
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    xdg_decoration_state: XdgDecorationState,
    shm_state: ShmState,
    seat_state: SeatState<State>,
    keyboard: KeyboardHandle<State>,
    pointer: PointerHandle<State>,
    seat: Seat<State>,
    space: Space<Window>,
    lcd_output: Option<Output>,
    hdmi_output: Option<Output>,
    pub backends: Option<Backends>,
    app_data: AppData,
    cursor_status: CursorImageStatus,
    serial_counter: SerialCounter,
    input_handler: InputHandler,
    has_pointer_device: bool,
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, _surface: &WlSurface) {
        if let Some(ref mut backends) = self.backends {
            backends.pending_render = true;
        }
    }
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<State> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<State>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<State>, image: CursorImageStatus) {
        self.cursor_status = image;
    }
}

impl smithay::wayland::shell::xdg::decoration::XdgDecorationHandler for State {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ServerSide);
        });
        toplevel.send_configure();
    }

    fn request_mode(&mut self, _toplevel: ToplevelSurface, _mode: Mode) {}

    fn unset_mode(&mut self, _toplevel: ToplevelSurface) {}
}

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();

        let window = Window::new_wayland_window(surface);
        self.space.map_element(window, (0, 0), true);

        if let Some(output) = self.space.outputs().next() {
            let window = self.space.elements().last().unwrap();
            window.send_frame(output, Duration::from_millis(0), None, |_, _| Some(output.clone()));
        }
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat_group::WlSeat, _serial: Serial) {}

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}

    fn toplevel_destroyed(&mut self, _surface: ToplevelSurface) {}

    fn popup_destroyed(&mut self, _surface: PopupSurface) {}
}

smithay::delegate_compositor!(State);
smithay::delegate_seat!(State);
smithay::delegate_shm!(State);
smithay::delegate_xdg_shell!(State);
smithay::delegate_xdg_decoration!(State);

fn create_lcd_output() -> Output {
    let output = Output::new(
        "ST7789V".to_string(),
        PhysicalProperties {
            make: "M5Stack".into(),
            model: "Cardputer Zero LCD".into(),
            size: (36_i32, 19_i32).into(),
            subpixel: Subpixel::Unknown,
        },
    );

    let lcd_mode = smithay::output::Mode {
        size: (320, 170).into(),
        refresh: 30_000,
    };

    output.set_preferred(lcd_mode);
    output.change_current_state(Some(lcd_mode), Some(Transform::Normal), None, None);

    output
}

fn create_hdmi_output(fps: u32) -> Output {
    let output = Output::new(
        "HDMI-A-1".to_string(),
        PhysicalProperties {
            make: "Generic".into(),
            model: "HDMI Monitor".into(),
            size: (530_i32, 300_i32).into(),
            subpixel: Subpixel::Unknown,
        },
    );

    let hdmi_mode = smithay::output::Mode {
        size: (1920, 1080).into(),
        refresh: (fps * 1000) as i32,
    };

    output.set_preferred(hdmi_mode);
    output.change_current_state(Some(hdmi_mode), Some(Transform::Normal), None, None);

    output
}

pub fn run(app_data: AppData) -> Result<(), Box<dyn std::error::Error>> {
    let mut display: Display<State> = Display::new()?;
    let dh = display.handle();

    let compositor_state = CompositorState::new::<State>(&dh);
    let xdg_shell_state = XdgShellState::new::<State>(&dh);
    let xdg_decoration_state = XdgDecorationState::new::<State>(&dh);
    let shm_state = ShmState::new::<State>(&dh, vec![]);

    let mut seat_state = SeatState::new();
    let mut seat = seat_state.new_wl_seat(&dh, "seat0");
    let keyboard = seat.add_keyboard(XkbConfig::default(), 200, 25)?;
    let pointer = seat.add_pointer();

    let mut space = Space::default();

    let lcd_output = create_lcd_output();
    space.map_output(&lcd_output, (0, 0));

    let hdmi_output = if app_data.hdmi_auto && hdmi::is_hdmi_connected() {
        let hdmi = create_hdmi_output(app_data.hdmi_fps);
        space.map_output(&hdmi, (320, 0));
        log::info!(
            "HDMI Screen #2 enabled — 1920x1080 @ {}fps (content display, right-of LCD)",
            app_data.hdmi_fps
        );
        Some(hdmi)
    } else {
        log::info!("HDMI output disabled (no monitor detected, or --hdmi-auto=false)");
        None
    };

    let listening_socket = ListeningSocketSource::with_name("wayland-0")?;
    let socket_name = listening_socket.socket_name().to_string_lossy().to_string();
    log::info!("Wayland socket: {}", socket_name);

    ctrlc::set_handler(|| {
        log::info!("SIGINT received, shutting down");
        RUNNING.store(false, Ordering::SeqCst);
    })?;

    let mut event_loop: calloop::EventLoop<State> = calloop::EventLoop::try_new()?;
    let loop_handle = event_loop.handle();

    loop_handle.insert_source(listening_socket, |client_stream, _metadata, state| {
        log::info!("New Wayland client connecting");
        match state.display_handle.insert_client(client_stream, Arc::new(ClientState::default())) {
            Ok(_client) => log::info!("Client connected via calloop"),
            Err(e) => log::warn!("Failed to insert client: {}", e),
        }
    })?;

    let signal = event_loop.get_signal();

    let backends = Backends::new(&loop_handle)?;

    let session_clone = backends.session.clone();
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<_>>(
        LibinputSessionInterface::from(session_clone),
    );
    libinput_context.udev_assign_seat("seat0")
        .map_err(|_| "Failed to assign seat to libinput")?;
    let input_backend = LibinputInputBackend::new(libinput_context);

    loop_handle.insert_source(input_backend, |event, _, state| {
        process_input_event(event, state);
    })?;

    let frame_interval = Duration::from_millis(1000 / app_data.target_fps as u64);
    loop_handle.insert_source(
        calloop::timer::Timer::from_duration(frame_interval),
        move |_time, _metadata, state| {
            if let Some(ref mut backends) = state.backends {
                if backends.should_render() {
                    let windows: Vec<Window> = state.space.elements().cloned().collect();
                    let output = state.lcd_output.as_ref().unwrap_or_else(|| {
                        state.space.outputs().next().unwrap()
                    });
                    let cursor_surface = if state.has_pointer_device {
                        match &state.cursor_status {
                            CursorImageStatus::Surface(s) => Some(s.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let pointer_location = state.pointer.current_location();
                    if !windows.is_empty() || cursor_surface.is_some() {
                        if let Err(e) = backends.render(&windows, output, cursor_surface.as_ref(), pointer_location) {
                            log::warn!("Render failed: {}", e);
                        }
                    }
                }
            }
            calloop::timer::TimeoutAction::ToDuration(frame_interval)
        },
    )?;

    start_client(&app_data, &socket_name);

    log::info!(
        "zeroday-comp running — Screen #1: LCD 320x170@{}fps (controls), Screen #2: HDMI 1920x1080@{}fps (content), client={}",
        app_data.target_fps,
        app_data.hdmi_fps,
        app_data.client_cmd
    );

    let input_handler = app_data.input_handler.clone();
    let mut state = State {
        display_handle: dh,
        compositor_state,
        xdg_shell_state,
        xdg_decoration_state,
        shm_state,
        seat_state,
        keyboard,
        pointer,
        seat,
        space,
        lcd_output: Some(lcd_output),
        hdmi_output,
        backends: Some(backends),
        app_data,
        cursor_status: CursorImageStatus::default_named(),
        serial_counter: SerialCounter::new(),
        input_handler,
        has_pointer_device: false,
    };

    log::info!("calloop event loop started — DRM rendering + libinput at {}fps", state.app_data.target_fps);

    while RUNNING.load(Ordering::SeqCst) {
        display.dispatch_clients(&mut state)?;
        display.flush_clients()?;
        event_loop.dispatch(Duration::from_millis(16), &mut state)?;
    }

    signal.stop();
    log::info!("zeroday-comp shutting down");
    Ok(())
}

fn process_input_event(event: InputEvent<LibinputInputBackend>, state: &mut State) {
    let keyboard = state.keyboard.clone();
    let pointer = state.pointer.clone();
    let serial_counter = &state.serial_counter;

    match event {
        InputEvent::Keyboard { event } => {
            let serial = serial_counter.next_serial();
            let time = event.time_msec();
            let keycode = event.key_code();
            let key_state = event.state();
            let evdev_keycode = event.key();

            if key_state == KeyState::Pressed {
                state.input_handler.handle_key_press(evdev_keycode);
            } else if key_state == KeyState::Released {
                state.input_handler.handle_key_release(evdev_keycode);
            }

            keyboard.input::<FnAction, _>(
                state,
                keycode,
                key_state,
                serial,
                time,
                |_, _mods, _keysym| {
                    FilterResult::Forward
                },
            );
        }
        InputEvent::PointerMotion { event } => {
            let serial = serial_counter.next_serial();
            let time = event.time_msec();
            let delta = event.delta();
            let current = pointer.current_location();
            let new_location = (current.x + delta.x, current.y + delta.y);

            let focus = surface_under_pointer(state, new_location);

            pointer.motion(
                state,
                focus,
                &MotionEvent {
                    location: new_location.into(),
                    serial,
                    time,
                },
            );

            if let Some(ref mut backends) = state.backends {
                backends.pending_render = true;
            }
        }
        InputEvent::PointerMotionAbsolute { event } => {
            let serial = serial_counter.next_serial();
            let time = event.time_msec();
            let location = event.position_transformed(smithay::utils::Size::from((320, 170)));

            let focus = surface_under_pointer(state, (location.x, location.y));

            pointer.motion(
                state,
                focus,
                &MotionEvent {
                    location,
                    serial,
                    time,
                },
            );

            if let Some(ref mut backends) = state.backends {
                backends.pending_render = true;
            }
        }
        InputEvent::PointerButton { event } => {
            let serial = serial_counter.next_serial();
            let time = event.time_msec();
            let button = event.button_code();
            let button_state = event.state();

            pointer.button(
                state,
                &ButtonEvent {
                    serial,
                    time,
                    button,
                    state: button_state,
                },
            );

            if button_state == ButtonState::Pressed {
                let location = pointer.current_location();
                if let Some((window, _)) = state.space.element_under(location.to_i32_round()) {
                    let window = window.clone();
                    state.space.raise_element(&window, true);
                }
                if let Some(surface) = get_focused_surface(state) {
                    keyboard.set_focus(state, Some(surface), serial);
                }
            }
        }
        InputEvent::PointerAxis { event } => {
            let time = event.time_msec();
            let source = event.source();

            let mut frame = AxisFrame::new(time).source(source);

            if let Some(vertical) = event.amount(Axis::Vertical) {
                frame = frame.value(Axis::Vertical, vertical);
            }
            if let Some(horizontal) = event.amount(Axis::Horizontal) {
                frame = frame.value(Axis::Horizontal, horizontal);
            }
            if let Some(v120_v) = event.amount_v120(Axis::Vertical) {
                frame = frame.v120(Axis::Vertical, v120_v as i32);
            }
            if let Some(v120_h) = event.amount_v120(Axis::Horizontal) {
                frame = frame.v120(Axis::Horizontal, v120_h as i32);
            }

            frame = frame
                .relative_direction(
                    Axis::Vertical,
                    event.relative_direction(Axis::Vertical),
                )
                .relative_direction(
                    Axis::Horizontal,
                    event.relative_direction(Axis::Horizontal),
                );

            pointer.axis(state, frame);
            pointer.frame(state);
        }
        InputEvent::DeviceAdded { device } => {
            if device.has_capability(DeviceCapability::Pointer) {
                state.has_pointer_device = true;
                log::info!("Pointer device added: {}", device.name());
            }
        }
        InputEvent::DeviceRemoved { device } => {
            if device.has_capability(DeviceCapability::Pointer) {
                state.has_pointer_device = false;
                state.cursor_status = CursorImageStatus::Hidden;
                if let Some(ref mut backends) = state.backends {
                    backends.pending_render = true;
                }
                log::info!("Pointer device removed");
            }
        }
        _ => {}
    }
}

fn surface_under_pointer(state: &State, location: (f64, f64)) -> Option<(WlSurface, smithay::utils::Point<f64, smithay::utils::Logical>)> {
    let point: smithay::utils::Point<f64, smithay::utils::Logical> = location.into();
    state.space.element_under(point)
        .and_then(|(window, loc)| {
            window.wl_surface().map(|s| (s.into_owned(), loc.to_f64()))
        })
}

fn get_focused_surface(state: &State) -> Option<WlSurface> {
    state.space.elements().next().and_then(|w| w.wl_surface().map(|s| s.into_owned()))
}

fn start_client(app_data: &AppData, socket_name: &str) {
    let cmd = app_data.client_cmd.clone();
    let args: Vec<String> = app_data
        .client_args
        .as_deref()
        .map(|a| a.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    let socket = socket_name.to_string();
    let hdmi_env = if app_data.hdmi_auto && hdmi::is_hdmi_connected() {
        "1"
    } else {
        "0"
    };

    let cursor_env = if app_data.no_cursor { "1" } else { "0" };
    let display_env = if hdmi_env == "1" { "hdmi" } else { "lcd" };
    let hdmi_fps = app_data.hdmi_fps;

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match Command::new(&cmd)
            .args(&args_ref)
            .env("WAYLAND_DISPLAY", &socket)
            .env("SDL_VIDEODRIVER", "wayland")
            .env("SDL_RENDER_DRIVER", "opengles2")
            .env("PYGAME_HIDE_SUPPORT_PROMPT", "1")
            .env("ZERODAY_HDMI", hdmi_env)
            .env("ZERODAY_DISPLAY", display_env)
            .env("ZERODAY_LCD_WIDTH", "320")
            .env("ZERODAY_LCD_HEIGHT", "170")
            .env("ZERODAY_HDMI_WIDTH", "1920")
            .env("ZERODAY_HDMI_HEIGHT", "1080")
            .env("ZERODAY_HDMI_FPS", hdmi_fps.to_string())
            .env("ZERODAY_COMP_NO_CURSOR", cursor_env)
            .spawn()
        {
            Ok(c) => log::info!("Client {} started (PID {})", cmd, c.id()),
            Err(e) => log::error!("Failed to start client {}: {}", cmd, e),
        }
    });
}