import { NotFoundException, ForbiddenException } from '@nestjs/common';
import { AlertsService } from './alerts.service';
import { AlertDirection, PriceAlert } from './alerts.entity';
import { CallsService } from '../calls/calls.service';
import { StakesService } from '../stakes/stakes.service';
import { BookmarksService } from '../bookmarks/bookmarks.service';
import { Repository } from 'typeorm';

// ── Mock factories ────────────────────────────────────────────────────────────

const mockAlertsRepo = () => ({
  create: jest.fn(),
  save: jest.fn(),
  find: jest.fn(),
  findOne: jest.fn(),
  remove: jest.fn(),
});

const mockCallsService = () => ({
  getCallOrThrow: jest.fn(),
});

const mockStakesService = () => ({
  hasStake: jest.fn(),
});

const mockBookmarksService = () => ({
  isBookmarked: jest.fn(),
});

// ── Suite ─────────────────────────────────────────────────────────────────────

describe('AlertsService', () => {
  let service: AlertsService;
  let alertsRepo: ReturnType<typeof mockAlertsRepo>;
  let callsService: ReturnType<typeof mockCallsService>;
  let stakesService: ReturnType<typeof mockStakesService>;
  let bookmarksService: ReturnType<typeof mockBookmarksService>;

  const userAddress = 'GABC123';
  const baseAlert: PriceAlert = {
    id: 'alert-1',
    userAddress,
    callId: 'call-1',
    tokenPair: 'XLM/USDC',
    targetPrice: 0.5,
    direction: AlertDirection.ABOVE,
    triggered: false,
    createdAt: new Date('2026-01-01T00:00:00Z'),
  };

  beforeEach(() => {
    alertsRepo = mockAlertsRepo();
    callsService = mockCallsService();
    stakesService = mockStakesService();
    bookmarksService = mockBookmarksService();

    service = new AlertsService(
      alertsRepo as unknown as Repository<PriceAlert>,
      callsService as unknown as CallsService,
      stakesService as unknown as StakesService,
      bookmarksService as unknown as BookmarksService,
    );
  });

  describe('createAlert', () => {
    const dto = {
      callId: 'call-1',
      tokenPair: 'XLM/USDC',
      targetPrice: 0.5,
      direction: AlertDirection.ABOVE,
    };

    it('throws NotFoundException when the call does not exist', async () => {
      callsService.getCallOrThrow.mockRejectedValue(
        new NotFoundException('Call not found'),
      );

      await expect(service.createAlert(userAddress, dto)).rejects.toThrow(
        NotFoundException,
      );
      expect(stakesService.hasStake).not.toHaveBeenCalled();
    });

    it('throws ForbiddenException when user has neither stake nor bookmark', async () => {
      callsService.getCallOrThrow.mockResolvedValue({
        id: 'call-1',
        title: 'BTC to 100k',
      });
      stakesService.hasStake.mockResolvedValue(false);
      bookmarksService.isBookmarked.mockResolvedValue(false);

      await expect(service.createAlert(userAddress, dto)).rejects.toThrow(
        ForbiddenException,
      );
      expect(alertsRepo.save).not.toHaveBeenCalled();
    });

    it('creates the alert when the user has a stake', async () => {
      callsService.getCallOrThrow.mockResolvedValue({
        id: 'call-1',
        title: 'BTC to 100k',
      });
      stakesService.hasStake.mockResolvedValue(true);
      bookmarksService.isBookmarked.mockResolvedValue(false);

      alertsRepo.create.mockReturnValue({
        userAddress,
        callId: dto.callId,
        triggered: false,
      });
      alertsRepo.save.mockResolvedValue({
        id: 'alert-1',
        userAddress,
        callId: dto.callId,
        triggered: false,
      });

      const result = await service.createAlert(userAddress, dto);

      expect(alertsRepo.create).toHaveBeenCalledWith(
        expect.objectContaining({
          userAddress,
          callId: dto.callId,
          triggered: false,
        }),
      );
      expect(alertsRepo.save).toHaveBeenCalled();
      expect(result).toMatchObject({ callId: 'call-1', userAddress });
    });

    it('creates the alert when the user has only a bookmark', async () => {
      callsService.getCallOrThrow.mockResolvedValue({
        id: 'call-1',
        title: 'BTC to 100k',
      });
      stakesService.hasStake.mockResolvedValue(false);
      bookmarksService.isBookmarked.mockResolvedValue(true);

      alertsRepo.create.mockReturnValue({
        userAddress,
        callId: dto.callId,
        triggered: false,
      });
      alertsRepo.save.mockResolvedValue({
        id: 'alert-1',
        userAddress,
        callId: dto.callId,
        triggered: false,
      });

      await expect(
        service.createAlert(userAddress, dto),
      ).resolves.toBeDefined();
    });
  });

  describe('removeAlert', () => {
    it('removes an alert owned by the user', async () => {
      alertsRepo.findOne.mockResolvedValue(baseAlert);

      await service.removeAlert(userAddress, 'alert-1');

      expect(alertsRepo.findOne).toHaveBeenCalledWith({
        where: { id: 'alert-1', userAddress },
      });
      expect(alertsRepo.remove).toHaveBeenCalledWith(baseAlert);
    });

    it('throws NotFoundException when the alert does not exist or is not owned by the user', async () => {
      alertsRepo.findOne.mockResolvedValue(null);

      await expect(service.removeAlert(userAddress, 'nope')).rejects.toThrow(
        NotFoundException,
      );
      expect(alertsRepo.remove).not.toHaveBeenCalled();
    });
  });

  describe('listActiveAlerts', () => {
    it('returns only non-triggered alerts with joined call details', async () => {
      alertsRepo.find.mockResolvedValue([baseAlert]);
      callsService.getCallOrThrow.mockResolvedValue({
        id: 'call-1',
        title: 'BTC to 100k',
      });

      const result = await service.listActiveAlerts(userAddress);

      expect(alertsRepo.find).toHaveBeenCalledWith(
        expect.objectContaining({ where: { userAddress, triggered: false } }),
      );
      expect(result).toEqual([
        expect.objectContaining({
          id: 'alert-1',
          targetPrice: 0.5,
          call: { id: 'call-1', title: 'BTC to 100k' },
        }),
      ]);
    });

    it('returns call: null when the related call lookup fails', async () => {
      alertsRepo.find.mockResolvedValue([baseAlert]);
      callsService.getCallOrThrow.mockRejectedValue(
        new NotFoundException('Call not found'),
      );

      const result = await service.listActiveAlerts(userAddress);

      expect(result[0].call).toBeNull();
    });
  });
});
