import { AlertDirection } from '../alerts.entity';

export class AlertCallSummaryDto {
  id: string;
  title: string;
}

export class AlertResponseDto {
  id: string;
  callId: string;
  targetPrice: number;
  direction: AlertDirection;
  triggered: boolean;
  createdAt: Date;
  call: AlertCallSummaryDto | null;
}
