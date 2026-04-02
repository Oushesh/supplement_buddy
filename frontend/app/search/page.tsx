"use client";

import { useState } from "react";
import SupplementCard from "../components/SupplementCard";

interface Supplement {
  id: number;
  name: string;
  brand: string;
  category: string;
  description: string;
  ingredients: string;
  serving_size: string;
}

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Supplement[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const apiUrl = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8000/api";

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;

    setLoading(true);
    setError(null);
    setSearched(true);

    try {
      const res = await fetch(
        `${apiUrl}/supplements/search/?q=${encodeURIComponent(query)}`
      );
      if (!res.ok) throw new Error(`API error: ${res.status}`);
      const data: Supplement[] = await res.json();
      setResults(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
      setResults([]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="max-w-3xl mx-auto px-4 py-12 space-y-8">
      <h1 className="text-3xl font-bold text-center">🔍 Search Supplements</h1>

      <form onSubmit={handleSearch} className="flex gap-2">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="e.g. vitamin C immune support, omega-3, magnesium sleep…"
          className="flex-1 border border-gray-300 rounded-xl px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          type="submit"
          disabled={loading}
          className="bg-blue-600 hover:bg-blue-700 disabled:bg-blue-300 text-white font-semibold px-6 py-2 rounded-xl transition-colors"
        >
          {loading ? "Searching…" : "Search"}
        </button>
      </form>

      {error && (
        <p className="text-red-500 text-sm text-center">⚠️ {error}</p>
      )}

      {searched && !loading && results.length === 0 && !error && (
        <p className="text-gray-500 text-center">No supplements found for "{query}".</p>
      )}

      <div className="grid gap-4">
        {results.map((supplement) => (
          <SupplementCard key={supplement.id} supplement={supplement} />
        ))}
      </div>
    </main>
  );
}
