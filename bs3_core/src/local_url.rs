use async_graphql::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, async_graphql::SimpleObject)]
pub struct LocalUrl {
    #[serde(serialize_with = "crate::proxy::serialize_proxy")]
    pub inner: url::Url,
}

impl Default for LocalUrl {
    fn default() -> Self {
        Self {
            inner: url::Url::parse("http://0.0.0.0:8080").expect("valid input"),
        }
    }
}

impl LocalUrl {
    pub fn try_from_port(port: Option<u16>) -> anyhow::Result<Self> {
        let mut local_url = Self::default();
        if let Some(port) = port {
            log::trace!("setting port {}", port);
            local_url
                .inner
                .set_port(Some(port))
                .map_err(|_e| anyhow::anyhow!("Could not set the port!"))?;
            Ok(local_url)
        } else {
            Ok(local_url)
        }
    }
}

impl<'de> serde::Deserialize<'de> for LocalUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let as_url = url::Url::parse(s.as_str()).map_err(serde::de::Error::custom)?;
        Ok(LocalUrl { inner: as_url })
    }
}

#[cfg(test)]
mod test {
    use crate::local_url::LocalUrl;

    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let lu = LocalUrl::try_from_port(Some(9090))?;
        let as_str = serde_json::to_string(&lu)?;
        let expected = r#""http://0.0.0.0:9090/""#;
        assert_eq!(as_str, expected);
        let as_url: LocalUrl = serde_json::from_str(expected)?;
        assert_eq!(as_url, LocalUrl::try_from_port(Some(9090))?);
        Ok(())
    }
}
