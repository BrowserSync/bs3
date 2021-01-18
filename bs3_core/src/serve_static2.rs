use async_graphql::{Object, Union};
use serde::de;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub struct ServeStaticConfig2 {
    pub routes: Vec<String>,
    #[serde(deserialize_with = "deserialize_dir")]
    pub dir: Option<PathBuf>,
}
