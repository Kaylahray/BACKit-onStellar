import { ImageResponse } from 'next/og';
import { NextRequest } from 'next/server';

export const runtime = 'edge';

export async function GET(req: NextRequest) {
  const { searchParams } = req.nextUrl;
  const id = searchParams.get('id');
  const title = searchParams.get('title') ?? 'Prediction Market';
  const pair = searchParams.get('pair') ?? 'XLM/USDC';
  const pool = searchParams.get('pool') ?? '0';
  const yes = searchParams.get('yes') ?? '50';
  const timeLeft = searchParams.get('timeLeft') ?? '';

  // Fetch live data if call id provided
  let displayTitle = title;
  let displayPair = pair;
  let displayPool = pool;
  let displayYes = yes;
  let displayTimeLeft = timeLeft;

  if (id) {
    try {
      const base = req.nextUrl.origin;
      const res = await fetch(`${base}/api/calls/${id}`, { next: { revalidate: 60 } });
      if (res.ok) {
        const data = await res.json();
        displayTitle = data.title ?? title;
        displayPair = data.pairId ?? pair;
        const totalYes = Number(data.totalYesStake ?? 0);
        const totalNo = Number(data.totalNoStake ?? 0);
        const total = totalYes + totalNo;
        displayPool = total.toFixed(0);
        displayYes = total > 0 ? Math.round((totalYes / total) * 100).toString() : '50';
        if (data.expiresAt) {
          const diff = new Date(data.expiresAt).getTime() - Date.now();
          if (diff > 0) {
            const h = Math.floor(diff / 3600000);
            displayTimeLeft = h > 0 ? `${h}h left` : `${Math.floor(diff / 60000)}m left`;
          } else {
            displayTimeLeft = 'Ended';
          }
        }
      }
    } catch {
      // use fallback values
    }
  }

  const noPercent = 100 - parseInt(displayYes);

  return new ImageResponse(
    (
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          width: '100%',
          height: '100%',
          background: '#080b14',
          padding: '48px',
          fontFamily: 'sans-serif',
          color: 'white',
        }}
      >
        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '32px' }}>
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '40px', height: '40px',
            borderRadius: '10px',
            background: 'linear-gradient(135deg, #22c55e, #3b82f6)',
          }}>
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5">
              <polyline points="22 7 13.5 15.5 8.5 10.5 2 17" />
              <polyline points="16 7 22 7 22 13" />
            </svg>
          </div>
          <span style={{ fontSize: '20px', fontWeight: 700, letterSpacing: '-0.5px' }}>
            BACK<span style={{ color: '#22c55e' }}>IT</span>
          </span>
          <span style={{ marginLeft: 'auto', fontSize: '13px', color: '#6b7280', background: '#1f2937', borderRadius: '20px', padding: '4px 12px' }}>
            Prediction Market on Stellar
          </span>
        </div>

        {/* Title */}
        <div style={{ fontSize: '36px', fontWeight: 800, lineHeight: 1.2, marginBottom: '16px', maxWidth: '900px' }}>
          {displayTitle}
        </div>

        {/* Pair + Pool */}
        <div style={{ display: 'flex', gap: '24px', marginBottom: '28px' }}>
          <span style={{ background: '#1f2937', borderRadius: '8px', padding: '6px 14px', fontSize: '15px', color: '#d1d5db' }}>
            {displayPair}
          </span>
          <span style={{ background: '#1f2937', borderRadius: '8px', padding: '6px 14px', fontSize: '15px', color: '#d1d5db' }}>
            Pool: {parseFloat(displayPool).toLocaleString()} USDC
          </span>
          {displayTimeLeft && (
            <span style={{ background: '#1f2937', borderRadius: '8px', padding: '6px 14px', fontSize: '15px', color: '#d1d5db' }}>
              ⏱ {displayTimeLeft}
            </span>
          )}
        </div>

        {/* UP/DOWN split bar */}
        <div style={{ display: 'flex', height: '16px', borderRadius: '8px', overflow: 'hidden', marginBottom: '10px', background: '#1f2937' }}>
          <div style={{ width: `${displayYes}%`, background: '#22c55e' }} />
          <div style={{ width: `${noPercent}%`, background: '#ef4444' }} />
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '13px' }}>
          <span style={{ color: '#22c55e' }}>▲ UP {displayYes}%</span>
          <span style={{ color: '#ef4444' }}>▼ DOWN {noPercent}%</span>
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 630,
    },
  );
}
