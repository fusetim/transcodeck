-- Your SQL goes here
CREATE TABLE IF NOT EXISTS media (
  media_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  basename TEXT,
  created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS fragment (
  fragment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  media_id UUID REFERENCES media(media_id) ON DELETE CASCADE NOT NULL,
  filename TEXT NOT NULL,
  fragment_number INT,
  encryption_key TEXT,
  retrieval_url TEXT,
  created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  deleted_at TIMESTAMPTZ
);

CREATE TYPE job_status AS ENUM ('pending', 'queued', 'in_progress', 'completed', 'failed', 'cancelled', 'deleted');
CREATE TYPE fragment_job_status AS ENUM ('pending', 'queued', 'reserved', 'in_progress', 'completed', 'failed', 'cancelled', 'deleted');

CREATE TABLE IF NOT EXISTS transcoding_job (
  transcoding_job_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  media_id UUID REFERENCES media(media_id) ON DELETE CASCADE NOT NULL,
  status job_status NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS transcoding_fragment_job (
  transcoding_fragment_job_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  transcoding_job_id UUID REFERENCES transcoding_job(transcoding_job_id) ON DELETE CASCADE NOT NULL,
  fragment_id UUID REFERENCES fragment(fragment_id) ON DELETE CASCADE NOT NULL,
  status fragment_job_status NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT now() NOT NULL,
  deleted_at TIMESTAMPTZ
);
