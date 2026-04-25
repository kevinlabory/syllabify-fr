// SPDX-License-Identifier: GPL-3.0-or-later
package com.dyscolor.syllabify;

/**
 * JNI wrapper for the syllabify-fr Rust library.
 *
 * <p>Load the native library once at class initialisation:
 * <pre>{@code
 * System.loadLibrary("syllabify_fr_jni");
 * // or with an explicit path:
 * System.load("/path/to/libsyllabify_fr_jni.so");
 * }</pre>
 *
 * <p>All methods are thread-safe (the underlying Rust library uses only
 * immutable global state after first initialisation).
 *
 * <h2>Example</h2>
 * <pre>{@code
 * String[] syls = SyllabifyFr.syllables("chocolat");
 * // syls = ["cho", "co", "lat"]
 *
 * String json = SyllabifyFr.syllabifyText("le chat dort");
 * // json = [{"kind":"word","syllables":["le"]},{"kind":"raw","text":" "}, ...]
 * }</pre>
 */
public class SyllabifyFr {

    static {
        System.loadLibrary("syllabify_fr_jni");
    }

    private SyllabifyFr() {}

    /**
     * Syllabifies a single French word.
     *
     * @param word the word to syllabify (case-insensitive)
     * @return array of syllables, e.g. {@code ["cho", "co", "lat"]} for {@code "chocolat"}
     */
    public static native String[] syllables(String word);

    /**
     * Syllabifies a full text, preserving punctuation and spaces.
     *
     * <p>Homographs (e.g. <i>le couvent</i> vs <i>elles couvent</i>) are
     * disambiguated according to the preceding word.
     *
     * @param text the text to process
     * @return JSON array of chunks:
     *         {@code [{"kind":"word","syllables":["cho","co","lat"]}, {"kind":"raw","text":" "}, ...]}
     */
    public static native String syllabifyText(String text);

    /**
     * Returns the phonemes of a single word as a JSON array.
     *
     * @param word the word to analyse
     * @return JSON array of {@code [code, letters]} pairs,
     *         e.g. {@code [["s^","ch"],["a","a"],["#","t"]]} for {@code "chat"}
     */
    public static native String phonemes(String word);

    /**
     * Renders a single word as HTML with alternating syllable spans.
     *
     * @param word the word to render
     * @return HTML string with {@code <span class="syl syl-a/b">} spans
     */
    public static native String renderWordHtml(String word);

    /**
     * Renders full text as HTML with syllable spans and liaison markers.
     *
     * @param text the text to render
     * @return HTML string with syllable spans and {@code <span class="liaison">} markers
     */
    public static native String renderHtml(String text);
}
