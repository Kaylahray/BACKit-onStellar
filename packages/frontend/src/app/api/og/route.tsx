import { ImageResponse } from 'next/og';

export const runtime = 'edge';

export async function GET(request: Request) {
  return new ImageResponse(
    (
      <div style={{ display: 'flex', fontSize: 40, color: 'black', background: 'white', width: '100%', height: '100%', alignItems: 'center', justifyContent: 'center' }}>
        BACKit Prediction Market
      </div>
    ),
    { width: 1200, height: 630 }
  );
}
