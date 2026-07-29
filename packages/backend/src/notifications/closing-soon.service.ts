import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { EventEmitter2 } from '@nestjs/event-emitter';
import { InjectRepository } from '@nestjs/typeorm';
import { Between, Repository } from 'typeorm';
import { Call, CallStatus } from '../calls/entities/call.entity';
import { Stake } from '../analytics/entities/stake.entity';
import { NotificationsService } from './notifications.service';
import { NotificationPreferencesService } from './notification-preferences.service';
import { NotificationType } from './notification-type.enum';
import { NotificationChannel } from './notification-channel.enum';

/** Window before close in which the first (standard) alert fires. */
const CLOSING_WINDOW_MINUTES = 60;
/** Threshold at/below which the second (urgent) alert fires. */
const URGENT_WINDOW_MINUTES = 15;

type AlertWave = 'standard' | 'urgent';

export interface ClosingSoonEvent {
  callId: string;
  title: string;
  userAddress: string;
  position: 'UP' | 'DOWN';
  minutesLeft: number;
  oddsRatio: number;
  wave: AlertWave;
}

@Injectable()
export class ClosingSoonService {
  private readonly logger = new Logger(ClosingSoonService.name);

  /**
   * Remembers `${callId}:${user}:${wave}` alerts already sent so the same
   * closing alert is never sent twice for the same user + call + wave.
   */
  private readonly sentAlerts = new Set<string>();

  constructor(
    @InjectRepository(Call)
    private readonly callsRepo: Repository<Call>,
    @InjectRepository(Stake)
    private readonly stakesRepo: Repository<Stake>,
    private readonly notificationsService: NotificationsService,
    private readonly preferencesService: NotificationPreferencesService,
    private readonly eventEmitter: EventEmitter2,
  ) {}

  /** Runs every 5 minutes; dispatches closing-soon alerts. */
  @Cron(CronExpression.EVERY_5_MINUTES)
  async handleClosingSoon(now: Date = new Date()): Promise<void> {
    const horizon = new Date(now.getTime() + CLOSING_WINDOW_MINUTES * 60_000);

    const calls = await this.callsRepo.find({
      where: {
        status: CallStatus.OPEN,
        endsAt: Between(now, horizon),
      },
    });

    for (const call of calls) {
      try {
        await this.processCall(call, now);
      } catch (err) {
        this.logger.error(
          `Failed processing closing-soon for call ${call.id}`,
          err as Error,
        );
      }
    }
  }

  private async processCall(call: Call, now: Date): Promise<void> {
    if (!call.endsAt) return;

    const minutesLeft = Math.max(
      0,
      Math.round((new Date(call.endsAt).getTime() - now.getTime()) / 60_000),
    );
    // Urgent (15-min) wave for high urgency, otherwise the standard 1-hour wave.
    const wave: AlertWave =
      minutesLeft <= URGENT_WINDOW_MINUTES ? 'urgent' : 'standard';
    const oddsRatio = this.computeOddsRatio(call);

    const stakes = await this.stakesRepo.find({ where: { callId: call.id } });

    for (const stake of stakes) {
      const key = `${call.id}:${stake.userAddress}:${wave}`;
      if (this.sentAlerts.has(key)) continue; // dedupe

      // Respect user preferences — skip if CALL_CLOSING is disabled.
      const enabled = await this.preferencesService.checkPreference(
        stake.userAddress,
        NotificationType.CALL_CLOSING,
        NotificationChannel.IN_APP,
      );
      if (!enabled) {
        this.sentAlerts.add(key); // don't re-check every run
        continue;
      }

      const position: 'UP' | 'DOWN' = stake.position === 'YES' ? 'UP' : 'DOWN';
      const message = `Call '${call.title}' ends in ${minutesLeft} minutes — your stake is on ${position}.`;

      await this.notificationsService.createNotification(
        stake.userAddress,
        NotificationType.CALL_CLOSING,
        message,
        call.id,
      );

      const payload: ClosingSoonEvent = {
        callId: call.id,
        title: call.title,
        userAddress: stake.userAddress,
        position,
        minutesLeft,
        oddsRatio,
        wave,
      };
      this.eventEmitter.emit('call.closing-soon', payload);

      this.sentAlerts.add(key);
    }
  }

  /** Current odds ratio (YES stake : NO stake). Returns 0 when no NO stake. */
  private computeOddsRatio(call: Call): number {
    const yes = Number(call.totalYesStake ?? 0);
    const no = Number(call.totalNoStake ?? 0);
    if (no <= 0) return 0;
    return Number((yes / no).toFixed(4));
  }
}
