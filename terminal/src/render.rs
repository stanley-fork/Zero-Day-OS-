use crate::status_bar::StatusBar;

pub struct Renderer {
    width: u32,
    height: u32,
    font_size: u16,
    cols: u16,
    rows: u16,
    screen: Vec<ScreenCell>,
    cursor_row: u16,
    cursor_col: u16,
    fg: u8,
    bg: u8,
}

#[derive(Clone)]
struct ScreenCell {
    ch: char,
    fg: u8,
    bg: u8,
}

impl ScreenCell {
    fn default() -> Self {
        Self { ch: ' ', fg: 7, bg: 0 }
    }
}

impl Renderer {
    pub fn new(width: u32, height: u32, font_size: u16) -> Self {
        let cols = (width / 8) as u16;
        let rows = ((height.saturating_sub(font_size as u32)) / font_size as u32) as u16;
        let cols = cols.max(40);
        let rows = rows.max(12);
        let screen = vec![ScreenCell::default(); (cols as usize) * (rows as usize)];

        Self {
            width,
            height,
            font_size,
            cols,
            rows,
            screen,
            cursor_row: 0,
            cursor_col: 0,
            fg: 7,
            bg: 0,
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn write_pty(&mut self, data: &[u8]) {
        for &byte in data {
            if byte >= 32 && byte < 127 {
                if (self.cursor_col as usize) < self.cols as usize && (self.cursor_row as usize) < self.rows as usize {
                    let idx = (self.cursor_row as usize) * (self.cols as usize) + (self.cursor_col as usize);
                    if idx < self.screen.len() {
                        self.screen[idx] = ScreenCell { ch: byte as char, fg: self.fg, bg: self.bg };
                    }
                }
                self.cursor_col += 1;
                if self.cursor_col >= self.cols {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= self.rows {
                        self.scroll_up();
                        self.cursor_row = self.rows - 1;
                    }
                }
            } else if byte == 10 {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
            } else if byte == 13 {
                self.cursor_col = 0;
            } else if byte == 8 {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        let cols = self.cols as usize;
        let rows = self.rows as usize;
        for row in 0..(rows - 1) {
            for col in 0..cols {
                let src = (row + 1) * cols + col;
                let dst = row * cols + col;
                if src < self.screen.len() && dst < self.screen.len() {
                    self.screen[dst] = self.screen[src].clone();
                }
            }
        }
        let last_row = (rows - 1) * cols;
        for col in 0..cols {
            let idx = last_row + col;
            if idx < self.screen.len() {
                self.screen[idx] = ScreenCell::default();
            }
        }
    }

    pub fn refresh_screen(&mut self) {
        // TODO: DRM/KMS framebuffer rendering
    }

    pub fn draw_status_bar(&mut self, status: &StatusBar) {
        let _ = status.render(self.width);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // TODO: Restore framebuffer
    }
}