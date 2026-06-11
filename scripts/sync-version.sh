#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
COMMIT_AUR=false

for arg in "$@"; do
  case "$arg" in
    --commit-aur) COMMIT_AUR=true ;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION_TXT="$ROOT/VERSION.txt"

if [ -z "$VERSION" ]; then
  VERSION="$(cat "$VERSION_TXT" | tr -d '[:space:]')"
fi

echo "=== Syncing version: $VERSION ==="

# PKGBUILD files (local copy + AUR git repo)
for f in "$ROOT/aur_build/PKGBUILD" "$ROOT/aur_build/codecontext-ai/PKGBUILD"; do
  if [ -f "$f" ]; then
    sed -i "s/^pkgver=.*/pkgver=$VERSION/" "$f"
    echo "  Updated: $f"
  fi
done

# .SRCINFO files
for f in "$ROOT/aur_build/.SRCINFO" "$ROOT/aur_build/codecontext-ai/.SRCINFO"; do
  if [ -f "$f" ]; then
    sed -i "s/^\([[:space:]]*\)pkgver = .*/\1pkgver = $VERSION/" "$f"
    sed -i "s|\(tags/\)v[^/]*\.tar\.gz|\1v$VERSION.tar.gz|g" "$f"
    sed -i "s/\(codecontext-ai-\)[0-9]*\.[0-9]*\.[0-9]*\(\.tar\.gz\)/\1$VERSION\2/g" "$f"
    echo "  Updated: $f"
  fi
done

echo ""
echo "=== Done. Version $VERSION synced to all build files ==="

if $COMMIT_AUR; then
  cd "$ROOT/aur_build/codecontext-ai"
  git add PKGBUILD .SRCINFO
  git commit -m "update to v$VERSION"
  git push
  cd "$ROOT"
  echo "  Pushed to AUR"
fi
