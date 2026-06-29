"use client";

import { useEffect, useState } from "react";
import { Bell, BellRing, Clock, Loader2 } from "lucide-react";
import {
  getNotificationPreferences,
  patchNotificationPreferences,
} from "@/lib/api";

const CALL_CLOSING = "CALL_CLOSING";
const IN_APP = "IN_APP";

interface Props {
  callId: number | string;
  walletAddress?: string;
  /** Only render when the user has a stake or bookmark on this call. */
  hasStakeOrBookmark: boolean;
}

export default function ClosingAlertToggle({
  callId: _callId,
  walletAddress,
  hasStakeOrBookmark,
}: Props) {
  const [enabled, setEnabled] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!walletAddress || !hasStakeOrBookmark) {
      setLoading(false);
      return;
    }
    let cancelled = false;
    getNotificationPreferences(walletAddress)
      .then((prefs) => {
        if (cancelled) return;
        const pref = prefs.find(
          (p) => p.notificationType === CALL_CLOSING && p.channel === IN_APP
        );
        // Default to true when no explicit preference is stored yet.
        setEnabled(pref ? pref.enabled : true);
      })
      .catch(() => {
        if (!cancelled) setError("Could not load alert preference.");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [walletAddress, hasStakeOrBookmark]);

  if (!walletAddress || !hasStakeOrBookmark) return null;

  const toggle = async () => {
    if (saving) return;
    const next = !enabled;
    setEnabled(next);
    setError(null);
    setSaving(true);
    try {
      await patchNotificationPreferences(walletAddress, [
        { notificationType: CALL_CLOSING, channel: IN_APP, enabled: next },
      ]);
    } catch {
      setEnabled(!next);
      setError("Failed to update. Please try again.");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-400">
        <Loader2 className="w-4 h-4 animate-spin" />
        <span>Loading alert preference…</span>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      <button
        type="button"
        onClick={toggle}
        disabled={saving}
        aria-pressed={enabled}
        className={`inline-flex items-center gap-2 rounded-xl border px-4 py-2.5 text-sm font-medium transition-colors disabled:opacity-60 ${
          enabled
            ? "border-amber-300 bg-amber-50 text-amber-700 hover:bg-amber-100"
            : "border-gray-200 bg-white text-gray-500 hover:border-gray-300 hover:text-gray-700"
        }`}
      >
        {saving ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : enabled ? (
          <span className="relative inline-flex">
            <BellRing className="w-4 h-4" />
            <Clock className="w-2.5 h-2.5 absolute -bottom-0.5 -right-1 text-amber-500" />
          </span>
        ) : (
          <Bell className="w-4 h-4" />
        )}
        {enabled ? "Closing alerts on" : "Notify me when closing"}
      </button>
      {error && <p className="text-xs text-red-500 pl-1">{error}</p>}
    </div>
  );
}
