#!/usr/bin/env python
"""Prepare supplement data for SPLADE fine-tuning.

This script converts raw supplement data (CSV or JSON) into the triplet format
expected by the fine-tuning script:

    query  |  positive_doc_id  |  negative_doc_id

Usage
-----
    python prepare_data.py \\
        --input supplements.csv \\
        --output data/train_triplets.jsonl \\
        --queries data/queries.jsonl

Input CSV / JSON columns expected
----------------------------------
    id, name, brand, category, description, ingredients, serving_size

Synthetic queries are generated from the supplement name + category to bootstrap
the training signal.  For production use, replace these with real user queries
collected from the application's search log.
"""

from __future__ import annotations

import argparse
import json
import random
from pathlib import Path
from typing import Any


QUERY_TEMPLATES = [
    "{name}",
    "{name} supplement",
    "{name} {category}",
    "best {name} for {category}",
    "{name} {brand}",
    "{category} supplement {name}",
    "what does {name} do",
    "{name} benefits",
]


def load_supplements(path: Path) -> list[dict[str, Any]]:
    if path.suffix == ".json":
        with path.open() as f:
            data = json.load(f)
        return data if isinstance(data, list) else data.get("supplements", [])
    try:
        import pandas as pd  # noqa: PLC0415

        df = pd.read_csv(path)
        return df.to_dict(orient="records")
    except ImportError:
        import csv  # noqa: PLC0415

        with path.open(newline="") as f:
            return list(csv.DictReader(f))


def generate_queries(supplement: dict[str, Any]) -> list[str]:
    name = supplement.get("name", "")
    brand = supplement.get("brand", "")
    category = supplement.get("category", "")
    queries = []
    for template in QUERY_TEMPLATES:
        try:
            q = template.format(name=name, brand=brand, category=category)
            if q.strip():
                queries.append(q.strip())
        except KeyError:
            pass
    return list(dict.fromkeys(queries))  # deduplicate preserving order


def build_triplets(
    supplements: list[dict[str, Any]],
    negatives_per_query: int = 3,
) -> list[dict[str, Any]]:
    """Build (query, positive_id, negative_id) triplets."""
    rng = random.Random(42)
    triplets: list[dict[str, Any]] = []
    ids = [str(s.get("id", i)) for i, s in enumerate(supplements)]

    for pos_idx, supplement in enumerate(supplements):
        queries = generate_queries(supplement)
        pos_id = ids[pos_idx]
        neg_pool = [id_ for id_ in ids if id_ != pos_id]
        for query in queries:
            negatives = rng.sample(neg_pool, min(negatives_per_query, len(neg_pool)))
            for neg_id in negatives:
                triplets.append(
                    {
                        "query": query,
                        "positive_id": pos_id,
                        "negative_id": neg_id,
                    }
                )

    return triplets


def save_corpus(supplements: list[dict[str, Any]], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w") as f:
        for s in supplements:
            doc = {
                "id": str(s.get("id", "")),
                "text": " ".join(
                    filter(
                        None,
                        [
                            s.get("name", ""),
                            s.get("brand", ""),
                            s.get("category", ""),
                            s.get("description", ""),
                            s.get("ingredients", ""),
                        ],
                    )
                ),
            }
            f.write(json.dumps(doc) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="Prepare supplement fine-tuning data")
    parser.add_argument("--input", required=True, help="Path to supplements CSV or JSON")
    parser.add_argument(
        "--output",
        default="data/train_triplets.jsonl",
        help="Path to write training triplets JSONL",
    )
    parser.add_argument(
        "--corpus",
        default="data/corpus.jsonl",
        help="Path to write corpus JSONL",
    )
    parser.add_argument(
        "--negatives",
        type=int,
        default=3,
        help="Number of hard negatives per query",
    )
    args = parser.parse_args()

    print(f"Loading supplements from {args.input} …")
    supplements = load_supplements(Path(args.input))
    print(f"  Loaded {len(supplements)} supplements.")

    print("Generating training triplets …")
    triplets = build_triplets(supplements, negatives_per_query=args.negatives)
    print(f"  Generated {len(triplets)} triplets.")

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w") as f:
        for t in triplets:
            f.write(json.dumps(t) + "\n")
    print(f"  Triplets written to {output_path}")

    corpus_path = Path(args.corpus)
    save_corpus(supplements, corpus_path)
    print(f"  Corpus written to {corpus_path}")


if __name__ == "__main__":
    main()
