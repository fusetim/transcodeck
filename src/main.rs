use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;

pub mod model;

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

