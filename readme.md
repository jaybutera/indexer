```
cargo run -- --root_domain stackoverflow.com/questions --limit 100
cargo run -- --root_domain stackoverflow.com/questions --query rust lifetimes
```

This is an investigation into the viability of creating high quality
domain-specific indexes of knowledge bases, such as the web, and search engines
which can find more meaningful, personal results than, say, Google.

With monolithic search engines today we suffer from their black box nature, not
knowing why they choose the results they show us and not being able to
delibarately configure that.

Furthermore, indexing data should be separate from the search algorithms over
the data. This way index databases can compose without imposing a specific
search strategy. Search algorithms can compete as general strategies or can be
applied for specific domains. For instance a programmer may want a search
algorithm with a strong semantic understanding of algorithms and code.

Freedom of choice in search strategies and diversity in indexing also makes SEO
more difficult to fit universally. New search and indexing methods can filter
out the Google/Facebook SEO-tailored garbage. So when you have a question like
"is there a non-mechanical split keyboard?", you find a conversation about
a niche product on reddit rather than a "top 10 keyboards of 2022" from PC
Magazine.

---

This decentralized search engine marketplace can be divided into three layers.

1. Raw web databases. Actively maintained with crawlers, and requiring fairly
   large storage capacity.

2. Indexes, which are finding novel and niche ways to index large raw data.

3. Search algorithms which may specialize for specific use cases and domains,
   or media types etc.


Search algorithms can run anywhere, they don't require especially high
performance storage and computation. Indexes and raw databases won't be stored
on a personal computer and likely people will not want to download them like
torrent files and host them on a local NAT. If not for any other reason than
that they often need to be actively maintained and updated.
