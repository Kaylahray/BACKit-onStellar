'use client';

import { useEffect, useState } from 'react';
import { Bell, BellOff, X } from 'lucide-react';

interface Alert {
  id: string;
  targetPrice: number;
  direction: 'above' | 'below';
}

interface Props {
  callId: number | string;
  currentPrice?: number;
  walletAddress?: string;
}

export default function PriceAlertToggle({ callId, currentPrice, walletAddress }: Props) {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [open, setOpen] = useState(false);
  const [targetPrice, setTargetPrice] = useState('');
  const [direction, setDirection] = useState<'above' | 'below'>('above');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!walletAddress) return;
    fetch(`/api/users/${walletAddress}/alerts?callId=${callId}`)
      .then(r => r.ok ? r.json() : [])
      .then(data => setAlerts(Array.isArray(data) ? data : []))
      .catch(() => null);
  }, [walletAddress, callId]);

  if (!walletAddress) return null;

  const validate = (): boolean => {
    const v = parseFloat(targetPrice);
    if (isNaN(v) || v <= 0) { setError('Enter a valid positive price.'); return false; }
    setError('');
    return true;
  };

  const handleSave = async () => {
    if (!validate()) return;
    setSaving(true);
    try {
      const res = await fetch(`/api/users/${walletAddress}/alerts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ callId, targetPrice: parseFloat(targetPrice), direction }),
      });
      if (res.ok) {
        const created = await res.json();
        setAlerts(prev => [...prev, created]);
        setTargetPrice('');
        setOpen(false);
      } else {
        setError('Failed to save alert.');
      }
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (alertId: string) => {
    await fetch(`/api/users/${walletAddress}/alerts/${alertId}`, { method: 'DELETE' });
    setAlerts(prev => prev.filter(a => a.id !== alertId));
  };

  return (
    <div className="mt-4">
      <button
        onClick={() => setOpen(v => !v)}
        className="flex items-center gap-2 text-sm font-medium text-indigo-600 hover:text-indigo-800 transition-colors"
      >
        {alerts.length > 0 ? <Bell className="w-4 h-4" /> : <BellOff className="w-4 h-4" />}
        {alerts.length > 0 ? `${alerts.length} Alert${alerts.length > 1 ? 's' : ''} Set` : 'Alert me when price reaches…'}
      </button>

      {open && (
        <div className="mt-3 rounded-xl border border-indigo-100 bg-indigo-50 p-4 space-y-3">
          <div className="flex gap-2">
            <input
              type="number"
              placeholder={currentPrice ? `Current: $${currentPrice}` : 'Target price'}
              value={targetPrice}
              onChange={e => setTargetPrice(e.target.value)}
              className="flex-1 rounded-lg border border-gray-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
            />
            <select
              value={direction}
              onChange={e => setDirection(e.target.value as 'above' | 'below')}
              className="rounded-lg border border-gray-200 px-3 py-2 text-sm focus:outline-none"
            >
              <option value="above">Above</option>
              <option value="below">Below</option>
            </select>
          </div>
          {error && <p className="text-xs text-red-500">{error}</p>}
          <button
            onClick={handleSave}
            disabled={saving}
            className="w-full rounded-lg bg-indigo-600 py-2 text-sm font-semibold text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
          >
            {saving ? 'Saving…' : 'Set Alert'}
          </button>

          {alerts.length > 0 && (
            <div className="space-y-1 pt-1 border-t border-indigo-100">
              {alerts.map(a => (
                <div key={a.id} className="flex items-center justify-between text-xs text-gray-600">
                  <span>You&apos;ll be notified when price goes {a.direction} ${a.targetPrice}</span>
                  <button onClick={() => handleDelete(a.id)} className="text-gray-400 hover:text-red-500 ml-2">
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
