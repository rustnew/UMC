-- UMC API — Migration 002: convert enum columns to TEXT
--
-- PostgreSQL ENUM types are painful to evolve: adding values requires
-- ALTER TYPE ... ADD VALUE (which cannot run inside a transaction block in
-- older versions), and reordering/removing values requires a full rewrite.
-- This migration converts every enum-backed column to TEXT, preserving the
-- existing values, so the schema can evolve freely.

-- users.plan: user_plan ('free', 'pro', 'enterprise') -> TEXT
ALTER TABLE users ALTER COLUMN plan DROP DEFAULT;
ALTER TABLE users ALTER COLUMN plan TYPE TEXT USING plan::text;
ALTER TABLE users ALTER COLUMN plan SET DEFAULT 'free';

-- conversion_jobs.status: job_status ('queued','running','done','failed','cancelled') -> TEXT
ALTER TABLE conversion_jobs ALTER COLUMN status DROP DEFAULT;
ALTER TABLE conversion_jobs ALTER COLUMN status TYPE TEXT USING status::text;
ALTER TABLE conversion_jobs ALTER COLUMN status SET DEFAULT 'queued';

-- Drop the now-unused enum types
DROP TYPE IF EXISTS user_plan;
DROP TYPE IF EXISTS job_status;