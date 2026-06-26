import { Controller, Post, Body, BadRequestException } from '@nestjs/common';
import { RelayService } from './relay.service';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';

import { IsString, IsNotEmpty } from 'class-validator';

class RelayTxDto {
  @IsString()
  @IsNotEmpty()
  xdr: string;
}

@ApiTags('relay')
@Controller('relay')
export class RelayController {
  constructor(private readonly relayService: RelayService) {}

  @Post('estimate-fee')
  @ApiOperation({ summary: 'Simulate a transaction and return estimated gas fee' })
  @ApiResponse({ status: 201, description: 'Fee estimate returned' })
  @ApiResponse({ status: 400, description: 'Invalid XDR' })
  async estimateFee(@Body() dto: RelayTxDto) {
    if (!dto.xdr) throw new BadRequestException('XDR string is required');
    try {
      return await this.relayService.estimateFee(dto.xdr);
    } catch (error) {
      if (error instanceof BadRequestException) throw error;
      throw new BadRequestException(`Fee estimation failed: ${error.message}`);
    }
  }

  @Post('tx')
  @ApiOperation({
    summary: 'Sponsor a transaction by co-signing and submitting',
  })
  @ApiResponse({ status: 201, description: 'Transaction submitted successfully' })
  @ApiResponse({ status: 400, description: 'Invalid XDR or unauthorized transaction' })
  async relay(@Body() dto: RelayTxDto) {
    if (!dto.xdr) throw new BadRequestException('XDR string is required');
    try {
      return await this.relayService.sponsorAndSubmit(dto.xdr);
    } catch (error) {
      if (error instanceof BadRequestException) throw error;
      throw new BadRequestException(`Relay failed: ${error.message}`);
    }
  }
}
