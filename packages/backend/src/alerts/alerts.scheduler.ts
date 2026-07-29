import { Injectable, Logger } from '@nestjs/common';
import { Cron } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { PriceAlert } from './alerts.entity';
import { isPriceNearTarget } from './alerts.utils';
import { TokensService } from '../token/tokens.service';
import { CallsService } from '../calls/calls.service';
import { NotificationsService } from '../notifications/notifications.service';
import { NotificationType } from '../notifications/notification-type.enum';

@Injectable()
export class AlertsScheduler {
  private readonly logger = new Logger(AlertsScheduler.name);

  constructor(
    @InjectRepository(PriceAlert)
    private readonly alertsRepo: Repository<PriceAlert>,
    private readonly tokensService: TokensService,
    private readonly callsService: CallsService,
    private readonly notificationsService: NotificationsService,
  ) {}

  // Every 60 seconds. Using an explicit cron string rather than
  // CronExpression.EVERY_MINUTE for clarity that this is a polling interval,
  // not "top of every minute" semantics.
  @Cron('*/60 * * * * *', { name: 'price-alert-poll' })
  async checkAlerts(): Promise<void> {
    const activeAlerts = await this.alertsRepo.find({
      where: { triggered: false },
    });
    if (activeAlerts.length === 0) return;

    const uniquePairs = [...new Set(activeAlerts.map((a) => a.tokenPair))];
    const priceByPair = await this.fetchPrices(uniquePairs);

    for (const alert of activeAlerts) {
      const currentPrice = priceByPair.get(alert.tokenPair);
      if (currentPrice === undefined) continue;

      if (isPriceNearTarget(currentPrice, Number(alert.targetPrice))) {
        await this.triggerAlert(alert, currentPrice);
      }
    }
  }

  private async fetchPrices(
    tokenPairs: string[],
  ): Promise<Map<string, number>> {
    const priceByPair = new Map<string, number>();

    await Promise.all(
      tokenPairs.map(async (pair) => {
        try {
          const { priceUsd } = await this.tokensService.getPairPrice(pair);
          if (priceUsd > 0) {
            priceByPair.set(pair, priceUsd);
          }
        } catch (err) {
          this.logger.warn(
            `Failed to fetch price for ${pair}: ${(err as Error).message}`,
          );
        }
      }),
    );

    return priceByPair;
  }

  private async triggerAlert(
    alert: PriceAlert,
    currentPrice: number,
  ): Promise<void> {
    const call = await this.callsService
      .getCallOrThrow(alert.callId)
      .catch(() => null);
    const callTitle = call?.title ?? alert.tokenPair;
    const target = Number(alert.targetPrice);
    const message = `${callTitle} (${alert.tokenPair}) is near your target of $${target} — current price $${currentPrice}`;

    await this.notificationsService.notify(
      alert.userAddress,
      NotificationType.PRICE_ALERT_TRIGGERED,
      message,
      alert.callId,
    );

    alert.triggered = true;
    await this.alertsRepo.save(alert);
  }
}
