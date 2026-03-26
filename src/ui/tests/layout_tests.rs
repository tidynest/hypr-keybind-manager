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

use crate::ui::builders::layout::{clamp_paned_position, IDEAL_RIGHT_PANEL_WIDTH};

#[test]
fn test_clamp_paned_position_uses_startup_width() {
    let position = clamp_paned_position(1000, 1000);
    assert_eq!(position, 1000 - IDEAL_RIGHT_PANEL_WIDTH);
}

#[test]
fn test_clamp_paned_position_limits_right_panel_growth() {
    let position = clamp_paned_position(1200, 700);
    let right_width = 1200 - position;

    assert_eq!(
        right_width, 400,
        "Right panel should cap at one third of the window"
    );
}

#[test]
fn test_clamp_paned_position_preserves_left_panel_minimum() {
    let position = clamp_paned_position(620, 100);
    assert!(
        position >= 620 - (620 / 3),
        "Left panel should retain most of the narrow window width"
    );
}
