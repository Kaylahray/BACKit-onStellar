import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  ManyToOne,
  JoinColumn,
  Unique,
  Index,
} from 'typeorm';
import { Call } from '../calls/entities/call.entity';

/**
 * A user's saved (bookmarked) market/call. Lets users track markets they are
 * interested in without staking. Unique per (userAddress, callId) so a market
 * cannot be bookmarked twice by the same user.
 */
@Entity('bookmarks')
@Unique('UQ_bookmarks_user_call', ['userAddress', 'callId'])
export class Bookmark {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  /** Wallet address of the user who created the bookmark. */
  @Index()
  @Column({ type: 'varchar', length: 64 })
  userAddress: string;

  /** Id of the bookmarked call (FK to the `calls` table). */
  @Column({ type: 'uuid' })
  callId: string;

  /** Joined call data, loaded on demand for the bookmark list. */
  @ManyToOne(() => Call, { onDelete: 'CASCADE' })
  @JoinColumn({ name: 'callId' })
  call: Call;

  @CreateDateColumn()
  createdAt: Date;
}
