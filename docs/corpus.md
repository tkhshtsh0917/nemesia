# Building a candidate corpus

[日本語](ja/corpus.md) · [Matching specification](matching.md)

## Default source

Generate Japanese completion terms from the same documents and fields that the host will search after submission. Use the document snapshot ID and a UTF-8 JSONL projection such as `{"id":"doc-1","text":"りんごを食べた。"}`. If the host searches questions, export question text. If it searches titles and bodies, export those fields in a deterministic order under one document ID. Exclude fields outside the search scope, such as display-only snippets. The input requires unique document IDs.

Use [Sudachi's Rust implementation](https://github.com/WorksApplications/sudachi.rs) in the native CLI, with one locked dictionary snapshot and a versioned extraction policy. Begin with Sudachi part-of-speech categories `名詞`, `動詞` and `形容詞` and split mode B. Reject empty keys and apply a configurable stopword/stoptag policy. The defaults exclude unknown words, numerals, dependent verbs and a short list of high-frequency terms. The policy's `aliases` field gives reviewed readings priority if the dictionary supplies only one. For example, a policy can add `nihon` for 日本 when Sudachi supplies `nippon`. Long vowels can be represented as `biitoruzu`, `bi-toruzu` and `びーとるず`.

## Extract, aggregate and compile

1. Keep Sudachi's normalized form for nouns, and normalized dictionary-form lemmas for verbs and adjectives. Group by the normalized display key. When multiple display forms remain, keep the one that sorts first by UTF-8 bytes. Keep lexical terms instead of splitting them into individual characters. For example, `食べた` can contribute to the displayed candidate `食べる`.
2. Count each term at most once per input document. Use the number of distinct source documents as its default `weight`. Repetition within one article does not raise its weight. Preserve explicitly reviewed aliases for eligible surface variants, and add dictionary readings and romanizations within the same per-candidate limit. Alias frequency does not affect rank. Keep distinct terms separate when they only share a reading.
3. Emit deterministic candidate JSONL. For extracted data, order terms by the normalized UTF-8 key and assign numeric IDs starting at 1. Those IDs are stable for the same candidate set and scoped to the index generation. External producers can provide their own stable IDs in direct candidate input. Record the document snapshot, input JSONL and candidate output hashes, dictionary/policy identity, candidate and alias counts, and exclusions by reason.
4. Run the builder on the candidate JSONL. The builder accepts hand-curated candidates or other languages through the same format. It produces the compact read-only index. Use bounded input lines, tokenization length, unique-document and unique-term counts, and alias limits. Hard budget errors fail the job and leave the published generation available. Publish each validated generation as one unit.

## Frequency and ranking metadata

Write a build-time JSONL sidecar keyed by candidate ID. Store `df` (documents containing the term), `cf` (total eligible token occurrences) and the experimental `bm25_sum` weight. Count the canonical term only. If it appears twice in document A and once in document B, `df = 2` and `cf = 3`. Store the ranking method, parameters and document statistics in provenance. Default to `weight = df`. In the browser, the selected integer weight only breaks ties after match quality and variant/key priority.

BM25 normally scores documents for a given query. For this experiment, treat each candidate term `t` as a single-term query, score each containing document `d`, and sum across the documents at build time. Use `tf` for occurrences of `t` in `d`, `N` for the number of input documents, `dl` for eligible token occurrences in `d` before deduplication, and `avgdl` for the mean of `dl` across all input documents. Start with `k1 = 1.2`, `b = 0.75` and the following positive IDF variant.

```text
idf(t) = ln(1 + (N - df(t) + 0.5) / (df(t) + 0.5))
bm25(t, d) = idf(t) * tf * (k1 + 1) / (tf + k1 * (1 - b + b * dl / avgdl))
bm25_sum(t) = sum of bm25(t, d) for documents that contain t
```

Use a bounded two-pass build with term counts per document and deterministic traversal order. The empty corpus produces no term scores. A candidate necessarily has a positive `avgdl` in its corpus. For the experiment, round `bm25_sum × 1,000` to the nearest integer (half up) and reject overflow or non-finite values. Run reproducibility checks with the recorded toolchain. The policy may use that integer as the ranking weight. Keep the sidecar as a build artifact and ship only the selected weight in the index. Compare with `df` because IDF rewards rarity and a term that stands out in its documents may still be a poor completion. [Robertson and Zaragoza, BM25 and Beyond](https://www.staff.city.ac.uk/~sbrp622/papers/foundations_bm25_review.pdf) · [Lucene BM25Similarity and its positive IDF](https://lucene.apache.org/core/10_3_1/core/org/apache/lucene/search/similarities/BM25Similarity.html).

## CLI and demo preparation

Generate candidates and the index with these commands:

```sh
nemesia extract --input documents.jsonl --dictionary system.dic --policy corpus-policy.json --output candidates.jsonl --stats term-stats.jsonl --query-rules query-rules.json --provenance extraction.json
nemesia build --input candidates.jsonl --output suggestions.nmsi
```

Extract the Core edition of [SudachiDict](https://github.com/WorksApplications/SudachiDict) and pass its `system_core.dic` file to `--dictionary`. Omit `--policy` to use the defaults. A minimal policy to try the alternate weight is `{"ranking":"bm25-sum"}`. The statistics, query rules and provenance outputs are optional. Choose fresh output paths for extraction and index building.

The core repository owns these commands, including parsing, morphological extraction, rules and provenance. An application can export its searchable fields to the same document JSONL format. The standalone [demo](https://github.com/tkhshtsh0917/nemesia-demo) prepares a frequency-ranked Japanese noun vocabulary from Common Crawl WET and builds its published index from a pinned vocabulary snapshot.

CI uses small deterministic fixtures. A larger benchmark can use a locked snapshot of representative documents and labelled queries. Pin the dataset revision, extraction output and attribution and preserve the applicable data license before distribution. Measure the number of source documents, distinct completion terms and alias keys separately. A dataset of 10,000 documents does not imply 10,000 suggestions.
