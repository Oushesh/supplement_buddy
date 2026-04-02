from django.urls import include, path
from rest_framework.routers import DefaultRouter

from .views import SearchQueryViewSet, SupplementViewSet

router = DefaultRouter()
router.register(r"supplements", SupplementViewSet, basename="supplement")
router.register(r"search-queries", SearchQueryViewSet, basename="searchquery")

urlpatterns = [
    path("", include(router.urls)),
]
