import { Controller, Post, Body, BadRequestException } from '@nestjs/common';
import { RelayService } from './relay.service';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';
import { Throttle } from '@nestjs/throttler';
import { IsString, IsNotEmpty } from 'class-validator';
import { SimulateTxDto } from './dto/simulate-tx.dto';
import { SimulationResultDto } from './dto/simulation-result.dto';
import { THROTTLER_GLOBAL_NAME } from '../throttler/throttler.constants';

class RelayTxDto {
  @IsString()
  @IsNotEmpty()
  xdr: string;
}

@ApiTags('relay')
@Controller('relay')
export class RelayController {
  constructor(private readonly relayService: RelayService) {}

  @Post('tx')
  @ApiOperation({
    summary: 'Sponsor a transaction by co-signing and submitting',
  })
  @ApiResponse({
    status: 201,
    description: 'Transaction submitted successfully',
  })
  @ApiResponse({
    status: 400,
    description: 'Invalid XDR or unauthorized transaction',
  })
  async relay(@Body() dto: RelayTxDto) {
    if (!dto.xdr) {
      throw new BadRequestException('XDR string is required');
    }

    try {
      const result = await this.relayService.sponsorAndSubmit(dto.xdr);
      return result;
    } catch (error) {
      if (error instanceof BadRequestException) {
        throw error;
      }
      throw new BadRequestException(`Relay failed: ${error.message}`);
    }
  }

  @Post('simulate')
  @Throttle({
    [THROTTLER_GLOBAL_NAME]: {
      limit: 20,
      ttl: 60000,
    },
  })
  @ApiOperation({
    summary: 'Simulate a Soroban transaction and return a human-readable breakdown',
  })
  @ApiResponse({
    status: 201,
    description: 'Simulation breakdown generated successfully',
    type: SimulationResultDto,
  })
  @ApiResponse({
    status: 400,
    description: 'Invalid transaction parameters or simulation failed',
  })
  async simulate(@Body() dto: SimulateTxDto): Promise<SimulationResultDto> {
    if (!dto.xdr) {
      throw new BadRequestException('XDR string is required');
    }
    return this.relayService.simulate(dto);
  }
}

