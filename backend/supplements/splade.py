"""SPLADE-based sparse retrieval helpers.

This module loads the fine-tuned SPLADE model (or the public base model when
no fine-tuned checkpoint is available) and exposes two functions:

    encode_query(text: str) -> dict[str, float]
        Produces a sparse vector representation of the query.

    find_similar_supplements(query_vector: dict, top_k: int) -> list[int]
        Returns the IDs of the top-k most similar supplements from the DB.

The SPLADE model path is configured via ``settings.SPLADE_MODEL_PATH``.
If the model or its dependencies are not installed the functions raise
``RuntimeError`` so callers can fall back gracefully.
"""

from __future__ import annotations

import logging
from typing import Dict, List

logger = logging.getLogger(__name__)

_model = None
_tokenizer = None


def _load_model():
    """Lazy-load the SPLADE model and tokenizer."""
    global _model, _tokenizer  # noqa: PLW0603
    if _model is not None:
        return _model, _tokenizer

    try:
        from django.conf import settings
        from transformers import AutoModelForMaskedLM, AutoTokenizer

        model_path = getattr(settings, "SPLADE_MODEL_PATH", "naver/splade-cocondenser-selfdistil")
        logger.info("Loading SPLADE model from '%s' …", model_path)
        _tokenizer = AutoTokenizer.from_pretrained(model_path)
        _model = AutoModelForMaskedLM.from_pretrained(model_path)
        _model.eval()
        logger.info("SPLADE model loaded successfully.")
    except Exception as exc:
        raise RuntimeError(f"Could not load SPLADE model: {exc}") from exc

    return _model, _tokenizer


def encode_query(text: str) -> Dict[str, float]:
    """Return a sparse SPLADE vector for *text*.

    The returned dict maps token strings to their importance weights.
    """
    import torch

    model, tokenizer = _load_model()

    inputs = tokenizer(text, return_tensors="pt", truncation=True, max_length=512)
    with torch.no_grad():
        outputs = model(**inputs)

    # SPLADE activation: log(1 + ReLU(logits)) aggregated over token positions
    logits = outputs.logits  # (1, seq_len, vocab_size)
    sparse_vec = torch.log1p(torch.relu(logits)).max(dim=1).values.squeeze(0)

    # Convert to a dict of {token: weight} keeping only non-zero entries
    non_zero = sparse_vec.nonzero(as_tuple=True)[0]
    vector: Dict[str, float] = {}
    for idx in non_zero.tolist():
        token = tokenizer.convert_ids_to_tokens(idx)
        vector[token] = float(sparse_vec[idx])

    return vector


def find_similar_supplements(query_vector: Dict[str, float], top_k: int = 10) -> List[int]:
    """Return the IDs of the *top_k* supplements most similar to *query_vector*.

    Similarity is computed as the dot-product of the query sparse vector with
    the pre-computed supplement SPLADE vectors stored in ``Supplement.splade_vector``.
    Supplements that have no stored vector are skipped.
    """
    from .models import Supplement

    supplements = Supplement.objects.exclude(splade_vector__isnull=True)

    scored: list[tuple[float, int]] = []
    for supplement in supplements:
        doc_vector: dict = supplement.splade_vector or {}
        score = sum(
            query_vector.get(token, 0.0) * weight
            for token, weight in doc_vector.items()
        )
        if score > 0:
            scored.append((score, supplement.id))

    scored.sort(reverse=True)
    return [id_ for _, id_ in scored[:top_k]]
