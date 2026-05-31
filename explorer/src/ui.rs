use crossterm::{
    event,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, EnableLineWrap, DisableLineWrap},
    cursor::{MoveTo, Hide, Show},
    style::{Color, SetForegroundColor, SetBackgroundColor, Attribute, SetAttribute, ResetColor, Print},
    execute, queue,
};
use std::io::{self, Write, BufWriter};
use crate::app::{self, App, Mode, InputKind, SortBy};
use crate::hexview::HexView;
use crate::fsops;

const STATUS_BAR_HEIGHT: u16 = 1;
const PATH_BAR_HEIGHT: u16 = 1;
const HELP_BAR_HEIGHT: u16 = 1;

pub fn run(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = BufWriter::new(io::stdout());
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, DisableLineWrap, Hide)?;
    stdout.flush()?;

    let result = run_app(app, &mut stdout);

    execute!(stdout, Show, EnableLineWrap, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}

fn run_app(app: &mut App, stdout: &mut BufWriter<io::Stdout>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        draw(app, stdout)?;
        stdout.flush()?;

        if !crate::input::handle_input(app)? {
            break;
        }
    }
    Ok(())
}

fn draw(app: &App, stdout: &mut BufWriter<io::Stdout>, ) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    if cols < 20 || rows < 10 {
        return Ok(());
    }

    queue!(stdout, MoveTo(0, 0))?;

    match app.mode {
        Mode::Navigate | Mode::Search | Mode::Input(_) | Mode::ConfirmDelete | Mode::ConfirmOverwrite => {
            draw_navigate(app, stdout, cols, rows)?;
        }
        Mode::HexView => draw_hex(app, stdout, cols, rows)?,
        Mode::Metadata => draw_metadata(app, stdout, cols, rows)?,
        Mode::BookmarkList => draw_bookmarks(app, stdout, cols, rows)?,
    }

    Ok(())
}

fn draw_navigate(app: &App, stdout: &mut impl Write, cols: u16, rows: u16) -> io::Result<()> {
    let path_row = 0u16;
    let file_start = 1u16;
    let file_end = rows.saturating_sub(STATUS_BAR_HEIGHT + HELP_BAR_HEIGHT);
    let status_row = rows.saturating_sub(STATUS_BAR_HEIGHT + HELP_BAR_HEIGHT);
    let help_row = rows.saturating_sub(HELP_BAR_HEIGHT);

    // Path bar
    queue!(stdout, MoveTo(0, path_row))?;
    let cwd_str = truncate_path(&app.cwd.to_string_lossy(), cols as usize);
    queue!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetBackgroundColor(Color::DarkGrey),
        SetAttribute(Attribute::Bold),
        Print(format!(" {:<width$}", cwd_str, width = cols as usize)),
        ResetColor,
    )?;

    // File list
    let visible_rows = (file_end.saturating_sub(file_start)) as usize;
    for i in 0..visible_rows {
        let idx = app.scroll_offset + i;
        queue!(stdout, MoveTo(0, file_start + i as u16))?;

        if idx < app.entries.len() {
            let entry = &app.entries[idx];
            let selected = idx == app.selected;
            let marked = app.marks.contains(&idx);

            let icon = if entry.is_dir { "/" } else if entry.is_symlink { "@" } else { " " };
            let mark_str = if marked { "*" } else { " " };

            let size_str = if entry.is_dir {
                "  - ".to_string()
            } else {
                format!("{:>4}", fsops::format_size(entry.size))
            };

            let name_width = cols as usize - 10;
            let name_str = truncate_str(&entry.name, name_width);

            let fg = if entry.is_dir {
                Color::Cyan
            } else if entry.is_symlink {
                Color::Magenta
            } else if entry.permissions & 0o111 != 0 {
                Color::Green
            } else {
                Color::White
            };

            let bg = if selected {
                Color::DarkBlue
            } else {
                Color::Reset
            };

            let line = format!("{}{}{} {:<width$}", mark_str, icon, size_str, name_str, width = name_width);
            queue!(
                stdout,
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print(format!(" {:<width$}", truncate_str(&line, cols as usize - 1), width = cols as usize - 1)),
                ResetColor,
            )?;
        } else {
            queue!(stdout, Print(format!("{:<width$}", "", width = cols as usize)))?;
        }
    }

    // Sort indicator
    let sort_str = match app.sort_by {
        SortBy::Type => "S:Type",
        SortBy::Name => "S:Name",
        SortBy::Size => "S:Size",
        SortBy::Modified => "S:Date",
    };

    // Status bar
    queue!(stdout, MoveTo(0, status_row))?;
    let entry_count = app.entries.len();
    let status_fg = if app.status_is_error { Color::Red } else { Color::Yellow };
    let status_text = if app.status_message.is_empty() {
        format!(" {}f {} /:find Alt+H:hex Alt+M:meta .:hidden", entry_count, sort_str)
    } else {
        format!(" {}", app.status_message)
    };
    queue!(
        stdout,
        SetForegroundColor(status_fg),
        SetBackgroundColor(Color::DarkGrey),
        Print(format!(" {:<width$}", truncate_str(&status_text, cols as usize - 1), width = cols as usize - 1)),
        ResetColor,
    )?;

    // Help bar
    queue!(stdout, MoveTo(0, help_row))?;
    let help_text = match &app.mode {
        Mode::Input(InputKind::Rename) => format!(" Rename: {}_  Enter=OK  Esc=Cancel", app.input_buffer),
        Mode::Input(InputKind::Mkdir) => format!(" New dir: {}_  Enter=OK  Esc=Cancel", app.input_buffer),
        Mode::Input(InputKind::SearchQuery) => format!(" Search: {}_  Enter=Find  Esc=Cancel", app.input_buffer),
        Mode::Input(InputKind::ZipArchive) => format!(" Zip name: {}_  Enter=Create  Esc=Cancel", app.input_buffer),
        Mode::ConfirmDelete => format!(" DELETE? Y/Enter=Yes  N/Esc=No"),
        _ => " jk/arrows nav Enter=open BS=back ^Y=cp ^X=cut ^V=paste ^D=del ^R=ren ^N=mkdir".to_string(),
    };
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        SetBackgroundColor(Color::Black),
        Print(format!(" {:<width$}", truncate_str(&help_text, cols as usize - 1), width = cols as usize - 1)),
        ResetColor,
    )?;

    Ok(())
}

fn draw_hex(app: &App, stdout: &mut impl Write, cols: u16, rows: u16) -> io::Result<()> {
    if let Some(ref path) = app.hex_view_path {
        let max_bytes = 65536;
        let hv = HexView::from_file(path, max_bytes).ok();

        queue!(stdout, MoveTo(0, 0))?;
        queue!(
            stdout,
            SetForegroundColor(Color::Cyan),
            SetAttribute(Attribute::Bold),
            Print(format!(" HEX: {}  Esc/q=Back", truncate_path(&path.display().to_string(), cols as usize - 16))),
            ResetColor,
        )?;

        if let Some(hv) = hv {
            let view_rows = (rows - 2) as usize;
            let mut view_hv = hv;
            view_hv.offset = app.hex_offset;
            let lines = view_hv.lines(view_rows);
            for (i, line) in lines.iter().enumerate() {
                if i as u16 >= rows.saturating_sub(2) { break; }
                queue!(stdout, MoveTo(0, 1 + i as u16))?;
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(truncate_str(line, cols as usize)),
                    ResetColor,
                )?;
            }
            queue!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(" Offset: {:08x}/{}  j/k/PgUp/PgDn/Home/End  Esc=Back", app.hex_offset, view_hv.data.len())),
                ResetColor,
            )?;
        }
    }
    Ok(())
}

fn draw_metadata(app: &App, stdout: &mut impl Write, cols: u16, rows: u16) -> io::Result<()> {
    if let Some(entry) = app.selected_entry() {
        let metadata = std::fs::symlink_metadata(&entry.path).ok();
        let link_target = if entry.is_symlink {
            std::fs::read_link(&entry.path).ok().map(|p| p.display().to_string())
        } else {
            None
        };

        let mut lines = Vec::new();

        lines.push(format!("Name: {}", entry.name));
        lines.push(format!("Path: {}", entry.path.display()));
        lines.push(format!("Type: {}", if entry.is_dir { "Directory" } else if entry.is_symlink { "Symlink" } else { "File" }));
        if let Some(ref target) = link_target {
            lines.push(format!("Target: {}", target));
        }
        lines.push(format!("Size: {} ({} bytes)", fsops::format_size(entry.size), entry.size));

        if let Some(meta) = metadata {
            lines.push(format!("Perms: {}", format_permissions(crate::fsops::get_permissions_mode(&meta))));
            lines.push(format!("UID: {}  GID: {}", entry.owner_uid, entry.group_gid));
            if let Ok(modified) = meta.modified() {
                let datetime: chrono::DateTime<chrono::Local> = modified.into();
                lines.push(format!("Modified: {}", datetime.format("%Y-%m-%d %H:%M:%S")));
            }
            if let Ok(created) = meta.created() {
                let datetime: chrono::DateTime<chrono::Local> = created.into();
                lines.push(format!("Created: {}", datetime.format("%Y-%m-%d %H:%M:%S")));
            }
        }

        if !entry.is_dir {
            lines.push(String::new());
            lines.push("Alt+H = Hex view".to_string());
        }

        for (i, line) in lines.iter().enumerate() {
            if i as u16 + 1 >= rows - 1 { break; }
            queue!(stdout, MoveTo(0, 1 + i as u16))?;
            queue!(
                stdout,
                SetForegroundColor(Color::White),
                Print(truncate_str(line, cols as usize)),
                ResetColor,
            )?;
        }
    }

    queue!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(" Esc/q = Back   Alt+H = Hex view"),
        ResetColor,
    )?;

    Ok(())
}

fn draw_bookmarks(app: &App, stdout: &mut impl Write, cols: u16, rows: u16) -> io::Result<()> {
    queue!(stdout, MoveTo(0, 0))?;
    queue!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(" Bookmarks  Enter=Go  Esc=Back"),
        ResetColor,
    )?;

    for (i, bm) in app.bookmarks.iter().enumerate() {
        if i as u16 + 1 >= rows.saturating_sub(1) { break; }
        let selected = i == app.selected;
        queue!(stdout, MoveTo(0, 1 + i as u16))?;
        let bg = if selected { Color::DarkBlue } else { Color::Reset };
        queue!(
            stdout,
            SetBackgroundColor(bg),
            SetForegroundColor(Color::Yellow),
            Print(format!(" {} {:<width$}", if selected { ">" } else { " " }, truncate_str(&format!("{} {}", bm.name, bm.path.display()), cols as usize - 4), width = cols as usize - 4)),
            ResetColor,
        )?;
    }

    queue!(stdout, MoveTo(0, rows.saturating_sub(1)))?;
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(" j/k=select  Enter=Go  Esc=Back"),
        ResetColor,
    )?;

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("...{}", &s[s.len().saturating_sub(max_len - 3)..])
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len - 3)..])
    }
}

fn format_permissions(mode: u32) -> String {
    let mut s = String::new();
    let bits = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    // File type
    s.push(if mode & 0o170000 != 0 { '-' } else { '-' });
    for &(bit, ch) in &bits {
        s.push(if mode & bit != 0 { ch } else { '-' });
    }
    s
}