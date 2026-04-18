#!/usr/bin/env python3
"""
Régénère tests/oracle.json depuis pylirecouleur.

Prérequis :
    pip install pylirecouleur

Usage :
    python3 build/generate_oracle.py [wordlist.txt]

Si wordlist.txt est fourni, un mot par ligne. Sinon, un corpus intégré de ~5000 mots courants.
"""
import json
import sys
from pathlib import Path

try:
    from lirecouleur.decoder import lcdecoder
except ImportError:
    print("Erreur : pylirecouleur non installé. Faites : pip install pylirecouleur")
    sys.exit(1)

OUT = Path(__file__).resolve().parent.parent / "tests" / "oracle.json"

# Corpus minimal de validation (à étendre)
DEFAULT_CORPUS = """
chat chien maman papa école arbre table chocolat éléphant
famille fille mille ville tranquille briller travailler grenouille
bouteille oreille abeille feuille mouillé
parlent mangent dorment prudent violent agent président
lion hier pied fief miel oiseau
hôtel haricot homme héros hibou
petite fenêtre cheval semaine bouteille
porte-monnaie arc-en-ciel grand-père
appeler addition attention allumer communauté passer
poisson chanteur monsieur aujourd'hui beaucoup maintenant
anticonstitutionnellement ophtalmologiste extraordinaire
l'école d'abord qu'il
crayon paysan royal voyage payer
eau au aux œuf bœuf cœur sœur
parler manger boulanger amer cher fier
numéro zéro cent mille million
""".split()


def main():
    if len(sys.argv) > 1:
        wordlist = Path(sys.argv[1])
        corpus = wordlist.read_text().split()
    else:
        corpus = DEFAULT_CORPUS

    corpus = sorted(set(w.strip() for w in corpus if w.strip()))

    oracle = {}
    errors = 0
    for w in corpus:
        try:
            chunks = lcdecoder.extract_syllables(w)
            oracle[w] = {"chunks": chunks}
        except Exception as e:
            oracle[w] = {"error": str(e)}
            errors += 1

    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", encoding="utf-8") as f:
        json.dump(oracle, f, ensure_ascii=False, indent=2)

    print(f"Écrit {OUT} ({len(oracle)} entrées, {errors} erreurs pylirecouleur)")


if __name__ == "__main__":
    main()
