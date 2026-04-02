#!/usr/bin/env python
"""Index supplements into the Django database with their SPLADE vectors.

Run this script after fine-tuning (or with the base model) to precompute the
SPLADE sparse vector for every supplement in the database and store it in the
``Supplement.splade_vector`` JSON field so that the search endpoint can do
fast dot-product retrieval without re-encoding documents at query time.

Usage
-----
    cd backend
    python ../scripts/finetune/index_supplements.py \\
        --model ../../backend/models/splade-supplements/best

Set ``DJANGO_SETTINGS_MODULE`` to point at your settings if it differs from
the default (``supplement_buddy.settings``).
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="Index supplement SPLADE vectors")
    parser.add_argument(
        "--model",
        default=os.getenv("SPLADE_MODEL_PATH", "naver/splade-cocondenser-selfdistil"),
        help="Path to fine-tuned SPLADE model or Hugging Face model ID",
    )
    parser.add_argument(
        "--batch_size",
        type=int,
        default=32,
    )
    args = parser.parse_args()

    # Bootstrap Django
    sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "backend"))
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "supplement_buddy.settings")
    import django  # noqa: PLC0415

    django.setup()

    import torch  # noqa: PLC0415
    from transformers import AutoModelForMaskedLM, AutoTokenizer  # noqa: PLC0415

    from supplements.models import Supplement  # noqa: PLC0415

    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Loading model '{args.model}' on {device} …")
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    model = AutoModelForMaskedLM.from_pretrained(args.model).to(device).eval()

    supplements = list(Supplement.objects.all())
    print(f"Indexing {len(supplements)} supplements …")

    batch_size = args.batch_size
    for start in range(0, len(supplements), batch_size):
        batch = supplements[start : start + batch_size]
        texts = [
            " ".join(
                filter(
                    None,
                    [s.name, s.brand, s.category, s.description, s.ingredients],
                )
            )
            for s in batch
        ]

        inputs = tokenizer(
            texts,
            padding=True,
            truncation=True,
            max_length=512,
            return_tensors="pt",
        ).to(device)
        with torch.no_grad():
            outputs = model(**inputs)

        vecs = torch.log1p(torch.relu(outputs.logits)).max(dim=1).values

        for supp, vec in zip(batch, vecs):
            non_zero = vec.nonzero(as_tuple=True)[0]
            vector = {
                tokenizer.convert_ids_to_tokens(int(idx)): float(vec[idx])
                for idx in non_zero.tolist()
            }
            supp.splade_vector = vector
            supp.save(update_fields=["splade_vector"])

        print(f"  Indexed {min(start + batch_size, len(supplements))}/{len(supplements)}")

    print("Done!")


if __name__ == "__main__":
    main()
