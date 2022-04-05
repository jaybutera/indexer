use soup::Soup;
use crate::crawler;
use crate::types::{DocId, Span, Term};
use std::collections::HashSet;

pub fn fuzzy_phrase(
    index: Vec<(Term, HashSet<(DocId, Span)>)>,
    db: sled::Db)
-> Vec<(isize, DocId, Span)> {
    let mut matches = vec![];
    let phrase = index.iter().map(|(t,_)| t.clone()).collect::<Vec<_>>().join(" ");
    if let Some((_, h)) = index.get(0) {
        for (docid, span) in h {
            let raw_doc = db.get(docid).expect(&format!("Couldn't find docid {} in db", docid)).unwrap();
            let doc = String::from_utf8_lossy(&raw_doc);

            let soup = Soup::new(&doc);
            let text = soup.text();
            let words = crawler::split_text(&text);

            // Fuzzy search the query phrase at the occurance of the first term
            /*
            dbg!(phrase.clone(),
                 words.clone().skip((span.0) as usize).take(index.len()).collect::<Vec<_>>().join(" "));
            */
            let res = sublime_fuzzy::FuzzySearch::new(
                    &phrase,
                    &words.skip((span.0) as usize).take(index.len()).collect::<Vec<_>>().join(" ")
                ).best_match();

            if let Some(m) = res {
                matches.push((m.score(), docid.clone(), span.clone()));
            }
        }

        // Sort by ascending scores
        matches.sort_by_key(|m| m.0);

        matches
    }
    else {
        vec![]
    }
}
