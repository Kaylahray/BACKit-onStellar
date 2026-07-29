import {
  Injectable,
  NotFoundException,
  ForbiddenException,
} from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { PriceAlert } from './alerts.entity';
import { CreateAlertDto } from './dto/create-alert.dto';
import { AlertResponseDto } from './dto/alert-response.dto';
import { CallsService } from '../calls/calls.service';
import { StakesService } from '../stakes/stakes.service';
import { BookmarksService } from '../bookmarks/bookmarks.service';

@Injectable()
export class AlertsService {
  constructor(
    @InjectRepository(PriceAlert)
    private readonly alertsRepo: Repository<PriceAlert>,
    private readonly callsService: CallsService,
    private readonly stakesService: StakesService,
    private readonly bookmarksService: BookmarksService,
  ) {}

  async createAlert(
    userAddress: string,
    dto: CreateAlertDto,
  ): Promise<PriceAlert> {
    // getCallOrThrow already throws NotFoundException('Call not found') for us.
    const call = await this.callsService.getCallOrThrow(dto.callId);

    const [hasStake, hasBookmark] = await Promise.all([
      this.stakesService.hasStake(userAddress, dto.callId),
      this.bookmarksService.isBookmarked(userAddress, dto.callId),
    ]);

    if (!hasStake && !hasBookmark) {
      throw new ForbiddenException(
        'You must stake or bookmark this call before setting a price alert',
      );
    }

    const alert = this.alertsRepo.create({
      userAddress,
      callId: dto.callId,
      tokenPair: dto.tokenPair,
      targetPrice: dto.targetPrice,
      direction: dto.direction,
      triggered: false,
    });

    return this.alertsRepo.save(alert);
  }

  async removeAlert(userAddress: string, id: string): Promise<void> {
    const alert = await this.alertsRepo.findOne({ where: { id, userAddress } });
    if (!alert) {
      throw new NotFoundException('Alert not found');
    }
    await this.alertsRepo.remove(alert);
  }

  async listActiveAlerts(userAddress: string): Promise<AlertResponseDto[]> {
    const alerts = await this.alertsRepo.find({
      where: { userAddress, triggered: false },
      order: { createdAt: 'DESC' },
    });

    return Promise.all(alerts.map((alert) => this.toResponseDto(alert)));
  }

  private async toResponseDto(alert: PriceAlert): Promise<AlertResponseDto> {
    const call = await this.callsService
      .getCallOrThrow(alert.callId)
      .catch(() => null);

    return {
      id: alert.id,
      callId: alert.callId,
      targetPrice: Number(alert.targetPrice),
      direction: alert.direction,
      triggered: alert.triggered,
      createdAt: alert.createdAt,
      call: call ? { id: call.id, title: call.title } : null,
    };
  }
}
