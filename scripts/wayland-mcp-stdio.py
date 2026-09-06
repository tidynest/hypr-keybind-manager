#!/usr/bin/env python3
import importlib
import os
import sys


def main() -> None:
    original_stdout = sys.stdout
    sys.stdout = sys.stderr
    try:
        module = importlib.import_module("wayland_mcp.server_mcp")
    finally:
        sys.stdout = original_stdout

    os.environ.setdefault("FASTMCP_SHOW_CLI_BANNER", "false")
    os.environ.setdefault("FASTMCP_LOG_LEVEL", "ERROR")
    module.mcp.run(show_banner=False)


if __name__ == "__main__":
    main()
