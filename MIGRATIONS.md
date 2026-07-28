# Database Migrations

The backend (`packages/backend`) uses [TypeORM](https://typeorm.io/) migrations to evolve the PostgreSQL schema. This document explains how migrations work in this project, how to generate/run/revert them, and the best practices to follow so we don't break the database in staging or production.

All configuration referenced below lives in [`packages/backend/src/database/data-source.ts`](packages/backend/src/database/data-source.ts) and [`packages/backend/package.json`](packages/backend/package.json).

## `synchronize` vs. migrations

TypeORM's `DataSource` has a `synchronize` option that, when `true`, automatically alters the database schema on every app boot to match the entity definitions. It's convenient for early prototyping but **destructive and unpredictable** to run against any database that other people or environments depend on — it can silently drop columns/tables, and two instances running it concurrently can race each other.

In this codebase, `synchronize` is **hardcoded to `false`** in `data-source.ts`:

```ts
// NEVER use synchronize in staging/production
synchronize: false,
```

with a comment explicitly warning it must never be turned on for staging/production. In other words: this project does not currently rely on `synchronize` at all — schema changes are expected to always go through migrations, in every environment, including local development. If you are working locally and want your entities reflected in the database, generate and run a migration rather than flipping `synchronize` to `true`. If you do experiment locally with `synchronize: true` for quick iteration, never let it run against a database that also has migrations applied, and never commit that change — the two mechanisms will fight each other (synchronize will "fix" the schema back to match entities, migrations will try to apply diffs on top of that, and the migrations table can drift out of sync with the real schema).

The `NODE_ENV` environment variable is otherwise used in `data-source.ts` to toggle `logging` (verbose SQL logging is on unless `NODE_ENV === 'production'`) and `ssl` (enabled only when `NODE_ENV === 'production'`) — but not to gate `synchronize`, since it's simply off unconditionally.

Other relevant configuration from `data-source.ts`:
- **Migrations glob**: `src/database/migrations/*.{ts,js}` (resolved relative to `process.cwd()`, so it works whether run via `ts-node` from source or as compiled JS from `dist/`).
- **Migrations table**: TypeORM tracks which migrations have run in a table named `typeorm_migrations` (via `migrationsTableName: 'typeorm_migrations'`), instead of the default `migrations` table name.
- **Entities glob**: `src/**/*.entity.{ts,js}` — this is what `migration:generate` diffs the database against.

As of this writing there are 10 migration files in `packages/backend/src/database/migrations/`, named `<unix-timestamp-ms>-<PascalCaseDescription>.ts` (e.g. `1740480000000-InitialSchema.ts`, `1760000000000-CreateFollowsTable.ts`). The leading timestamp is what TypeORM uses to order migrations, so always let the generator produce it — don't hand-pick timestamps.

## Commands

> **Note on naming:** the backend's `package.json` does not define scripts literally named `migration:generate`/`migration:run`/`migration:revert`. It defines wrapper scripts named `typeorm:generate`, `typeorm:run`, `typeorm:revert`, and `typeorm:show`, which internally invoke the TypeORM CLI's `migration:generate`, `migration:run`, `migration:revert`, and `migration:show` commands respectively (with `--dataSource src/database/data-source.ts` already wired in). Use the `typeorm:*` script names below — they are the real, working entry points in this repo.

These scripts must run with `packages/backend` as the working directory for `ts-node`, dotenv, and the `src/`-relative globs in `data-source.ts` to resolve correctly. From the repo root, use pnpm's `--filter` flag (the backend package is named `backend` in its `package.json`, and the repo is a pnpm workspace per `pnpm-workspace.yaml` / the root `package.json`'s `workspaces` field), which pnpm runs with that package's directory as the cwd:

```bash
# From the repo root:
pnpm --filter backend run typeorm:generate -- src/database/migrations/DescriptiveName
pnpm --filter backend run typeorm:run
pnpm --filter backend run typeorm:revert
pnpm --filter backend run typeorm:show
```

Equivalently, you can `cd packages/backend` first and drop the `--filter backend` part:

```bash
cd packages/backend
npm run typeorm:generate -- src/database/migrations/DescriptiveName
npm run typeorm:run
npm run typeorm:revert
npm run typeorm:show
```

Make sure your `.env` in `packages/backend` points at a real, reachable Postgres database (`DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD`, `DB_NAME`) before running any of these — migrations connect to the database defined by `dataSourceOptions` in `data-source.ts`.

### Generate a migration

```bash
pnpm --filter backend run typeorm:generate -- src/database/migrations/AddSomeColumn
```

This diffs your current entity definitions (`*.entity.ts`) against the live database schema and writes a new file under `src/database/migrations/` with an `up()` (the change) and a `down()` (the reverse of that change), prefixed with the current timestamp. It requires a reachable database that already reflects the *previous* migration state — run `typeorm:run` first if your local DB isn't up to date.

### Run (apply) pending migrations

```bash
pnpm --filter backend run typeorm:run
```

Applies every migration in `src/database/migrations/` that hasn't yet been recorded in the `typeorm_migrations` table, in timestamp order.

### Revert the last migration

```bash
pnpm --filter backend run typeorm:revert
```

Runs the `down()` method of the most recently applied migration and removes its row from `typeorm_migrations`. Only the single latest migration is reverted per invocation — run it multiple times to step back further.

### Check migration status

```bash
pnpm --filter backend run typeorm:show
```

Lists every migration file found by the glob and marks each with `[X]` (applied) or `[ ]` (pending), based on the `typeorm_migrations` table.

## Best practices

- **Always review the generated SQL before committing.** `migration:generate` does its best, but schema diffing can produce unintended drops, wrong column types, or missing indexes/foreign keys. Open the generated file and read the `up()`/`down()` SQL (or query builder calls) line by line before trusting it.
- **Test both the forward migration *and* its revert before merging.** Run `typeorm:run`, verify the schema/data end up as expected, then run `typeorm:revert` and confirm it cleanly undoes the change (and that `typeorm:run` again reproduces the same result). A migration whose `down()` doesn't work is a trap for whoever needs to roll back in production.
- **Never edit a migration that has already been merged or applied anywhere** (another developer's machine, staging, production). Once a migration file has run somewhere, its timestamp and contents are effectively part of the historical record in the `typeorm_migrations` table. Editing it after the fact causes checksum/behavior drift between environments. If you need to fix or extend something, **create a brand-new migration** that makes the corrective change.
- Keep each migration focused on one logical schema change where practical — it makes review and selective revert easier.
- Never enable `synchronize: true` against any database that also has migrations tracked in `typeorm_migrations` — see the [`synchronize` vs. migrations](#synchronize-vs-migrations) section above.

## Adding a new migration: step-by-step

1. **Update the entity/entities** in `packages/backend/src/**/*.entity.ts` to reflect the schema change you want.
2. **Generate** the migration:
   ```bash
   pnpm --filter backend run typeorm:generate -- src/database/migrations/DescriptiveName
   ```
3. **Review** the generated file in `packages/backend/src/database/migrations/` — read the SQL/query-builder calls in both `up()` and `down()`, and correct anything the auto-diff got wrong (column defaults, index names, destructive drops, data backfills, etc.).
4. **Test the forward migration**: run `pnpm --filter backend run typeorm:run` against a local/test database and confirm the resulting schema (and any data transformations) are correct.
5. **Test the revert**: run `pnpm --filter backend run typeorm:revert` and confirm the schema returns to its prior state with no errors, then run `typeorm:run` again to make sure it's re-appliable.
6. **Commit** the migration file alongside the entity changes that motivated it, so they land in the same PR/commit and stay in sync.
