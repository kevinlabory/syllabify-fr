// SPDX-License-Identifier: GPL-3.0-or-later
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let html = syllabify_fr::render_html(s);
        // Invariant XSS : pas de balise <script> non échappée dans la sortie.
        // Si le fuzzer trouve un input qui produit "<script>", on a un bug
        // d'échappement.
        debug_assert!(
            !html.contains("<script>"),
            "render_html a produit une <script> brute pour input {:?}",
            s
        );
    }
});
