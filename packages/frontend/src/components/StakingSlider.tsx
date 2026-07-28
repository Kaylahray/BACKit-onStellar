import React, { useState } from "react";

export const StakingSlider: React.FC = () => {
  const [percentage, setPercentage] = useState<number>(0);
  return (
    <div>
      <input type="range" min="0" max="100" value={percentage} onChange={(e) => setPercentage(Number(e.target.value))} />
      <button onClick={() => setPercentage(25)}>25%</button>
      <button onClick={() => setPercentage(50)}>50%</button>
      <button onClick={() => setPercentage(75)}>75%</button>
      <button onClick={() => setPercentage(100)}>Max</button>
    </div>
  );
};
