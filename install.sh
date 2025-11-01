#!/usr/bin/env bash
set -e # Exit on any error

# Colour output helpers
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Colour

print_error() {
  echo -e "${RED}✗${NC} $1"
}

print_info() {
  echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
  echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
  echo -e "${YELLOW}⚠${NC} $1"
}

# Configuration
BINARY_NAME="hypr-keybind-manager"
INSTALL_DIR="$HOME/.local/bin"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"

print_info "Starting installation of $BINARY_NAME..."
echo

# Step 1: Build the release binary
print_info "Building release binary (this may take a minute...)"
if cargo build --release; then
  print_success "Build completed successfully."
else
  print_error "Build failed. Please check the output above."
  exit 1
fi
echo

# Step 2: Create installation directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
  print_info "Creating $INSTALL_DIR directory..."
  mkdir -p "$INSTALL_DIR"
  print_success "Directory created."
else
  print_info "Installation directory already exists."
fi
echo

# Step 3: Copy the binary
print_info "Installing $BINARY_NAME to $INSTALL_DIR..."
if cp target/release/$BINARY_NAME "$BINARY_PATH"; then
  print_success "Binary installed successfully."
else
  print_error "Failed to copy binary to $INSTALL_DIR"
  exit 1
fi
echo

# Step 4: Check if installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  print_warning "$INSTALL_DIR is not in your PATH."
  echo "  Add this line to your shell config file (~/.bashrc, ~/.zshrc, etc.):"
  echo
  echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo
  echo "  Then restart your shell or run < source ~/.bashrc >, < source ~/.zshrc >, ..."
  echo
else
  print_success "$INSTALL_DIR is in your PATH"
fi
echo

# Step 5: Print usage instructions
print_success "Installation complete!"
echo
print_info "Next steps:"
echo
echo "  1. Add a keybinding to your Hyprland config (~/.config/hypr/hyprland.conf)."
echo "     => OBS: Make sure that it doesn't conflict with an existing binding."
echo "     Example keybinding:"
echo
echo "     bind = SUPER, B, exec, $BINARY_NAME"
echo
echo "  2. Reload Hyprland to apply the changes:"
echo
echo "     hyprctl reload"
echo
echo "  3. Press SUPER+B (or your corresponding, chosen keybinding) to launch Hyprland Keybinding Manager!"
echo
print_info "For more information, see the README.md file."
