// SPDX-License-Identifier: GPL-3.0-or-later
//! WebAssembly bindings for syllabify-fr.
//!
//! Exposes:
//! - `syllables(word)` → `string[]`
//! - `syllabifyText(text)` → `Array<{kind:"word",syllables:string[]} | {kind:"raw",text:string}>`
//! - `phonemes(word)` → `Array<[string, string]>` (code, letters)
//! - `renderHtml(text)` → `string` (spans syllabiques + liaisons)
//! - `renderWordHtml(word)` → `string`
//! - `highlightLetters(word, preset, mode?)` → `string` (HTML, confusions de lettres)

use js_sys::{Array, Object, Reflect};
use syllabify_fr::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use syllabify_fr::{
    render_html as core_render_html, render_word_html as core_render_word_html, syllabify_text,
    syllables as core_syllables, TextChunk,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn syllables(word: &str) -> Vec<String> {
    core_syllables(word)
}

#[wasm_bindgen(js_name = syllabifyText)]
pub fn syllabify_text_js(text: &str) -> Array {
    let out = Array::new();
    for chunk in syllabify_text(text) {
        let obj = Object::new();
        match chunk {
            TextChunk::Word(syls) => {
                let arr = Array::new();
                for s in syls {
                    arr.push(&JsValue::from_str(&s));
                }
                let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("word"));
                let _ = Reflect::set(&obj, &JsValue::from_str("syllables"), &arr);
            }
            TextChunk::Raw(raw) => {
                let _ = Reflect::set(&obj, &JsValue::from_str("kind"), &JsValue::from_str("raw"));
                let _ = Reflect::set(&obj, &JsValue::from_str("text"), &JsValue::from_str(&raw));
            }
            _ => {
                let _ = Reflect::set(
                    &obj,
                    &JsValue::from_str("kind"),
                    &JsValue::from_str("unknown"),
                );
            }
        }
        out.push(&obj);
    }
    out
}

#[wasm_bindgen]
pub fn phonemes(word: &str) -> Array {
    let out = Array::new();
    for (code, letters) in syllabify_fr::phonemes(word) {
        let pair = Array::new();
        pair.push(&JsValue::from_str(&code));
        pair.push(&JsValue::from_str(&letters));
        out.push(&pair);
    }
    out
}

#[wasm_bindgen(js_name = renderHtml)]
pub fn render_html(text: &str) -> String {
    core_render_html(text)
}

#[wasm_bindgen(js_name = renderWordHtml)]
pub fn render_word_html(word: &str) -> String {
    core_render_word_html(word)
}

fn preset_rules(name: &str) -> Option<Vec<LetterRule>> {
    match name {
        "bdpq" => Some(presets::bdpq()),
        "mnu" => Some(presets::mnu()),
        "pir-pri" | "pir_pri" => Some(presets::pir_pri()),
        _ => None,
    }
}

fn parse_mode(mode: Option<String>) -> RenderMode {
    match mode.as_deref() {
        Some("classes") => RenderMode::Classes,
        _ => RenderMode::Inline,
    }
}

/// Highlight confusable letters in `word` using a named preset.
///
/// `preset` accepts `"bdpq"`, `"mnu"`, or `"pir-pri"`. `mode` accepts
/// `"inline"` (default) or `"classes"`. Returns HTML with `<span>` wrappers
/// around the targeted letters; on unknown preset the input word is returned
/// unchanged (HTML-escaped, no spans).
#[wasm_bindgen(js_name = highlightLetters)]
pub fn highlight_letters(word: &str, preset: &str, mode: Option<String>) -> String {
    let rules = preset_rules(preset).unwrap_or_default();
    let spans = match_letters(word, &rules);
    render_letters_html(word, &spans, &rules, parse_mode(mode))
}
