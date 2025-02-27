use {
    clap::Parser,
    dangod_types::{home_dir, DANGOD_APP_DIR},
    std::path::PathBuf,
    tracing::level_filters::LevelFilter,
};

pub mod genesis;

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
    Build {
        #[arg(long)]
        docker: bool,
        #[arg(long, default_value = "88888888")]
        hyperlane_domain: u32,
    },
    BuildMsgs,
    Generate {
        counter: usize,
    },
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
        home_dir()?.join(DANGOD_APP_DIR)
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
        Command::Build {
            docker,
            hyperlane_domain,
        } => genesis::build(app_dir, docker, hyperlane_domain),
        Command::Generate { counter } => genesis::generate_random(app_dir, counter),
        Command::GenerateStatic => genesis::generate_static(app_dir),
        Command::Reset => genesis::reset(),
        Command::BuildMsgs => genesis::build_msgs(app_dir),
    }
}
