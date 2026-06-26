// SPDX-License-Identifier: GPL-3.0-or-later
//
// Smoke tests for the Swift wrappers. Run with `swift test` on macOS after
// `swift/scripts/build-xcframework.sh` has produced the binary target.

import XCTest
@testable import SyllabifyFr

final class SyllabifyFrTests: XCTestCase {

    func testSyllablesChocolat() {
        XCTAssertEqual(SyllabifyFr.syllables("chocolat"), ["cho", "co", "lat"])
    }

    func testSyllablesFamille() {
        XCTAssertEqual(SyllabifyFr.syllables("famille"), ["fa", "mi", "lle"])
    }

    func testSyllablesEmpty() {
        XCTAssertEqual(SyllabifyFr.syllables(""), [])
    }

    func testSyllabifyTextHomograph() {
        // "les couvent" : `couvent` désambigué par `les` → nom (lieu)
        let chunks = SyllabifyFr.syllabifyText("les couvent")
        XCTAssertEqual(chunks.count, 3)
        if case .word(let s) = chunks[0] { XCTAssertEqual(s, ["les"]) }
        if case .raw(let t) = chunks[1] { XCTAssertEqual(t, " ") }
        if case .word(let s) = chunks[2] { XCTAssertEqual(s.count, 2) }
    }

    func testPhonemesChat() {
        let p = SyllabifyFr.phonemes("chat")
        XCTAssertFalse(p.isEmpty)
        let letters = p.map(\.letters).joined()
        XCTAssertEqual(letters, "chat")
    }

    func testRenderWordHtml() {
        let html = SyllabifyFr.renderWordHtml("chat")
        XCTAssertTrue(html.contains("<span"))
        XCTAssertTrue(html.contains("syl-a"))
    }

    func testHighlightBdpq() {
        let html = SyllabifyFr.highlightLetters("dépit", preset: .bdpq)
        XCTAssertTrue(html.contains("color:"))
    }

    /// Audit #3 — pas de caractère de contrôle brut dans la sortie JSON.
    func testSyllabifyTextEscapesControlChars() {
        let chunks = SyllabifyFr.syllabifyText("a\u{0001}b")
        // Doit produire au moins un chunk word et un chunk raw.
        XCTAssertGreaterThanOrEqual(chunks.count, 2)
    }
}
