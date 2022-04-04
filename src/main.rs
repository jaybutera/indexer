use blake3;
use std::sync::{Arc, RwLock};
use std::collections::{HashSet, HashMap};
use serde_json::Value;
use soup::prelude::*;

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

async fn bfs_crawl(root_domain: &str) {
    // BFS on the links, each link is an entry into a hashmap
    let start_url = format!("http://www.{root_domain}");
    let mut visited: HashSet<String> = HashSet::new();
    //let mut queue = Arc::new(RwLock::new(vec![start_url]));
    let mut queue = vec![start_url];
    let mut index = HashMap::new();

    let mut count_down = 10;
    //let mut index = Arc::new(RwLock::new(HashMap::new()));

    while let Some(url) = queue.pop() {
        count_down -= 1;
        if count_down == 0 {
            break;
        }

        dbg!(format!("visiting {url}"));

        if let Ok(resp) = get_html(&url).await {
            let soup = Soup::new(&resp);

            visited.insert(url.clone());
            for l in get_links_with_root(&soup, root_domain).difference(&visited) {
                queue.push(l.clone());
            }

            let text = soup.text();
            let word_set = text_to_set(text).await;
            //dbg!("{:?}", word_set.clone());
            dbg!("{} words", word_set.iter().count());
            index.insert(blake3::hash(&url.into_bytes()).to_string(), word_set);
        }
    }

    //let ser_index = index.into_iter().map(|(k,v)| k.
    std::fs::write("db", bincode::serialize(&index).unwrap());
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let url = "http://www.reddit.com/r/Bitcoin";
    let url = "reddit.com/r/Bitcoin";
    bfs_crawl(url).await;

    Ok(())
}
