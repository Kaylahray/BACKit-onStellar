import { NotFoundException, ForbiddenException } from '@nestjs/common';
import { AlertsService } from './alerts.service';
import { AlertDirection, PriceAlert } from './alerts.entity';

describe('AlertsService', () => {
  let service: AlertsService;
  let alertsRepo: any;
  let callsService: any;
  let stakesService: any;
  let bookmarksService: any;

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
    alertsRepo = {
      create: jest.fn((data) => ({ ...data })),
      save: jest.fn((alert) => Promise.resolve({ id: 'alert-1', ...alert })),
      find: jest.fn(),
      findOne: jest.fn(),
      remove: jest.fn(),
    };

    callsService = {
      getCallOrThrow: jest.fn(),
    };

    stakesService = {
      hasStake: jest.fn(),
    };

    bookmarksService = {
      isBookmarked: jest.fn(),
    };

    service = new AlertsService(
      alertsRepo,
      callsService,
      stakesService,
      bookmarksService,
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
