import {
  IsString,
  IsNotEmpty,
  IsNumber,
  IsPositive,
  IsEnum,
} from 'class-validator';
import { AlertDirection } from '../alerts.entity';

export class CreateAlertDto {
  @IsString()
  @IsNotEmpty()
  callId: string;

  @IsString()
  @IsNotEmpty()
  tokenPair: string;

  @IsNumber()
  @IsPositive()
  targetPrice: number;

  @IsEnum(AlertDirection)
  direction: AlertDirection;
}
