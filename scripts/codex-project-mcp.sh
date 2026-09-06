#!/bin/sh
set -eu

ROOT="/home/bakri/RustroverProjects/hypr-keybind-manager"

exec codex \
  -C "$ROOT" \
  -c "mcp_servers.wayland.command=\"$ROOT/scripts/wayland-mcp-session.sh\"" \
  -c 'mcp_servers.wayland.args=[]' \
  -c 'mcp_servers.wayland.enabled=true' \
  -c 'mcp_servers.wayland.startup_timeout_sec=20' \
  -c 'mcp_servers.wayland.tool_timeout_sec=60' \
  -c 'mcp_servers.wayland.env.MCP_CONFIG_DIR="/home/bakri/.roo"' \
  -c 'mcp_servers.wayland.env_vars=["OPENROUTER_API_KEY"]' \
  -c "mcp_servers.playwright.command=\"$ROOT/scripts/playwright-mcp-session.sh\"" \
  -c 'mcp_servers.playwright.args=[]' \
  -c 'mcp_servers.playwright.enabled=true' \
  -c 'mcp_servers.playwright.startup_timeout_sec=30' \
  -c 'mcp_servers.playwright.tool_timeout_sec=120' \
  -c 'mcp_servers.filesystem.command="npx"' \
  -c 'mcp_servers.filesystem.args=["-y","@modelcontextprotocol/server-filesystem","/home/bakri/RustroverProjects/hypr-keybind-manager","/home/bakri/.codex/memories","/tmp"]' \
  -c 'mcp_servers.filesystem.enabled=true' \
  -c 'mcp_servers.filesystem.startup_timeout_sec=60' \
  -c 'mcp_servers.filesystem.tool_timeout_sec=60' \
  -c 'mcp_servers.hyprland.enabled=false' \
  "$@"
