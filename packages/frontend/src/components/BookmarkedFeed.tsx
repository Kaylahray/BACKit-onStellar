"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import CallCard from "./CallCard";
import { CallCardSkeleton } from "./CardCallSkeleton";
import { EmptyState } from "./EmptyState";
import { useWalletContext } from "./WalletContext";
import { fetchBookmarks } from "@/lib/api";

type BookmarkedCall = Record<string, unknown> & { id: string };

/**
 * Renders the connected user's bookmarked markets. Handles the wallet-not-
 * connected, loading, empty, and error states required by the bookmark UI.
 */
export default function BookmarkedFeed({ address }: { address?: string } = {}) {
  const { publicKey } = useWalletContext();
  // Use the explicit profile address when provided, otherwise the connected user.
  const targetAddress = address ?? publicKey;
  const [items, setItems] = useState<BookmarkedCall[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!targetAddress) return;
    let active = true;
    setLoading(true);
    setError(null);
    fetchBookmarks(targetAddress)
      .then((res) => {
        if (!active) return;
        const calls = res.data
          .map((bookmark) => bookmark.call)
          .filter((call): call is BookmarkedCall => Boolean(call));
        setItems(calls);
      })
      .catch(() => {
        if (active) setError("Failed to load your bookmarked markets.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [targetAddress]);

  if (!targetAddress) {
    return (
      <EmptyState text="Connect your wallet to see your bookmarked markets." />
    );
  }

  if (loading) {
    return (
      <div className="space-y-4 mt-4">
        {[...Array(4)].map((_, i) => (
          <CallCardSkeleton key={i} />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center text-red-600 py-8 font-medium">{error}</div>
    );
  }

  if (items.length === 0) {
    return <EmptyState text="No bookmarked markets yet" />;
  }

  return (
    <div className="space-y-4 mt-4">
      {items.map((call) => (
        <Link key={call.id} href={`/calls/${call.id}`} className="block">
          <CallCard call={call} />
        </Link>
      ))}
    </div>
  );
}
