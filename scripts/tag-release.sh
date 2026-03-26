#!/usr/bin/env bash
# Automated release tagging script
# Updates version across all project files and creates git tag
# Usage: ./scripts/tag-release.sh <version>
# Example: ./scripts/tag-release.sh 1.0.4

set -e

if [ -z "$1" ]; then
    echo "❌ Error: Version number required"
    echo "Usage: ./scripts/tag-release.sh <version>"
    echo "Example: ./scripts/tag-release.sh 1.0.4"
    exit 1
fi

NEW_VERSION="$1"

# Validate version format (semantic versioning)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "❌ Error: Invalid version format. Use semantic versioning (e.g., 1.0.4)"
    exit 1
fi

echo "🚀 Starting release process for v$NEW_VERSION"
echo ""

# Check if tag already exists
if git tag -l | grep -q "^v$NEW_VERSION$"; then
    echo "❌ Error: Tag v$NEW_VERSION already exists"
    echo "   Existing tags:"
    git tag -l | tail -5
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   Please commit or stash them first"
    git status --short
    exit 1
fi

echo "📝 Step 1/6: Updating Cargo.toml"
sed -i "s/^version = \"[0-9]\+\.[0-9]\+\.[0-9]\+\"/version = \"$NEW_VERSION\"/" Cargo.toml
echo "  ✅ Cargo.toml updated to v$NEW_VERSION"
echo ""

echo "📝 Step 2/6: Synchronising version to documentation files"
./scripts/sync-version.sh
if [ -f CHANGELOG.md ] && ! grep -q "^## \[$NEW_VERSION\]" CHANGELOG.md; then
    RELEASE_DATE=$(date +%F)
    awk -v version="$NEW_VERSION" -v release_date="$RELEASE_DATE" '
        BEGIN { inserted = 0 }
        {
            print
            if (!inserted && $0 == "## [Unreleased]") {
                print ""
                print "## [" version "] - " release_date
                print ""
                print "### Changed"
                print "- Release preparation for v" version "."
                inserted = 1
            }
        }
    ' CHANGELOG.md > CHANGELOG.md.tmp
    mv CHANGELOG.md.tmp CHANGELOG.md
    echo "  ✅ Added CHANGELOG.md release stub for v$NEW_VERSION"
fi
echo ""

echo "📝 Step 3/6: Updating Cargo.lock"
cargo check --quiet 2>/dev/null || true
echo "  ✅ Cargo.lock updated"
echo ""

echo "📝 Step 4/6: Committing version bump"
git add Cargo.toml Cargo.lock README.md SECURITY.md docs/ARCHITECTURE.md docs/DESIGN_DECISIONS.md PKGBUILD CHANGELOG.md
git commit -m "Bump version to $NEW_VERSION"
echo "  ✅ Changes committed"
echo ""

echo "📝 Step 5/6: Creating git tag v$NEW_VERSION"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
echo "  ✅ Tag created"
echo ""

echo "📝 Step 6/6: Reminder to push to GitHub"
echo "  ⚠️  Don't forget to push!"
echo ""

echo "✨ Release v$NEW_VERSION ready!"
echo ""
echo "📤 To push to GitHub, run:"
echo "   git push origin main"
echo "   git push origin v$NEW_VERSION"
echo ""
echo "Or in one command:"
echo "   git push origin main && git push origin v$NEW_VERSION"
