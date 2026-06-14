use clap::{Parser, Subcommand};

mod api;
mod format;

#[derive(Parser)]
#[command(name = "worldcup")]
#[command(about = "FIFA World Cup 2026 live info CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Next,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let events = match &cli.command {
        Some(Commands::Next) => api::fetch_events(Some(7)).await?,
        None => api::fetch_events(None).await?,
    };

    if events.is_empty() {
        println!("No matches found.");
        return Ok(());
    }

    let is_next = matches!(&cli.command, Some(Commands::Next));
    let rows = format::build_table_rows(&events, is_next);

    let table = format::render_table(rows);
    println!("{}", table);

    Ok(())
}
