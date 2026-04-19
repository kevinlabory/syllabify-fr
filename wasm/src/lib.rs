// SPDX-License-Identifier: GPL-3.0-or-later
//! WebAssembly bindings for syllabify-fr.
//!
//! Exposes three entry points:
//! - `syllables(word)` → `string[]`
//! - `syllabifyText(text)` → `Array<{kind:"word",syllables:string[]} | {kind:"raw",text:string}>`
//! - `phonemes(word)` → `Array<[string, string]>` (code, letters)

use js_sys::{Array, Object, Reflect};
use syllabify_fr::{syllabify_text, syllables as core_syllables, TextChunk};
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
