use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{model, schema, TranscodeCommand};
use model::{JobStatus, FragmentJobStatus};

pub async fn new_transcode(db: &mut PgConnection, cmd: TranscodeCommand) -> Result<()> {
    let media_id = Uuid::parse_str(&cmd.media_id)?;
    let media = schema::media::table
        .filter(schema::media::media_id.eq(media_id))
        .first::<model::Media>(db)?;

    if cmd.ffmpeg_command.is_empty() {
        bail!("ffmpeg_command cannot be empty");
    }

    let mut job = model::NewTranscodingJob {
        media_id: media.media_id,
        ffmpeg_command: cmd.ffmpeg_command,
        status: JobStatus::Pending,
    };

    if cmd.start {
        job.status = JobStatus::Queued;
    }

    let job_id = diesel::insert_into(schema::transcoding_job::table)
        .values(&job)
        .returning(schema::transcoding_job::transcoding_job_id)
        .get_result::<Uuid>(db)?;

    let fragments = schema::fragment::table
        .filter(schema::fragment::media_id.eq(media_id))
        .load::<model::Fragment>(db)?;

    let mut fragment_jobs = Vec::new();
    for fragment in fragments.iter() {
        let mut job = model::NewTranscodingFragmentJob {
            transcoding_job_id: job_id,
            fragment_id: fragment.fragment_id,
            status: FragmentJobStatus::Pending,
        };
        if cmd.start {
            job.status = FragmentJobStatus::Queued;
        }
        fragment_jobs.push(job);
    }
    diesel::insert_into(schema::transcoding_fragment_job::table)
        .values(&fragment_jobs)
        .execute(db)?;

    if cmd.start {
        println!("Transcoding job added and started: {} ({} fragments)", job_id, fragments.len());
    } else {
        println!("Transcoding job added: {} ({} fragments)", job_id, fragments.len());
    }

    Ok(())
}

