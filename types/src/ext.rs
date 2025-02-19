use {
    grug::JsonDeExt,
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
