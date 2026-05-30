use std::io::Read;
use std::io::Write;

pub struct PtySession {
    reader: Option<Box<dyn Read + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
}

impl PtySession {
    pub fn new(shell: &str, cols: u16, rows: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = portable_pty::CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("COLUMNS", cols.to_string());
        cmd.env("LINES", rows.to_string());

        let _child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let master: Box<dyn portable_pty::MasterPty + Send> = pair.master;

        Ok(Self {
            reader: Some(Box::new(reader)),
            writer: Some(Box::new(writer)),
            master: Some(master),
        })
    }

    pub fn read_output(&mut self, buf: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(ref mut reader) = self.reader {
            match reader.read(buf) {
                Ok(0) => Ok(0),
                Ok(n) => Ok(n),
                Err(e) => Err(e.into()),
            }
        } else {
            Ok(0)
        }
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut writer) = self.writer {
            writer.write_all(data)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut master) = self.master {
            master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }
        Ok(())
    }
}