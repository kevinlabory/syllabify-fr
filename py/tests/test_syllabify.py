"""
Tests Python pour syllabify-fr.

Prérequis : maturin develop -m py/Cargo.toml --features extension-module
"""
import syllabify_fr


def test_syllables_chocolat():
    assert syllabify_fr.syllables("chocolat") == ["cho", "co", "lat"]


def test_syllables_famille():
    assert syllabify_fr.syllables("famille") == ["fa", "mi", "lle"]


def test_syllables_empty():
    # chaîne vide → une syllabe vide (comportement de la bibliothèque core)
    assert syllabify_fr.syllables("") == [""]


def test_syllabify_text_word_chunks():
    chunks = syllabify_fr.syllabify_text("le chat dort")
    words = [c for c in chunks if c["kind"] == "word"]
    assert len(words) >= 3
    assert words[0]["syllables"] == ["le"]


def test_syllabify_text_raw_chunks():
    chunks = syllabify_fr.syllabify_text("bonjour !")
    raws = [c for c in chunks if c["kind"] == "raw"]
    assert any(r["text"].strip() == "!" for r in raws)


def test_syllabify_text_multiword():
    chunks = syllabify_fr.syllabify_text("anticonstitutionnellement")
    words = [c for c in chunks if c["kind"] == "word"]
    # long word → many syllables
    assert len(words) == 1
    assert len(words[0]["syllables"]) >= 7


def test_phonemes_returns_pairs():
    pairs = syllabify_fr.phonemes("chat")
    assert len(pairs) > 0
    assert all(isinstance(code, str) and isinstance(letters, str) for code, letters in pairs)


def test_phonemes_covers_letters():
    pairs = syllabify_fr.phonemes("chat")
    covered = "".join(letters for _, letters in pairs)
    assert covered == "chat"


def test_render_word_html_has_spans():
    html = syllabify_fr.render_word_html("chocolat")
    assert "syl-a" in html and "syl-b" in html


def test_render_html_full_text():
    html = syllabify_fr.render_html("les hôtels")
    assert "syl-a" in html


def test_render_html_liaison():
    html = syllabify_fr.render_html("les hôtels")
    assert "liaison" in html
