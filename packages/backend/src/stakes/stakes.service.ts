import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Stake } from './entities/stake.entity';

@Injectable()
export class StakesService {
  constructor(
    @InjectRepository(Stake)
    private readonly stakesRepo: Repository<Stake>,
  ) {}

  /** Whether `userAddress` has an existing stake on `callId`. */
  async hasStake(userAddress: string, callId: string): Promise<boolean> {
    if (!userAddress) return false;
    const count = await this.stakesRepo.count({
      where: { userAddress, callId },
    });
    return count > 0;
  }
}
