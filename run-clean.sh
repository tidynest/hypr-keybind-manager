#!/usr/bin/env zsh
cargo run -- gui -c /tmp/hyprland-test.conf 2>&1 | grep --line-buffered -vE 'Gtk-WARNING|GtkGizmo|Unknown key' | awk 'NF'
