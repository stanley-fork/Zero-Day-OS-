use crate::hdmi;
use crate::input::InputHandler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::process::Command;

use smithay::desktop::{Space, Window};
use smithay::input::keyboard::XkbConfig;
use smithay::input::keyboard::KeyboardHandle;
use smithay::input::pointer::{CursorImageStatus, PointerHandle};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Output, PhysicalProperties, Subpixel};
use smithay::reexports::wayland_server::{self, Client, Display};
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState};
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
};
use smithay::wayland::shell::xdg::decoration::XdgDecorationState;
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::utils::{Serial, Transform};
use wayland_server::protocol::wl_buffer;
use wayland_server::protocol::wl_surface::WlSurface;
use wayland_server::protocol::wl_seat as wl_seat_group;
use wayland_server::ListeningSocket;

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

impl wayland_server::backend::ClientData for ClientState {
    fn initialized(&self, _client_id: wayland_server::backend::ClientId) {}
    fn disconnected(&self, _client_id: wayland_server::backend::ClientId, _reason: wayland_server::backend::DisconnectReason) {}
}

pub struct State {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    xdg_decoration_state: XdgDecorationState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,
    keyboard: KeyboardHandle<Self>,
    pointer: PointerHandle<Self>,
    seat: Seat<Self>,
    space: Space<Window>,
    lcd_output: Option<Output>,
    hdmi_output: Option<Output>,
    app_data: AppData,
    cursor_status: CursorImageStatus,
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

    fn commit(&mut self, _surface: &WlSurface) {}
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
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
    let display: Display<State> = Display::new()?;
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

    let listener = ListeningSocket::bind("wayland-0")?;
    log::info!("Wayland socket: wayland-0");

    start_client(app_data.clone(), "wayland-0");

    log::info!(
        "zeroday-comp running — Screen #1: LCD 320x170@{}fps (controls), Screen #2: HDMI 1920x1080@{}fps (content), client={}",
        app_data.target_fps,
        app_data.hdmi_fps,
        app_data.client_cmd
    );

    ctrlc::set_handler(|| {
        log::info!("SIGINT received, shutting down");
        RUNNING.store(false, Ordering::SeqCst);
    })?;

    let frame_duration = Duration::from_millis(1000 / app_data.target_fps as u64);

    let mut state = State {
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
        app_data,
        cursor_status: CursorImageStatus::default_named(),
    };

    loop {
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(frame_duration);
    }

    log::info!("zeroday-comp shutting down");
    drop(state);
    Ok(())
}

fn start_client(app_data: AppData, socket_name: &str) {
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
            .env("ZERODAY_HDMI_FPS", app_data.hdmi_fps.to_string())
            .env("ZERODAY_COMP_NO_CURSOR", cursor_env)
            .spawn()
        {
            Ok(c) => log::info!("Client {} started (PID {})", cmd, c.id()),
            Err(e) => log::error!("Failed to start client {}: {}", cmd, e),
        }
    });
}