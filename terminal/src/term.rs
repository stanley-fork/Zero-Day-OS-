use crate::fn_keys::{FnKeyHandler, FnAction};
use crate::render::Renderer;
use crate::status_bar::StatusBar;
use crate::pty::PtySession;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static RUNNING: AtomicBool = AtomicBool::new(true);

pub struct Terminal {
    cols: u16,
    rows: u16,
    font_size: u16,
    width: u32,
    height: u32,
    no_status: bool,
    shell: String,
    command: Option<String>,
    fn_handler: FnKeyHandler,
    renderer: Renderer,
    status_bar: StatusBar,
}

impl Terminal {
    pub fn new(args: &crate::Args) -> Self {
        let status_bar_height: u32 = if args.no_status { 0 } else { args.font_size as u32 };
        let cols = (args.width / 8) as u16;
        let rows = ((args.height.saturating_sub(status_bar_height)) / args.font_size as u32) as u16;
        let cols = cols.max(40);
        let rows = rows.max(12);

        Self {
            cols,
            rows,
            font_size: args.font_size,
            width: args.width,
            height: args.height,
            no_status: args.no_status,
            shell: args.shell.clone(),
            command: args.command.clone(),
            fn_handler: FnKeyHandler::new(),
            renderer: Renderer::new(args.width, args.height, args.font_size),
            status_bar: StatusBar::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        ctrlc::set_handler(|| {
            RUNNING.store(false, Ordering::SeqCst);
        })?;

        let cmd = self.command.as_deref().unwrap_or(&self.shell);
        let mut pty = PtySession::new(cmd, self.cols, self.rows)?;
        let mut input_buf = [0u8; 4096];

        log::info!("Terminal {}x{} ({} cols x {} rows), shell: {}", self.width, self.height, self.cols, self.rows, cmd);

        self.renderer.init()?;
        self.status_bar.update();

        while RUNNING.load(Ordering::SeqCst) {
            let output = pty.read_output(&mut input_buf);
            match output {
                Ok(n) if n > 0 => {
                    let data = &input_buf[..n];
                    self.renderer.write_pty(data);
                    self.renderer.refresh_screen();
                }
                Ok(_) => {}
                Err(_) => break,
            }

            if !self.no_status && self.status_bar.should_update() {
                self.status_bar.update();
                self.renderer.draw_status_bar(&self.status_bar);
            }

            std::thread::sleep(Duration::from_millis(16));
        }

        log::info!("zeroday-term exiting");
        Ok(())
    }
}