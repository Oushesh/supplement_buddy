import Link from "next/link";

export default function Home() {
  return (
    <main className="flex flex-col items-center justify-center min-h-screen p-8">
      <div className="max-w-2xl w-full text-center space-y-6">
        <h1 className="text-5xl font-bold tracking-tight">💊 Supplement Buddy</h1>
        <p className="text-xl text-gray-600">
          AI-powered supplement search using SPLADE sparse retrieval. Find the
          exact supplement you need — fast.
        </p>
        <Link
          href="/search"
          className="inline-block bg-blue-600 hover:bg-blue-700 text-white font-semibold px-8 py-3 rounded-xl transition-colors"
        >
          Search Supplements
        </Link>
      </div>
    </main>
  );
}
