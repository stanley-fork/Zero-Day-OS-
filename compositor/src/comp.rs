use crate::hdmi;
use crate::input::InputHandler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::process::Command;

use smithay::desktop::{Space, Window};
use smithay::input::keyboard::{self, KeyboardHandler, Keysym, XkbConfig, ModifierState};
use smithay::input::pointer::{self, PointerHandler};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Output, PhysicalProperties, Subpixel};
use smithay::reexports::wayland_server::Display;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
use smithay::utils::{Logical, Point, Rectangle, Size, Transform};

static RUNNING: AtomicBool = AtomicBool::new(true);

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

struct State {
    compositor: CompositorState,
    xdg_shell: XdgShellState,
    shm: ShmState,
    seat_state: SeatState<Self>,
    keyboard: keyboard::KeyboardState<Self>,
    pointer: pointer::PointerHandle<Self>,
    seat: Seat<Self>,
    space: Space<Window>,
    lcd_output: Option<Output>,
    hdmi_output: Option<Output>,
    app_data: AppData,
}

impl KeyboardHandler for State {
    fn on_keyboard_key(
        &mut self,
        key: keyboard::Key,
        state: keyboard::KeyState,
        _serial: u32,
        _time: u32,
        _handle: &keyboard::KeyboardHandle<Self>,
    ) {
        if state == keyboard::KeyState::Pressed {
            self.app_data.input_handler.handle_key_press(key.code as u32);
        } else {
            self.app_data.input_handler.handle_key_release(key.code as u32);
        }
    }

    fn on_keyboard_modifiers(
        &mut self,
        _mods: keyboard::ModifiersState,
        _serial: u32,
        _handle: &keyboard::KeyboardHandle<Self>,
    ) {
    }
}

impl PointerHandler for State {
    fn on_pointer_motion(
        &mut self,
        _pos: Point<f64, Logical>,
        _serial: u32,
        _time: u32,
        _handle: &pointer::PointerHandle<Self>,
    ) {
    }

    fn on_pointer_button(
        &mut self,
        _button: pointer::Button,
        _state: pointer::ButtonState,
        _serial: u32,
        _time: u32,
        _handle: &pointer::PointerHandle<Self>,
    ) {
    }
}

fn create_lcd_output() -> Output {
    let output = Output::new(
        "ST7789V",
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

    output.change_current_state(Some(lcd_mode), Some(Transform::Normal), None, None);
    output.with_state(|state| {
        state.modes = vec![lcd_mode];
    });

    output
}

fn create_hdmi_output(fps: u32) -> Output {
    let output = Output::new(
        "HDMI-A-1",
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

    output.change_current_state(Some(hdmi_mode), Some(Transform::Normal), None, None);
    output.with_state(|state| {
        state.modes = vec![hdmi_mode];
    });

    output
}

pub fn run(app_data: AppData) -> Result<(), Box<dyn std::error::Error>> {
    let display = Display::<State>::new()?;
    let dh = display.handle();

    let compositor = CompositorState::new::<State>(&dh);
    let xdg_shell = XdgShellState::new::<State>(&dh);
    let shm = ShmState::new::<State>(&dh, []);

    let (seat_state, _) = SeatState::new();
    let seat = seat_state.add_seat(&dh, "seat0".into());
    let keyboard = keyboard::KeyboardState::new(&seat, &dh, XkbConfig::default(), 200, 25)?;
    let pointer = pointer::PointerHandle::new(&seat, &dh);

    let mut space = Space::default();

    let lcd_output = create_lcd_output();
    space.map_output(&lcd_output, (0, 0));

    let hdmi_output = if app_data.hdmi_auto && hdmi::is_hdmi_connected() {
        let hdmi = create_hdmi_output(app_data.hdmi_fps);
        space.map_output(&hdmi, (320, 0));
        log::info!(
            "HDMI output enabled — 1920x1080 @ {}fps (Monitor 2, position: right-of LCD)",
            app_data.hdmi_fps
        );
        Some(hdmi)
    } else {
        log::info!("HDMI output disabled (no monitor detected, or disable with --hdmi-auto=false)");
        None
    };

    let socket_name = display.socket_name()?.to_string_lossy().into_owned();
    log::info!("Wayland socket: {}", socket_name);

    start_client(&app_data, &socket_name);

    log::info!(
        "zeroday-comp running — LCD 320x170@{}fps, HDMI 1920x1080@{}fps, client={}",
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
        compositor,
        xdg_shell,
        shm,
        seat_state,
        keyboard,
        pointer,
        seat,
        space,
        lcd_output: Some(lcd_output),
        hdmi_output,
        app_data,
    };

    loop {
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(frame_duration);
    }

    log::info!("zeroday-comp shutting down");
    Ok(())
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
            .env("ZERODAY_LCD_WIDTH", "320")
            .env("ZERODAY_LCD_HEIGHT", "170")
            .env("ZERODAY_HDMI_WIDTH", "1920")
            .env("ZERODAY_HDMI_HEIGHT", "1080")
            .spawn()
        {
            Ok(c) => log::info!("Client {} started (PID {})", cmd, c.id()),
            Err(e) => log::error!("Failed to start client {}: {}", cmd, e),
        }
    });
}