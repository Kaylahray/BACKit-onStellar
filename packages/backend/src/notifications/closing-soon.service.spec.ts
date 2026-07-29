import { Test, TestingModule } from '@nestjs/testing';
import { getRepositoryToken } from '@nestjs/typeorm';
import { EventEmitter2 } from '@nestjs/event-emitter';
import { ClosingSoonService } from './closing-soon.service';
import { Call, CallStatus } from '../calls/entities/call.entity';
import { Stake } from '../analytics/entities/stake.entity';
import { NotificationsService } from './notifications.service';
import { NotificationPreferencesService } from './notification-preferences.service';
import { NotificationType } from './notification-type.enum';

const NOW = new Date('2026-01-01T12:00:00.000Z');

function makeCall(minutesLeft: number, overrides: Partial<Call> = {}): Call {
  return {
    id: `call-${minutesLeft}`,
    title: 'BTC > 100k',
    status: CallStatus.OPEN,
    endsAt: new Date(NOW.getTime() + minutesLeft * 60_000),
    totalYesStake: '30',
    totalNoStake: '10',
    ...overrides,
  } as Call;
}

function makeStake(userAddress: string, position: 'YES' | 'NO'): Stake {
  return {
    id: `s-${userAddress}`,
    callId: 'x',
    userAddress,
    position,
  } as Stake;
}

describe('ClosingSoonService', () => {
  let service: ClosingSoonService;

  const callsRepo = { find: jest.fn() };
  const stakesRepo = { find: jest.fn() };
  const notificationsService = { createNotification: jest.fn() };
  const preferencesService = { checkPreference: jest.fn() };
  const eventEmitter = { emit: jest.fn() };

  beforeEach(async () => {
    jest.clearAllMocks();
    preferencesService.checkPreference.mockResolvedValue(true);
    stakesRepo.find.mockResolvedValue([]);
    callsRepo.find.mockResolvedValue([]);

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        ClosingSoonService,
        { provide: getRepositoryToken(Call), useValue: callsRepo },
        { provide: getRepositoryToken(Stake), useValue: stakesRepo },
        { provide: NotificationsService, useValue: notificationsService },
        {
          provide: NotificationPreferencesService,
          useValue: preferencesService,
        },
        { provide: EventEmitter2, useValue: eventEmitter },
      ],
    }).compile();

    service = module.get(ClosingSoonService);
  });

  it('notifies stakers of a call closing in the standard (1h) window', async () => {
    callsRepo.find.mockResolvedValue([makeCall(45)]);
    stakesRepo.find.mockResolvedValue([makeStake('GUSER1', 'YES')]);

    await service.handleClosingSoon(NOW);

    expect(notificationsService.createNotification).toHaveBeenCalledWith(
      'GUSER1',
      NotificationType.CALL_CLOSING,
      "Call 'BTC > 100k' ends in 45 minutes — your stake is on UP.",
      'call-45',
    );
  });

  it('maps a NO position to DOWN and fires the urgent (<=15m) wave', async () => {
    callsRepo.find.mockResolvedValue([makeCall(10)]);
    stakesRepo.find.mockResolvedValue([makeStake('GUSER2', 'NO')]);

    await service.handleClosingSoon(NOW);

    expect(notificationsService.createNotification).toHaveBeenCalledWith(
      'GUSER2',
      NotificationType.CALL_CLOSING,
      expect.stringContaining('your stake is on DOWN'),
      'call-10',
    );
    expect(eventEmitter.emit).toHaveBeenCalledWith(
      'call.closing-soon',
      expect.objectContaining({
        wave: 'urgent',
        minutesLeft: 10,
        position: 'DOWN',
        oddsRatio: 3,
      }),
    );
  });

  it('emits call.closing-soon with odds + position for WebSocket broadcast', async () => {
    callsRepo.find.mockResolvedValue([makeCall(30)]);
    stakesRepo.find.mockResolvedValue([makeStake('GUSER3', 'YES')]);

    await service.handleClosingSoon(NOW);

    expect(eventEmitter.emit).toHaveBeenCalledWith(
      'call.closing-soon',
      expect.objectContaining({
        callId: 'call-30',
        userAddress: 'GUSER3',
        wave: 'standard',
        oddsRatio: 3,
        position: 'UP',
      }),
    );
  });

  it('deduplicates: the same user+call+wave is not alerted twice', async () => {
    callsRepo.find.mockResolvedValue([makeCall(30)]);
    stakesRepo.find.mockResolvedValue([makeStake('GUSER4', 'YES')]);

    await service.handleClosingSoon(NOW);
    await service.handleClosingSoon(NOW); // second run, same window

    expect(notificationsService.createNotification).toHaveBeenCalledTimes(1);
  });

  it('skips users who disabled CALL_CLOSING notifications', async () => {
    callsRepo.find.mockResolvedValue([makeCall(30)]);
    stakesRepo.find.mockResolvedValue([makeStake('GUSER5', 'YES')]);
    preferencesService.checkPreference.mockResolvedValue(false);

    await service.handleClosingSoon(NOW);

    expect(notificationsService.createNotification).not.toHaveBeenCalled();
    expect(eventEmitter.emit).not.toHaveBeenCalled();
  });

  it('sends both waves across time as a call approaches expiry', async () => {
    const stakes = [makeStake('GUSER6', 'YES')];
    stakesRepo.find.mockResolvedValue(stakes);

    // First run: 40 minutes out → standard wave.
    callsRepo.find.mockResolvedValue([makeCall(40, { id: 'call-x' })]);
    await service.handleClosingSoon(NOW);

    // Later run: 8 minutes out → urgent wave (distinct dedupe key).
    callsRepo.find.mockResolvedValue([
      makeCall(8, {
        id: 'call-x',
        endsAt: new Date(NOW.getTime() + 8 * 60_000),
      }),
    ]);
    await service.handleClosingSoon(NOW);

    expect(notificationsService.createNotification).toHaveBeenCalledTimes(2);
    const waves = (eventEmitter.emit.mock.calls as unknown[][]).map(
      (c) => (c[1] as { wave: string }).wave,
    );
    expect(waves).toEqual(['standard', 'urgent']);
  });

  it('does nothing when no calls are closing soon', async () => {
    callsRepo.find.mockResolvedValue([]);
    await service.handleClosingSoon(NOW);
    expect(notificationsService.createNotification).not.toHaveBeenCalled();
  });
});
