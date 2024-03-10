use age::secrecy::ExposeSecret;
use age::{decryptor::RecipientsDecryptor, Decryptor, Identity, Recipient};
use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use reqwest::Client;
use std::iter;
use std::path::PathBuf;
use tempdir::TempDir;
use tokio_util::compat::{
    FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt, TokioAsyncReadCompatExt,
    TokioAsyncWriteCompatExt,
};
use uuid::Uuid;

use crate::{model, schema, DaemonCommand};
use model::{FragmentJobStatus, JobStatus};

pub async fn daemon(db: &mut PgConnection, cmd: DaemonCommand) -> Result<()> {
    println!("Starting daemon...");

    let mut http = reqwest::Client::new();

    loop {
        // Searching for a fragment job that is queued
        let job = schema::transcoding_fragment_job::table
            .filter(schema::transcoding_fragment_job::status.eq(FragmentJobStatus::Queued))
            .select((
                schema::transcoding_fragment_job::transcoding_job_id,
                schema::transcoding_fragment_job::transcoding_fragment_job_id,
                schema::transcoding_fragment_job::fragment_id,
            ))
            .order(schema::transcoding_fragment_job::created_at.asc())
            .first::<model::JobResume>(db)
            .optional()?;

        if let Some(model::JobResume {
            transcoding_fragment_job_id,
            transcoding_job_id,
            fragment_id,
        }) = job
        {
            // Updating the fragment job to in progress, if it is still queued
            let changed = diesel::update(schema::transcoding_fragment_job::table)
                .set(schema::transcoding_fragment_job::status.eq(FragmentJobStatus::InProgress))
                .filter(
                    schema::transcoding_fragment_job::transcoding_fragment_job_id
                        .eq(transcoding_fragment_job_id),
                )
                .filter(schema::transcoding_fragment_job::status.eq(FragmentJobStatus::Queued))
                .execute(db)?;
            if changed <= 0 {
                continue;
            }

            println!("Starting fragment job: {}", transcoding_fragment_job_id);
            let fragment = schema::fragment::table
                .filter(schema::fragment::fragment_id.eq(fragment_id))
                .first::<model::Fragment>(db)?;
            let ffmpeg_command = schema::transcoding_job::table
                .filter(schema::transcoding_job::transcoding_job_id.eq(transcoding_job_id))
                .select(schema::transcoding_job::ffmpeg_command)
                .first::<String>(db)?;

            // Update the parent transcoding job to in progress, if it is still queued.
            if (diesel::update(schema::transcoding_job::table)
                .set(schema::transcoding_job::status.eq(JobStatus::InProgress))
                .filter(schema::transcoding_job::transcoding_job_id.eq(transcoding_job_id))
                .filter(schema::transcoding_job::status.eq(JobStatus::Queued))
                .execute(db)?
                > 0)
            {
                println!("Started transcoding job: {}", transcoding_job_id);
            }

            let tempdir = TempDir::new(&format!(
                "transcodeck-job-{}",
                transcoding_fragment_job_id.as_hyphenated().to_string()
            ))?;

            // Download the media fragment
            if fragment.retrieval_url.is_none() {
                bail!("Fragment retrieval URL is missing");
            }
            let fragment_url = format!("{}/{}", fragment.retrieval_url.unwrap(), fragment.filename);
            let fragment_path = tempdir.path().join(&fragment.filename);
            let mut fragment_file = tokio::fs::File::create(&fragment_path).await?;
            println!("Downloading fragment: {}", fragment_url);
            let mut response = http.get(&fragment_url).send().await?;
            let mut bytes = response.bytes().await?;
            tokio::io::copy(&mut bytes.as_ref(), &mut fragment_file).await?;
            println!("Fragment downloaded: {}", fragment_path.display());

            // Decrypt the media fragment if needed
            if fragment.encryption_key.is_some() {
                let key = age::x25519::Identity::from_str(fragment.encryption_key.as_ref())?;
                let mut output_path = tempdir.path().join(&fragment.filename);
                output_path.set_extension("mkv");
                decrypt_file(fragment_path, output_path, key).await?;
                println!("Fragment decrypted: {}", output_path.display());
            }

            // Transcode the media fragment
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    Ok(())
}

async fn decrypt_file(input: PathBuf, output: PathBuf, key: impl Identity + Send) -> Result<()> {
    let mut input_file = tokio::fs::File::open(input).await?;
    let mut output_file = tokio::fs::File::create(output).await?;

    let mut input_compat = input_file.compat();
    let decryptor = Decryptor::new_async(&mut input_compat).await;
    match decryptor {
        Ok(age::Decryptor::Recipients(d)) => {
            let mut decrypted = d.decrypt_async(iter::once(&key as &dyn age::Identity))?;
            futures::io::copy(&mut decrypted, &mut output_file.compat()).await?;
        }
        Ok(_) => bail!("Unsupported decryptor"),
        Err(err) => bail!("Failed to create decryptor: {}", err),
    }
    Ok(())
}
