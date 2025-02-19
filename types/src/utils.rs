use std::path::PathBuf;

pub fn home_dir() -> anyhow::Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to find home directory"))
}

pub fn cometbft_genesis_path() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".cometbft/config/genesis.json"))
}

pub fn cometbft_config_path() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".cometbft/config/config.toml"))
}
