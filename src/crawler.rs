use soup::prelude::*;
use std::collections::{VecDeque, HashSet, HashMap};
use crate::types::{DocId, Term, Span, ForwardIndex, InvertedIndex};
use std::sync::{Arc, RwLock};
use tokio::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlError {
    #[error("reqwest error")]
    ReqwestErr(#[from] reqwest::Error),
    #[error("unsuccessful http request")]
    BadHttp(reqwest::Response),
    //#[error("failed to get html text")]
    //ParseErr(#[from] Result<String, reqwest::Error>),
}


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
    start_url: String,
    limit: u64)
{
    //let start_url = format!("https://{root_domain}");
    //let start_url = format!("{root_domain}");
    let mut visited: HashSet<DocId> = HashSet::new();
    let queue: Arc<RwLock<VecDeque<DocId>>> = Arc::new(RwLock::new(VecDeque::from([start_url.clone()])));
    let mut last = queue.write().unwrap().pop_front();
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
            let re = regex::Regex::new(r"(https|http)://*").unwrap();
            let root_domain = re.replace(&start_url, "");
            dbg!(root_domain.clone());
            let mut new_links: VecDeque<String> = get_links_with_root(&soup, &root_domain)
                .difference(&visited)
                .into_iter().cloned()
                .collect();

            // Fix protocol-less urls (section 4.2 RFC 3986)
            new_links = new_links.into_iter().map(|l|
                if l.starts_with("//") {
                    ["http:", &l].join("")
                } else {
                    l
                })
                .collect();

            queue.write().unwrap().append(&mut new_links);

            let text = soup.text();
            let word_set = text_to_set(text).await;
            dbg!(word_set.iter().count());

            index.write().unwrap().insert(url, word_set);
        } else {
            panic!("No more links to follow!");
        }

        // TODO kind of a hack
        //last = Some(queue.write().unwrap().remove(0));
        last = queue.write().unwrap().pop_front();
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
//async fn get_html(url: &str) -> reqwest::Result<String> {
async fn get_html(url: &str) -> Result<String, CrawlError> {
    let resp = reqwest::get(url)
        .await?;
    if !resp.status().is_success() {
        //dbg!("HTTP get: {:?}", resp);
        Err(CrawlError::BadHttp(resp))
    } else {
        resp.text().await
            .map_err(CrawlError::ReqwestErr)
    }
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
