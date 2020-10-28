use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug, Eq, PartialEq)]
pub struct Args {
    #[structopt(long = "index")]
    pub index: Option<PathBuf>,
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>
}

#[test]
fn test_parse_args_with_index() {
    let args = Args::from_iter(vec!["prog", "--index", "index.htm", "."]);
    let expected = Args {
        index: Some(PathBuf::from("index.htm")),
        paths: vec![PathBuf::from(".")],
    };
    assert_eq!(expected, args);
}

#[test]
fn test_parse_with_paths() {
    let cmd = "prog . fixtures".split(" ");
    let args = Args::from_iter(cmd);
    let expected = Args {
        index: None,
        paths: vec![PathBuf::from("."), PathBuf::from("fixtures")],
    };
    assert_eq!(expected, args);
}
