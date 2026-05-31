use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Navigate,
    Search,
    HexView,
    Metadata,
    BookmarkList,
    ConfirmDelete,
    ConfirmOverwrite,
    Input(InputKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputKind {
    Rename,
    Mkdir,
    SearchQuery,
    ZipArchive,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: u32,
    pub owner_uid: u32,
    pub group_gid: u32,
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
}

pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub show_hidden: bool,
    pub sort_by: SortBy,
    pub sort_reverse: bool,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub status_message: String,
    pub status_is_error: bool,
    pub clipboard: Option<ClipboardEntry>,
    pub marks: Vec<usize>,
    pub bookmarks: Vec<Bookmark>,
    pub hex_view_path: Option<PathBuf>,
    pub hex_offset: usize,
    pub search_results: Vec<PathBuf>,
    pub search_idx: usize,
    pub history: Vec<PathBuf>,
    pub history_idx: usize,
    pub confirm_target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub path: PathBuf,
    pub is_cut: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

impl App {
    pub fn new(start_dir: PathBuf, show_hidden: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut app = App {
            cwd: start_dir.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            mode: Mode::Navigate,
            show_hidden,
            sort_by: SortBy::Type,
            sort_reverse: false,
            input_buffer: String::new(),
            input_cursor: 0,
            status_message: String::new(),
            status_is_error: false,
            clipboard: None,
            marks: Vec::new(),
            bookmarks: vec![
                Bookmark { name: "Home".into(), path: dirs_home() },
                Bookmark { name: "Root".into(), path: PathBuf::from("/") },
                Bookmark { name: "Loot".into(), path: PathBuf::from("/opt/cardputer/loot") },
                Bookmark { name: "Config".into(), path: PathBuf::from("/opt/cardputer/config") },
                Bookmark { name: "Capture".into(), path: PathBuf::from("/opt/cardputer/capture") },
                Bookmark { name: "TMP".into(), path: PathBuf::from("/tmp") },
            ],
            hex_view_path: None,
            hex_offset: 0,
            search_results: Vec::new(),
            search_idx: 0,
            history: vec![start_dir.clone()],
            history_idx: 0,
            confirm_target: None,
        };
        app.reload()?;
        Ok(app)
    }

    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.entries = crate::fsops::read_dir_sorted(&self.cwd, self.show_hidden, self.sort_by, self.sort_reverse)?;
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
        if self.entries.is_empty() {
            self.selected = 0;
        }
        Ok(())
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if path.is_dir() {
            let canonical = path.canonicalize().unwrap_or(path.clone());
            self.history.truncate(self.history_idx + 1);
            self.history.push(canonical.clone());
            self.history_idx = self.history.len() - 1;
            self.cwd = canonical;
            self.selected = 0;
            self.scroll_offset = 0;
            self.status_message.clear();
            self.status_is_error = false;
            self.reload()?;
        }
        Ok(())
    }

    pub fn go_back(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            let path = self.history[self.history_idx].clone();
            self.cwd = path;
            self.selected = 0;
            self.scroll_offset = 0;
            self.reload()?;
        }
        Ok(())
    }

    pub fn go_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.history_idx < self.history.len() - 1 {
            self.history_idx += 1;
            let path = self.history[self.history_idx].clone();
            self.cwd = path;
            self.selected = 0;
            self.scroll_offset = 0;
            self.reload()?;
        }
        Ok(())
    }

    pub fn go_parent(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.cwd.parent() {
            self.navigate_to(parent.to_path_buf())?;
        }
        Ok(())
    }

    pub fn enter_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(entry) = self.selected_entry().cloned() {
            if entry.is_dir {
                self.navigate_to(entry.path)?;
            } else {
                self.hex_view_path = Some(entry.path.clone());
                self.hex_offset = 0;
                self.mode = Mode::HexView;
            }
        }
        Ok(())
    }

    pub fn set_status(&mut self, msg: &str, is_error: bool) {
        self.status_message = msg.to_string();
        self.status_is_error = is_error;
    }

    pub fn clear_status(&mut self) {
        self.status_message.clear();
        self.status_is_error = false;
    }

    pub fn toggle_mark(&mut self) {
        if self.marks.contains(&self.selected) {
            self.marks.retain(|&m| m != self.selected);
        } else {
            self.marks.push(self.selected);
        }
    }

    pub fn mark_all(&mut self) {
        self.marks = (0..self.entries.len()).collect();
    }

    pub fn clear_marks(&mut self) {
        self.marks.clear();
    }

    pub fn marked_entries(&self) -> Vec<&FileEntry> {
        if self.marks.is_empty() {
            vec![self.selected_entry()].into_iter().flatten().collect()
        } else {
            self.marks.iter().filter_map(|&i| self.entries.get(i)).collect()
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/root"))
}