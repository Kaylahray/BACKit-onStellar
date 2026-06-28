import { MigrationInterface, QueryRunner } from 'typeorm';

export class AddStreakAndStakeComment1760000010000 implements MigrationInterface {
  name = 'AddStreakAndStakeComment1760000010000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    // Win streak fields on users table
    await queryRunner.query(`
      ALTER TABLE "users"
        ADD COLUMN IF NOT EXISTS "currentWinStreak" integer NOT NULL DEFAULT 0,
        ADD COLUMN IF NOT EXISTS "bestWinStreak" integer NOT NULL DEFAULT 0
    `);

    // Optional stake comment field
    await queryRunner.query(`
      ALTER TABLE "stakes"
        ADD COLUMN IF NOT EXISTS "comment" varchar(140)
    `);

    // Add streak badge types to the badge type enum if not already there
    await queryRunner.query(`
      DO $$
      BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'STREAK_THREE' AND enumtypid = 'badge_type_enum'::regtype) THEN
          ALTER TYPE "badge_type_enum" ADD VALUE 'STREAK_THREE';
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'STREAK_FIVE' AND enumtypid = 'badge_type_enum'::regtype) THEN
          ALTER TYPE "badge_type_enum" ADD VALUE 'STREAK_FIVE';
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'STREAK_TEN' AND enumtypid = 'badge_type_enum'::regtype) THEN
          ALTER TYPE "badge_type_enum" ADD VALUE 'STREAK_TEN';
        END IF;
      END
      $$;
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`ALTER TABLE "users" DROP COLUMN IF EXISTS "currentWinStreak"`);
    await queryRunner.query(`ALTER TABLE "users" DROP COLUMN IF EXISTS "bestWinStreak"`);
    await queryRunner.query(`ALTER TABLE "stakes" DROP COLUMN IF EXISTS "comment"`);
  }
}
