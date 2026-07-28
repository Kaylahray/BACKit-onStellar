export interface CacheConfigOptions {
  ttlFeed: number; // 30s
  ttlProfile: number; // 300s
  ttlLeaderboard: number; // 120s
}

export const defaultCacheConfigOptions: CacheConfigOptions = {
  ttlFeed: 30,
  ttlProfile: 300,
  ttlLeaderboard: 120,
};
