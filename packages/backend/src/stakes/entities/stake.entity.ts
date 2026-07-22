import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  Index,
} from 'typeorm';

@Entity('stakes')
@Index(['userAddress', 'callId'])
export class Stake {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 128 })
  userAddress: string;

  @Column({ type: 'varchar', length: 64 })
  callId: string;

  @Column({ type: 'decimal', precision: 24, scale: 10 })
  amount: number;

  @CreateDateColumn()
  createdAt: Date;
}
