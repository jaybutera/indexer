mod cli;
mod types;
mod crawler;
mod scoring;

use crawler::{invert_index_process, crawler_process};
use types::{Span, DocId, Term, InvertedIndex, ForwardIndex};
use structopt::StructOpt;
use std::sync::{Arc, RwLock};
use std::collections::{HashSet, HashMap};
use crate::cli::Opt;


async fn bfs_crawl(root_domain: &str, limit: u64) {
    // BFS on the links, each link is an entry into a hashmap
    let index: Arc<RwLock<ForwardIndex>> = Arc::new(RwLock::new(HashMap::new()));

    let inverter_task = invert_index_process(index.clone());
    let crawler = crawler_process(index, root_domain, limit);

    tokio::join!(inverter_task, crawler);
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    if let Some(keywords) = opt.keywords {
        let pages_db = sled::open("pages_db")?;
        let index_db = sled::open("db")?;

        let mut l: Vec<(String, HashSet<(DocId, Span)>)> = Vec::new();
        //println!("{:?}", index_db.iter().map(|r| if let Ok((k,_)) = r { String::from_utf8_lossy(&k).to_string() } else { "-".into() }).collect::<Vec<_>>());
        for k in keywords {
            if let Ok(Some(v)) = index_db.get(k.clone()) {
                l.push((k, bincode::deserialize(&v).expect("failed to deserialize index db")));
            }
        }

        //println!("{l:?}");
        let matches = scoring::fuzzy_phrase(l, pages_db.clone());

        if matches.is_empty() {
            println!("no matches found");
        }
        for m in matches {
            let raw_doc = pages_db.get(&m.1).unwrap().unwrap();
            let doc = String::from_utf8_lossy(&raw_doc);
            let soup = soup::Soup::new(&doc);
            let text = soup.text();
            let words = crawler::split_text(&text);
            let excerpt = words.skip((m.2.0-2) as usize).take(10).collect::<Vec<_>>().join(" ");

            println!("{}\n- \"{}\"", m.1, excerpt);
        }

    }
    else {
        bfs_crawl(&opt.url_root, opt.limit).await;
    }

    Ok(())
}
