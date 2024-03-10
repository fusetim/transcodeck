use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use age::{Recipient};
use age::secrecy::ExposeSecret;
use tempdir::TempDir;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt, FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

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

    let tmp_dir = TempDir::new(&format!("transcodeck-{}", media_id.as_hyphenated().to_string()))?;
    let mut fragments = Vec::new();

    if cmd.fragment > 0 {
        let output_dir = if cmd.encrypted {
            tmp_dir.path().to_path_buf()
        } else {
            cmd.output_dir.clone().unwrap_or_else(|| {
                let mut output_dir = cmd.input.clone();
                output_dir.set_extension("");
                output_dir
            })
        };
        tokio::fs::create_dir_all(&output_dir).await?;

        println!("Fragmenting media into {} second pieces", cmd.fragment);
        let _fragments = fragment_media(cmd.input.clone(), output_dir, cmd.fragment as usize).await?;
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
            retrieval_url: None,
        };
        fragments.push(fragment);
    }

    if cmd.encrypted {
        println!("Encrypting {} media...", fragments.len());
        let output_dir = cmd.output_dir.unwrap_or_else(|| {
            let mut output_dir = cmd.input.clone();
            output_dir.set_extension("");
            output_dir
        });

        tokio::fs::create_dir_all(&output_dir).await?;

        for fragment in &mut fragments {
            let input = tmp_dir.path().join(&fragment.filename);
            let mut output = output_dir.join(&fragment.filename);
            output.set_extension("age");
            let identity = age::x25519::Identity::generate();
            let pubkey = identity.to_public();
            fragment.filename = output.file_name().unwrap().to_string_lossy().to_string();
            encrypt_file(input, output, Box::new(pubkey)).await?;
            fragment.encryption_key = Some(identity.to_string().expose_secret().to_owned());
        }
    }

    if let Some(base_url) = cmd.retrieval_url {
        for fragment in &mut fragments {
            fragment.retrieval_url = Some(format!("{}/{}", base_url, fragment.filename));
        }
    }

    if let Err(e) = tmp_dir.close() {
        eprintln!("Failed to clean up temporary directory: {}", e);
    }

    println!("Adding {} fragments to the database.", fragments.len());
    diesel::insert_into(schema::fragment::table)
        .values(&fragments)
        .execute(db)?;

    Ok(())
}

pub async fn fragment_media(input: impl AsRef<Path>, output_dir: impl AsRef<Path>, duration: usize) -> Result<Vec<model::NewFragment>> {
    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-stats")
        .arg("-y")
        .arg("-i")
        .arg(input.as_ref())
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("copy")
        .arg("-f")
        .arg("segment")
        .arg("-segment_time")
        .arg(duration.to_string())
        .arg("-reset_timestamps")
        .arg("1")
        .arg(output_dir.as_ref().join("fragment-%03d.mkv"))
        .status()
        .await?;

    if !status.success() {
        bail!("Failed to fragment media: status={:?}", status.code());
    }
    
    let mut fragments = vec![];
    let mut fragment_number = 0;

    // Waiting for completion of the command


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

async fn encrypt_file(input: impl AsRef<Path>, output: impl AsRef<Path>, pubkey: Box<dyn Recipient + Send>) -> Result<()> {
    let encryptor = age::Encryptor::with_recipients(vec![pubkey]).expect("Failed to create encryptor");

    let mut input_file = tokio::fs::File::open(input).await?;
    let mut output_file = tokio::fs::File::create(output).await?;

    let mut enc_writer = encryptor.wrap_async_output(output_file.compat()).await?;
    futures::io::copy(&mut input_file.compat(), &mut enc_writer).await?;

    Ok(())
}