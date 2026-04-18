#!/usr/bin/env node
/**
 * Régénère tests/oracle.json depuis LireCouleur 6 (référence JS).
 *
 * Usage :
 *   node build/generate_oracle.js [path/to/wordlist.txt]
 */
const fs = require('fs');
const path = require('path');

const LC6_PATH = process.env.LC6_PATH || '/home/claude/lc6/devlc6-main/devlc6-main/js/lirecouleur/module.js';
const ROOT = path.resolve(__dirname, '..');
const OUT = path.join(ROOT, 'tests', 'oracle.json');

let content = fs.readFileSync(LC6_PATH, 'utf-8');
content += '\nglobalThis.LireCouleur = LireCouleur;\nglobalThis.LCPhoneme = LCPhoneme;\nglobalThis.LCSyllabe = LCSyllabe;\n';
eval(content);

function syllabifyWord(word, mode = 'std') {
    const phons = LireCouleur.extrairePhonemes(word);
    const sylls = LireCouleur.extraireSyllabes(phons, mode, 'ecrit');
    return sylls.map(s => s.phonemes.map(p => p.lettres).join(''));
}

// Oracle format identique à la v5 : {mot: {chunks: [[syllabes] ou "raw"]}}
// Pour un mot isolé, toujours un seul chunk Word.

// Corpus : si on passe un fichier, l'utiliser, sinon prendre le corpus par défaut intégré.
let corpus;
if (process.argv[2]) {
    corpus = fs.readFileSync(process.argv[2], 'utf-8').split(/\s+/).filter(Boolean);
} else {
    // Corpus fourni sous forme de liste de mots dans build/data/corpus.txt si présent
    const corpusFile = path.join(ROOT, 'build', 'data', 'corpus.txt');
    if (fs.existsSync(corpusFile)) {
        corpus = fs.readFileSync(corpusFile, 'utf-8').split(/\s+/).filter(Boolean);
    } else {
        corpus = ['chat', 'famille', 'parlent', 'prudent', 'lion', 'hier',
                  'chocolat', 'anticonstitutionnellement'];
    }
}

corpus = Array.from(new Set(corpus.filter(w => w.length > 0))).sort();

const oracle = {};
let errors = 0;
for (const w of corpus) {
    try {
        const sylls = syllabifyWord(w, 'std');
        oracle[w] = { chunks: [sylls] };
    } catch (e) {
        oracle[w] = { error: e.message };
        errors++;
    }
}

fs.writeFileSync(OUT, JSON.stringify(oracle, null, 2), 'utf-8');
console.log(`Oracle: ${Object.keys(oracle).length} entrées (${errors} erreurs) → ${OUT}`);
