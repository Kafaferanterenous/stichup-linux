#!/bin/bash
set -e
APP=StichUp
ARCH=x86_64
RELEASE_DIR="target/AppImage"
mkdir -p "$RELEASE_DIR"
cargo build --release
APPDIR="/tmp/AppDir-stichup-$RANDOM"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/256x256/apps"
cp target/release/stichup "$APPDIR/usr/bin/stichup"
cp packaging/stichup.desktop "$APPDIR/usr/share/applications/stichup.desktop"
test -f packaging/stichup.png && cp packaging/stichup.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/stichup.png"
ln -sf usr/bin/stichup "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"
LINUXDEPLOY="$(command -v linuxdeploy 2>/dev/null || echo packaging/linuxdeploy.AppImage)"
if [ ! -x "$LINUXDEPLOY" ]; then
  echo "linuxdeploy missing" >&2
  exit 1
fi
"$LINUXDEPLOY" --appdir "$APPDIR" --output appimage 2>/dev/null || "$LINUXDEPLOY" --appdir "$APPDIR" --output appimage
mv "${APP}-${ARCH}.AppImage" "$RELEASE_DIR/" 2>/dev/null || mv *.AppImage "$RELEASE_DIR/" 2>/dev/null || true
echo "AppImage in $RELEASE_DIR"
