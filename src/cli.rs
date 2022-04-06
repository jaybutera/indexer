//use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    /// Search pages for a set of keywords
    #[structopt(short, long)]
    pub query: Option<Vec<String>>,

    /// Max number of links to visit during indexing
    #[structopt(short, long, default_value="10")]
    pub limit: u64,

    /// Root domain (after www) to search. Search will stay within the root.
    /// For example reddit.com/r/bitcoin
    #[structopt(short, long)]
    pub url_root: String,
}
