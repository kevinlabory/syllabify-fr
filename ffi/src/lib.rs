use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use syllabify_fr::{phonemes, render_html, render_word_html, syllabify_text, syllables, TextChunk};

// --- JSON helpers (no serde dependency) ---

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

fn json_syllables_array(syls: &[String]) -> String {
    let items: Vec<String> = syls.iter().map(|s| json_str(s)).collect();
    format!("[{}]", items.join(","))
}

fn json_chunk(chunk: &TextChunk) -> String {
    match chunk {
        TextChunk::Word(syls) => {
            format!(
                "{{\"kind\":\"word\",\"syllables\":{}}}",
                json_syllables_array(syls)
            )
        }
        TextChunk::Raw(text) => {
            format!("{{\"kind\":\"raw\",\"text\":{}}}", json_str(text))
        }
        _ => "{\"kind\":\"unknown\"}".to_string(),
    }
}

fn json_chunks(chunks: &[TextChunk]) -> String {
    let items: Vec<String> = chunks.iter().map(json_chunk).collect();
    format!("[{}]", items.join(","))
}

fn json_phonemes(pairs: &[(String, String)]) -> String {
    let items: Vec<String> = pairs
        .iter()
        .map(|(code, letters)| format!("[{},{}]", json_str(code), json_str(letters)))
        .collect();
    format!("[{}]", items.join(","))
}

// --- Helpers for safe string conversion ---

unsafe fn c_str_to_rust<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

fn rust_string_to_c(s: String) -> *mut c_char {
    CString::new(s)
        .map(|cs| cs.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

// --- Public C API ---

/// Free a string previously returned by a `syllabify_*` function.
/// Passing NULL is a no-op.
#[no_mangle]
pub unsafe extern "C" fn syllabify_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Syllabify a word, returning syllables joined by hyphens.
///
/// Example: `"chocolat"` → `"cho-co-lat"`
///
/// Returns NULL on NULL input or invalid UTF-8.
/// The caller must free the result with `syllabify_free()`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_syllables(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(syllables(word).join("-"))
}

/// Syllabify full text, returning a JSON array of chunk objects.
///
/// Each chunk is either `{"kind":"word","syllables":["cho","co","lat"]}` or
/// `{"kind":"raw","text":" "}`.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_text_json(text: *const c_char) -> *mut c_char {
    let text = match c_str_to_rust(text) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(json_chunks(&syllabify_text(text)))
}

/// Get phonemes for a word as a JSON array of `[code, letters]` pairs.
///
/// Example: `"chat"` → `[["c","ch"],["V","a"],["c","t"]]`
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_phonemes(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(json_phonemes(&phonemes(word)))
}

/// Render HTML for a single word with `<span class="syl-a/b">` syllable spans.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_render_word_html(word: *const c_char) -> *mut c_char {
    let word = match c_str_to_rust(word) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(render_word_html(word))
}

/// Render HTML for full text with syllable spans and liaison markers.
///
/// Returns NULL on NULL input.
/// The caller must free the result with `syllabify_free()`.
#[no_mangle]
pub unsafe extern "C" fn syllabify_render_html(text: *const c_char) -> *mut c_char {
    let text = match c_str_to_rust(text) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    rust_string_to_c(render_html(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn round_trip(
        f: unsafe extern "C" fn(*const c_char) -> *mut c_char,
        input: &str,
    ) -> String {
        let cstr = CString::new(input).unwrap();
        let ptr = f(cstr.as_ptr());
        assert!(!ptr.is_null(), "unexpected NULL for input {:?}", input);
        let result = CStr::from_ptr(ptr).to_str().unwrap().to_owned();
        syllabify_free(ptr);
        result
    }

    #[test]
    fn test_syllabify_syllables_chocolat() {
        let result = unsafe { round_trip(syllabify_syllables, "chocolat") };
        assert_eq!(result, "cho-co-lat");
    }

    #[test]
    fn test_syllabify_syllables_famille() {
        let result = unsafe { round_trip(syllabify_syllables, "famille") };
        assert_eq!(result, "fa-mi-lle");
    }

    #[test]
    fn test_syllabify_syllables_null_returns_null() {
        let ptr = unsafe { syllabify_syllables(std::ptr::null()) };
        assert!(ptr.is_null());
    }

    #[test]
    fn test_syllabify_text_json_word_chunk() {
        let result = unsafe { round_trip(syllabify_text_json, "le chat dort") };
        assert!(result.starts_with('['), "expected JSON array: {result}");
        assert!(
            result.contains("\"kind\":\"word\""),
            "missing word chunk: {result}"
        );
        assert!(
            result.contains("\"syllables\""),
            "missing syllables: {result}"
        );
    }

    #[test]
    fn test_syllabify_text_json_raw_chunk() {
        let result = unsafe { round_trip(syllabify_text_json, "bonjour !") };
        assert!(
            result.contains("\"kind\":\"raw\""),
            "missing raw chunk: {result}"
        );
    }

    #[test]
    fn test_syllabify_phonemes_json() {
        let result = unsafe { round_trip(syllabify_phonemes, "chat") };
        assert!(result.starts_with('['), "expected JSON array: {result}");
        assert!(result.contains("[["), "expected nested arrays: {result}");
    }

    #[test]
    fn test_syllabify_render_word_html_has_spans() {
        let result = unsafe { round_trip(syllabify_render_word_html, "chocolat") };
        assert!(
            result.contains("syl-a") || result.contains("syl-b"),
            "missing syllable spans: {result}"
        );
    }

    #[test]
    fn test_syllabify_render_html_has_spans() {
        let result = unsafe { round_trip(syllabify_render_html, "les hôtels") };
        assert!(
            result.contains("syl-a") || result.contains("syl-b"),
            "missing syllable spans: {result}"
        );
    }

    #[test]
    fn test_syllabify_free_null_is_noop() {
        // Must not crash or panic
        unsafe { syllabify_free(std::ptr::null_mut()) };
    }
}
