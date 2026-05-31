use std::path::PathBuf;
use crate::app::{App, Bookmark};

pub fn load_bookmarks(path: &std::path::Path) -> Vec<Bookmark> {
    if path.exists() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        content.lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let parts: Vec<&str> = l.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some(Bookmark {
                        name: parts[0].trim().to_string(),
                        path: PathBuf::from(parts[1].trim()),
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn save_bookmarks(path: &std::path::Path, bookmarks: &[Bookmark]) -> Result<(), Box<dyn std::error::Error>> {
    let content = bookmarks.iter()
        .map(|b| format!("{}={}", b.name, b.path.display()))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, content)?;
    Ok(())
}

pub fn add_bookmark(app: &mut App) {
    if let Some(entry) = app.selected_entry().cloned() {
        let name = if entry.is_dir {
            entry.name.clone()
        } else {
            entry.name.clone()
        };
        let path = entry.path.clone();
        if !app.bookmarks.iter().any(|b| b.path == path) {
            app.bookmarks.push(Bookmark { name, path });
            app.set_status(&format!("Bookmark added: {}", entry.name), false);
        } else {
            app.set_status("Already bookmarked", false);
        }
    }
}

pub fn remove_bookmark(app: &mut App, idx: usize) {
    if idx < app.bookmarks.len() {
        let name = app.bookmarks[idx].name.clone();
        app.bookmarks.remove(idx);
        app.set_status(&format!("Bookmark removed: {}", name), false);
    }
}