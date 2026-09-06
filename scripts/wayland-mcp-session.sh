#!/bin/sh
set -eu

export FASTMCP_SHOW_CLI_BANNER="false"
export FASTMCP_LOG_LEVEL="ERROR"
export MCP_CONFIG_DIR="${MCP_CONFIG_DIR-}"
export OPENROUTER_API_KEY="${OPENROUTER_API_KEY-}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY-}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR-}"

exec /home/bakri/.local/share/pipx/venvs/wayland-mcp/bin/python \
  /home/bakri/RustroverProjects/hypr-keybind-manager/scripts/wayland-mcp-stdio.py \
  "$@"
