use crate::config::Config;
use structopt::StructOpt;

#[derive(Debug, Clone, Default)]
pub struct BrowserSync {
    pub config: Config
}

impl BrowserSync {
    pub fn from_args(args: impl Iterator<Item = String>) -> Self {
        let config: Config = Config::from_iter(args);
        Self { config }
    }
    pub fn try_from_args<'a>(args: impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let config: Config = Config::from_iter_safe(args)?;
        Ok(Self { config })
    }
}

#[test]
fn test_try() {
    let args = vec!["prog", ".", "hello"].into_iter();
    let bs = BrowserSync::try_from_args(args);
    dbg!(bs);
}
