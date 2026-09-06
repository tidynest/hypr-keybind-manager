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

//! Transient status line under the header bar
//!
//! Shows short notices such as "Applied to Hyprland" or "Config changed on
//! disk, reloaded" and hides itself again after a few seconds. GTK4 without
//! libadwaita has no toast widget, so this is a Revealer with a label.

use gtk4::{Box as GtkBox, Label, Orientation, Revealer, RevealerTransitionType, prelude::*};
use std::{cell::Cell, rc::Rc, time::Duration};

/// How long a notice stays visible
const SHOW_FOR: Duration = Duration::from_secs(5);

/// Self-hiding notice line
pub struct StatusBanner {
    revealer: Revealer,
    label: Label,
    /// Bumped on every `show`, so an old hide timer does not hide a newer notice
    generation: Rc<Cell<u32>>,
}

impl Default for StatusBanner {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBanner {
    /// Creates a hidden banner
    pub fn new() -> Self {
        let label = Label::builder()
            .xalign(0.0)
            .wrap(true)
            .hexpand(true)
            .build();

        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .build();
        row.add_css_class("status-banner");
        row.append(&label);

        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideDown)
            .transition_duration(200)
            .reveal_child(false)
            .child(&row)
            .build();

        Self {
            revealer,
            label,
            generation: Rc::new(Cell::new(0)),
        }
    }

    /// Returns the root widget for adding to a container
    pub fn widget(&self) -> &Revealer {
        &self.revealer
    }

    /// Shows `text` for a few seconds
    pub fn show(&self, text: &str) {
        self.label.set_label(text);
        self.revealer.set_reveal_child(true);

        let generation = self.generation.get().wrapping_add(1);
        self.generation.set(generation);

        let revealer = self.revealer.clone();
        let current = self.generation.clone();
        glib::timeout_add_local_once(SHOW_FOR, move || {
            if current.get() == generation {
                revealer.set_reveal_child(false);
            }
        });
    }
}
