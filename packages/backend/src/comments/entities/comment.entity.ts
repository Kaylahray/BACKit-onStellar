import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  ManyToOne,
  OneToMany,
  JoinColumn,
  Index,
} from 'typeorm';
import { Call } from '../../calls/entities/call.entity';

@Entity('comments')
@Index('IDX_comment_call_created', ['callId', 'createdAt'])
@Index('IDX_comment_author_call', ['authorAddress', 'callId'])
export class Comment {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'uuid' })
  @Index()
  callId: string;

  @ManyToOne(() => Call, { onDelete: 'CASCADE' })
  @JoinColumn({ name: 'callId' })
  call: Call;

  @Column({ type: 'varchar', length: 56 })
  @Index()
  authorAddress: string;

  @Column({ type: 'varchar', length: 2000 })
  content: string;

  @Column({ type: 'uuid', nullable: true })
  @Index()
  parentId: string | null;

  @ManyToOne(() => Comment, (comment) => comment.replies, {
    onDelete: 'CASCADE',
    nullable: true,
  })
  @JoinColumn({ name: 'parentId' })
  parent: Comment | null;

  @OneToMany(() => Comment, (comment) => comment.parent)
  replies: Comment[];

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
