import { Controller, Get, Post, Delete, Param, Body } from '@nestjs/common';
import { AlertsService } from './alerts.service';
import { CreateAlertDto } from './dto/create-alert.dto';
import { AlertResponseDto } from './dto/alert-response.dto';
import { PriceAlert } from './alerts.entity';

@Controller('users/:address/alerts')
export class AlertsController {
  constructor(private readonly alertsService: AlertsService) {}

  @Post()
  create(
    @Param('address') address: string,
    @Body() dto: CreateAlertDto,
  ): Promise<PriceAlert> {
    return this.alertsService.createAlert(address, dto);
  }

  @Get()
  list(@Param('address') address: string): Promise<AlertResponseDto[]> {
    return this.alertsService.listActiveAlerts(address);
  }

  @Delete(':id')
  remove(
    @Param('address') address: string,
    @Param('id') id: string,
  ): Promise<void> {
    return this.alertsService.removeAlert(address, id);
  }
}
