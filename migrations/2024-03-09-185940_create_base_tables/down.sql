-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS media, fragment, transcoding_job, transcoding_fragment_job CASCADE;
