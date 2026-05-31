use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;
use crate::app::{FileEntry, SortBy};
use std::os::unix::fs::MetadataExt as UnixMetadataExt;

pub fn read_dir_sorted(
    dir: &Path,
    show_hidden: bool,
    sort_by: SortBy,
    reverse: bool,
) -> Result<Vec<FileEntry>, Box<dyn std::error::Error>> {
    let mut entries: Vec<FileEntry> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !show_hidden {
                if let Some(name) = e.file_name().to_str() {
                    return !name.starts_with('.');
                }
            }
            true
        })
        .filter_map(|e| {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let metadata = e.metadata().ok()?;
            let is_dir = metadata.is_dir();
            let is_symlink = metadata.file_type().is_symlink();
            let size = if is_dir { 0 } else { metadata.len() };
            let modified = metadata.modified().ok()?;
            let permissions = metadata.mode();
            let owner_uid = metadata.uid();
            let group_gid = metadata.gid();
            Some(FileEntry {
                name,
                path,
                is_dir,
                is_symlink,
                size,
                modified,
                permissions,
                owner_uid,
                group_gid,
            })
        })
        .collect();

    entries.sort_by(|a, b| {
        let cmp = match sort_by {
            SortBy::Type => {
                a.is_dir.cmp(&b.is_dir).reverse().then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Size => a.size.cmp(&b.size),
            SortBy::Modified => a.modified.cmp(&b.modified),
        };
        if reverse { cmp.reverse() } else { cmp }
    });

    Ok(entries)
}

pub fn get_permissions_mode(meta: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode()
}

pub fn copy_file(src: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = src.file_name().ok_or("No filename")?;
    let dest = dest_dir.join(file_name);
    if dest.exists() {
        return Err(format!("File exists: {}", dest.display()).into());
    }
    std::fs::copy(src, &dest)?;
    Ok(())
}

pub fn move_file(src: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = src.file_name().ok_or("No filename")?;
    let dest = dest_dir.join(file_name);
    if dest.exists() && src != dest {
        return Err(format!("File exists: {}", dest.display()).into());
    }
    std::fs::rename(src, &dest)?;
    Ok(())
}

pub fn delete_entry(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn rename_entry(src: &Path, new_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dest = src.parent().unwrap_or(Path::new(".")).join(new_name);
    std::fs::rename(src, &dest)?;
    Ok(())
}

pub fn create_dir(parent: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = parent.join(name);
    std::fs::create_dir(&path)?;
    Ok(())
}

pub fn create_zip(files: &[PathBuf], dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(dest)?;
    let mut zw = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for src_path in files {
        if src_path.is_dir() {
            add_dir_to_zip(&mut zw, src_path, src_path, options.clone())?;
        } else {
            let name = src_path.file_name().unwrap_or_default().to_string_lossy();
            let mut f = std::fs::File::open(src_path)?;
            zw.start_file(name.as_ref(), options.clone())?;
            std::io::copy(&mut f, &mut zw)?;
        }
    }
    zw.finish()?;
    Ok(())
}

fn add_dir_to_zip(
    zw: &mut zip::ZipWriter<std::fs::File>,
    dir: &Path,
    base: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let name = relative.to_string_lossy();

        if path.is_dir() {
            zw.add_directory(name.as_ref(), options.clone())?;
            add_dir_to_zip(zw, &path, base, options.clone())?;
        } else {
            let mut f = std::fs::File::open(&path)?;
            zw.start_file(name.as_ref(), options.clone())?;
            std::io::copy(&mut f, zw)?;
        }
    }
    Ok(())
}

pub fn extract_zip(archive: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(archive)?;
    let mut za = zip::ZipArchive::new(file)?;
    for i in 0..za.len() {
        let mut entry = za.by_index(i)?;
        let outpath = dest_dir.join(entry.name());
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }
    Ok(())
}

pub fn search_files(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let regex = regex::Regex::new(&format!("(?i){}", regex::escape(pattern)));
    let mut results = Vec::new();
    for entry in WalkDir::new(root).max_depth(8).into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy();
        if let Ok(ref re) = regex {
            if re.is_match(&name) {
                results.push(entry.path().to_path_buf());
            }
        } else if name.to_lowercase().contains(&pattern.to_lowercase()) {
            results.push(entry.path().to_path_buf());
        }
    }
    results
}

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if size >= GB { format!("{:.1}G", size as f64 / GB as f64) }
    else if size >= MB { format!("{:.1}M", size as f64 / MB as f64) }
    else if size >= KB { format!("{:.1}K", size as f64 / KB as f64) }
    else { format!("{}B", size) }
}