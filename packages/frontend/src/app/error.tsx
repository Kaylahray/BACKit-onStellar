"use client";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="min-h-[60vh] flex flex-col items-center justify-center px-4">
      <div className="text-6xl mb-4">⚠️</div>
      <h1 className="text-2xl font-bold text-gray-900 mb-2">Something Went Wrong</h1>
      <p className="text-gray-600 mb-8 text-center max-w-md">
        An unexpected error occurred. Please try again.
      </p>
      <button
        onClick={reset}
        className="px-6 py-3 bg-indigo-600 text-white rounded-lg font-medium hover:bg-indigo-700 transition-colors"
      >
        Try Again
      </button>
    </div>
  );
}
