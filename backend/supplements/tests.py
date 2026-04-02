"""Tests for the supplements API."""

from django.test import TestCase
from django.urls import reverse
from rest_framework.test import APIClient

from supplements.models import Supplement


class SupplementSearchTestCase(TestCase):
    def setUp(self):
        self.client = APIClient()
        Supplement.objects.create(
            name="Vitamin C",
            brand="HealthPlus",
            category="Vitamins",
            description="Supports immune health",
            ingredients="Ascorbic Acid",
        )
        Supplement.objects.create(
            name="Fish Oil",
            brand="OmegaBest",
            category="Fatty Acids",
            description="Omega-3 fatty acids for heart health",
            ingredients="EPA, DHA",
        )

    def test_list_supplements(self):
        url = reverse("supplement-list")
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertEqual(len(response.data["results"]), 2)

    def test_search_returns_results(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "vitamin"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Vitamin C", names)

    def test_search_requires_query(self):
        url = reverse("supplement-search")
        response = self.client.get(url)
        self.assertEqual(response.status_code, 400)

    def test_create_supplement(self):
        url = reverse("supplement-list")
        data = {
            "name": "Magnesium",
            "brand": "PureMag",
            "category": "Minerals",
            "description": "Essential mineral for muscle function",
            "ingredients": "Magnesium Glycinate",
        }
        response = self.client.post(url, data, format="json")
        self.assertEqual(response.status_code, 201)
        self.assertEqual(Supplement.objects.count(), 3)
