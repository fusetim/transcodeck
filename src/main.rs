use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub mod model;
pub mod schema;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long, env = "DATABASE_URL")]
    db_uri: String,    

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "List all media in the database")]
    ListMedia,
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
        Command::ListMedia => {
            println!("ListMedia");
        }
    }

    Ok(())
}

