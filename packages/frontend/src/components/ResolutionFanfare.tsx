import React, { useEffect, useState } from 'react';

interface ResolutionFanfareProps {
  outcome: 'YES' | 'NO';
  userWon?: boolean;
  payoutAmount?: number;
}

export const ResolutionFanfare: React.FC<ResolutionFanfareProps> = ({
  outcome,
  userWon,
  payoutAmount,
}) => {
  const [showCelebration, setShowCelebration] = useState(false);

  useEffect(() => {
    const key = `resolution_seen_${outcome}`;
    if (!sessionStorage.getItem(key)) {
      setShowCelebration(true);
      sessionStorage.setItem(key, 'true');
    }
  }, [outcome]);

  if (!showCelebration) return null;

  return (
    <div className="rounded-lg p-6 text-center bg-gradient-to-r from-emerald-500 to-teal-600 text-white shadow-lg my-4">
      {userWon ? (
        <div>
          <h2 className="text-2xl font-bold">🎉 Congratulations! You Won!</h2>
          <p className="mt-2 text-lg">Payout: +{payoutAmount || 0} USDC</p>
        </div>
      ) : (
        <div>
          <h3 className="text-xl font-semibold">Market Resolved: {outcome}</h3>
          <p className="mt-1 opacity-90">Better luck next time!</p>
        </div>
      )}
    </div>
  );
};
