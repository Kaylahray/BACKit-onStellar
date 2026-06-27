'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { CheckCircle, XCircle, Clock, Users } from 'lucide-react';

interface CreatedCall {
  id: string;
  title: string;
  status: string;
  createdAt: string;
  resolvedAt?: string;
  expiresAt?: string;
  totalYesStake: string;
  totalNoStake: string;
  uniqueStakers: string;
}

interface Stats {
  totalCreated: number;
  totalResolved: number;
  totalStakeVolumeAttracted: number;
}

interface Props {
  address: string;
}

const STATUS_BADGE: Record<string, { label: string; cls: string; icon: React.ReactNode }> = {
  OPEN: { label: 'Open', cls: 'bg-green-100 text-green-700', icon: <Clock className="w-3 h-3" /> },
  RESOLVED_YES: { label: 'Resolved', cls: 'bg-blue-100 text-blue-700', icon: <CheckCircle className="w-3 h-3" /> },
  RESOLVED_NO: { label: 'Resolved', cls: 'bg-blue-100 text-blue-700', icon: <CheckCircle className="w-3 h-3" /> },
  CANCELLED: { label: 'Cancelled', cls: 'bg-red-100 text-red-700', icon: <XCircle className="w-3 h-3" /> },
};

export default function CreatorDashboard({ address }: Props) {
  const [calls, setCalls] = useState<CreatedCall[]>([]);
  const [stats, setStats] = useState<Stats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      fetch(`/api/users/${address}/created-calls`).then(r => r.json()),
      fetch(`/api/users/${address}/created-calls/stats`).then(r => r.json()),
    ])
      .then(([callsRes, statsRes]) => {
        setCalls(callsRes.data ?? []);
        setStats(statsRes);
      })
      .finally(() => setLoading(false));
  }, [address]);

  if (loading) return <div className="py-8 text-center text-gray-400 animate-pulse">Loading markets…</div>;

  if (!calls.length) {
    return (
      <div className="py-12 text-center">
        <p className="text-gray-500 mb-4">You haven&apos;t created any markets yet.</p>
        <Link href="/create" className="inline-block rounded-xl bg-indigo-600 px-5 py-2.5 text-sm font-semibold text-white hover:bg-indigo-700">
          Create your first prediction
        </Link>
      </div>
    );
  }

  const totalPool = (c: CreatedCall) => parseFloat(c.totalYesStake) + parseFloat(c.totalNoStake);

  return (
    <div className="space-y-5">
      {stats && (
        <div className="grid grid-cols-3 gap-3 rounded-xl border border-gray-200 bg-gray-50 p-4 text-center text-sm">
          <div><p className="text-2xl font-bold text-gray-900">{stats.totalCreated}</p><p className="text-gray-500">Created</p></div>
          <div><p className="text-2xl font-bold text-gray-900">{stats.totalResolved}</p><p className="text-gray-500">Resolved</p></div>
          <div><p className="text-2xl font-bold text-gray-900">{stats.totalStakeVolumeAttracted.toFixed(0)}</p><p className="text-gray-500">Volume</p></div>
        </div>
      )}

      <div className="grid gap-3 sm:grid-cols-2">
        {calls.map(call => {
          const badge = STATUS_BADGE[call.status] ?? STATUS_BADGE.OPEN;
          return (
            <Link key={call.id} href={`/calls/${call.id}`} className="rounded-xl border border-gray-200 p-4 hover:shadow-md transition-shadow block">
              <div className="flex items-start justify-between gap-2 mb-2">
                <p className="font-medium text-gray-900 text-sm leading-snug">{call.title}</p>
                <span className={`flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium flex-shrink-0 ${badge.cls}`}>
                  {badge.icon}{badge.label}
                </span>
              </div>
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <span className="flex items-center gap-1"><Users className="w-3 h-3" />{call.uniqueStakers}</span>
                <span>Pool: {totalPool(call).toFixed(2)} USDC</span>
              </div>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
