# syllabify-fr-jni

JNI bindings for [syllabify-fr](https://crates.io/crates/syllabify-fr) — French syllabification for Java, Kotlin, and Android.

## Build

```bash
# Shared library (.so / .dylib / .dll)
cargo build --release -p syllabify-fr-jni

# Output:
#   Linux   → target/release/libsyllabify_fr_jni.so
#   macOS   → target/release/libsyllabify_fr_jni.dylib
#   Windows → target/release/syllabify_fr_jni.dll
```

## Usage

### 1. Add the native library to your project

Copy the compiled library to a location on `java.library.path`, then load it:

```java
System.loadLibrary("syllabify_fr_jni");
// or with an explicit path:
System.load("/path/to/libsyllabify_fr_jni.so");
```

### 2. Copy `SyllabifyFr.java`

Copy `java/com/dyscolor/syllabify/SyllabifyFr.java` into your project (keep the package structure).

### 3. Call the API

```java
import com.dyscolor.syllabify.SyllabifyFr;

// Syllabify a word
String[] syls = SyllabifyFr.syllables("chocolat");
// → ["cho", "co", "lat"]

// Syllabify full text (JSON output)
String json = SyllabifyFr.syllabifyText("le petit chat");
// → [{"kind":"word","syllables":["le"]},{"kind":"raw","text":" "},...]

// Phonemes (JSON output)
String phonemes = SyllabifyFr.phonemes("chat");
// → [["s^","ch"],["a","a"],["#","t"]]

// HTML rendering
String html = SyllabifyFr.renderWordHtml("chocolat");
// → <span class="word"><span class="syl syl-a">cho</span>...</span>

String fullHtml = SyllabifyFr.renderHtml("les hôtels");
// → HTML with syllable spans and liaison markers
```

## Android

For Android, place the `.so` for each ABI under `app/src/main/jniLibs/<ABI>/`:

```
app/src/main/jniLibs/
  arm64-v8a/libsyllabify_fr_jni.so
  armeabi-v7a/libsyllabify_fr_jni.so
  x86_64/libsyllabify_fr_jni.so
```

Cross-compile with [cross](https://github.com/cross-rs/cross):

```bash
cross build --release -p syllabify-fr-jni --target aarch64-linux-android
cross build --release -p syllabify-fr-jni --target armv7-linux-androideabi
cross build --release -p syllabify-fr-jni --target x86_64-linux-android
```

## API reference

| Method | Input | Output |
|---|---|---|
| `syllables(String)` | word | `String[]` — syllables |
| `syllabifyText(String)` | text | JSON `String` — array of chunks |
| `phonemes(String)` | word | JSON `String` — array of `[code, letters]` pairs |
| `renderWordHtml(String)` | word | HTML `String` |
| `renderHtml(String)` | text | HTML `String` |

All methods are thread-safe. `syllabifyText` and `phonemes` return JSON strings; parse them with your preferred library (Gson, Jackson, Moshi, etc.).

## Licence

GPL-3.0-or-later — see [LICENSE](../LICENSE).
