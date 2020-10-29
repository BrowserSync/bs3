use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use serde::{de, Deserialize, Deserializer};
use std::fmt;

pub trait ServeStatic: Default {
    fn serve_static_config(&self) -> Vec<ServeStaticConfig>;
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ServeStaticConfig {
    #[serde(deserialize_with = "deserialize_dir")]
    DirOnly(PathBuf),
    Multi {
        routes: Vec<PathBuf>,
        #[serde(deserialize_with = "deserialize_dir")]
        dir: PathBuf
    }
}

impl ServeStaticConfig {
    pub fn from_dir_only(path: impl Into<PathBuf>) -> Self {
        ServeStaticConfig::DirOnly(path.into())
    }
}

impl Default for ServeStaticConfig {
    fn default() -> Self {
        ServeStaticConfig::from_dir_only(".")
    }
}

impl FromStr for ServeStaticConfig {
    type Err = ServeStaticError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("fromStr called with :) {} ", s);
        Ok(ServeStaticConfig::from_dir_only(s))
    }
}


#[derive(Error, Debug)]
pub enum ServeStaticError {
    #[error("Invalid serve static option")]
    Invalid,
    #[error("unknown serve static error")]
    Unknown,
}


///
/// Helpers for deserializing a dir argument
///
/// todo: add verification here
///
pub fn deserialize_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: de::Deserializer<'de>,
{
    struct DirVisitor;

    impl<'de> de::Visitor<'de> for DirVisitor {
        type Value = PathBuf;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either `7.1`, `7.2`, `7.3` or `7.4`")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            // let r: Result<PathBuf, _> = Ok();
            // r.map_err(E::custom)
            match ServeStaticConfig::from_str(v) {
                Ok(ServeStaticConfig::DirOnly(pb)) => Ok(pb),
                _ => unreachable!("should not get here when deserializing a dir")
            }
        }
    }

    deserializer.deserialize_any(DirVisitor)
}
