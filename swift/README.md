# syllabify-fr-swift

Swift Package for [`syllabify-fr`](https://github.com/kevinlabory/syllabify-fr) — French syllabification for iOS apps. Idiomatic Swift wrappers over the C ABI exported by `syllabify-fr-ffi`.

## Licence

**GPL v3 or later.** Derived from LireCouleur 6 (Marie-Pierre & Luc Brungard, GPL v3). Any application distributing this library must comply with GPL v3 terms.

## One-time setup (macOS)

```bash
# Xcode CLI tools (provides xcodebuild + lipo)
xcode-select --install

# Rust toolchains: iOS (product) + macOS (host-side `swift test`)
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios \
                  aarch64-apple-darwin x86_64-apple-darwin
```

## Build the XCFramework

```bash
bash swift/scripts/build-xcframework.sh
```

This produces `swift/XCFramework/SyllabifyFr.xcframework`, bundling :

| Slice | Use |
|---|---|
| `aarch64-apple-ios` | Physical iPhone / iPad (arm64 device) |
| `aarch64-apple-ios-sim` + `x86_64-apple-ios` (lipo) | iOS Simulator (Apple Silicon + Intel) |
| macOS universal (arm64 + x86_64) | **Dev-only** : lets `swift test` run on the Mac host — the shipped product stays iOS-only |

The XCFramework is **not committed** to the repo (`.gitignore`) — it is a build artifact rebuilt per release.

## Use from your Xcode project

1. In Xcode: **File → Add Package Dependencies… → Add Local…** and select the `swift/` directory.
2. Add `SyllabifyFr` to your target's *Frameworks, Libraries, and Embedded Content*.
3. `import SyllabifyFr` in your Swift code.

## Usage

```swift
import SyllabifyFr

// Single word → syllables
SyllabifyFr.syllables("chocolat")
// → ["cho", "co", "lat"]

// Full text with homograph disambiguation
let chunks = SyllabifyFr.syllabifyText("le couvent accueille les moines")
for chunk in chunks {
    switch chunk {
    case .word(let syllables):
        print("word:", syllables.joined(separator: "-"))
    case .raw(let text):
        print("raw:", text)
    }
}

// Phonemes
let phons = SyllabifyFr.phonemes("hier")
for p in phons {
    print("\(p.code) ← \(p.letters)")
}
// → j ← i
//   e^_comp ← e
//   r ← r

// HTML rendering
let html = SyllabifyFr.renderHtml("les hôtels")
// → <span class="word">…</span> <span class="liaison" data-with="z"></span>…

// Highlight confusable letters (b/d/p/q etc.)
let highlighted = SyllabifyFr.highlightLetters("dépit", preset: .bdpq)
// → "<span style=\"color:#1e8e3e\">d</span>é<span style=\"color:#d93025\">p</span>it"
```

## API

| Swift | Returns | Notes |
|---|---|---|
| `SyllabifyFr.syllables(_:)` | `[String]` | Empty array on empty input |
| `SyllabifyFr.syllabifyText(_:)` | `[TextChunk]` | Enum with `.word(syllables:)` / `.raw(text:)` |
| `SyllabifyFr.phonemes(_:)` | `[Phoneme]` | `(code, letters)` pairs |
| `SyllabifyFr.renderWordHtml(_:)` | `String` | Syllable spans only |
| `SyllabifyFr.renderHtml(_:)` | `String` | Spans + liaison markers |
| `SyllabifyFr.highlightLetters(_:preset:mode:)` | `String` | `mode` defaults to `.inline` |

All entry points are pure functions; no shared state, safe to call from any thread.

## Testing

The XCFramework must be built first (see above), since it bundles the macOS
slice that hosts the tests.

```bash
# On the Mac host (macOS slice) — fastest, validates the wrapper logic:
cd swift
swift test

# On an iOS Simulator (validates the actual shipping slices):
xcodebuild test -scheme SyllabifyFr \
  -destination 'platform=iOS Simulator,name=iPhone 16'
# list your simulators with: xcrun simctl list devices available
```

`swift test` runs against the macOS slice (host), which fully exercises the
platform-independent wrapper logic (JSON parsing, memory management, enum
mapping). The `xcodebuild ... -destination 'iOS Simulator'` path validates the
exact slices the iOS app links against.

## Distribution

This package is **local-only** for the time being. Reference it from another project with:

```swift
.package(path: "/path/to/syllabify-fr/swift")
```

If you publish it later (CocoaPods / SwiftPM registry / GitHub Releases), document the source-of-truth here.
