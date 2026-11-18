# Matching specification

[日本語](ja/matching.md) · [Architecture](architecture.md) · [Roadmap](roadmap.md)

This is the proposed v0.1 contract. Milestone M1 freezes the reference cases before tuning.

## Candidates and normalization

The CLI accepts UTF-8 JSONL. Each row contains a stable `u32` `id`, a nonempty `text` to return when chosen, an optional `u32` `weight` (default `0`), and optional string `aliases`. One row represents one completion. Candidate IDs must be unique. A display string that appears more than once is a build error so the producer can decide which ID and weight to keep. Different display strings may share a match key. Repeated normalized keys within a row are stored once with the best key tier.

```json
{
  "id": 42,
  "text": "ビートルズ メンバー",
  "weight": 20,
  "aliases": ["beatles members", "びーとるず メンバー", "bi-toruzu menba-"]
}
```

Normalize match keys and queries with the same Rust implementation and pinned Unicode data. Apply NFKC, Unicode default lowercase, then collapse runs of Unicode whitespace to one ASCII space. Remove leading whitespace. Trim trailing space from stored keys and preserve a single trailing space in nonempty queries. Distance counts Unicode scalar values after normalization. Keep the original display string for rendering and selection.

The canonical match key is the normalized display text. Other keys are explicitly supplied aliases or full-phrase readings produced during the build. Whole-phrase readings preserve order, including leading words. Standalone synonyms, dropped leading words and substrings are excluded from automatic alias generation in v0.1. The corpus producer can explicitly supply alternate phrases as aliases.

## Acceptance and order

For a normalized nonempty query `q` and key `s`, compute the minimum Levenshtein distance from `q` to any nonempty leading prefix of `s`. The permitted edits are insertion, deletion and substitution, each with cost one. Characters after the chosen prefix are free. Swapping two different neighboring characters is evaluated as ordinary edits. An empty or whitespace-only input returns zero suggestions.

The default edit budget for each normalized query variant is `0` at 1–2 scalars, `1` at 3–5, and `2` at 6 or more. A variant ending in a normalized space uses budget `0` to preserve the word boundary. These defaults are subject to the M1 quality check.

| Input                  | Key                | Expected                                              |
| ---------------------- | ------------------ | ----------------------------------------------------- |
| `bi`                   | `bi-toruzu menba-` | Distance 0                                            |
| `bi`                   | `possibility`      | Excluded                                              |
| `toky st`              | `tokyo station`    | Distance 1                                            |
| `new ` (ends in space) | `newark`           | Excluded                                              |
| `new ` (ends in space) | `new york`         | Distance 0                                            |
| `びーt`                | `bi-toruzu menba-` | Distance 0 when the derived `bi-t` variant is enabled |

Score each eligible path by distance, then original query before derived query, then canonical key before alias. For each candidate ID, keep the strongest path over all of its keys and both query variants. Rank unique candidates by that path score, then by weight descending and ID ascending. Keep the first `k` after comparing all viable paths or proving that skipped paths cannot alter top-k. An alias cannot contribute a second suggestion or raise the candidate's weight. Candidate equality is by ID throughout runtime ranking.

CLI and native Rust accept the original query and one optional derived variant in the same request as WASM. The browser may derive one distinct mixed kana/Latin variant with a bounded table. It sends both variants whenever a conversion is applicable, including when the original matches. The engine normalizes and ranks both streams together in the same index generation.

## Correctness oracle

Use exhaustive scans and scalar Levenshtein DP over all relevant nonempty prefixes, followed by an independent full sort and deduplication in tests. Compare ordered IDs, distances and tie breaks, including cases where a late alias improves an earlier candidate. Include the examples above, Japanese, normalization collisions, astral characters, combining marks, a complete query, one-character inputs, shared keys, high-weight fuzzy candidates and failed requests followed by a valid query. Use exhaustive tiny alphabets and seeded random corpora to check the optimized index.

The prefix distance follows the established fuzzy prefix definition. The edit policy, query variants and ranking above are Nemesia decisions. [Vespa fuzzy prefix documentation](https://docs.vespa.ai/en/querying/text-matching.html#fuzzy-prefix-match) · [Bast and Celikik, Efficient Fuzzy Search in Large Text Collections](https://ad-publications.cs.uni-freiburg.de/TOIS_fuzzy_BC_2013.pdf).
