#!/usr/bin/env bash
# Cargo runner that filters GTK warnings from output
# This provides clean terminal output for all cargo run commands

"$@" 2>&1 | grep --line-buffered -vE 'Gtk-WARNING|GtkGizmo|Unknown key' | awk 'NF'

# Cargo runs test binaries through this script too. A pipeline reports the
# status of its last command, awk, so without this a failing test run passed.
exit "${PIPESTATUS[0]}"
