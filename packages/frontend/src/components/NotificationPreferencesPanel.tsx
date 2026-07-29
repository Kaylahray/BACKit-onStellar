"use client";

import { useEffect, useState } from "react";
import { Bell, BellOff, Loader2, RefreshCw } from "lucide-react";
import {
  getNotificationPreferences,
  patchNotificationPreferences,
  type NotificationPreference,
} from "@/lib/api";

interface NotifTypeMeta {
  type: string;
  label: string;
  description: string;
}

const TYPES: NotifTypeMeta[] = [
  {
    type: "CALL_CLOSING",
    label: "Call Closing Soon",
    description: "Get alerted 60 min and 15 min before a market you've staked on closes.",
  },
  {
    type: "BACKED_CALL",
    label: "Backed Call",
    description: "Someone backs a call you created.",
  },
  {
    type: "CALL_ENDED",
    label: "Call Ended",
    description: "A market you participated in has ended.",
  },
  {
    type: "PAYOUT_READY",
    label: "Payout Ready",
    description: "Your winnings are ready to claim.",
  },
  {
    type: "CALL_RESOLVED",
    label: "Call Resolved",
    description: "A market you participated in has been resolved.",
  },
  {
    type: "NEW_FOLLOWER",
    label: "New Follower",
    description: "Someone followed your profile.",
  },
  {
    type: "STAKE_UPDATE",
    label: "Stake Update",
    description: "Significant movement on a market you've staked on.",
  },
];

const CHANNELS = ["IN_APP", "EMAIL"] as const;
type Channel = (typeof CHANNELS)[number];

const CHANNEL_LABELS: Record<Channel, string> = {
  IN_APP: "In-app",
  EMAIL: "Email",
};

function isEnabled(
  prefs: NotificationPreference[],
  type: string,
  channel: Channel
): boolean {
  const p = prefs.find(
    (x) => x.notificationType === type && x.channel === channel
  );
  return p ? p.enabled : true; // default enabled when no record exists yet
}

interface Props {
  walletAddress: string;
}

export default function NotificationPreferencesPanel({
  walletAddress,
}: Props) {
  const [prefs, setPrefs] = useState<NotificationPreference[]>([]);
  const [loading, setLoading] = useState(true);
  const [savingKey, setSavingKey] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getNotificationPreferences(walletAddress);
      setPrefs(data);
    } catch {
      setError("Could not load notification preferences.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, [walletAddress]); // eslint-disable-line react-hooks/exhaustive-deps

  const toggle = async (type: string, channel: Channel) => {
    const key = `${type}:${channel}`;
    if (savingKey === key) return;

    const currentlyEnabled = isEnabled(prefs, type, channel);
    const next = !currentlyEnabled;

    // Optimistic update
    setPrefs((prev) => {
      const idx = prev.findIndex(
        (p) => p.notificationType === type && p.channel === channel
      );
      if (idx === -1) {
        return [...prev, { notificationType: type, channel, enabled: next }];
      }
      const updated = [...prev];
      updated[idx] = { ...updated[idx], enabled: next };
      return updated;
    });

    setSavingKey(key);
    try {
      const updated = await patchNotificationPreferences(walletAddress, [
        { notificationType: type, channel, enabled: next },
      ]);
      // Merge server response into local state
      setPrefs((prev) => {
        const map = new Map(
          prev.map((p) => [`${p.notificationType}:${p.channel}`, p])
        );
        for (const u of updated) {
          map.set(`${u.notificationType}:${u.channel}`, u);
        }
        return Array.from(map.values());
      });
    } catch {
      // Revert
      setPrefs((prev) => {
        const idx = prev.findIndex(
          (p) => p.notificationType === type && p.channel === channel
        );
        if (idx === -1) return prev;
        const reverted = [...prev];
        reverted[idx] = { ...reverted[idx], enabled: currentlyEnabled };
        return reverted;
      });
      setError("Failed to save. Please try again.");
    } finally {
      setSavingKey(null);
    }
  };

  return (
    <div className="bg-white rounded-2xl border border-gray-100 shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
        <div className="flex items-center gap-2">
          <Bell className="w-4 h-4 text-indigo-600" />
          <h3 className="text-sm font-semibold text-gray-900">
            Notification Preferences
          </h3>
        </div>
        <button
          onClick={load}
          disabled={loading}
          title="Refresh"
          className="text-gray-400 hover:text-gray-600 disabled:opacity-40 transition-colors"
        >
          <RefreshCw className={`w-4 h-4 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {error && (
        <div className="px-6 py-3 bg-red-50 text-xs text-red-600 border-b border-red-100">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex items-center justify-center py-12 gap-2 text-sm text-gray-400">
          <Loader2 className="w-4 h-4 animate-spin" />
          Loading preferences…
        </div>
      ) : (
        <div className="divide-y divide-gray-50">
          {/* Column header */}
          <div className="flex items-center px-6 py-2 bg-gray-50/60">
            <div className="flex-1" />
            {CHANNELS.map((ch) => (
              <div
                key={ch}
                className="w-16 text-center text-[10px] font-semibold uppercase tracking-wider text-gray-400"
              >
                {CHANNEL_LABELS[ch]}
              </div>
            ))}
          </div>

          {TYPES.map(({ type, label, description }) => {
            const isHighlighted = type === "CALL_CLOSING";
            return (
              <div
                key={type}
                className={`flex items-center px-6 py-4 ${
                  isHighlighted ? "bg-amber-50/40" : ""
                }`}
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1.5">
                    <p className="text-sm font-medium text-gray-800">{label}</p>
                    {isHighlighted && (
                      <span className="text-[10px] font-semibold px-1.5 py-0.5 rounded-full bg-amber-100 text-amber-700">
                        New
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-gray-400 mt-0.5 leading-snug">
                    {description}
                  </p>
                </div>

                {CHANNELS.map((ch) => {
                  const on = isEnabled(prefs, type, ch);
                  const key = `${type}:${ch}`;
                  const busy = savingKey === key;
                  return (
                    <div key={ch} className="w-16 flex justify-center">
                      <button
                        type="button"
                        onClick={() => toggle(type, ch)}
                        disabled={busy}
                        aria-pressed={on}
                        aria-label={`${on ? "Disable" : "Enable"} ${label} via ${CHANNEL_LABELS[ch]}`}
                        className="group relative"
                      >
                        {busy ? (
                          <Loader2 className="w-5 h-5 animate-spin text-indigo-400" />
                        ) : on ? (
                          <Bell className="w-5 h-5 text-indigo-500 group-hover:text-indigo-700 transition-colors" fill="currentColor" />
                        ) : (
                          <BellOff className="w-5 h-5 text-gray-300 group-hover:text-gray-500 transition-colors" />
                        )}
                      </button>
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
