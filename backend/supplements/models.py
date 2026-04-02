from django.db import models


class Supplement(models.Model):
    """A supplement product that can be searched and recommended."""

    name = models.CharField(max_length=255)
    brand = models.CharField(max_length=255, blank=True)
    category = models.CharField(max_length=100, blank=True)
    description = models.TextField(blank=True)
    ingredients = models.TextField(
        blank=True,
        help_text="Comma-separated list of active ingredients.",
    )
    serving_size = models.CharField(max_length=100, blank=True)
    # Sparse vector representation produced by the SPLADE model (stored as JSON)
    splade_vector = models.JSONField(null=True, blank=True)

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ["name"]

    def __str__(self) -> str:
        return f"{self.name} ({self.brand})" if self.brand else self.name


class SearchQuery(models.Model):
    """Stores user search queries for analytics and fine-tuning data collection."""

    query = models.CharField(max_length=512)
    results = models.JSONField(default=list)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        verbose_name_plural = "search queries"
        ordering = ["-created_at"]

    def __str__(self) -> str:
        return self.query
