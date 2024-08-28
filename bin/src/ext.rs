use std::path::{Path, PathBuf};

use grug::{Binary, JsonDeExt};
use home::home_dir;
use serde::de::DeserializeOwned;

pub trait PathBuffExt {
    fn read<T: DeserializeOwned>(&self) -> anyhow::Result<T>;

    fn read_raw(&self) -> anyhow::Result<Vec<u8>>;
}

impl PathBuffExt for PathBuf {
    fn read<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        std::fs::read(self)
            .map_err(|_| anyhow::anyhow!("Failed to read file: {:?}", self))?
            .deserialize_json()
            .map_err(Into::into)
    }

    fn read_raw(&self) -> anyhow::Result<Vec<u8>> {
        std::fs::read(self).map_err(Into::into)
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

pub fn read_wasm_file(filename: &str) -> anyhow::Result<Binary> {
    let path = PathBuf::from(format!(
        "{}/../artifacts/{filename}",
        env!("CARGO_MANIFEST_DIR")
    ));

    path.read_raw().map(Into::into)
}
