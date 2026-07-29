import 'reflect-metadata';
import { DataSource, DataSourceOptions } from 'typeorm';
import * as dotenv from 'dotenv';
import * as path from 'path';

dotenv.config();

const isProduction = process.env.NODE_ENV === 'production';

// Use process.cwd() so this file works whether loaded via ts-node (src/)
// or as compiled JS (dist/), without relying on __dirname.
const root = process.cwd();

export const dataSourceOptions: DataSourceOptions = {
  type: 'postgres',
  host: process.env.DB_HOST ?? 'localhost',
  port: Number(process.env.DB_PORT ?? 5432),
  username: process.env.DB_USERNAME ?? 'postgres',
  password: process.env.DB_PASSWORD ?? '',
  database: process.env.DB_NAME ?? 'backit',

  // NEVER use synchronize in staging/production
  synchronize: false,

  entities: [path.join(root, 'src', '**', '*.entity.{ts,js}')],

  migrations: [path.join(root, 'src', 'database', 'migrations', '*.{ts,js}')],
  migrationsTableName: 'typeorm_migrations',

  logging: !isProduction,
  logger: 'advanced-console',

  ssl: isProduction ? { rejectUnauthorized: true } : false,
};

/** Singleton DataSource used both by the NestJS app and the TypeORM CLI. */
const AppDataSource = new DataSource(dataSourceOptions);

export default AppDataSource;
