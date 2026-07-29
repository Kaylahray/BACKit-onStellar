export async function searchTokens(query: string) {
  if (!query) return [];

  const res = await fetch(`/api/tokens?search=${query}`);
  if (!res.ok) throw new Error("Failed to fetch tokens");

  return res.json();
}

// ── Global search ──────────────────────────────────────────────────────────

export interface SearchResultMarket {
  type: "market";
  id: string;
  title: string;
  token: string;
  outcome?: string;
  totalStake: number;
}

export interface SearchResultUser {
  type: "user";
  address: string;
  displayName?: string;
  winRate: number;
  totalCalls: number;
}

export interface SearchResultToken {
  type: "token";
  symbol: string;
  name: string;
  address: string;
  price?: number;
}

export interface SearchResponse {
  markets: SearchResultMarket[];
  users: SearchResultUser[];
  tokens: SearchResultToken[];
}

/**
 * Query the unified search endpoint.
 * GET /search?q=<query>
 */
export async function fetchSearch(query: string): Promise<SearchResponse> {
  if (!query.trim()) {
    return { markets: [], users: [], tokens: [] };
  }
  const res = await fetch(
    `${BACKEND_URL}/search?q=${encodeURIComponent(query.trim())}`
  );
  if (!res.ok) throw new Error("Search request failed");
  return res.json() as Promise<SearchResponse>;
}

export async function fetchFeed(
  type: "for-you" | "following",
  cursor?: string,
  filters?: { status: string | null }
) {
  const params = new URLSearchParams();
  params.set("type", type);
  if (cursor) params.set("cursor", cursor);
  if (filters?.status) params.set("status", filters.status);

  const res = await fetch(`/api/feed?${params.toString()}`);

  if (!res.ok) {
    throw new Error("Failed to fetch feed");
  }

  return res.json();
}

// ── Notifications ──────────────────────────────────────────────────────────

const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL || "http://localhost:3000";

export interface Notification {
  id: number;
  userId: string;
  type: "BACKED_CALL" | "CALL_ENDED" | "PAYOUT_READY" | "NEW_FOLLOWER";
  referenceId?: string;
  address?: string; // for profile
  message: string;
  readStatus: boolean;
  createdAt: string;
}

export interface NotificationsResponse {
  data: Notification[];
  totalCount: number;
  hasNext: boolean;
  unreadCount: number;
}

export async function fetchNotifications(
  userId: string,
  limit = 20,
  offset = 0
): Promise<NotificationsResponse> {
  const params = new URLSearchParams({
    userId,
    limit: String(limit),
    offset: String(offset),
  });
  const res = await fetch(`${BACKEND_URL}/notifications?${params.toString()}`);
  if (!res.ok) throw new Error("Failed to fetch notifications");
  return res.json();
}

export async function markNotificationsRead(
  userId: string,
  ids?: number[]
): Promise<{ updated: number }> {
  const params = new URLSearchParams({ userId });
  const res = await fetch(
    `${BACKEND_URL}/notifications/mark-read?${params.toString()}`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(ids ? { ids } : {}),
    }
  );
  if (!res.ok) throw new Error("Failed to mark notifications as read");
  return res.json();
}

// ── Bookmarks (#374) ─────────────────────────────────────────────────────────

export interface BookmarkRecord {
  id: string;
  userAddress: string;
  callId: string;
  createdAt: string;
  // Joined call data returned by GET /users/:address/bookmarks
  call?: Record<string, unknown> & { id: string };
}

export interface PaginatedBookmarks {
  data: BookmarkRecord[];
  total: number;
  page: number;
  limit: number;
}

/** Add a bookmark for `address` on `callId`. POST /users/:address/bookmarks */
export async function addBookmark(
  address: string,
  callId: string
): Promise<void> {
  const res = await fetch(
    `${BACKEND_URL}/users/${encodeURIComponent(address)}/bookmarks`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ callId }),
    }
  );
  // 409 means it is already bookmarked — treat as success (idempotent toggle).
  if (!res.ok && res.status !== 409) {
    throw new Error("Failed to add bookmark");
  }
}

/** Remove a bookmark. DELETE /users/:address/bookmarks/:callId */
export async function removeBookmark(
  address: string,
  callId: string
): Promise<void> {
  const res = await fetch(
    `${BACKEND_URL}/users/${encodeURIComponent(address)}/bookmarks/${encodeURIComponent(callId)}`,
    { method: "DELETE" }
  );
  // 404 means it was not bookmarked — treat as success (idempotent toggle).
  if (!res.ok && res.status !== 404) {
    throw new Error("Failed to remove bookmark");
  }
}

// ── Notification Preferences ───────────────────────────────────────────────

export interface NotificationPreference {
  notificationType: string;
  channel: string;
  enabled: boolean;
}

/** GET /users/:address/notification-preferences */
export async function getNotificationPreferences(
  address: string
): Promise<NotificationPreference[]> {
  const res = await fetch(
    `${BACKEND_URL}/users/${encodeURIComponent(address)}/notification-preferences`
  );
  if (!res.ok) return [];
  return res.json() as Promise<NotificationPreference[]>;
}

/** PATCH /users/:address/notification-preferences */
export async function patchNotificationPreferences(
  address: string,
  preferences: NotificationPreference[]
): Promise<NotificationPreference[]> {
  const res = await fetch(
    `${BACKEND_URL}/users/${encodeURIComponent(address)}/notification-preferences`,
    {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ preferences }),
    }
  );
  if (!res.ok) throw new Error("Failed to update notification preferences");
  return res.json() as Promise<NotificationPreference[]>;
}

/** Paginated list of a user's bookmarked calls (with joined call data). */
export async function fetchBookmarks(
  address: string,
  page = 1,
  limit = 20
): Promise<PaginatedBookmarks> {
  const params = new URLSearchParams({
    page: String(page),
    limit: String(limit),
  });
  const res = await fetch(
    `${BACKEND_URL}/users/${encodeURIComponent(address)}/bookmarks?${params.toString()}`
  );
  if (!res.ok) {
    throw new Error("Failed to fetch bookmarks");
  }
  return res.json() as Promise<PaginatedBookmarks>;
}
