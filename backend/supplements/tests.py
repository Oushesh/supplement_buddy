"""Tests for the supplements API."""

from unittest.mock import patch

from django.test import TestCase
from django.urls import reverse
from rest_framework.test import APIClient

from supplements.models import SearchQuery, Supplement
from supplements.serializers import SearchQuerySerializer, SupplementSerializer


# ---------------------------------------------------------------------------
# Model tests
# ---------------------------------------------------------------------------

class SupplementModelTestCase(TestCase):
    def test_str_with_brand(self):
        supplement = Supplement(name="Vitamin C", brand="HealthPlus")
        self.assertEqual(str(supplement), "Vitamin C (HealthPlus)")

    def test_str_without_brand(self):
        supplement = Supplement(name="Vitamin C", brand="")
        self.assertEqual(str(supplement), "Vitamin C")

    def test_default_fields(self):
        supplement = Supplement.objects.create(name="Test Supplement")
        self.assertEqual(supplement.brand, "")
        self.assertEqual(supplement.category, "")
        self.assertEqual(supplement.description, "")
        self.assertEqual(supplement.ingredients, "")
        self.assertEqual(supplement.serving_size, "")
        self.assertIsNone(supplement.splade_vector)
        self.assertIsNotNone(supplement.created_at)
        self.assertIsNotNone(supplement.updated_at)

    def test_ordering_is_by_name(self):
        Supplement.objects.create(name="Zinc")
        Supplement.objects.create(name="Ashwagandha")
        Supplement.objects.create(name="Magnesium")
        names = list(Supplement.objects.values_list("name", flat=True))
        self.assertEqual(names, sorted(names))


class SearchQueryModelTestCase(TestCase):
    def test_str(self):
        sq = SearchQuery(query="vitamin C")
        self.assertEqual(str(sq), "vitamin C")

    def test_default_results_is_empty_list(self):
        sq = SearchQuery.objects.create(query="test")
        self.assertEqual(sq.results, [])

    def test_ordering_is_most_recent_first(self):
        SearchQuery.objects.create(query="first")
        SearchQuery.objects.create(query="second")
        queries = list(SearchQuery.objects.values_list("query", flat=True))
        self.assertEqual(queries[0], "second")
        self.assertEqual(queries[1], "first")


# ---------------------------------------------------------------------------
# Serializer tests
# ---------------------------------------------------------------------------

class SupplementSerializerTestCase(TestCase):
    def setUp(self):
        self.supplement = Supplement.objects.create(
            name="Zinc",
            brand="ZinCo",
            category="Minerals",
            description="Immune support",
            ingredients="Zinc Gluconate",
            serving_size="15mg",
        )

    def test_contains_expected_fields(self):
        data = SupplementSerializer(self.supplement).data
        for field in ("id", "name", "brand", "category", "description",
                      "ingredients", "serving_size", "created_at", "updated_at"):
            self.assertIn(field, data)

    def test_values_are_correct(self):
        data = SupplementSerializer(self.supplement).data
        self.assertEqual(data["name"], "Zinc")
        self.assertEqual(data["brand"], "ZinCo")
        self.assertEqual(data["serving_size"], "15mg")

    def test_splade_vector_not_in_fields(self):
        data = SupplementSerializer(self.supplement).data
        self.assertNotIn("splade_vector", data)


class SearchQuerySerializerTestCase(TestCase):
    def setUp(self):
        self.sq = SearchQuery.objects.create(query="magnesium", results=[1, 2, 3])

    def test_contains_expected_fields(self):
        data = SearchQuerySerializer(self.sq).data
        for field in ("id", "query", "results", "created_at"):
            self.assertIn(field, data)

    def test_values_are_correct(self):
        data = SearchQuerySerializer(self.sq).data
        self.assertEqual(data["query"], "magnesium")
        self.assertEqual(data["results"], [1, 2, 3])


# ---------------------------------------------------------------------------
# Supplement CRUD API tests
# ---------------------------------------------------------------------------

class SupplementCRUDTestCase(TestCase):
    def setUp(self):
        self.client = APIClient()
        self.supplement = Supplement.objects.create(
            name="Vitamin D",
            brand="SunVit",
            category="Vitamins",
            description="Bone health support",
            ingredients="Cholecalciferol",
            serving_size="1000IU",
        )

    def test_retrieve_supplement(self):
        url = reverse("supplement-detail", args=[self.supplement.id])
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.data["name"], "Vitamin D")
        self.assertEqual(response.data["brand"], "SunVit")

    def test_retrieve_nonexistent_supplement_returns_404(self):
        url = reverse("supplement-detail", args=[99999])
        response = self.client.get(url)
        self.assertEqual(response.status_code, 404)

    def test_update_supplement(self):
        url = reverse("supplement-detail", args=[self.supplement.id])
        data = {
            "name": "Vitamin D3",
            "brand": "SunVit",
            "category": "Vitamins",
            "description": "Bone and immune health support",
            "ingredients": "Cholecalciferol",
            "serving_size": "2000IU",
        }
        response = self.client.put(url, data, format="json")
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.data["name"], "Vitamin D3")
        self.assertEqual(response.data["serving_size"], "2000IU")

    def test_partial_update_supplement(self):
        url = reverse("supplement-detail", args=[self.supplement.id])
        response = self.client.patch(url, {"brand": "MegaVit"}, format="json")
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.data["brand"], "MegaVit")
        self.assertEqual(response.data["name"], "Vitamin D")

    def test_delete_supplement(self):
        url = reverse("supplement-detail", args=[self.supplement.id])
        response = self.client.delete(url)
        self.assertEqual(response.status_code, 204)
        self.assertFalse(Supplement.objects.filter(id=self.supplement.id).exists())

    def test_list_pagination_returns_count_and_results(self):
        url = reverse("supplement-list")
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertIn("count", response.data)
        self.assertIn("results", response.data)
        self.assertEqual(response.data["count"], 1)


# ---------------------------------------------------------------------------
# Original search tests (preserved)
# ---------------------------------------------------------------------------

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


# ---------------------------------------------------------------------------
# Advanced search tests
# ---------------------------------------------------------------------------

class SupplementSearchAdvancedTestCase(TestCase):
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
        Supplement.objects.create(
            name="Zinc Gluconate",
            brand="ZincCo",
            category="Minerals",
            description="Essential mineral for immune function",
            ingredients="Zinc Gluconate",
        )

    def test_search_by_brand(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "OmegaBest"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Fish Oil", names)

    def test_search_by_description(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "heart health"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Fish Oil", names)

    def test_search_by_ingredients(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "Ascorbic"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Vitamin C", names)

    def test_search_case_insensitive(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "VITAMIN"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Vitamin C", names)

    def test_search_no_results(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "xyznonexistentterm"})
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.data, [])

    def test_search_whitespace_only_query_returns_400(self):
        url = reverse("supplement-search")
        response = self.client.get(url, {"q": "   "})
        self.assertEqual(response.status_code, 400)

    def test_search_persists_search_query(self):
        url = reverse("supplement-search")
        self.client.get(url, {"q": "zinc"})
        self.assertEqual(SearchQuery.objects.count(), 1)
        sq = SearchQuery.objects.first()
        self.assertEqual(sq.query, "zinc")

    def test_search_stores_result_ids_in_search_query(self):
        url = reverse("supplement-search")
        self.client.get(url, {"q": "vitamin"})
        sq = SearchQuery.objects.first()
        vitamin_c = Supplement.objects.get(name="Vitamin C")
        self.assertIn(vitamin_c.id, sq.results)

    def test_search_no_match_stores_empty_results(self):
        url = reverse("supplement-search")
        self.client.get(url, {"q": "xyznonexistentterm"})
        sq = SearchQuery.objects.first()
        self.assertEqual(sq.results, [])

    def test_search_splade_failure_falls_back_to_text(self):
        """When SPLADE raises, the view falls back to text search gracefully."""
        url = reverse("supplement-search")
        # encode_query raises RuntimeError when torch is absent (as in test env)
        response = self.client.get(url, {"q": "fish"})
        self.assertEqual(response.status_code, 200)
        names = [s["name"] for s in response.data]
        self.assertIn("Fish Oil", names)


# ---------------------------------------------------------------------------
# Search Query API tests
# ---------------------------------------------------------------------------

class SearchQueryAPITestCase(TestCase):
    def setUp(self):
        self.client = APIClient()
        SearchQuery.objects.create(query="vitamin", results=[1, 2])
        SearchQuery.objects.create(query="omega", results=[3])

    def test_list_search_queries(self):
        url = reverse("searchquery-list")
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertEqual(response.data["count"], 2)

    def test_list_search_queries_returns_paginated_response(self):
        url = reverse("searchquery-list")
        response = self.client.get(url)
        self.assertIn("results", response.data)
        self.assertIn("count", response.data)

    def test_retrieve_search_query(self):
        sq = SearchQuery.objects.first()
        url = reverse("searchquery-detail", args=[sq.id])
        response = self.client.get(url)
        self.assertEqual(response.status_code, 200)
        self.assertIn("query", response.data)

    def test_search_queries_post_not_allowed(self):
        url = reverse("searchquery-list")
        response = self.client.post(url, {"query": "test"}, format="json")
        self.assertEqual(response.status_code, 405)

    def test_search_queries_delete_not_allowed(self):
        sq = SearchQuery.objects.first()
        url = reverse("searchquery-detail", args=[sq.id])
        response = self.client.delete(url)
        self.assertEqual(response.status_code, 405)


# ---------------------------------------------------------------------------
# SPLADE module unit tests
# ---------------------------------------------------------------------------

class SpladeModuleTestCase(TestCase):
    def setUp(self):
        self.s1 = Supplement.objects.create(
            name="Vitamin C",
            splade_vector={"vitamin": 2.0, "immune": 1.5, "ascorbic": 1.0},
        )
        self.s2 = Supplement.objects.create(
            name="Fish Oil",
            splade_vector={"omega": 2.0, "fatty": 1.0, "dha": 1.5},
        )
        self.s3 = Supplement.objects.create(
            name="Zinc",
            splade_vector=None,
        )

    def test_find_similar_supplements_returns_matching_ids(self):
        from supplements.splade import find_similar_supplements

        results = find_similar_supplements({"vitamin": 1.0, "immune": 0.5})
        self.assertIn(self.s1.id, results)
        self.assertNotIn(self.s2.id, results)

    def test_find_similar_supplements_skips_null_vectors(self):
        from supplements.splade import find_similar_supplements

        results = find_similar_supplements({"vitamin": 1.0})
        self.assertNotIn(self.s3.id, results)

    def test_find_similar_supplements_empty_db(self):
        from supplements.splade import find_similar_supplements

        Supplement.objects.all().delete()
        self.assertEqual(find_similar_supplements({"vitamin": 1.0}), [])

    def test_find_similar_supplements_respects_top_k(self):
        from supplements.splade import find_similar_supplements

        for i in range(5):
            Supplement.objects.create(
                name=f"Extra {i}",
                splade_vector={"vitamin": float(i + 1)},
            )
        results = find_similar_supplements({"vitamin": 1.0}, top_k=3)
        self.assertLessEqual(len(results), 3)

    def test_find_similar_supplements_ranking_order(self):
        from supplements.splade import find_similar_supplements

        # s1 dot {"vitamin": 1.0, "immune": 1.0} = 2.0 + 1.5 = 3.5
        # s2 has no overlap
        results = find_similar_supplements({"vitamin": 1.0, "immune": 1.0})
        self.assertEqual(results[0], self.s1.id)

    def test_find_similar_supplements_no_overlap_returns_empty(self):
        from supplements.splade import find_similar_supplements

        results = find_similar_supplements({"xyztoken": 1.0})
        self.assertEqual(results, [])

    def test_encode_query_raises_when_torch_unavailable(self):
        from supplements.splade import encode_query

        # torch is not installed in the test environment; the function raises
        # ImportError (ModuleNotFoundError) for a missing import or RuntimeError
        # when the model itself cannot be loaded.
        with self.assertRaises((RuntimeError, ImportError)):
            encode_query("test query")

    def test_load_model_raises_runtime_error_without_transformers(self):
        """_load_model() propagates import errors as RuntimeError."""
        import supplements.splade as splade_module

        # Reset cached model so _load_model() runs again
        original_model = splade_module._model
        original_tokenizer = splade_module._tokenizer
        splade_module._model = None
        splade_module._tokenizer = None
        try:
            with patch.dict("sys.modules", {"torch": None, "transformers": None}):
                with self.assertRaises(RuntimeError):
                    splade_module._load_model()
        finally:
            splade_module._model = original_model
            splade_module._tokenizer = original_tokenizer
