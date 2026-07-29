import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { PriceAlert } from './alerts.entity';
import { AlertsService } from './alerts.service';
import { AlertsController } from './alerts.controller';
import { AlertsScheduler } from './alerts.scheduler';
import { CallsModule } from '../calls/calls.module';
import { BookmarksModule } from '../bookmarks/bookmarks.module';
import { NotificationsModule } from '../notifications/notifications.module';
import { TokensModule } from 'src/token/tokens.module';
import { StakesModule } from 'src/stakes/stakes.module';

@Module({
  imports: [
    TypeOrmModule.forFeature([PriceAlert]),
    CallsModule,
    StakesModule,
    BookmarksModule,
    TokensModule,
    NotificationsModule,
  ],
  controllers: [AlertsController],
  providers: [AlertsService, AlertsScheduler],
  exports: [AlertsService],
})
export class AlertsModule {}
