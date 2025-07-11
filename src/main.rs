use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;
use md5;
use dirs;
use ini::Ini;
use mime_guess::get_mime_extensions_str;

/// Calculates the MD5 hash of the file URI as GNOME does
fn compute_thumbnail_hash(path: &Path) -> String {
    let uri = format!("file://{}", path.to_string_lossy());
    format!("{:x}", md5::compute(uri))
}

/// Removes the thumbnails corresponding to the given file
fn purge_thumbnail(hash: &str, cache_dir: &Path) {
    for size in &["normal", "large"] {
        let thumb_path = cache_dir.join(size).join(format!("{}.png", hash));
        if thumb_path.exists() {
            match fs::remove_file(&thumb_path) {
                Ok(_) => println!("Removed thumbnail: {}", thumb_path.display()),
                Err(e) => eprintln!("Failed to remove thumbnail {}: {}", thumb_path.display(), e),
            }
        }
    }
}

/// Checks if an extension is among those supported by thumbnailers
fn is_thumbnail_candidate(path: &Path, valid_exts: &HashSet<String>) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        valid_exts.contains(&ext.to_lowercase())
    } else {
        false
    }
}

/// Handles a deleted file: purges its thumbnail if applicable
fn handle_deleted(path: &Path, cache_dir: &Path) {
    let hash = compute_thumbnail_hash(path);
    purge_thumbnail(&hash, cache_dir);
}

/// Checks if a path is inside the thumbnail cache directory
fn is_in_thumbnail_cache(path: &Path, cache_dir: &Path) -> bool {
    path.starts_with(cache_dir)
}

/// Reads installed thumbnailers and extracts supported extensions
fn collect_supported_extensions() -> HashSet<String> {
    let mut extensions = HashSet::new();

    let dirs = vec![
        PathBuf::from("/usr/share/thumbnailers"),
        dirs::data_dir().unwrap_or_default().join("thumbnailers"),
    ];

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("thumbnailer") {
                    continue;
                }

                if let Ok(conf) = Ini::load_from_file(&path) {
                    if let Some(section) = conf.section(Some("Thumbnailer Entry")) {
                        if let Some(mime_list) = section.get("MimeType") {
                            for mime in mime_list.split(';').filter(|s| !s.is_empty()) {
                                if let Some(exts) = get_mime_extensions_str(mime) {
                                    for ext in exts {
                                        extensions.insert(ext.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    extensions
}

fn main() -> notify::Result<()> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let cache_dir = home_dir.join(".cache").join("thumbnails");

    let valid_extensions = collect_supported_extensions();

    println!(
        "Extensions being watched for thumbnail cleanup: {}",
        valid_extensions.iter().map(|ext| format!(".{}", ext)).collect::<Vec<_>>().join(", ")
    );


    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    println!("Thumbnail purge daemon running. Watching {}", home_dir.display());
    watcher.watch(&home_dir, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if matches!(
                    event.kind,
                    EventKind::Remove(_) | EventKind::Modify(notify::event::ModifyKind::Name(_))
                ) {
                    for path in event.paths {
                        if is_in_thumbnail_cache(&path, &cache_dir) {
                            continue;
                        }
                        if is_thumbnail_candidate(&path, &valid_extensions) {
                            handle_deleted(&path, &cache_dir);
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("Watcher error: {:?}", e),
            Err(e) => eprintln!("Channel receive error: {:?}", e),
        }
    }
}
