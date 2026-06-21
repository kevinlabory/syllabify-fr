// SPDX-License-Identifier: GPL-3.0-or-later
//
// Idiomatic Swift wrappers over the C ABI exported by `syllabify-fr-ffi`.
// Designed for app consumption: every function returns native Swift types
// (`[String]`, `enum TextChunk`, `[(code: String, letters: String)]`) and
// hides the manual memory management of the underlying C strings.

import CSyllabifyFr
import Foundation

/// A chunk produced by `SyllabifyFr.syllabifyText`: either a word with its
/// syllables, or raw text (whitespace/punctuation) between words.
public enum TextChunk: Equatable {
    case word(syllables: [String])
    case raw(text: String)
}

/// A `(phonetic code, source letters)` pair as produced by
/// `SyllabifyFr.phonemes`.
public struct Phoneme: Equatable {
    public let code: String
    public let letters: String

    public init(code: String, letters: String) {
        self.code = code
        self.letters = letters
    }
}

/// Highlight preset for `SyllabifyFr.highlightLetters`. The string values match
/// the names accepted by the underlying C ABI.
public enum HighlightPreset: String {
    case bdpq = "bdpq"
    case mnu = "mnu"
    case pirPri = "pir-pri"
}

/// Rendering mode for HTML output.
public enum RenderMode: String {
    case inline = "inline"
    case classes = "classes"
}

/// Static façade — all entry points are pure functions.
public enum SyllabifyFr {

    // MARK: - Syllabes

    /// Segment a single word into its syllables.
    ///
    /// - Parameter word: a French word, e.g. `"chocolat"`.
    /// - Returns: the syllables in order, e.g. `["cho", "co", "lat"]`.
    ///            Empty array if the input is empty.
    public static func syllables(_ word: String) -> [String] {
        guard let cstr = callReturningString({ syllabify_syllables($0) }, word) else {
            return []
        }
        if cstr.isEmpty { return [] }
        return cstr.components(separatedBy: "-")
    }

    /// Syllabify a full text with homograph disambiguation.
    ///
    /// - Parameter text: arbitrary French text.
    /// - Returns: an ordered list of chunks; `.word(syllables:)` for actual
    ///            words and `.raw(text:)` for whitespace/punctuation between
    ///            them. Empty array if input is empty or parsing fails.
    public static func syllabifyText(_ text: String) -> [TextChunk] {
        guard let json = callReturningString({ syllabify_text_json($0) }, text),
              let data = json.data(using: .utf8),
              let raw = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]]
        else {
            return []
        }
        return raw.compactMap { dict in
            switch dict["kind"] as? String {
            case "word":
                let syls = (dict["syllables"] as? [String]) ?? []
                return .word(syllables: syls)
            case "raw":
                let text = (dict["text"] as? String) ?? ""
                return .raw(text: text)
            default:
                return nil
            }
        }
    }

    /// Phonemes for a single word.
    ///
    /// - Parameter word: a French word.
    /// - Returns: list of `(code, letters)` pairs. The concatenation of
    ///            `letters` reproduces the cleaned word.
    public static func phonemes(_ word: String) -> [Phoneme] {
        guard let json = callReturningString({ syllabify_phonemes($0) }, word),
              let data = json.data(using: .utf8),
              let raw = try? JSONSerialization.jsonObject(with: data) as? [[String]]
        else {
            return []
        }
        return raw.compactMap { pair in
            guard pair.count == 2 else { return nil }
            return Phoneme(code: pair[0], letters: pair[1])
        }
    }

    // MARK: - HTML

    /// HTML for a single word with `<span class="syl syl-a/b">…</span>`
    /// alternating syllable spans.
    public static func renderWordHtml(_ word: String) -> String {
        return callReturningString({ syllabify_render_word_html($0) }, word) ?? ""
    }

    /// HTML for full text: syllable spans + `<span class="liaison">` inter-word
    /// liaison markers.
    public static func renderHtml(_ text: String) -> String {
        return callReturningString({ syllabify_render_html($0) }, text) ?? ""
    }

    /// Highlight confusable letters in `word` using a named preset.
    /// On unknown preset the word is returned HTML-escaped without spans
    /// (matches the C ABI contract).
    public static func highlightLetters(
        _ word: String,
        preset: HighlightPreset,
        mode: RenderMode = .inline
    ) -> String {
        return callReturningStringThreeArgs(
            { syllabify_highlight_letters($0, $1, $2) },
            word,
            preset.rawValue,
            mode.rawValue
        ) ?? ""
    }

    // MARK: - Internal: bridging helpers

    /// Common wrapper: call a 1-arg C fn returning `char*`, convert to a Swift
    /// String, free the C buffer. Returns nil if the C call returned NULL.
    private static func callReturningString(
        _ fn: (UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?,
        _ arg: String
    ) -> String? {
        return arg.withCString { ptr in
            guard let raw = fn(ptr) else { return nil }
            defer { syllabify_free(raw) }
            return String(cString: raw)
        }
    }

    /// Variant for a 3-arg C fn (used by `highlight_letters`).
    private static func callReturningStringThreeArgs(
        _ fn: (UnsafePointer<CChar>?, UnsafePointer<CChar>?, UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?,
        _ a: String,
        _ b: String,
        _ c: String
    ) -> String? {
        return a.withCString { pa in
            b.withCString { pb in
                c.withCString { pc in
                    guard let raw = fn(pa, pb, pc) else { return nil }
                    defer { syllabify_free(raw) }
                    return String(cString: raw)
                }
            }
        }
    }
}
