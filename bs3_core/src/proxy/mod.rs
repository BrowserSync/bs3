use actix_service::Service;

use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serializer};

pub mod service;

pub trait Proxy: Default {
    fn proxies(&self) -> Vec<ProxyTarget>;
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ProxyTarget {
    #[serde(serialize_with = "serialize_proxy")]
    pub target: url::Url,
}

fn serialize_proxy<S>(input: &url::Url, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let as_string = input.to_string();
    serializer.serialize_str(&as_string)
}

impl FromStr for ProxyTarget {
    type Err = ProxyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProxyTarget {
            target: url::Url::parse(s)?,
        })
    }
}

impl<'de> Deserialize<'de> for ProxyTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
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
fn test_serialize() {
    let p =
        ProxyTarget::from_str("http://www.example.com?hello-there=shane+Osbourne").expect("test");
    let str = serde_json::to_string_pretty(&p).expect("json");
    let expected = r#"{
  "target": "http://www.example.com/?hello-there=shane+Osbourne"
}"#;
    assert_eq!(str, expected);
}

#[test]
fn test_from_str_err() {
    let p = ProxyTarget::from_str("///");
    println!("|{}|", p.unwrap_err());
}
