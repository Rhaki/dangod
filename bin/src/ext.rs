use {
    grug::JsonDeExt,
    home::home_dir,
    serde::de::DeserializeOwned,
    std::{
        path::{Path, PathBuf},
        str::FromStr,
    },
};

pub trait PathBuffExt {
    fn read<T: DeserializeOwned>(&self) -> anyhow::Result<T>;

    fn read_string<T>(&self) -> anyhow::Result<T>
    where
        T: FromStr,
        T::Err: std::error::Error + Send + Sync + 'static;
}

impl PathBuffExt for PathBuf {
    fn read<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        std::fs::read(self)
            .map_err(|_| anyhow::anyhow!("Failed to read file: {:?}", self))?
            .deserialize_json()
            .map_err(Into::into)
    }

    fn read_string<T>(&self) -> anyhow::Result<T>
    where
        T: FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        Ok(std::fs::read_to_string(self)
            .map_err(|_| anyhow::anyhow!("Failed to read file: {:?}", self))?
            .parse()?)
        // .map_err(Into::into)
    }
}

pub trait Writer {
    fn write_pretty_json<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()>;
}

impl<T> Writer for T
where
    T: serde::Serialize,
{
    fn write_pretty_json<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        std::fs::write(path.as_ref(), serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

pub fn g_home_dir() -> anyhow::Result<PathBuf> {
    home_dir().ok_or_else(|| anyhow::anyhow!("Failed to find home directory"))
}

pub fn cometbft_genesis_path() -> anyhow::Result<PathBuf> {
    Ok(g_home_dir()?.join(".cometbft/config/genesis.json"))
}

pub fn cometbft_config_path() -> anyhow::Result<PathBuf> {
    Ok(g_home_dir()?.join(".cometbft/config/config.toml"))
}
