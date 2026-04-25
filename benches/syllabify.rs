use criterion::{black_box, criterion_group, criterion_main, Criterion};
use syllabify_fr::{syllabify_text, syllables};

fn bench_syllables_short(c: &mut Criterion) {
    c.bench_function("syllables/chocolat", |b| {
        b.iter(|| syllables(black_box("chocolat")))
    });
}

fn bench_syllables_long(c: &mut Criterion) {
    c.bench_function("syllables/anticonstitutionnellement", |b| {
        b.iter(|| syllables(black_box("anticonstitutionnellement")))
    });
}

fn bench_syllabify_text(c: &mut Criterion) {
    let text = "Le petit chat noir dort sur le canapé bleu et rêve de souris.";
    c.bench_function("syllabify_text/sentence", |b| {
        b.iter(|| syllabify_text(black_box(text)))
    });
}

fn bench_syllabify_text_cold(c: &mut Criterion) {
    // Mesure le coût de la première initialisation du cache regex (warm-up inclus).
    let words = [
        "famille",
        "chocolat",
        "parlent",
        "prudent",
        "président",
        "lion",
        "hier",
        "pied",
        "œil",
        "grand-père",
        "hôtel",
        "haricot",
    ];
    c.bench_function("syllabify_text/12_words", |b| {
        b.iter(|| {
            for w in &words {
                black_box(syllables(black_box(w)));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_syllables_short,
    bench_syllables_long,
    bench_syllabify_text,
    bench_syllabify_text_cold,
);
criterion_main!(benches);
