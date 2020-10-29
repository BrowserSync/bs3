use std::path::PathBuf;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use crate::serve_static::{ServeStatic, ServeStaticConfig};
use crate::browser_sync::BrowserSync;

#[derive(Default, StructOpt, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[structopt(long = "serve-static", short = "ss")]
    #[serde(rename = "serveStatic")]
    pub serve_static: Option<Vec<ServeStaticConfig>>,
    #[structopt(long = "index")]
    pub index: Option<String>,
    #[structopt(parse(from_os_str))]
    #[serde(default)]
    pub trailing_paths: Vec<PathBuf>,
}

impl ServeStatic for Config {
    fn serve_static_config(&self) -> Vec<ServeStaticConfig> {
        let mut output = vec![];
        for pb in &self.trailing_paths {
            output.push(ServeStaticConfig::from_dir_only(&pb))
        }
        output.extend(self.serve_static.clone().unwrap_or(vec![]));
        output
    }
}

#[test]
fn test_deserialize() -> std::io::Result<()> {
    let input = r#"
    {
        "serveStatic": [
            {
                "routes": ["/node_modules", "react"],
                "dir": "node_modules"
            },
            "static"
        ],
        "trailing_paths": ["."]
    }
    "#;
    let config = serde_json::from_str::<Config>(input)?;
    let ss = config.serve_static_config();
    assert_eq!(ss[0], ServeStaticConfig::from_dir_only("."));
    Ok(())
}


#[test]
fn test_from_args() -> anyhow::Result<()> {
    let args = "prog . --serve-static static";
    let bs = BrowserSync::try_from_args(args.split(" "))?;
    let ss = bs.config.serve_static_config();
    assert_eq!(
        vec![
            ServeStaticConfig::from_dir_only("."),
            ServeStaticConfig::from_dir_only("static"),
        ],
        ss
    );
    Ok(())
}
