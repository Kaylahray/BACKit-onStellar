"use client";

import { useState } from "react";
import { Bookmark } from "lucide-react";
import { useWalletContext } from "./WalletContext";
import { addBookmark, removeBookmark } from "@/lib/api";

interface BookmarkButtonProps {
  callId: string;
  /** Initial bookmark state (e.g. `call.isBookmarked` from an authed request). */
  initialBookmarked?: boolean;
  /** Initial saved count (e.g. `call.bookmarkCount`). */
  initialCount?: number;
  /** Show the "N saved" count next to the icon. */
  showCount?: boolean;
  className?: string;
}

/**
 * Accessible bookmark toggle with optimistic UI. The icon fills immediately on
 * click and reverts if the API call fails. Keyboard navigable (native button)
 * with an aria-label and aria-pressed that describe the toggle state.
 */
export default function BookmarkButton({
  callId,
  initialBookmarked = false,
  initialCount = 0,
  showCount = true,
  className = "",
}: BookmarkButtonProps) {
  const { publicKey } = useWalletContext();
  const [bookmarked, setBookmarked] = useState(initialBookmarked);
  const [count, setCount] = useState(initialCount);
  const [pending, setPending] = useState(false);

  const toggle = async (e: React.MouseEvent) => {
    // CallCard is wrapped in a Link — don't navigate when toggling.
    e.preventDefault();
    e.stopPropagation();

    if (!publicKey || pending) return;

    const nextBookmarked = !bookmarked;

    // Optimistic update.
    setBookmarked(nextBookmarked);
    setCount((c) => Math.max(0, c + (nextBookmarked ? 1 : -1)));
    setPending(true);

    try {
      if (nextBookmarked) {
        await addBookmark(publicKey, callId);
      } else {
        await removeBookmark(publicKey, callId);
      }
    } catch {
      // Revert on error.
      setBookmarked(!nextBookmarked);
      setCount((c) => Math.max(0, c + (nextBookmarked ? -1 : 1)));
    } finally {
      setPending(false);
    }
  };

  const label = !publicKey
    ? "Connect a wallet to bookmark this market"
    : bookmarked
      ? "Remove bookmark"
      : "Bookmark this market";

  return (
    <button
      type="button"
      onClick={toggle}
      disabled={!publicKey || pending}
      aria-label={label}
      aria-pressed={bookmarked}
      title={label}
      className={`inline-flex items-center gap-1 text-sm transition-colors disabled:opacity-50 ${
        bookmarked ? "text-indigo-600" : "text-gray-500 hover:text-indigo-600"
      } ${className}`}
    >
      <Bookmark
        className="w-4 h-4"
        fill={bookmarked ? "currentColor" : "none"}
        aria-hidden="true"
      />
      {showCount && (
        <span className="tabular-nums">
          {count} <span className="sr-only">markets </span>saved
        </span>
      )}
    </button>
  );
}
