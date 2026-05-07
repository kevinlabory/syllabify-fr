use ::syllabify_fr as core;
use core::letters::{match_letters, presets, render_letters_html, LetterRule, RenderMode};
use core::TextChunk;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[pyfunction]
#[pyo3(name = "syllables")]
fn syllables_py(word: &str) -> Vec<String> {
    core::syllables(word)
}

#[pyfunction]
#[pyo3(name = "syllabify_text")]
fn syllabify_text_py<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for chunk in core::syllabify_text(text) {
        let d = PyDict::new(py);
        match chunk {
            TextChunk::Word(syls) => {
                d.set_item("kind", "word")?;
                d.set_item("syllables", syls)?;
            }
            TextChunk::Raw(s) => {
                d.set_item("kind", "raw")?;
                d.set_item("text", s)?;
            }
            _ => {
                d.set_item("kind", "unknown")?;
            }
        }
        list.append(d)?;
    }
    Ok(list)
}

#[pyfunction]
#[pyo3(name = "phonemes")]
fn phonemes_py(word: &str) -> Vec<(String, String)> {
    core::phonemes(word)
}

#[pyfunction]
#[pyo3(name = "render_html")]
fn render_html_py(text: &str) -> String {
    core::render_html(text)
}

#[pyfunction]
#[pyo3(name = "render_word_html")]
fn render_word_html_py(word: &str) -> String {
    core::render_word_html(word)
}

fn preset_rules(name: &str) -> Option<Vec<LetterRule>> {
    match name {
        "bdpq" => Some(presets::bdpq()),
        "mnu" => Some(presets::mnu()),
        "pir-pri" | "pir_pri" => Some(presets::pir_pri()),
        _ => None,
    }
}

#[pyfunction]
#[pyo3(name = "highlight_letters", signature = (word, preset, mode = "inline"))]
fn highlight_letters_py(word: &str, preset: &str, mode: &str) -> PyResult<String> {
    let rules = preset_rules(preset).ok_or_else(|| {
        PyValueError::new_err(format!(
            "unknown preset {:?}: expected one of \"bdpq\", \"mnu\", \"pir-pri\"",
            preset
        ))
    })?;
    let render_mode = match mode {
        "inline" => RenderMode::Inline,
        "classes" => RenderMode::Classes,
        other => {
            return Err(PyValueError::new_err(format!(
                "unknown mode {:?}: expected \"inline\" or \"classes\"",
                other
            )))
        }
    };
    let spans = match_letters(word, &rules);
    Ok(render_letters_html(word, &spans, &rules, render_mode))
}

#[pymodule]
fn syllabify_fr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(syllables_py, m)?)?;
    m.add_function(wrap_pyfunction!(syllabify_text_py, m)?)?;
    m.add_function(wrap_pyfunction!(phonemes_py, m)?)?;
    m.add_function(wrap_pyfunction!(render_html_py, m)?)?;
    m.add_function(wrap_pyfunction!(render_word_html_py, m)?)?;
    m.add_function(wrap_pyfunction!(highlight_letters_py, m)?)?;
    Ok(())
}
