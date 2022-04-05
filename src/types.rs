use std::collections::{HashMap, HashSet};

/// The from and to word positions of a [Term] in a document
pub type Span = (u64, u64);
/// An identifier by which things are indexed (like words in a doc or tags)
pub type Term = String;
/// A document id, typically a url to an html page
pub type DocId = String;
//type Index = HashMap<String, (String, Span)>;
/// An index mapping a [Term] to documents and the location where it occurs
pub type InvertedIndex = HashMap<Term, HashSet<(DocId, Span)>>;
pub type ForwardIndex = HashMap<DocId, HashSet<(Term, Span)>>;
