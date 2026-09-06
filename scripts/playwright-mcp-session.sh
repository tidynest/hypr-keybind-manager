#!/bin/sh
set -eu

exec npx -y @playwright/mcp@latest --executable-path /usr/bin/chromium "$@"
