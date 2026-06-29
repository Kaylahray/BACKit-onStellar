"use client";

import clsx from "clsx";

export type FeedTab = "for-you" | "following" | "bookmarked";

const TABS: { id: FeedTab; label: string }[] = [
  { id: "for-you", label: "For You" },
  { id: "following", label: "Following" },
  { id: "bookmarked", label: "Bookmarked" },
];

export default function FeedTabs({
  active,
  onChange,
}: {
  active: FeedTab;
  onChange: (tab: FeedTab) => void;
}) {
  return (
    <div className="flex border-b mb-4">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          className={clsx(
            "flex-1 p-3 text-sm font-medium",
            active === tab.id ? "border-b-2 border-black" : "text-gray-400"
          )}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
