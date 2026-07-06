#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Build a multi-slice XCFramework from `syllabify-fr-ffi`.
#
# Produced slices:
#   - aarch64-apple-ios       (device, arm64)
#   - aarch64-apple-ios-sim   (simulator on Apple Silicon Mac)
#   - x86_64-apple-ios        (simulator on Intel Mac)
#   - macOS universal         (arm64 + x86_64) — DEV-ONLY : permet à
#                             `swift test` de s'exécuter sur l'hôte macOS.
#                             Le produit reste iOS-only ; l'app iOS linke
#                             ses slices iOS, jamais celle-ci.
#
# Output: swift/XCFramework/SyllabifyFr.xcframework
#
# Requirements (macOS):
#   xcode-select --install
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios \
#                     aarch64-apple-darwin x86_64-apple-darwin
#
# Usage:
#   bash swift/scripts/build-xcframework.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SWIFT_DIR="$ROOT_DIR/swift"
BUILD_DIR="$SWIFT_DIR/.build/xcframework"
OUT_DIR="$SWIFT_DIR/XCFramework"
HEADER="$ROOT_DIR/ffi/include/syllabify_fr.h"
LIB_NAME="libsyllabify_fr_ffi.a"
FRAMEWORK_NAME="SyllabifyFr"

if [[ "$(uname)" != "Darwin" ]]; then
  echo "error: this script must be run on macOS (requires xcodebuild + lipo)." >&2
  exit 1
fi

if [[ ! -f "$HEADER" ]]; then
  echo "error: header not found at $HEADER" >&2
  exit 1
fi

echo "=== cross-compile ffi crate for 3 iOS slices + 2 macOS (dev test) ==="
cd "$ROOT_DIR"
cargo build --release -p syllabify-fr-ffi --target aarch64-apple-ios
cargo build --release -p syllabify-fr-ffi --target aarch64-apple-ios-sim
cargo build --release -p syllabify-fr-ffi --target x86_64-apple-ios
# macOS slices — uniquement pour permettre `swift test` sur l'hôte.
cargo build --release -p syllabify-fr-ffi --target aarch64-apple-darwin
cargo build --release -p syllabify-fr-ffi --target x86_64-apple-darwin

echo "=== prepare headers + modulemap directory ==="
HEADERS_DIR="$BUILD_DIR/Headers"
mkdir -p "$HEADERS_DIR"
cp "$HEADER" "$HEADERS_DIR/"

cat > "$HEADERS_DIR/module.modulemap" <<'EOF'
module CSyllabifyFr {
  header "syllabify_fr.h"
  link "syllabify_fr_ffi"
  export *
}
EOF

echo "=== combine the two simulator slices via lipo ==="
SIM_DIR="$BUILD_DIR/sim-universal"
mkdir -p "$SIM_DIR"
lipo -create \
  "$ROOT_DIR/target/aarch64-apple-ios-sim/release/$LIB_NAME" \
  "$ROOT_DIR/target/x86_64-apple-ios/release/$LIB_NAME" \
  -output "$SIM_DIR/$LIB_NAME"

echo "=== combine the two macOS slices via lipo (dev test host) ==="
MAC_DIR="$BUILD_DIR/macos-universal"
mkdir -p "$MAC_DIR"
lipo -create \
  "$ROOT_DIR/target/aarch64-apple-darwin/release/$LIB_NAME" \
  "$ROOT_DIR/target/x86_64-apple-darwin/release/$LIB_NAME" \
  -output "$MAC_DIR/$LIB_NAME"

echo "=== assemble XCFramework ==="
rm -rf "$OUT_DIR/$FRAMEWORK_NAME.xcframework"
mkdir -p "$OUT_DIR"
xcodebuild -create-xcframework \
  -library "$ROOT_DIR/target/aarch64-apple-ios/release/$LIB_NAME" \
  -headers "$HEADERS_DIR" \
  -library "$SIM_DIR/$LIB_NAME" \
  -headers "$HEADERS_DIR" \
  -library "$MAC_DIR/$LIB_NAME" \
  -headers "$HEADERS_DIR" \
  -output "$OUT_DIR/$FRAMEWORK_NAME.xcframework"

echo
echo "✓ XCFramework produced at $OUT_DIR/$FRAMEWORK_NAME.xcframework"
echo "  Use it in Xcode by adding $SWIFT_DIR as a local Swift Package."
