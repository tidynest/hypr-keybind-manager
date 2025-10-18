#!/usr/bin/env bash
# Sync version numbers across all project documentation
# Uses Cargo.toml as the single source of truth

set -e

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
    echo "❌ Error: Could not extract version from Cargo.toml"
    exit 1
fi

echo "📦 Syncing version to: $VERSION"

# Update SECURITY.md
sed -i "s/^\*\*Version\*\*: [0-9]\+\.[0-9]\+\.[0-9]\+/**Version**: $VERSION/" SECURITY.md
echo "  ✅ Updated SECURITY.md"

# Update ARCHITECTURE.md
sed -i "s/^\*\*Version\*\*: [0-9]\+\.[0-9]\+\.[0-9]\+/**Version**: $VERSION/" docs/ARCHITECTURE.md
echo "  ✅ Updated docs/ARCHITECTURE.md"

# Update DESIGN_DECISIONS.md
sed -i "s/^\*\*Version\*\*: [0-9]\+\.[0-9]\+\.[0-9]\+/**Version**: $VERSION/" docs/DESIGN_DECISIONS.md
echo "  ✅ Updated docs/DESIGN_DECISIONS.md"

echo "✨ Version sync complete! All files now at v$VERSION"
