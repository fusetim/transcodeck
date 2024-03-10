use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent, command::ffmpeg_is_installed};
use uuid::Uuid;
use std::path::{Path, PathBuf};

use crate::{model, schema, AddMediaCommand};

pub async fn add_media(db: &mut PgConnection, cmd: AddMediaCommand) -> Result<()> {
    let media = model::NewMedia {
        basename: cmd.input.file_stem().map(|s| s.to_string_lossy().to_string()),
    };
    let media = diesel::insert_into(schema::media::table)
        .values(&media)
        .get_result::<model::Media>(db)?;
    let media_id = media.media_id;

    println!("Media added: {}", media_id);

    let mut fragments = Vec::new();

    if cmd.fragment > 0 {
        let output_dir = cmd.output_dir.unwrap_or_else(|| {
            let mut output_dir = cmd.input.clone();
            output_dir.set_extension("");
            output_dir
        });
        tokio::fs::create_dir_all(&output_dir).await?;

        println!("Fragmenting media into {} second pieces", cmd.fragment);
        let _fragments = fragment_media(cmd.input, output_dir, cmd.fragment as usize).await?;
        for fragment in _fragments {
            fragments.push(model::NewFragment {
                media_id,
                filename: fragment.filename,
                fragment_number: fragment.fragment_number,
                encryption_key: None,
                retrieval_url: None,
            });
        }
    } else {
        let fragment = model::NewFragment {
            media_id,
            filename: cmd.input.file_name().unwrap().to_string_lossy().to_string(),
            fragment_number: None,
            encryption_key: None,
            retrieval_url: cmd.retrieval_url,
        };
        fragments.push(fragment);
    }

    println!("Adding {} fragments to the database.", fragments.len());
    diesel::insert_into(schema::fragment::table)
        .values(&fragments)
        .execute(db)?;

    Ok(())
}

pub async fn fragment_media(input: impl AsRef<Path>, output_dir: impl AsRef<Path>, duration: usize) -> Result<Vec<model::NewFragment>> {
    if (!ffmpeg_is_installed()) {
        bail!("ffmpeg is not installed");
    }

    let mut ffmpeg_cmd = FfmpegCommand::new_with_path("/usr/bin/ffmpeg");
    ffmpeg_cmd
        .input(input.as_ref().to_str().unwrap())
        .codec_video("copy")
        .codec_audio("copy")
        .args(&["-segment_time", duration.to_string().as_str()])
        .args(&["-f", "segment"])
        .output(format!("{}/fragment_%06d.mkv", output_dir.as_ref().to_str().unwrap()))
        .print_command();
    //let mut cmd : std::process::Command = ffmpeg_cmd.into();
    //let mut async_cmd = tokio::process::Command::from(cmd);
    //let mut child = async_cmd.spawn()?;
    
    let mut fragments = vec![];
    let mut fragment_number = 0;

    // Waiting for completion of the command
    //let _ = child.wait().await;
    ffmpeg_cmd.spawn()?.wait()?;

    // Listing the files in the output directory
    let mut dir = tokio::fs::read_dir(output_dir.as_ref()).await?;
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let fragment = model::NewFragment {
                media_id: Uuid::nil(),
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                fragment_number: Some(fragment_number),
                encryption_key: None,
                retrieval_url: None,
            };
            fragments.push(fragment);
            fragment_number += 1;
        }
    }

    Ok(fragments)
}