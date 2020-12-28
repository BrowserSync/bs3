#[derive(Debug, Clone)]
pub struct LocalUrl(pub url::Url);

impl Default for LocalUrl {
    fn default() -> Self {
        Self(url::Url::parse("http://0.0.0.0:8080").expect("valid input"))
    }
}

impl LocalUrl {
    pub fn try_from_port(port: Option<u16>) -> anyhow::Result<Self> {
        let mut local_url = Self::default();
        if let Some(port) = port {
            log::trace!("setting port {}", port);
            local_url
                .0
                .set_port(Some(port))
                .map_err(|_e| anyhow::anyhow!("Could not set the port!"))?;
            Ok(local_url)
        } else {
            Ok(local_url)
        }
    }
}
