use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::path::PathBuf;

pub mod add_media;
pub mod add_transcode;
pub mod daemon;
pub mod model;
pub mod schema;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URI of the database to connect to
    #[clap(long, env = "DATABASE_URL")]
    db_uri: String,

    /// FFmpeg bin to use for transcoding
    #[clap(long, env = "FFMPEG_BIN", default_value = "ffmpeg")]
    ffmpeg_bin: String,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Add media to the database")]
    AddMedia(AddMediaCommand),

    #[command(about = "Add a transcoding job")]
    Transcode(TranscodeCommand),

    #[command(about = "Start the transcoding daemon")]
    Daemon(DaemonCommand),

    //    #[command(about = "Add a transcoding fragment job")]
    //    TranscodeFragment(TranscodeFragmentCommand),
    #[command(about = "List all media in the database")]
    ListMedia,
}

#[derive(Parser, Debug)]
pub struct AddMediaCommand {
    /// The path to the media file to add
    input: PathBuf,

    /// The URL to use for retrieval of this particular media.
    #[clap(short, long)]
    retrieval_url: Option<String>,

    /// Encryption flag, if set, the media will be encrypted.
    #[clap(short, long, default_value = "false")]
    encrypted: bool,

    /// Fragment the media into smaller pieces, every n seconds.
    /// If set to 0, the media will not be fragmented.
    #[clap(short, long, default_value = "0")]
    fragment: u32,

    /// The output path where every fragment will be stored.
    /// If not set, the fragments will be stored in a sub-directory where the input file is stored.
    /// If the media is not fragmented nor encrypted, this flag is ignored.
    #[clap(short, long)]
    output_dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct TranscodeCommand {
    /// The media ID to transcode
    media_id: String,

    /// The ffmpeg command to use for transcoding
    ffmpeg_command: String,

    /// Start flag, if set, the transcoding job will be queued to be processed immediately.
    #[clap(short, long, default_value = "false")]
    start: bool,
}

// #[derive(Parser, Debug)]
// pub struct TranscodeFragmentCommand {
//     /// The fragment ID to transcode
//     fragment_id: String,
//
//     /// The ffmpeg command to use for transcoding
//     ffmpeg_command: String,
// }

#[derive(Parser, Debug)]
pub struct DaemonCommand {
    /// Output directory for transcoded media
    output_dir: PathBuf,

    /// Reserve flag, should the daemon try to reserve more jobs than it can process?
    #[clap(short, long, default_value = "false")]
    reserve: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("Error loading .env file: {}", err);
    }
    pretty_env_logger::init();
    let args = Args::parse();

    let mut db = PgConnection::establish(&args.db_uri)?;
    let ffmpeg_bin = args.ffmpeg_bin.clone();

    match args.cmd {
        Command::AddMedia(cmd) => {
            add_media::add_media(&mut db, cmd, &ffmpeg_bin).await?;
        }
        Command::Daemon(cmd) => daemon::daemon(&mut db, cmd, &ffmpeg_bin).await?,
        Command::ListMedia => {
            println!("ListMedia");
        }
        Command::Transcode(cmd) => add_transcode::new_transcode(&mut db, cmd).await?,
    }

    Ok(())
}
