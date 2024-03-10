use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::path::PathBuf;

pub mod model;
pub mod schema;
pub mod add_media;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URI of the database to connect to
    #[clap(long, env = "DATABASE_URL")]
    db_uri: String,    

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Add media to the database")]
    AddMedia(AddMediaCommand),

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

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("Error loading .env file: {}", err);
    }
    pretty_env_logger::init();
    let args = Args::parse();

    let mut db = PgConnection::establish(&args.db_uri)?;

    match args.cmd {
        Command::AddMedia(cmd) => {
            add_media::add_media(&mut db, cmd).await?;
        }
        Command::ListMedia => {
            println!("ListMedia");
        }
    }

    Ok(())
}

