import { MigrationInterface, QueryRunner } from 'typeorm';

export class CreatePriceAlertsAndStakes1760000020000 implements MigrationInterface {
  name = 'CreatePriceAlertsAndStakes1760000020000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    // alert_direction enum
    await queryRunner.query(`
      DO $$ BEGIN
        CREATE TYPE "alert_direction_enum" AS ENUM ('ABOVE', 'BELOW');
      EXCEPTION WHEN duplicate_object THEN NULL;
      END $$
    `);

    // price_alerts table
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "price_alerts" (
        "id"          uuid              NOT NULL DEFAULT uuid_generate_v4(),
        "userAddress" character varying(128) NOT NULL,
        "callId"      character varying(64)  NOT NULL,
        "tokenPair"   character varying(32)  NOT NULL,
        "targetPrice" numeric(24, 10)   NOT NULL,
        "direction"   "alert_direction_enum" NOT NULL,
        "triggered"   boolean           NOT NULL DEFAULT false,
        "createdAt"   TIMESTAMP         NOT NULL DEFAULT now(),
        CONSTRAINT "PK_price_alerts" PRIMARY KEY ("id")
      )
    `);

    await queryRunner.query(`
      CREATE INDEX IF NOT EXISTS "IDX_price_alerts_userAddress"
        ON "price_alerts" ("userAddress")
    `);

    // stakes table (placeholder – swap with the real table if one already exists)
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "stakes" (
        "id"          uuid              NOT NULL DEFAULT uuid_generate_v4(),
        "userAddress" character varying(128) NOT NULL,
        "callId"      character varying(64)  NOT NULL,
        "amount"      numeric(24, 10)   NOT NULL,
        "createdAt"   TIMESTAMP         NOT NULL DEFAULT now(),
        CONSTRAINT "PK_stakes" PRIMARY KEY ("id")
      )
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `DROP INDEX IF EXISTS "IDX_price_alerts_userAddress"`,
    );
    await queryRunner.query(`DROP TABLE IF EXISTS "price_alerts"`);
    await queryRunner.query(`DROP TABLE IF EXISTS "stakes"`);
    await queryRunner.query(`DROP TYPE IF EXISTS "alert_direction_enum"`);
  }
}
