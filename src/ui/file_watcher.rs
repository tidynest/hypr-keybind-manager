// Copyright 2025 Eric Jingryd (tidynest@proton.me)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! File system watcher for live config file monitoring
//!
//! Uses OS-level file watching (Linux inotify) via the notify crate.
//! Zero CPU overhead when file unchanged, instant UI refresh on modification.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
};

/// Watches Hyprland.conf file for modifications and notifies via callback
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    /// The config file, events for other files in its directory are ignored
    path: PathBuf,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        // Watch the directory, not the file: the app replaces the file by
        // atomic rename, and a watch on the old inode would go dead after
        // the first write.
        let path = path.canonicalize().unwrap_or(path);
        let dir = path.parent().ok_or("config file has no parent directory")?;
        watcher.watch(dir, RecursiveMode::NonRecursive)?;

        Ok(FileWatcher {
            _watcher: watcher,
            rx,
            path,
        })
    }

    /// Checks for file modification events (non-blocking)
    pub fn check_for_changes(&self) -> bool {
        let mut changed = false;
        while let Ok(event_result) = self.rx.try_recv() {
            let Ok(event) = event_result else { continue };
            let touches_config = event.paths.iter().any(|p| p == &self.path);
            let is_write = matches!(
                event.kind,
                notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_)
            );
            if touches_config && is_write {
                changed = true;
            }
        }
        changed
    }
}
