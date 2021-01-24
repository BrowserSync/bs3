use std::str::FromStr;

use serde::{de, Serializer};

pub mod proxy_resp_mod;
pub mod service;

pub trait Proxy: Default {
    fn proxies(&self) -> Vec<ProxyTarget>;
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProxyTarget {
    #[serde(
        serialize_with = "serialize_proxy",
        deserialize_with = "deserialize_json_string"
    )]
    pub target: url::Url,
    pub paths: Vec<std::path::PathBuf>,
}

#[async_graphql::Object]
impl ProxyTarget {
    async fn target(&self) -> String {
        self.target.to_string()
    }
    async fn paths(&self) -> Vec<String> {
        self.paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<String>>()
    }
}

pub fn serialize_proxy<S>(input: &url::Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let as_string = input.to_string();
    serializer.serialize_str(&as_string)
}

pub fn deserialize_json_string<'de, D>(deserializer: D) -> Result<url::Url, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    url::Url::parse(s).map_err(de::Error::custom)
}

impl FromStr for ProxyTarget {
    type Err = ProxyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = s.split('~').collect();
        match items.len() {
            1 => {
                let url_part = items.get(0).expect("index 0 cannot fail here");
                Ok(ProxyTarget {
                    target: url::Url::parse(url_part)?,
                    paths: vec![],
                })
            }
            2 => {
                // separate each part
                let paths_part = items.get(0).expect("index 0 item present");
                let url_part = items.get(1).expect("index 1 item present");

                // process the paths
                let paths = paths_part
                    .split(',')
                    .map(std::path::PathBuf::from)
                    .collect::<Vec<std::path::PathBuf>>();

                // process the url
                let target = url::Url::parse(url_part)?;

                Ok(ProxyTarget { target, paths })
            }
            _ => todo!("cannot process proxy with more thn 3 segments when splitting on `~`"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error(
        "invalid proxy target: {0}

    Valid examples:

        bs3 --proxy http://example.com
        bs3 --proxy https://www.example.com

    "
    )]
    InvalidTarget(#[from] url::ParseError),
}

#[test]
fn test_serialize() -> anyhow::Result<()> {
    let p = ProxyTarget::from_str("http://www.example.com?hello-there=shane+Osbourne")?;
    let str = serde_json::to_string_pretty(&p)?;
    let expected = r#"{
  "target": "http://www.example.com/?hello-there=shane+Osbourne",
  "paths": []
}"#;
    assert_eq!(str, expected);
    Ok(())
}

#[test]
fn test_serialize_with_path() -> anyhow::Result<()> {
    let p = ProxyTarget::from_str("/gql~http://www.example.com/gql")?;
    let str = serde_json::to_string_pretty(&p)?;
    let expected = r#"{
  "target": "http://www.example.com/gql",
  "paths": [
    "/gql"
  ]
}"#;
    assert_eq!(str, expected);
    Ok(())
}

#[test]
fn test_deserialize_with_path() -> anyhow::Result<()> {
    let input = r#"{
  "target": "http://www.example.com/gql",
  "paths": [
    "/gql"
  ]
}"#;
    let str = serde_json::from_str::<ProxyTarget>(&input)?;
    assert_eq!(str.target.path(), "/gql");
    Ok(())
}

#[test]
fn test_from_str_err() {
    let p = ProxyTarget::from_str("///");
    println!("|{}|", p.unwrap_err());
}

#[test]
fn test_with_local_paths() {
    let input = "/gql,/~https://countries.trevorblades.com";
    let target = ProxyTarget::from_str(input).expect("test");
    assert_eq!(
        target.paths,
        vec![
            std::path::PathBuf::from("/gql"),
            std::path::PathBuf::from("/")
        ]
    );
    assert_eq!(
        target.target.into_string(),
        "https://countries.trevorblades.com/"
    );
}

#[test]
fn test_with_local_and_remote_paths() {
    let input = "/gql~https://countries.trevorblades.com/gql";
    let target = ProxyTarget::from_str(input).expect("test");
    assert_eq!(target.paths, vec![std::path::PathBuf::from("/gql")]);
    assert_eq!(
        target.target.into_string(),
        "https://countries.trevorblades.com/gql"
    );
}
