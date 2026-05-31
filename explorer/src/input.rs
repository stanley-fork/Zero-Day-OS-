use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;
use crate::app::{App, Mode, InputKind, ClipboardEntry, SortBy};
use crate::hexview::HexView;
use crate::fsops;
use crate::search;

pub fn handle_input(app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if event::poll(Duration::from_millis(100))? {
        let ev = event::read()?;
        match ev {
            Event::Key(key) => handle_key(app, key),
            Event::Mouse(mouse) => handle_mouse(app, mouse),
            Event::Resize(..) => Ok(true),
            _ => Ok(true),
        }
    } else {
        Ok(true)
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    // Clone mode to avoid borrow issues
    let mode = app.mode.clone();
    match mode {
        Mode::Navigate => handle_navigate(app, key),
        Mode::Search => handle_search_mode(app, key),
        Mode::HexView => handle_hex(app, key),
        Mode::Metadata => handle_metadata(app, key),
        Mode::BookmarkList => handle_bookmarks(app, key),
        Mode::ConfirmDelete => handle_confirm(app, key),
        Mode::ConfirmOverwrite => handle_confirm(app, key),
        Mode::Input(kind) => handle_input_mode(app, key, &kind),
    }
}

fn handle_navigate(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if ctrl => {
            if app.selected > 0 {
                app.selected -= 1;
                if app.selected < app.scroll_offset {
                    app.scroll_offset = app.selected;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') if ctrl => {
            if !app.entries.is_empty() && app.selected < app.entries.len() - 1 {
                app.selected += 1;
                let view_height = 18usize;
                if app.selected >= app.scroll_offset + view_height {
                    app.scroll_offset = app.selected.saturating_sub(view_height - 1);
                }
            }
        }
        KeyCode::Up if !ctrl => {
            if app.selected > 0 {
                app.selected -= 1;
                if app.selected < app.scroll_offset {
                    app.scroll_offset = app.selected;
                }
            }
        }
        KeyCode::Down if !ctrl => {
            if !app.entries.is_empty() && app.selected < app.entries.len() - 1 {
                app.selected += 1;
                let view_height = 18usize;
                if app.selected >= app.scroll_offset + view_height {
                    app.scroll_offset = app.selected.saturating_sub(view_height - 1);
                }
            }
        }
        KeyCode::Left => { app.go_parent()?; }
        KeyCode::Right | KeyCode::Enter => { app.enter_selected()?; }
        KeyCode::Home => { app.selected = 0; app.scroll_offset = 0; }
        KeyCode::End => {
            if !app.entries.is_empty() {
                app.selected = app.entries.len() - 1;
                app.scroll_offset = app.selected.saturating_sub(8);
            }
        }
        KeyCode::PageUp => {
            app.selected = app.selected.saturating_sub(10);
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::PageDown => {
            if !app.entries.is_empty() {
                app.selected = (app.selected + 10).min(app.entries.len() - 1);
                app.scroll_offset = app.scroll_offset + 10;
            }
        }
        KeyCode::Backspace => { app.go_back()?; }

        // File operations
        KeyCode::Char('y') if ctrl => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                app.clipboard = Some(ClipboardEntry { path: entry.path.clone(), is_cut: false });
                app.set_status(&format!("Copy: {}", entry.name), false);
            }
        }
        KeyCode::Char('x') if ctrl => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                app.clipboard = Some(ClipboardEntry { path: entry.path.clone(), is_cut: true });
                app.set_status(&format!("Cut: {}", entry.name), false);
            }
        }
        KeyCode::Char('v') if ctrl => {
            if let Some(ref clip) = app.clipboard.clone() {
                let dest_dir = app.cwd.clone();
                let is_cut = clip.is_cut;
                let src_path = clip.path.clone();
                if is_cut {
                    match fsops::move_file(&src_path, &dest_dir) {
                        Ok(()) => { app.set_status("Moved", false); app.clipboard = None; app.reload()?; }
                        Err(e) => app.set_status(&format!("Move error: {}", e), true),
                    }
                } else {
                    match fsops::copy_file(&src_path, &dest_dir) {
                        Ok(()) => { app.set_status("Copied", false); app.reload()?; }
                        Err(e) => app.set_status(&format!("Copy error: {}", e), true),
                    }
                }
            }
        }
        KeyCode::Char('d') if ctrl => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                app.confirm_target = Some(entry.name.clone());
                app.mode = Mode::ConfirmDelete;
            }
        }
        KeyCode::Char('r') if ctrl => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                app.input_buffer = entry.name;
                app.input_cursor = app.input_buffer.len();
                app.mode = Mode::Input(InputKind::Rename);
            }
        }
        KeyCode::Char('n') if ctrl => {
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.mode = Mode::Input(InputKind::Mkdir);
        }
        KeyCode::Char('z') if ctrl => {
            app.input_buffer = "archive.zip".to_string();
            app.input_cursor = app.input_buffer.len();
            app.mode = Mode::Input(InputKind::ZipArchive);
        }
        KeyCode::Char('e') if ctrl => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                if entry.name.ends_with(".zip") {
                    match fsops::extract_zip(&entry.path, &app.cwd) {
                        Ok(()) => { app.set_status("Extracted", false); app.reload()?; }
                        Err(e) => app.set_status(&format!("Extract error: {}", e), true),
                    }
                } else {
                    app.set_status("Not a zip file", true);
                }
            }
        }
        KeyCode::Char('o') if ctrl => { app.go_back()?; }
        KeyCode::Char('i') if ctrl => { app.go_forward()?; }

        // View modes
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::ALT) => {
            if let Some(entry) = app.entries.get(app.selected).cloned() {
                if !entry.is_dir {
                    app.hex_view_path = Some(entry.path);
                    app.hex_offset = 0;
                    app.mode = Mode::HexView;
                }
            }
        }
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.mode = Mode::Metadata;
        }
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.mode = Mode::BookmarkList;
        }
        KeyCode::Char('/') | KeyCode::Char('f') if ctrl => {
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.mode = Mode::Input(InputKind::SearchQuery);
        }

        // Marking
        KeyCode::Char(' ') => { app.toggle_mark(); }
        KeyCode::Char('a') if ctrl => { app.mark_all(); }
        KeyCode::Char('u') if ctrl => { app.clear_marks(); }

        // Sort
        KeyCode::Char('s') if ctrl => {
            app.sort_by = match app.sort_by {
                SortBy::Type => SortBy::Name,
                SortBy::Name => SortBy::Size,
                SortBy::Size => SortBy::Modified,
                SortBy::Modified => SortBy::Type,
            };
            app.sort_reverse = false;
            app.reload()?;
        }

        // Hidden files
        KeyCode::Char('.') => {
            app.show_hidden = !app.show_hidden;
            app.reload()?;
            app.set_status(&format!("Hidden: {}", if app.show_hidden { "ON" } else { "OFF" }), false);
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            return Ok(false);
        }

        _ => {}
    }
    Ok(true)
}

fn handle_hex(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Up => { app.hex_offset = app.hex_offset.saturating_sub(16); }
        KeyCode::Down => { app.hex_offset = app.hex_offset.saturating_add(16); }
        KeyCode::PageUp => { app.hex_offset = app.hex_offset.saturating_sub(16 * 15); }
        KeyCode::PageDown => { app.hex_offset = app.hex_offset.saturating_add(16 * 15); }
        KeyCode::Home => { app.hex_offset = 0; }
        KeyCode::End => {
            if let Some(path) = app.hex_view_path.clone() {
                if let Ok(meta) = std::fs::metadata(&path) {
                    app.hex_offset = meta.len().saturating_sub(16 * 5) as usize;
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.hex_view_path = None;
            app.mode = Mode::Navigate;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_metadata(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
        app.mode = Mode::Navigate;
    }
    Ok(true)
}

fn handle_bookmarks(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Up => {
            if app.selected > 0 { app.selected -= 1; }
        }
        KeyCode::Down => {
            if app.selected < app.bookmarks.len() - 1 { app.selected += 1; }
        }
        KeyCode::Enter => {
            if let Some(bm) = app.bookmarks.get(app.selected).cloned() {
                let path = bm.path.clone();
                app.mode = Mode::Navigate;
                let _ = app.navigate_to(path);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Navigate;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_confirm(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            if app.mode == Mode::ConfirmDelete {
                let entries: Vec<_> = app.marked_entries().into_iter().cloned().collect();
                if entries.is_empty() {
                    if let Some(target) = app.confirm_target.take() {
                        let path = app.cwd.join(&target);
                        match fsops::delete_entry(&path) {
                            Ok(()) => app.set_status("Deleted", false),
                            Err(e) => app.set_status(&format!("Delete error: {}", e), true),
                        }
                    }
                } else {
                    for entry in &entries {
                        match fsops::delete_entry(&entry.path) {
                            Ok(()) => {}
                            Err(e) => app.set_status(&format!("Error: {}", e), true),
                        }
                    }
                    app.clear_marks();
                    app.set_status(&format!("Deleted {} items", entries.len()), false);
                }
            }
            app.mode = Mode::Navigate;
            app.confirm_target = None;
            app.reload()?;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::Navigate;
            app.confirm_target = None;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_input_mode(app: &mut App, key: KeyEvent, kind: &InputKind) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Enter => {
            let input = app.input_buffer.clone();
            let kind = kind.clone();
            match kind {
                InputKind::Rename => {
                    if let Some(entry) = app.entries.get(app.selected).cloned() {
                        match fsops::rename_entry(&entry.path, &input) {
                            Ok(()) => { app.set_status(&format!("Renamed to {}", input), false); app.reload()?; }
                            Err(e) => app.set_status(&format!("Rename error: {}", e), true),
                        }
                    }
                }
                InputKind::Mkdir => {
                    match fsops::create_dir(&app.cwd, &input) {
                        Ok(()) => { app.set_status(&format!("Created: {}", input), false); app.reload()?; }
                        Err(e) => app.set_status(&format!("Mkdir error: {}", e), true),
                    }
                }
                InputKind::SearchQuery => {
                    search::search_current_dir(app, &input);
                }
                InputKind::ZipArchive => {
                    let entries: Vec<_> = app.marked_entries().into_iter().cloned().collect();
                    let paths: Vec<_> = entries.iter().map(|e| e.path.clone()).collect();
                    let dest = app.cwd.join(&input);
                    if paths.is_empty() {
                        app.set_status("No files selected", true);
                    } else {
                        match fsops::create_zip(&paths, &dest) {
                            Ok(()) => { app.set_status(&format!("Created: {}", input), false); app.clear_marks(); app.reload()?; }
                            Err(e) => app.set_status(&format!("Zip error: {}", e), true),
                        }
                    }
                }
            }
            app.mode = Mode::Navigate;
            app.input_buffer.clear();
        }
        KeyCode::Esc => {
            app.mode = Mode::Navigate;
            app.input_buffer.clear();
        }
        KeyCode::Backspace => {
            if app.input_cursor > 0 {
                app.input_buffer.remove(app.input_cursor - 1);
                app.input_cursor -= 1;
            }
        }
        KeyCode::Delete => {
            if app.input_cursor < app.input_buffer.len() {
                app.input_buffer.remove(app.input_cursor);
            }
        }
        KeyCode::Left => {
            if app.input_cursor > 0 { app.input_cursor -= 1; }
        }
        KeyCode::Right => {
            if app.input_cursor < app.input_buffer.len() { app.input_cursor += 1; }
        }
        KeyCode::Home => { app.input_cursor = 0; }
        KeyCode::End => { app.input_cursor = app.input_buffer.len(); }
        KeyCode::Char(c) => {
            app.input_buffer.insert(app.input_cursor, c);
            app.input_cursor += 1;
        }
        _ => {}
    }
    Ok(true)
}

fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Navigate;
            app.search_results.clear();
        }
        KeyCode::Char('n') => search::next_result(app),
        KeyCode::Char('N') => search::prev_result(app),
        _ => {}
    }
    Ok(true)
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Result<bool, Box<dyn std::error::Error>> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if app.selected > 0 {
                app.selected -= 1;
                if app.selected < app.scroll_offset {
                    app.scroll_offset = app.selected;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if !app.entries.is_empty() && app.selected < app.entries.len() - 1 {
                app.selected += 1;
            }
        }
        _ => {}
    }
    Ok(true)
}