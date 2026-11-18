# NMSI v1 byte layout

[日本語](ja/format.md) · [Architecture](architecture.md)

All integers are unsigned 32-bit little-endian. Offsets below are zero-based. The first 32 bytes are the header.

| Offset | Header field              |
| ------ | ------------------------- |
| 0–3    | ASCII `NMSI`              |
| 4–7    | Format version, 1         |
| 8–11   | Normalization revision, 1 |
| 12–15  | Total byte length         |
| 16–19  | Candidate count           |
| 20–23  | Trie node count           |
| 24–27  | Edge count                |
| 28–31  | Posting count             |

The sections follow immediately, with no alignment padding:

| Section    | Bytes per record | Fields in order                                                         |
| ---------- | ---------------: | ----------------------------------------------------------------------- |
| Candidates |               16 | Candidate ID, weight, absolute display byte offset, display byte length |
| Nodes      |               16 | First edge ordinal, edge count, first posting ordinal, posting count    |
| Edges      |                8 | Unicode scalar value, child node ordinal                                |
| Postings   |                8 | Candidate ordinal, key tier (0 canonical, 1 alias)                      |
| Strings    |         Variable | Consecutive UTF-8 display strings in candidate order                    |

Candidates are sorted by ID. The root is node 0. Each other node has exactly one incoming edge from a lower ordinal. The edges of a node are ascending by scalar. Postings within a node are ascending by candidate ordinal and encode at most one tier per candidate. The reader checks counts, section arithmetic, contiguous ranges, UTF-8, depth, edge ordering and posting references before exposing the index.

The query ABI has its own version because the interface can change independently of this byte layout. Acquire the reader and builder from a version that supports the same format and normalization revision.
