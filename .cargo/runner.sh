#!/usr/bin/env bash
# Cargo runner that filters GTK warnings from output
# This provides clean terminal output for all cargo run commands

exec "$@" 2>&1 | grep --line-buffered -vE 'Gtk-WARNING|GtkGizmo|Unknown key' | awk 'NF'
