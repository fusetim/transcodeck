-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS media, fragment, transcoding_job, transcoding_fragment_job CASCADE;
DROP TYPE IF EXISTS job_status, fragment_job_status;