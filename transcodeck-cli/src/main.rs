use clap::{Parser, Subcommand};
use anyhow::{Result, anyhow, bail};
use sea_orm::{Database, DatabaseConnection};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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

    let mut db : DatabaseConnection = Database::connect(args.db_uri).await?;

    match args.cmd {
        Command::ListMedia => {
            println!("ListMedia");
        }
    }

    db.close().await?;
    Ok(())
}
