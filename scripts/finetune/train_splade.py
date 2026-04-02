#!/usr/bin/env python
"""Fine-tune a SPLADE model on supplement search data.

This script fine-tunes the SPLADE e-commerce model
(https://huggingface.co/thierrydamiba/splade-ecommerce-esci) on a
supplement-specific triplet dataset produced by ``prepare_data.py``.

Architecture
------------
SPLADE (SParse Lexical AnD Expansion) models produce sparse term-weight
vectors from a masked-language-model backbone.  We fine-tune using a
pairwise in-batch negatives + hard negatives loss.

Usage
-----
    # 1. Install dependencies
    pip install -r requirements.txt

    # 2. Prepare data
    python prepare_data.py --input supplements.csv

    # 3. Fine-tune
    python train_splade.py \\
        --model_name thierrydamiba/splade-ecommerce-esci \\
        --train_triplets data/train_triplets.jsonl \\
        --corpus data/corpus.jsonl \\
        --output_dir ../../backend/models/splade-supplements \\
        --epochs 3 \\
        --batch_size 16

After training the checkpoint is saved to ``output_dir``.  Point the Django
backend at it via the ``SPLADE_MODEL_PATH`` environment variable.
"""

from __future__ import annotations

import argparse
import json
import logging
import os
from pathlib import Path
from typing import Any

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Dataset helpers
# ---------------------------------------------------------------------------


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    with path.open() as f:
        return [json.loads(line) for line in f if line.strip()]


class SupplementTripletDataset:
    """Minimal map-style dataset for (query, pos_text, neg_text) triplets."""

    def __init__(
        self,
        triplets: list[dict[str, Any]],
        corpus: dict[str, str],
    ):
        self.triplets = triplets
        self.corpus = corpus

    def __len__(self) -> int:
        return len(self.triplets)

    def __getitem__(self, idx: int) -> dict[str, str]:
        t = self.triplets[idx]
        return {
            "query": t["query"],
            "positive": self.corpus.get(t["positive_id"], ""),
            "negative": self.corpus.get(t["negative_id"], ""),
        }


# ---------------------------------------------------------------------------
# SPLADE sparse vector helpers
# ---------------------------------------------------------------------------


def encode_batch(texts: list[str], model: Any, tokenizer: Any, device: str) -> Any:
    """Return a (batch, vocab) sparse activation tensor."""
    import torch  # noqa: PLC0415

    inputs = tokenizer(
        texts,
        padding=True,
        truncation=True,
        max_length=512,
        return_tensors="pt",
    ).to(device)
    with torch.no_grad():
        outputs = model(**inputs)
    # SPLADE aggregation: max-pool over token positions
    vecs = torch.log1p(torch.relu(outputs.logits)).max(dim=1).values
    return vecs


# ---------------------------------------------------------------------------
# Training loop
# ---------------------------------------------------------------------------


def train(args: argparse.Namespace) -> None:
    import torch  # noqa: PLC0415
    from torch.utils.data import DataLoader  # noqa: PLC0415
    from transformers import AutoModelForMaskedLM, AutoTokenizer  # noqa: PLC0415

    device = "cuda" if torch.cuda.is_available() else "cpu"
    logger.info("Using device: %s", device)

    # --- Load model ---------------------------------------------------------
    logger.info("Loading base model '%s' …", args.model_name)
    tokenizer = AutoTokenizer.from_pretrained(args.model_name)
    model = AutoModelForMaskedLM.from_pretrained(args.model_name)
    model.to(device)
    model.train()

    # --- Load data ----------------------------------------------------------
    logger.info("Loading triplets from %s …", args.train_triplets)
    triplets = load_jsonl(Path(args.train_triplets))
    logger.info("  %d triplets loaded.", len(triplets))

    logger.info("Loading corpus from %s …", args.corpus)
    corpus_records = load_jsonl(Path(args.corpus))
    corpus = {r["id"]: r["text"] for r in corpus_records}
    logger.info("  %d documents in corpus.", len(corpus))

    dataset = SupplementTripletDataset(triplets, corpus)
    loader = DataLoader(dataset, batch_size=args.batch_size, shuffle=True, drop_last=True)

    # --- Optimizer ----------------------------------------------------------
    optimizer = torch.optim.AdamW(model.parameters(), lr=args.lr)

    # --- Training loop ------------------------------------------------------
    lambda_reg = args.lambda_reg
    best_loss = float("inf")

    for epoch in range(1, args.epochs + 1):
        epoch_loss = 0.0
        for step, batch in enumerate(loader):
            optimizer.zero_grad()

            q_vecs = encode_batch(batch["query"], model, tokenizer, device)
            p_vecs = encode_batch(batch["positive"], model, tokenizer, device)
            n_vecs = encode_batch(batch["negative"], model, tokenizer, device)

            # Pairwise margin ranking loss (positive score > negative score)
            pos_scores = (q_vecs * p_vecs).sum(dim=-1)
            neg_scores = (q_vecs * n_vecs).sum(dim=-1)
            ranking_loss = torch.relu(args.margin - pos_scores + neg_scores).mean()

            # FLOPS regularisation to encourage sparsity
            reg = lambda_reg * (
                torch.abs(q_vecs).sum(dim=-1).mean()
                + torch.abs(p_vecs).sum(dim=-1).mean()
            )

            loss = ranking_loss + reg
            loss.backward()
            optimizer.step()

            epoch_loss += loss.item()
            if step % 50 == 0:
                logger.info(
                    "Epoch %d/%d  step %d/%d  loss=%.4f",
                    epoch,
                    args.epochs,
                    step,
                    len(loader),
                    loss.item(),
                )

        avg_loss = epoch_loss / len(loader)
        logger.info("Epoch %d finished.  Avg loss: %.4f", epoch, avg_loss)

        if avg_loss < best_loss:
            best_loss = avg_loss
            output = Path(args.output_dir) / "best"
            output.mkdir(parents=True, exist_ok=True)
            model.save_pretrained(str(output))
            tokenizer.save_pretrained(str(output))
            logger.info("  Best model saved to %s", output)

    # Save final checkpoint
    final = Path(args.output_dir) / "final"
    final.mkdir(parents=True, exist_ok=True)
    model.save_pretrained(str(final))
    tokenizer.save_pretrained(str(final))
    logger.info("Final model saved to %s", final)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description="Fine-tune SPLADE for supplement search")
    parser.add_argument(
        "--model_name",
        default="thierrydamiba/splade-ecommerce-esci",
        help="Hugging Face model ID or local path to the base SPLADE model",
    )
    parser.add_argument(
        "--train_triplets",
        default="data/train_triplets.jsonl",
        help="Training triplets produced by prepare_data.py",
    )
    parser.add_argument(
        "--corpus",
        default="data/corpus.jsonl",
        help="Corpus produced by prepare_data.py",
    )
    parser.add_argument(
        "--output_dir",
        default="../../backend/models/splade-supplements",
        help="Directory to save the fine-tuned model",
    )
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--batch_size", type=int, default=16)
    parser.add_argument("--lr", type=float, default=2e-5)
    parser.add_argument("--margin", type=float, default=1.0, help="Ranking loss margin")
    parser.add_argument(
        "--lambda_reg",
        type=float,
        default=5e-4,
        help="FLOPS regularisation coefficient",
    )
    args = parser.parse_args()
    train(args)


if __name__ == "__main__":
    main()
