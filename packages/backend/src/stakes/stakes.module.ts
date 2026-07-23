import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Stake } from './entities/stake.entity';
import { StakesService } from './stakes.service';

@Module({
  imports: [TypeOrmModule.forFeature([Stake])],
  providers: [StakesService],
  exports: [StakesService],
})
export class StakesModule {}
