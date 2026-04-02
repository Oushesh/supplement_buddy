# Supplement Buddy

AI-powered supplement search MVP.  
Semantic search is driven by a SPLADE sparse retrieval model fine-tuned on supplement data starting from [`thierrydamiba/splade-ecommerce-esci`](https://huggingface.co/thierrydamiba/splade-ecommerce-esci).

## Project Structure

```
supplement_buddy/
├── backend/                      # Django REST API
│   ├── supplement_buddy/         # Django project (settings, urls, wsgi)
│   ├── supplements/              # Supplements app (models, views, search)
│   │   ├── models.py             # Supplement + SearchQuery models
│   │   ├── views.py              # REST viewsets + /search endpoint
│   │   ├── splade.py             # SPLADE encode/retrieve helpers
│   │   ├── serializers.py
│   │   ├── urls.py
│   │   └── tests.py
│   ├── requirements.txt
│   ├── Dockerfile
│   └── .env.example
├── frontend/                     # Next.js 14 (App Router) frontend
│   ├── app/
│   │   ├── page.tsx              # Landing page
│   │   ├── search/page.tsx       # Supplement search UI
│   │   ├── components/
│   │   │   └── SupplementCard.tsx
│   │   └── globals.css
│   ├── package.json
│   ├── next.config.js
│   └── tsconfig.json
├── scripts/
│   └── finetune/                 # SLM fine-tuning scripts
│       ├── prepare_data.py       # Build training triplets from supplement catalogue
│       ├── train_splade.py       # Fine-tune SPLADE on supplement data
│       ├── index_supplements.py  # Pre-compute + store SPLADE vectors in Django DB
│       ├── requirements.txt
│       └── README.md
├── docker-compose.yml
└── README.md
```

## Getting Started

### With Docker Compose (recommended)

```bash
# 1. Copy and edit the backend environment file
cp backend/.env.example backend/.env

# 2. Start all services
docker-compose up --build

# 3. Run Django migrations (first time only)
docker-compose exec backend python manage.py migrate
docker-compose exec backend python manage.py createsuperuser
```

- **Frontend:** http://localhost:3000  
- **Backend API:** http://localhost:8000/api  
- **Django Admin:** http://localhost:8000/admin

### Local development (without Docker)

#### Backend

```bash
cd backend
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env   # edit as needed
python manage.py migrate
python manage.py runserver
```

#### Frontend

```bash
cd frontend
npm install
npm run dev
```

## Fine-tuning the SPLADE Model

See [`scripts/finetune/README.md`](scripts/finetune/README.md) for the full workflow:

1. `prepare_data.py` — convert your supplement catalogue to training triplets  
2. `train_splade.py` — fine-tune from the ESCI e-commerce checkpoint  
3. `index_supplements.py` — precompute & store SPLADE vectors in the DB  

Set `SPLADE_MODEL_PATH` in `backend/.env` to point at the fine-tuned checkpoint.

## API Endpoints

| Method | URL | Description |
|--------|-----|-------------|
| GET | `/api/supplements/` | List all supplements (paginated) |
| POST | `/api/supplements/` | Create a supplement |
| GET | `/api/supplements/<id>/` | Retrieve a supplement |
| GET | `/api/supplements/search/?q=<query>` | SPLADE semantic search |
| GET | `/api/search-queries/` | List past search queries |

