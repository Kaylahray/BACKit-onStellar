import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
  ManyToOne,
  JoinColumn,
} from 'typeorm';

export enum CallStatus {
  PENDING = 'pending',
  SETTLED = 'settled',
  CANCELLED = 'cancelled',
}

export enum CallOutcome {
  WON = 'won',
  LOST = 'lost',
}

@Entity('prediction_calls')
@Index('IDX_prediction_call_user_status', ['userId', 'status'])
@Index('IDX_prediction_call_settled', ['settledAt'])
@Index('IDX_prediction_call_status_outcome', ['status', 'outcome'])
export class PredictionCall {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column('uuid')
  @Index()
  userId: string;

  @Column({ type: 'enum', enum: CallStatus, default: CallStatus.PENDING })
  status: CallStatus;

  @Column({ type: 'enum', enum: CallOutcome, nullable: true })
  outcome: CallOutcome | null;

  @Column({ type: 'decimal', precision: 18, scale: 6, default: 0 })
  profitUsdc: number;

  @Column({ type: 'decimal', precision: 18, scale: 6, default: 0 })
  stakeUsdc: number;

  @Column({ type: 'timestamp', nullable: true })
  settledAt: Date | null;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}

@Entity('leaderboard_snapshots')
@Index('IDX_leaderboard_snap_period_date', ['period', 'snapshotDate'])
export class LeaderboardSnapshot {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column('uuid')
  @Index()
  userId: string;

  @Column()
  username: string;

  @Column({ nullable: true })
  avatarUrl: string;

  @Column({ type: 'int', default: 0 })
  totalCalls: number;

  @Column({ type: 'int', default: 0 })
  wonCalls: number;

  @Column({ type: 'int', default: 0 })
  lostCalls: number;

  @Column({ type: 'decimal', precision: 10, scale: 4, default: 0 })
  winRate: number;

  @Column({ type: 'decimal', precision: 18, scale: 6, default: 0 })
  totalProfit: number;

  @Column({ type: 'int', default: 0 })
  rank: number;

  @Column({ default: 'all' })
  period: string; // 'all' | 'month'

  @Column({ type: 'date' })
  @Index()
  snapshotDate: Date;

  @CreateDateColumn()
  createdAt: Date;
}
