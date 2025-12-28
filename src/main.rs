use color_eyre::Result;
use clap::Parser;
use tnj::{Config, Database, Profile, cli::{Cli, Commands}};

fn main() -> Result<()> {
    // Set up error reporting with color-eyre
    color_eyre::install()?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Determine profile: --dev flag enables dev mode, otherwise use prod
    let profile = if cli.dev {
        Profile::Dev
    } else {
        Profile::Prod
    };

    // Load configuration with the determined profile
    // Note: --config option is parsed but not yet used to override config path
    // This can be enhanced in the future if needed
    let config = Config::load_with_profile(profile)?;

    // Initialize database
    let db_path = config.get_database_path();
    let db = Database::new(
        db_path.to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Database path contains invalid UTF-8"))?
    )?;

    // Dispatch to appropriate command handler
    match cli.command {
        Commands::Tui => {
            let app = tnj::tui::App::new(config, db)?;
            tnj::tui::run_event_loop(app)?;
        }
        Commands::AddTask { title, due, tags } => {
            tnj::cli::handle_add_task(title, due, tags, &db)?;
        }
        Commands::AddNote { title, content, tags } => {
            tnj::cli::handle_add_note(title, content, tags, &db)?;
        }
        Commands::AddJournal { content, title, tags } => {
            tnj::cli::handle_add_journal(content, title, tags, &db)?;
        }
    }

    Ok(())
}
