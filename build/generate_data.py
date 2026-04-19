#!/usr/bin/env python3
"""
Génère src/data.rs à partir de build/data/lirecouleur_v6.json.

Le JSON est extrait du fichier js/lirecouleur/module.js de LireCouleur 6
(licence GPL v3, © Marie-Pierre & Luc Brungard).
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "build" / "data" / "lirecouleur_v6.json"
OUT = ROOT / "src" / "data.rs"

with SRC.open(encoding="utf-8") as f:
    payload = json.load(f)

automaton = payload["automaton"]
mots_osse = payload["mots_osse"]

special_fns = set()
for letter, entry in automaton.items():
    rules_list, rules_map = entry[0], entry[1]
    for rule_name, rule_def in rules_map.items():
        cle = rule_def[0]
        if isinstance(cle, str) and cle:  # filtrer les chaînes vides (règles '@' sans condition)
            special_fns.add(cle)

special_variants = sorted(special_fns)


def rust_string(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def to_camel(name):
    return "".join(w.capitalize() for w in name.split("_"))


def emit_rule(cle):
    if isinstance(cle, str):
        return f"RuleKind::Special(Special::{to_camel(cle)})"
    plus = cle.get("+")
    minus = cle.get("-")
    plus_str = rust_string(plus) if plus is not None else '""'
    minus_str = rust_string(minus) if minus is not None else '""'
    has_plus = "true" if plus is not None else "false"
    has_minus = "true" if minus is not None else "false"
    return (
        f"RuleKind::Context {{ plus: {plus_str}, minus: {minus_str}, "
        f"has_plus: {has_plus}, has_minus: {has_minus} }}"
    )


letters_code = []
for letter in sorted(automaton.keys()):
    entry = automaton[letter]
    rules_list, rules_map = entry[0], entry[1]
    rules_entries = []
    for rule_name in rules_list:
        rule_def = rules_map[rule_name]
        cle, phoneme, step = rule_def[0], rule_def[1], rule_def[2]
        rules_entries.append(
            f"        Rule {{ name: {rust_string(rule_name)}, "
            f"kind: {emit_rule(cle)}, phoneme: {rust_string(phoneme)}, step: {step} }},"
        )
    default_rule = None
    endword_rule = None
    if "*" in rules_map:
        rd = rules_map["*"]
        # LC6 applique * comme règle par défaut SANS tester sa condition
        # (cf. module.js l. 811-816 : `phoneme = aut['*'][1]; pas = aut['*'][2];`).
        # Donc quelle que soit la `cle`, on prend * comme règle par défaut pure.
        default_rule = (rust_string(rd[1]), rd[2])
    if "@" in rules_map:
        rd = rules_map["@"]
        cle = rd[0]
        has_cond = isinstance(cle, dict) and bool(cle)
        if has_cond:
            rules_entries.append(
                f'        Rule {{ name: "__at", '
                f"kind: {emit_rule(cle)}, phoneme: {rust_string(rd[1])}, step: {rd[2]} }},"
            )
            endword_rule = None
        else:
            endword_rule = (rust_string(rd[1]), rd[2])

    default_code = (
        f"Some(({default_rule[0]}, {default_rule[1]}))" if default_rule else "None"
    )
    endword_code = (
        f"Some(({endword_rule[0]}, {endword_rule[1]}))" if endword_rule else "None"
    )
    rules_block = "\n".join(rules_entries) if rules_entries else ""
    letters_code.append(
        f"""    ({rust_string(letter)}, LetterEntry {{
        rules: &[
{rules_block}
        ],
        default: {default_code},
        end_of_word: {endword_code},
    }}),"""
    )

automaton_rs = "\n".join(letters_code)


def emit_str_array(name, values):
    sorted_vals = sorted(set(values))
    joined = ",\n    ".join(rust_string(v) for v in sorted_vals)
    return f"pub static {name}: &[&str] = &[\n    {joined},\n];\n\n"


db_blocks = []
db_blocks.append(emit_str_array("VERBES_IER", payload["verbes_ier"]))
db_blocks.append(emit_str_array("VERBES_MER", payload["verbes_mer"]))
db_blocks.append(emit_str_array("VERBES_ENTER", payload["verbes_enter"]))
db_blocks.append(emit_str_array("MOTS_ENT", payload["mots_ent"]))
db_blocks.append(emit_str_array("EXCEPTIONS_FINAL_ER", payload["exceptions_final_er"]))
db_blocks.append(emit_str_array("POSSIBLES_NC_AI_FINAL", payload["possibles_nc_ai_final"]))
db_blocks.append(emit_str_array("POSSIBLES_AVOIR", payload["possibles_avoir"]))
db_blocks.append(emit_str_array("MOTS_S_FINAL", payload["mots_s_final"]))
db_blocks.append(emit_str_array("MOTS_T_FINAL", payload["mots_t_final"]))
db_blocks.append(emit_str_array("EXCEPTIONS_FINAL_TIEN", payload["exceptions_final_tien"]))
db_blocks.append(emit_str_array("MOTS_OSSE", mots_osse))
db_blocks.append(emit_str_array("EXCEPTIONS_EN_FINAL", payload["exceptions_en_final"]))
db_blocks.append(emit_str_array("DETERMINANTS", payload["determinant"]))
db_blocks.append(emit_str_array("PRONOMS", payload["pronom"]))
db_blocks.append(emit_str_array("LIAISONS_AVAL", payload.get("liaisons_aval", [])))

homographes = payload.get("homographesNonHomophones", {})
homo_entries = []
for mot in sorted(homographes.keys()):
    variants = homographes[mot]
    variants_code = []
    for v in variants:
        prec = v["precedent"]
        codage = v["codage"]
        prec_joined = ", ".join(rust_string(w) for w in prec)
        codage_joined = ", ".join(
            f"({rust_string(c['phoneme'])}, {rust_string(c['lettres'])})" for c in codage
        )
        variants_code.append(
            f"        HomographVariant {{ precedent: &[{prec_joined}], "
            f"codage: &[{codage_joined}] }},"
        )
    variants_block = "\n".join(variants_code)
    homo_entries.append(
        f"""    ({rust_string(mot)}, &[
{variants_block}
    ]),"""
    )
homographes_rs = "\n".join(homo_entries)

variants_code = "\n".join(f"    {to_camel(fn)},  // {fn}" for fn in special_variants)

output = f'''// SPDX-License-Identifier: GPL-3.0-or-later
// Generated from LireCouleur 6 (forge.apps.education.fr) by build/generate_data.py
// Original work © Marie-Pierre & Luc Brungard (LireCouleur), GPL v3
// DO NOT EDIT MANUALLY

#![allow(clippy::all)]

/// Entrée de l'automate pour une lettre.
#[derive(Debug, Clone, Copy)]
pub struct LetterEntry {{
    pub rules: &'static [Rule],
    pub default: Option<(&'static str, usize)>,
    pub end_of_word: Option<(&'static str, usize)>,
}}

/// Règle nommée de l'automate.
#[derive(Debug, Clone, Copy)]
pub struct Rule {{
    pub name: &'static str,
    pub kind: RuleKind,
    pub phoneme: &'static str,
    pub step: usize,
}}

#[derive(Debug, Clone, Copy)]
pub enum RuleKind {{
    /// Règle contextuelle : regex lookahead (plus) et lookbehind (minus).
    Context {{
        plus: &'static str,
        minus: &'static str,
        has_plus: bool,
        has_minus: bool,
    }},
    /// Règle spéciale gérée par une fonction dédiée.
    Special(Special),
}}

/// Fonctions spéciales de l'automate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Special {{
{variants_code}
}}

/// Un variant d'homographe non homophone.
#[derive(Debug, Clone, Copy)]
pub struct HomographVariant {{
    /// Liste des mots pouvant précéder pour déclencher ce codage.
    pub precedent: &'static [&'static str],
    /// Suite de (phoneme, lettres).
    pub codage: &'static [(&'static str, &'static str)],
}}

/// Automate : une entrée par lettre.
pub static AUTOMATON: &[(&str, LetterEntry)] = &[
{automaton_rs}
];

/// Homographes non homophones (v6).
pub static HOMOGRAPHES: &[(&str, &[HomographVariant])] = &[
{homographes_rs}
];

// ================= DATABASES =================

{"".join(db_blocks)}'''

OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(output, encoding="utf-8")
print(f"Écrit {OUT} ({len(output):,} octets, {output.count(chr(10))} lignes)")
