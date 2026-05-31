use std::path::PathBuf;
use crate::app::App;

pub fn search_current_dir(app: &mut App, pattern: &str) {
    app.search_results = crate::fsops::search_files(&app.cwd, pattern);
    app.search_idx = 0;
    if app.search_results.is_empty() {
        app.set_status("No results found", false);
    } else {
        app.set_status(&format!("Found {} results", app.search_results.len()), false);
        jump_to_result(app);
    }
}

pub fn next_result(app: &mut App) {
    if !app.search_results.is_empty() {
        app.search_idx = (app.search_idx + 1) % app.search_results.len();
        jump_to_result(app);
    }
}

pub fn prev_result(app: &mut App) {
    if !app.search_results.is_empty() {
        if app.search_idx == 0 {
            app.search_idx = app.search_results.len() - 1;
        } else {
            app.search_idx -= 1;
        }
        jump_to_result(app);
    }
}

fn jump_to_result(app: &mut App) {
    if let Some(path) = app.search_results.get(app.search_idx).cloned() {
        if let Some(parent) = path.parent() {
            if parent != app.cwd {
                let _ = app.navigate_to(parent.to_path_buf());
            }
            if let Some(idx) = app.entries.iter().position(|e| e.path == path) {
                app.selected = idx;
                app.scroll_offset = app.selected.saturating_sub(3);
            }
        }
    }
}