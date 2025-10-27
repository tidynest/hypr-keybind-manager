//! File system watcher for live config file monitoring
//!
//! Uses OS-level file watching (Linux inotify) via the notify crate.
//! Zero CPU overhead when file unchanged, instant UI refresh on modification.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

/// Watches Hyprland.conf file for modifications and notifies via callback
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(move |res| {
            let _ = tx.send(res);
        },
          Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        Ok(FileWatcher {
            _watcher: watcher,
            rx,
        })
    }

    /// Checks for file modification events (non-blocking)
    pub fn check_for_changes(&self) -> bool {
        while let Ok(event_result) = self.rx.try_recv() {
            if let Ok(event) = event_result {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    return true;
                }
            }
        }
        false
    }
}