import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { ConflictException, NotFoundException } from '@nestjs/common';
import { BookmarksService } from './bookmarks.service';
import { Bookmark } from './bookmarks.entity';
import { Call } from '../calls/entities/call.entity';

const USER = 'GUSER000000000000000000000000000000000000000000000000000';
const CALL_ID = '7c9e6679-7425-40de-944b-e07fc1f90ae7';

describe('BookmarksService', () => {
  let service: BookmarksService;

  const bookmarksRepo = {
    findOne: jest.fn(),
    create: jest.fn(),
    save: jest.fn(),
    remove: jest.fn(),
    findAndCount: jest.fn(),
    count: jest.fn(),
    find: jest.fn(),
    createQueryBuilder: jest.fn(),
  };

  const callsRepo = {
    findOne: jest.fn(),
  };

  beforeEach(async () => {
    jest.clearAllMocks();
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        BookmarksService,
        { provide: getRepositoryToken(Bookmark), useValue: bookmarksRepo },
        { provide: getRepositoryToken(Call), useValue: callsRepo },
      ],
    }).compile();

    service = module.get<BookmarksService>(BookmarksService);
  });

  describe('addBookmark', () => {
    it('creates a bookmark when the call exists and is not already bookmarked', async () => {
      callsRepo.findOne.mockResolvedValue({ id: CALL_ID });
      bookmarksRepo.findOne.mockResolvedValue(null);
      const created = { userAddress: USER, callId: CALL_ID };
      bookmarksRepo.create.mockReturnValue(created);
      bookmarksRepo.save.mockResolvedValue({ id: 'bm-1', ...created });

      const result = await service.addBookmark(USER, CALL_ID);

      expect(bookmarksRepo.create).toHaveBeenCalledWith({
        userAddress: USER,
        callId: CALL_ID,
      });
      expect(bookmarksRepo.save).toHaveBeenCalledWith(created);
      expect(result).toEqual({ id: 'bm-1', ...created });
    });

    it('throws NotFoundException when the call does not exist', async () => {
      callsRepo.findOne.mockResolvedValue(null);

      await expect(service.addBookmark(USER, CALL_ID)).rejects.toBeInstanceOf(
        NotFoundException,
      );
      expect(bookmarksRepo.save).not.toHaveBeenCalled();
    });

    it('throws ConflictException (409) on a duplicate bookmark', async () => {
      callsRepo.findOne.mockResolvedValue({ id: CALL_ID });
      bookmarksRepo.findOne.mockResolvedValue({ id: 'bm-existing' });

      await expect(service.addBookmark(USER, CALL_ID)).rejects.toBeInstanceOf(
        ConflictException,
      );
      expect(bookmarksRepo.save).not.toHaveBeenCalled();
    });
  });

  describe('removeBookmark', () => {
    it('removes an existing bookmark', async () => {
      const bookmark = { id: 'bm-1', userAddress: USER, callId: CALL_ID };
      bookmarksRepo.findOne.mockResolvedValue(bookmark);
      bookmarksRepo.remove.mockResolvedValue(bookmark);

      await service.removeBookmark(USER, CALL_ID);

      expect(bookmarksRepo.remove).toHaveBeenCalledWith(bookmark);
    });

    it('throws NotFoundException when the bookmark does not exist', async () => {
      bookmarksRepo.findOne.mockResolvedValue(null);

      await expect(
        service.removeBookmark(USER, CALL_ID),
      ).rejects.toBeInstanceOf(NotFoundException);
      expect(bookmarksRepo.remove).not.toHaveBeenCalled();
    });
  });

  describe('getBookmarks', () => {
    it('returns a paginated list with joined call data', async () => {
      const rows = [{ id: 'bm-1', callId: CALL_ID, call: { id: CALL_ID } }];
      bookmarksRepo.findAndCount.mockResolvedValue([rows, 1]);

      const result = await service.getBookmarks(USER, 2, 10);

      expect(bookmarksRepo.findAndCount).toHaveBeenCalledWith({
        where: { userAddress: USER },
        relations: ['call'],
        order: { createdAt: 'DESC' },
        skip: 10, // (page 2 - 1) * limit 10
        take: 10,
      });
      expect(result).toEqual({ data: rows, total: 1, page: 2, limit: 10 });
    });

    it('clamps invalid page/limit to safe defaults', async () => {
      bookmarksRepo.findAndCount.mockResolvedValue([[], 0]);

      const result = await service.getBookmarks(USER, 0, 0);

      expect(bookmarksRepo.findAndCount).toHaveBeenCalledWith(
        expect.objectContaining({ skip: 0, take: 20 }),
      );
      expect(result).toEqual({ data: [], total: 0, page: 1, limit: 20 });
    });
  });

  describe('getBookmarkCount', () => {
    it('counts bookmarks for a call', async () => {
      bookmarksRepo.count.mockResolvedValue(5);
      await expect(service.getBookmarkCount(CALL_ID)).resolves.toBe(5);
      expect(bookmarksRepo.count).toHaveBeenCalledWith({
        where: { callId: CALL_ID },
      });
    });
  });

  describe('getBookmarkCounts', () => {
    it('returns an empty map for an empty input', async () => {
      await expect(service.getBookmarkCounts([])).resolves.toEqual({});
      expect(bookmarksRepo.createQueryBuilder).not.toHaveBeenCalled();
    });

    it('aggregates counts per callId', async () => {
      const qb = {
        select: jest.fn().mockReturnThis(),
        addSelect: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        groupBy: jest.fn().mockReturnThis(),
        getRawMany: jest.fn().mockResolvedValue([
          { callId: 'a', count: '3' },
          { callId: 'b', count: '1' },
        ]),
      };
      bookmarksRepo.createQueryBuilder.mockReturnValue(qb);

      await expect(service.getBookmarkCounts(['a', 'b'])).resolves.toEqual({
        a: 3,
        b: 1,
      });
    });
  });

  describe('isBookmarked', () => {
    it('returns true when a bookmark exists', async () => {
      bookmarksRepo.count.mockResolvedValue(1);
      await expect(service.isBookmarked(USER, CALL_ID)).resolves.toBe(true);
    });

    it('returns false when none exists', async () => {
      bookmarksRepo.count.mockResolvedValue(0);
      await expect(service.isBookmarked(USER, CALL_ID)).resolves.toBe(false);
    });

    it('returns false for an empty/unauthenticated address without querying', async () => {
      await expect(service.isBookmarked('', CALL_ID)).resolves.toBe(false);
      expect(bookmarksRepo.count).not.toHaveBeenCalled();
    });
  });

  describe('getBookmarkedCallIds', () => {
    it('returns the set of bookmarked callIds for a user', async () => {
      bookmarksRepo.find.mockResolvedValue([{ callId: 'a' }, { callId: 'c' }]);
      const result = await service.getBookmarkedCallIds(USER, ['a', 'b', 'c']);
      expect([...result].sort()).toEqual(['a', 'c']);
    });

    it('returns an empty set when address or ids are empty', async () => {
      await expect(service.getBookmarkedCallIds('', ['a'])).resolves.toEqual(
        new Set(),
      );
      await expect(service.getBookmarkedCallIds(USER, [])).resolves.toEqual(
        new Set(),
      );
      expect(bookmarksRepo.find).not.toHaveBeenCalled();
    });
  });
});
