import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  Index,
} from 'typeorm';

export enum AlertDirection {
  ABOVE = 'ABOVE',
  BELOW = 'BELOW',
}

/**
 * A user-configured price alert tied to a specific call/token pair.
 * Polled by AlertsScheduler; once triggered it is left in place (triggered=true)
 * rather than deleted, so it still shows up in history / doesn't re-fire.
 */
@Entity('price_alerts')
export class PriceAlert {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 128 })
  @Index()
  userAddress: string;

  @Column({ type: 'varchar', length: 64 })
  callId: string;

  @Column({ type: 'varchar', length: 32 })
  tokenPair: string;

  @Column({ type: 'decimal', precision: 24, scale: 10 })
  targetPrice: number;

  @Column({ type: 'enum', enum: AlertDirection })
  direction: AlertDirection;

  @Column({ type: 'boolean', default: false })
  triggered: boolean;

  @CreateDateColumn()
  createdAt: Date;
}
