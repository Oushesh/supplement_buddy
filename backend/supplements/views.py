"""Views for the supplements app.

Search is powered by the fine-tuned SPLADE model loaded at startup.  When the
model is not available (e.g. first run) the endpoint falls back to a simple
case-insensitive text search so the API remains usable without the GPU/model.
"""

import logging

from django.conf import settings
from django.db.models import Q
from rest_framework import status, viewsets
from rest_framework.decorators import action
from rest_framework.response import Response

from .models import SearchQuery, Supplement
from .serializers import SearchQuerySerializer, SupplementSerializer
from .splade import encode_query, find_similar_supplements

logger = logging.getLogger(__name__)


class SupplementViewSet(viewsets.ModelViewSet):
    """CRUD + semantic search for supplements."""

    queryset = Supplement.objects.all()
    serializer_class = SupplementSerializer

    @action(detail=False, methods=["get"], url_path="search")
    def search(self, request):
        """
        GET /api/supplements/search/?q=<query>

        Returns supplements ranked by SPLADE similarity.  Falls back to
        basic text search when the model is unavailable.
        """
        query = request.query_params.get("q", "").strip()
        if not query:
            return Response(
                {"detail": "Query parameter 'q' is required."},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            query_vector = encode_query(query)
            ranked_ids = find_similar_supplements(query_vector)
            supplements = Supplement.objects.filter(id__in=ranked_ids)
            # Preserve ranking order
            id_order = {id_: idx for idx, id_ in enumerate(ranked_ids)}
            supplements = sorted(supplements, key=lambda s: id_order.get(s.id, 999))
        except Exception as exc:
            logger.warning("SPLADE search failed, falling back to text search: %s", exc)
            supplements = Supplement.objects.filter(
                Q(name__icontains=query)
                | Q(description__icontains=query)
                | Q(ingredients__icontains=query)
                | Q(brand__icontains=query)
            )

        # Persist query for analytics / future fine-tuning
        SearchQuery.objects.create(
            query=query,
            results=[s.id for s in supplements],
        )

        serializer = SupplementSerializer(supplements, many=True)
        return Response(serializer.data)


class SearchQueryViewSet(viewsets.ReadOnlyModelViewSet):
    """Read-only view of past search queries (useful for analytics)."""

    queryset = SearchQuery.objects.all()
    serializer_class = SearchQuerySerializer
