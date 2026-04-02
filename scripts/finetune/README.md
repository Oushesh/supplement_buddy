# Fine-tuning Scripts

Scripts to fine-tune a SPLADE sparse retrieval model for supplement search.

## Background

[SPLADE](https://github.com/naver/splade) is a sparse neural retrieval model
that builds on masked-language models to produce sparse, interpretable term-weight
vectors.  We start from the e-commerce checkpoint
[`thierrydamiba/splade-ecommerce-esci`](https://huggingface.co/thierrydamiba/splade-ecommerce-esci)
and fine-tune it on supplement-specific triplets.

## Quick Start

```bash
# 1. Install dependencies (preferably in a virtual environment)
pip install -r requirements.txt

# 2. Prepare training data from your supplement CSV or JSON
python prepare_data.py \
    --input supplements.csv \
    --output data/train_triplets.jsonl \
    --corpus data/corpus.jsonl

# 3. Fine-tune the model
python train_splade.py \
    --model_name thierrydamiba/splade-ecommerce-esci \
    --train_triplets data/train_triplets.jsonl \
    --corpus data/corpus.jsonl \
    --output_dir ../../backend/models/splade-supplements \
    --epochs 3 \
    --batch_size 16

# 4. Index the fine-tuned vectors into the Django database
cd ../../backend
python ../scripts/finetune/index_supplements.py \
    --model models/splade-supplements/best
```

Then set `SPLADE_MODEL_PATH=models/splade-supplements/best` in your Django
`.env` file to use the fine-tuned model for search.

## Scripts

| Script | Purpose |
|---|---|
| `prepare_data.py` | Converts supplement catalogue (CSV/JSON) into training triplets and a corpus file |
| `train_splade.py` | Fine-tunes the SPLADE model using pairwise ranking loss + FLOPS regularisation |
| `index_supplements.py` | Pre-computes SPLADE vectors for all supplements in the Django DB |

## Input Data Format

`prepare_data.py` accepts a CSV or JSON file with columns/keys:

```
id, name, brand, category, description, ingredients, serving_size
```

A minimal working example:

```csv
id,name,brand,category,description,ingredients
1,Vitamin C,HealthPlus,Vitamins,Antioxidant support,Ascorbic Acid
2,Fish Oil,OmegaBest,Fatty Acids,Omega-3 for heart health,EPA DHA
```
