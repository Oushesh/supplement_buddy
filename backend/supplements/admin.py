from django.contrib import admin

from .models import SearchQuery, Supplement


@admin.register(Supplement)
class SupplementAdmin(admin.ModelAdmin):
    list_display = ["name", "brand", "category", "created_at"]
    search_fields = ["name", "brand", "description", "ingredients"]
    list_filter = ["category"]


@admin.register(SearchQuery)
class SearchQueryAdmin(admin.ModelAdmin):
    list_display = ["query", "created_at"]
    readonly_fields = ["results"]
