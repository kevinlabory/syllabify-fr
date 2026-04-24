// SPDX-License-Identifier: GPL-3.0-or-later
//! Smoke tests for the WASM bindings. Run with `wasm-pack test --node wasm/`.

use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn syllables_chocolat() {
    assert_eq!(
        syllabify_fr_wasm::syllables("chocolat"),
        vec!["cho", "co", "lat"]
    );
}

#[wasm_bindgen_test]
fn syllables_famille() {
    assert_eq!(
        syllabify_fr_wasm::syllables("famille"),
        vec!["fa", "mi", "lle"]
    );
}

#[wasm_bindgen_test]
fn syllables_parlent() {
    assert_eq!(syllabify_fr_wasm::syllables("parlent"), vec!["par", "lent"]);
}

#[wasm_bindgen_test]
fn render_html_produit_spans_et_liaison() {
    let html = syllabify_fr_wasm::render_html("les hôtels");
    assert!(html.contains(r#"class="syl syl-a""#), "got: {}", html);
    assert!(
        html.contains(r#"class="liaison" data-with="z""#),
        "got: {}",
        html
    );
}

#[wasm_bindgen_test]
fn render_word_html_chocolat() {
    let html = syllabify_fr_wasm::render_word_html("chocolat");
    assert_eq!(
        html,
        r#"<span class="word"><span class="syl syl-a">cho</span><span class="syl syl-b">co</span><span class="syl syl-a">lat</span></span>"#
    );
}

#[wasm_bindgen_test]
fn syllabify_text_handles_homographs() {
    use js_sys::{Array, Reflect};
    use wasm_bindgen::JsValue;

    let result: Array = syllabify_fr_wasm::syllabify_text_js("le couvent accueille");
    let mut serialized = Vec::new();
    for i in 0..result.length() {
        let obj = result.get(i);
        let kind = Reflect::get(&obj, &JsValue::from_str("kind"))
            .unwrap()
            .as_string()
            .unwrap();
        if kind == "word" {
            let syls: Array = Reflect::get(&obj, &JsValue::from_str("syllables"))
                .unwrap()
                .into();
            let joined: Vec<String> = (0..syls.length())
                .map(|i| syls.get(i).as_string().unwrap())
                .collect();
            serialized.push(joined.join("-"));
        } else {
            let raw = Reflect::get(&obj, &JsValue::from_str("text"))
                .unwrap()
                .as_string()
                .unwrap();
            serialized.push(format!("[{}]", raw));
        }
    }
    // "couvent" after "le" is the noun form, -ent pronounced (a~)
    assert!(
        serialized.iter().any(|s| s == "cou-vent"),
        "got: {:?}",
        serialized
    );
}
