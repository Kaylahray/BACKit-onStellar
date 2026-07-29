import React, { useState } from "react";

export const FeedToggle: React.FC = () => {
  const [view, setView] = useState<"grid" | "list">("grid");
  return (
    <div>
      <button onClick={() => setView("grid")}>Grid</button>
      <button onClick={() => setView("list")}>List</button>
      <p>Current View: {view}</p>
    </div>
  );
};
