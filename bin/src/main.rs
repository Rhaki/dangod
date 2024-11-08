use {
    clap::Parser, ext::g_home_dir, genesis::GenesisCommand, std::path::PathBuf,
    tracing::level_filters::LevelFilter,
};

mod ext;
mod genesis;
mod types;

const DEFAULT_APP_DIR: &str = ".dagnod";

#[derive(Parser)]
#[command(author, version, about, next_display_order = None)]
struct Cli {
    #[arg(long, global = true)]
    home: Option<PathBuf>,

    #[arg(long, global = true, default_value = "info")]
    tracing_level: LevelFilter,

    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    #[command(subcommand, next_display_order = None, alias = "g")]
    Genesis(GenesisCommand),
    Start,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let app_dir = if let Some(dir) = cli.home {
        dir
    } else {
        g_home_dir()?.join(DEFAULT_APP_DIR)
    };

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }

    match cli.command {
        Command::Genesis(cmd) => cmd.run(app_dir),
        Command::Start => {
            // Start the app
            std::process::Command::new("cometbft")
                .arg("start")
                .spawn()?;

            std::process::Command::new("dango").arg("start").status()?;

            Ok(())
        }
    }
}
