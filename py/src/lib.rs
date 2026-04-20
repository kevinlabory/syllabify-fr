use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use ::syllabify_fr as core;
use core::TextChunk;

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

#[pymodule]
fn syllabify_fr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(syllables_py, m)?)?;
    m.add_function(wrap_pyfunction!(syllabify_text_py, m)?)?;
    m.add_function(wrap_pyfunction!(phonemes_py, m)?)?;
    m.add_function(wrap_pyfunction!(render_html_py, m)?)?;
    m.add_function(wrap_pyfunction!(render_word_html_py, m)?)?;
    Ok(())
}
