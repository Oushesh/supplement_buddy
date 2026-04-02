from rest_framework import serializers

from .models import SearchQuery, Supplement


class SupplementSerializer(serializers.ModelSerializer):
    class Meta:
        model = Supplement
        fields = [
            "id",
            "name",
            "brand",
            "category",
            "description",
            "ingredients",
            "serving_size",
            "created_at",
            "updated_at",
        ]


class SearchQuerySerializer(serializers.ModelSerializer):
    class Meta:
        model = SearchQuery
        fields = ["id", "query", "results", "created_at"]
        read_only_fields = ["results", "created_at"]
