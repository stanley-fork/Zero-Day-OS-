use std::io::Read;

pub struct HexView {
    pub data: Vec<u8>,
    pub offset: usize,
    pub path: String,
}

impl HexView {
    pub fn from_file(path: &std::path::Path, max_bytes: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut f = std::fs::File::open(path)?;
        let mut buf = vec![0u8; max_bytes];
        let n = f.read(&mut buf)?;
        buf.truncate(n);
        Ok(HexView {
            data: buf,
            offset: 0,
            path: path.display().to_string(),
        })
    }

    pub fn lines(&self, width: usize) -> Vec<String> {
        let bytes_per_line = 16;
        let mut lines = Vec::new();
        let start = self.offset;
        let mut i = start;
        let view_rows = width;

        while i < self.data.len() && lines.len() < view_rows {
            let addr = i;
            let mut hex_part = String::new();
            let mut ascii_part = String::new();

            for j in 0..bytes_per_line {
                if i + j < self.data.len() {
                    hex_part.push_str(&format!("{:02x} ", self.data[i + j]));
                    let ch = self.data[i + j];
                    if ch >= 0x20 && ch < 0x7f {
                        ascii_part.push(ch as char);
                    } else {
                        ascii_part.push('.');
                    }
                } else {
                    hex_part.push_str("   ");
                    ascii_part.push(' ');
                }
            }
            i += bytes_per_line;

            lines.push(format!("{:08x}  {} {}", addr, hex_part, ascii_part));
        }
        lines
    }

    pub fn scroll_down(&mut self, rows: usize) {
        let bytes_per_line = 16;
        self.offset = self.offset.saturating_add(rows * bytes_per_line).min(self.data.len().saturating_sub(1));
    }

    pub fn scroll_up(&mut self, rows: usize) {
        let bytes_per_line = 16;
        self.offset = self.offset.saturating_sub(rows * bytes_per_line);
    }

    pub fn goto_start(&mut self) {
        self.offset = 0;
    }

    pub fn goto_end(&mut self) {
        let bytes_per_line = 16;
        self.offset = self.data.len().saturating_sub(bytes_per_line * 5);
    }
}