#!/usr/bin/env node
// SPDX-License-Identifier: GPL-3.0-or-later
/**
 * Extrait toutes les données algorithmiques de LireCouleur 6 (module.js)
 * vers build/data/lirecouleur_v6.json, utilisé ensuite par generate_data.py.
 *
 * Usage :
 *   LC6_PATH=/chemin/vers/lirecouleur/js/lirecouleur/module.js node build/extract_v6_data.js
 */
const fs = require('fs');
const path = require('path');

const LC6_PATH = process.env.LC6_PATH
    || path.join(__dirname, '..', '..', 'lirecouleur', 'js', 'lirecouleur', 'module.js');
const OUT = path.join(__dirname, 'data', 'lirecouleur_v6.json');

if (!fs.existsSync(LC6_PATH)) {
    console.error(`Erreur : ${LC6_PATH} introuvable.`);
    console.error(`Définissez LC6_PATH vers le module.js de LireCouleur 6.`);
    process.exit(1);
}

let content = fs.readFileSync(LC6_PATH, 'utf-8');
content += '\nglobalThis.LireCouleur = LireCouleur;\n';
eval(content);

const data = {
    automaton: LireCouleur.autom,
    verbes_ier: LireCouleur.verbes_ier,
    verbes_mer: LireCouleur.verbes_mer,
    verbes_enter: LireCouleur.verbes_enter,
    mots_ent: LireCouleur.mots_ent,
    exceptions_final_er: LireCouleur.exceptions_final_er,
    possibles_nc_ai_final: LireCouleur.possibles_nc_ai_final,
    possibles_avoir: LireCouleur.possibles_avoir,
    mots_s_final: LireCouleur.mots_s_final,
    mots_t_final: LireCouleur.mots_t_final,
    exceptions_final_tien: LireCouleur.exceptions_final_tien,
    exceptions_er_final: LireCouleur.exceptions_er_final,
    exceptions_en_final: LireCouleur.exceptions_en_final,
    determinant: LireCouleur.determinant,
    pronom: LireCouleur.pronom,
    homographesNonHomophones: LireCouleur.homographesNonHomophones,
};

// mots_osse est défini localement dans post_traitement_o_ouvert_ferme ; on l'extrait
const motsOsseMatch = content.match(/let mots_osse = (\[[^\]]+\])/);
data.mots_osse = motsOsseMatch ? JSON.parse(motsOsseMatch[1]) : [];

// listeMotsLiaison est défini localement dans liaisonAval ; on l'extrait
const liaisonAvalMatch = content.match(/let listeMotsLiaison = (\[[\s\S]+?\])/);
data.liaisons_aval = liaisonAvalMatch ? JSON.parse(liaisonAvalMatch[1]) : [];

fs.mkdirSync(path.dirname(OUT), { recursive: true });
fs.writeFileSync(OUT, JSON.stringify(data, null, 2), 'utf-8');
console.log(`Écrit ${OUT}`);
console.log(`  automate : ${Object.keys(data.automaton).length} lettres`);
console.log(`  homographes : ${Object.keys(data.homographesNonHomophones).length}`);
console.log(`  mots_osse : ${data.mots_osse.length}`);
console.log(`  liaisons_aval : ${data.liaisons_aval.length}`);
