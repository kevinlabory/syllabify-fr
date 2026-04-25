# syllabify-fr-ffi

C FFI bindings for [syllabify-fr](https://github.com/kevinlabory/syllabify-fr) — a Rust port of
[LireCouleur 6](https://lirecouleur.forge.apps.education.fr/), a French syllabification library
for reading instruction.

## Build

```bash
# Shared library (.so / .dylib / .dll) + static library (.a / .lib)
cargo build --release -p syllabify-fr-ffi

# Artifacts
# target/release/libsyllabify_fr_ffi.so   (Linux)
# target/release/libsyllabify_fr_ffi.a
```

## Header

A pre-generated C header is available at `include/syllabify_fr.h`.

Regenerate from source (requires [cbindgen](https://github.com/mozilla/cbindgen)):

```bash
cbindgen --config ffi/cbindgen.toml --crate syllabify-fr-ffi \
         --output ffi/include/syllabify_fr.h
```

## API

```c
#include "syllabify_fr.h"

// All returned strings must be freed with syllabify_free().
// NULL input → NULL output (safe).

char *syllabify_syllables(const char *word);     // "chocolat" → "cho-co-lat"
char *syllabify_text_json(const char *text);     // JSON [{kind,syllables/text}]
char *syllabify_phonemes(const char *word);      // JSON [[code,letters],…]
char *syllabify_render_word_html(const char *word); // HTML <span> syllables
char *syllabify_render_html(const char *text);   // HTML full text + liaisons
void  syllabify_free(char *ptr);                 // release memory (NULL-safe)
```

## Example (C)

```c
#include <stdio.h>
#include "syllabify_fr.h"

int main(void) {
    char *result = syllabify_syllables("chocolat");
    printf("%s\n", result);  // cho-co-lat
    syllabify_free(result);
    return 0;
}
```

Compile:

```bash
gcc example.c -L../../target/release -lsyllabify_fr_ffi \
    -I../../ffi/include -o example
```

## Language bindings using this FFI

- **Python** (ctypes): load `libsyllabify_fr_ffi.so` via `ctypes.CDLL`
- **Java** (JNI / JNA): use `com.sun.jna.Library` or JNI
- **Swift**: import with `@_silgen_name` or a bridging header

For an idiomatic Python interface, prefer the native
[syllabify-fr-py](../py/README.md) package (PyO3).

## License

GPL-3.0-or-later — derived from [LireCouleur 6](https://lirecouleur.forge.apps.education.fr/)
by Marie-Pierre & Luc Brungard.
