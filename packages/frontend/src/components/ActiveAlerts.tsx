"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { X, Bell } from "lucide-react";

interface Alert {
  id: string;
  callId: string | number;
  targetPrice: number;
  direction: "above" | "below";
  callTitle?: string;
  currentPrice?: number;
}

export default function ActiveAlerts({ walletAddress }: { walletAddress: string }) {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(`/api/users/${walletAddress}/alerts`)
      .then((r) => (r.ok ? r.json() : []))
      .then((d) => setAlerts(Array.isArray(d) ? d : []))
      .catch(() => null)
      .finally(() => setLoading(false));
  }, [walletAddress]);

  if (loading) return <div className="h-8 w-40 bg-gray-100 rounded animate-pulse" />;
  if (!alerts.length) return null;

  const remove = async (id: string) => {
    await fetch(`/api/users/${walletAddress}/alerts/${id}`, { method: "DELETE" });
    setAlerts((prev) => prev.filter((a) => a.id !== id));
  };

  return (
    <div className="mt-6">
      <h3 className="flex items-center gap-2 text-sm font-semibold text-gray-700 mb-3">
        <Bell className="w-4 h-4" /> Active Alerts
      </h3>
      <div className="space-y-2">
        {alerts.map((a) => (
          <div key={a.id} className="flex items-center justify-between rounded-xl border border-indigo-100 bg-indigo-50 px-4 py-2 text-sm">
            <div className="flex flex-col">
              {a.callTitle && (
                <Link href={`/calls/${a.callId}`} className="font-medium text-indigo-700 hover:underline text-xs truncate max-w-xs">
                  {a.callTitle}
                </Link>
              )}
              <span className="text-gray-600 text-xs">
                Notify when price goes <strong>{a.direction}</strong> ${a.targetPrice}
                {a.currentPrice != null && (
                  <span className="ml-2 text-gray-400">(current: ${a.currentPrice})</span>
                )}
              </span>
            </div>
            <button onClick={() => remove(a.id)} className="ml-3 text-gray-400 hover:text-red-500 flex-shrink-0">
              <X className="w-4 h-4" />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
