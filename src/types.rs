use std::collections::{HashMap, HashSet};

pub type Span = (u64, u64);
pub type DocId = String;
//type Index = HashMap<String, (String, Span)>;
pub type InvertedIndex = HashMap<String, HashSet<(DocId, Span)>>;
