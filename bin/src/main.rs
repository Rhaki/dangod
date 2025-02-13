use {clap::Parser, std::path::PathBuf, tracing::level_filters::LevelFilter};

pub mod ext;
pub mod genesis;
pub mod types;

pub use {ext::*, genesis::*, types::*};

pub const DEFAULT_APP_DIR: &str = ".dagnod";

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
    Build,
    Generate { counter: usize },
    GenerateStatic,
    Reset,
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
        Command::Start => {
            std::process::Command::new("cometbft")
                .arg("start")
                .spawn()?;
            std::process::Command::new("dango").arg("start").status()?;
            Ok(())
        }
        Command::Build => genesis::build(app_dir),
        Command::Generate { counter } => genesis::generate_random(app_dir, counter),
        Command::GenerateStatic => genesis::generate_static(app_dir),
        Command::Reset => genesis::reset(),
    }
}
