interface Supplement {
  id: number;
  name: string;
  brand: string;
  category: string;
  description: string;
  ingredients: string;
  serving_size: string;
}

export default function SupplementCard({
  supplement,
}: {
  supplement: Supplement;
}) {
  return (
    <div className="bg-white border border-gray-200 rounded-2xl p-5 shadow-sm hover:shadow-md transition-shadow space-y-2">
      <div className="flex items-start justify-between gap-2">
        <h2 className="text-lg font-semibold">{supplement.name}</h2>
        {supplement.category && (
          <span className="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded-full whitespace-nowrap">
            {supplement.category}
          </span>
        )}
      </div>
      {supplement.brand && (
        <p className="text-sm text-gray-500">{supplement.brand}</p>
      )}
      {supplement.description && (
        <p className="text-sm text-gray-700">{supplement.description}</p>
      )}
      {supplement.ingredients && (
        <p className="text-xs text-gray-400">
          <span className="font-medium">Ingredients:</span>{" "}
          {supplement.ingredients}
        </p>
      )}
      {supplement.serving_size && (
        <p className="text-xs text-gray-400">
          <span className="font-medium">Serving size:</span>{" "}
          {supplement.serving_size}
        </p>
      )}
    </div>
  );
}
