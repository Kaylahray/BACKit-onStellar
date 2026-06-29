import {
  ConflictException,
  Injectable,
  NotFoundException,
} from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { In, Repository } from 'typeorm';
import { Bookmark } from './bookmarks.entity';
import { Call } from '../calls/entities/call.entity';

export interface PaginatedBookmarks {
  data: Bookmark[];
  total: number;
  page: number;
  limit: number;
}

@Injectable()
export class BookmarksService {
  constructor(
    @InjectRepository(Bookmark)
    private readonly bookmarksRepo: Repository<Bookmark>,
    @InjectRepository(Call)
    private readonly callsRepo: Repository<Call>,
  ) {}

  /**
   * Adds a bookmark for `userAddress` on `callId`.
   * @throws NotFoundException when the call does not exist.
   * @throws ConflictException (409) when the bookmark already exists.
   */
  async addBookmark(userAddress: string, callId: string): Promise<Bookmark> {
    const call = await this.callsRepo.findOne({ where: { id: callId } });
    if (!call) {
      throw new NotFoundException('Call not found');
    }

    const existing = await this.bookmarksRepo.findOne({
      where: { userAddress, callId },
    });
    if (existing) {
      throw new ConflictException('Market already bookmarked');
    }

    return this.bookmarksRepo.save(
      this.bookmarksRepo.create({ userAddress, callId }),
    );
  }

  /**
   * Removes a user's bookmark for a call.
   * @throws NotFoundException when no such bookmark exists.
   */
  async removeBookmark(userAddress: string, callId: string): Promise<void> {
    const bookmark = await this.bookmarksRepo.findOne({
      where: { userAddress, callId },
    });
    if (!bookmark) {
      throw new NotFoundException('Bookmark not found');
    }
    await this.bookmarksRepo.remove(bookmark);
  }

  /**
   * Paginated list of a user's bookmarks with full joined call data,
   * most-recent first.
   */
  async getBookmarks(
    userAddress: string,
    page = 1,
    limit = 20,
  ): Promise<PaginatedBookmarks> {
    const safePage = page < 1 ? 1 : page;
    const safeLimit = limit < 1 ? 20 : limit;

    const [data, total] = await this.bookmarksRepo.findAndCount({
      where: { userAddress },
      relations: ['call'],
      order: { createdAt: 'DESC' },
      skip: (safePage - 1) * safeLimit,
      take: safeLimit,
    });

    return { data, total, page: safePage, limit: safeLimit };
  }

  /** Number of users who have bookmarked a call. */
  async getBookmarkCount(callId: string): Promise<number> {
    return this.bookmarksRepo.count({ where: { callId } });
  }

  /** Bookmark counts for several calls at once, keyed by callId. */
  async getBookmarkCounts(callIds: string[]): Promise<Record<string, number>> {
    const counts: Record<string, number> = {};
    if (callIds.length === 0) {
      return counts;
    }
    const rows = await this.bookmarksRepo
      .createQueryBuilder('bookmark')
      .select('bookmark.callId', 'callId')
      .addSelect('COUNT(*)', 'count')
      .where('bookmark.callId IN (:...callIds)', { callIds })
      .groupBy('bookmark.callId')
      .getRawMany<{ callId: string; count: string }>();
    for (const row of rows) {
      counts[row.callId] = Number(row.count);
    }
    return counts;
  }

  /** Whether `userAddress` has bookmarked `callId`. */
  async isBookmarked(userAddress: string, callId: string): Promise<boolean> {
    if (!userAddress) {
      return false;
    }
    const count = await this.bookmarksRepo.count({
      where: { userAddress, callId },
    });
    return count > 0;
  }

  /**
   * Subset of `callIds` that `userAddress` has bookmarked. Useful for tagging a
   * list of calls with `isBookmarked` in a single query.
   */
  async getBookmarkedCallIds(
    userAddress: string,
    callIds: string[],
  ): Promise<Set<string>> {
    if (!userAddress || callIds.length === 0) {
      return new Set();
    }
    const rows = await this.bookmarksRepo.find({
      where: { userAddress, callId: In(callIds) },
      select: ['callId'],
    });
    return new Set(rows.map((row) => row.callId));
  }
}
