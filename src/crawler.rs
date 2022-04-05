use soup::prelude::*;
use std::collections::{HashSet, HashMap};
use crate::types::{DocId, Term, Span, ForwardIndex, InvertedIndex};
use std::sync::{Arc, RwLock};
use tokio::time::Duration;

pub async fn invert_index_process(
    forward_index: Arc<RwLock<ForwardIndex>>,
) {
    let db = sled::open("db").unwrap();

    loop {
        // Wipe the forward index
        let elems = forward_index.write().unwrap().drain().collect::<Vec<_>>();
        let mut index: InvertedIndex = HashMap::new();

        // Fill into the inverted index
        for (docid, hs) in elems {
            for (term, span) in hs {
                let h = index.entry(term).or_insert(HashSet::new());
                h.insert((docid.clone(), span));
            }
            //db.insert(docid, bincode::serialize(&span).unwrap()).unwrap();
        }

        for (term, entry) in index {
            db.insert(term, bincode::serialize(&entry).unwrap()).unwrap();
        }

        //std::fs::write("db", bincode::serialize(&*index.read().unwrap()).unwrap());

        db.flush().unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn crawler_process(
    index: Arc<RwLock<ForwardIndex>>,
    root_domain: &str,
    limit: u64)
{
    let start_url = format!("http://www.{root_domain}");
    let mut visited: HashSet<DocId> = HashSet::new();
    let queue: Arc<RwLock<Vec<DocId>>> = Arc::new(RwLock::new(vec![start_url]));
    let mut last = queue.write().unwrap().pop();
    let db = sled::open("pages_db").unwrap();

    let mut count_down = limit;

    while let Some(url) = last {
        count_down -= 1;
        if count_down == 0 {
            break;
        }

        dbg!(format!("visiting {url}"));

        if let Ok(resp) = get_html(&url).await {
            // Insert page in db
            db.insert(url.as_bytes(), resp.as_bytes()).unwrap();

            let soup = Soup::new(&resp);

            visited.insert(url.clone());
            let mut new_links = get_links_with_root(&soup, root_domain)
                .difference(&visited)
                .into_iter().cloned()
                .collect();
            queue.write().unwrap().append(&mut new_links);

            let text = soup.text();
            let word_set = text_to_set(text).await;
            dbg!(word_set.iter().count());

            // TODO
            // Write a concurrent task which converts this index into the inverted index and saves
            // to disk.
            index.write().unwrap().insert(url, word_set);
        }

        last = queue.write().unwrap().pop();
    }
}

/*
async fn get_json(url: &str) -> reqwest::Result<Value> {
    reqwest::get(url)
        .await?
        .json::<Value>()
        .await
}
*/

/// Get html from a url
async fn get_html(url: &str) -> reqwest::Result<String> {
    let resp = reqwest::get(url)
        .await?;
    if !resp.status().is_success() {
        panic!("HTTP get: {:?}", resp);
    }
    resp.text()
        .await
}

/// Find all links in an html document that have the provided root
fn get_links_with_root(soup: &Soup, root: &str) -> HashSet<DocId> {
    soup.tag("a")
        .find_all()
        .map(|n| n.get("href").unwrap_or("".into()))
        .filter(|s| s != "")
        .filter(|s| s.contains(root))
        .collect()
}

/// Generate a set of words and their corresponding locations in a string
async fn text_to_set(text: String) -> HashSet<(Term, Span)> {
    tokio::task::spawn_blocking(move || {
        let words = split_text(&text);//text.split(&[' ', '"', ':', ',', '\\', '\n', '.']);
        let mut hm = std::collections::HashSet::new();

        for (i, word) in words.enumerate() {
            hm.insert((word.into(), (i as u64, i as u64)));
        }
        hm
    }).await.expect("tokio join error")
}

pub fn split_text(text: &str) -> std::str::Split<'_, &[char; 7]> {
    text.split(&[' ', '"', ':', ',', '\\', '\n', '.'])
}
