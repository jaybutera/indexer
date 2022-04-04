mod cli;

use structopt::StructOpt;
use std::sync::{Arc, RwLock};
use std::collections::{HashSet, HashMap};
//use sublime_fuzzy::FuzzySearch;
use serde_json::Value;
use soup::prelude::*;
use crate::cli::Opt;

async fn get_json(url: &str) -> reqwest::Result<Value> {
    reqwest::get(url)
        .await?
        .json::<Value>()
        .await
}

async fn get_html(url: &str) -> reqwest::Result<String> {
    let resp = reqwest::get(url)
        .await?;
    if !resp.status().is_success() {
        panic!("HTTP get: {:?}", resp);
    }
    resp.text()
        .await
}

async fn text_to_set(text: String) -> HashSet<String> {
    tokio::task::spawn_blocking(move || {
        let words = text.split(&[' ', '"', ':', ',', '\\', '\n', '.']);
        let mut hm = std::collections::HashSet::new();

        for word in words {
            hm.insert(word.into());
        }
        hm
    }).await.expect("tokio join error")
}

fn get_links_with_root(soup: &Soup, root: &str) -> HashSet<String> {
    soup.tag("a")
        .find_all()
        .map(|n| n.get("href").unwrap_or("".into()))
        .filter(|s| s != "")
        .filter(|s| s.contains(root))
        .collect()
}

async fn bfs_crawl(root_domain: &str, limit: u64) {
    // BFS on the links, each link is an entry into a hashmap
    let start_url = format!("http://www.{root_domain}");
    let mut visited: HashSet<String> = HashSet::new();
    let queue = Arc::new(RwLock::new(vec![start_url]));
    let index: Arc<RwLock<HashMap<String, HashSet<String>>>> = Arc::new(RwLock::new(HashMap::new()));
    //let mut queue = vec![start_url];
    //let mut index = HashMap::new();

    let mut count_down = limit;

    let mut last = queue.write().unwrap().pop();
    //while let Some(url) = queue.write().expect("first lock failed").pop() {
    while let Some(url) = last {
        count_down -= 1;
        if count_down == 0 {
            break;
        }

        dbg!(format!("visiting {url}"));

        if let Ok(resp) = get_html(&url).await {
            let soup = Soup::new(&resp);

            visited.insert(url.clone());
            let mut new_links = get_links_with_root(&soup, root_domain)
                .difference(&visited)
                .into_iter().cloned()
                .collect();
            queue.write().unwrap().append(&mut new_links);
            /*
            for l in get_links_with_root(&soup, root_domain).difference(&visited) {
                queue.write().expect("wtf").push(l.clone());
            }
            */

            let text = soup.text();
            let word_set = text_to_set(text).await;
            //dbg!("{:?}", word_set.clone());
            //dbg!(format!("{} words", word_set.iter().count()));
            dbg!(word_set.iter().count());
            //index.write().unwrap().insert(blake3::hash(&url.into_bytes()).to_string(), word_set);
            index.write().unwrap().insert(url, word_set);
        }

        last = queue.write().unwrap().pop();
    }

    //let ser_index = index.into_iter().map(|(k,v)| k.
    std::fs::write("db", bincode::serialize(&*index.read().unwrap()).unwrap());
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    if let Some(keywords) = opt.keywords {
        let db: HashMap<String, HashSet<String>> = bincode::deserialize(&std::fs::read("db").unwrap()).unwrap();
        let mut res_pages = HashSet::new();
        for (url, page) in db {
            //let res = FuzzySearch::new("bitcoin bull run"
            if keywords.iter().fold(true, |acc, k| acc && page.contains(k)) {
                res_pages.insert(url);
            }
        }

        for page in res_pages {
            println!("{page}");
        }
    }
    else {
        bfs_crawl(&opt.url_root, opt.limit).await;
    }

    Ok(())
}
