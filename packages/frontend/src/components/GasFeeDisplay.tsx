"use client";

import { useEffect, useState } from "react";
import { Info } from "lucide-react";

interface FeeResult {
  estimatedGasXLM: string;
  estimatedGasUSD: string;
  sponsored: boolean;
}

interface Props {
  /** Pass a transaction XDR to get a real estimate, or omit for a static fallback. */
  xdr?: string;
}

export default function GasFeeDisplay({ xdr }: Props) {
  const [fee, setFee] = useState<FeeResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(false);

    const fetchFee = async () => {
      try {
        if (xdr) {
          const res = await fetch("/api/relay/estimate-fee", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ xdr }),
          });
          if (!res.ok) throw new Error();
          const data: FeeResult = await res.json();
          if (!cancelled) setFee(data);
        } else {
          // Static fallback when no XDR available yet
          if (!cancelled) setFee({ estimatedGasXLM: "~0.001", estimatedGasUSD: "~$0.0001", sponsored: false });
        }
      } catch {
        if (!cancelled) setError(true);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    fetchFee();
    return () => { cancelled = true; };
  }, [xdr]);

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-xs text-gray-400">
        <span className="h-3 w-24 bg-gray-100 rounded animate-pulse" />
      </div>
    );
  }

  if (error || !fee) return null;

  if (fee.sponsored) {
    return (
      <p className="text-xs text-green-600 font-medium flex items-center gap-1">
        ✅ Gas Sponsored — $0.00 fee!
      </p>
    );
  }

  return (
    <p className="text-xs text-gray-500 flex items-center gap-1">
      Estimated Gas Fee: {fee.estimatedGasXLM} XLM ({fee.estimatedGasUSD} USD)
      <span title="Gas fees are charged by the Stellar network for transaction processing." className="cursor-help">
        <Info className="w-3 h-3 inline" />
      </span>
    </p>
  );
}
