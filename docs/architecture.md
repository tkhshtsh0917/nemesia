# Architecture

[日本語](ja/architecture.md) · [Matching specification](matching.md) · [Roadmap](roadmap.md)

## Repository and execution boundaries

| Owner                                                        | Responsibility                                                                               |
| ------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| `nemesia-index`                                              | Versioned binary layout, checked reader, shared data types and shared text normalization     |
| `nemesia-engine`                                             | Query preparation, prefix matching, ranking and query budgets                                |
| `nemesia-builder`                                            | Deterministic compilation of candidate JSONL into the index                                  |
| `nemesia-analyzer`                                           | Native-only Japanese candidate and reading extraction with an explicit dictionary and policy |
| `nemesia-cli`                                                | Native `extract`, `build`, `inspect` and `suggest` commands                                  |
| `nemesia-wasm`                                               | Thin read/query binding for the same engine over the same index bytes                        |
| [nemesia-demo](https://github.com/tkhshtsh0917/nemesia-demo) | TypeScript adapter, demo fixtures, accessible combobox, browser tests and Pages              |

A build job exports the searchable fields of the same document snapshot used by the downstream search service. It runs the extractor and builder. The web service or CDN serves hashed assets. In the browser, a Worker loads the index and requests completions from WASM. The page offers candidate text and passes the completed query to the host search on selection or submission. A standalone demo can show the completed query without calling a search backend.

The WASM dependency graph consists of the reader, the engine, shared normalization and the boundary. Tokenizers, dictionaries, corpus ingestion and the writer run only at build time. The browser sends no query or completion to a remote service until the host submits it. v0.1 uses immutable full snapshots and has no vector search, query-log collection, or delta protocol.

## Index, transport and boundary

M2 freezes a simple little-endian format, with the provisional extension `.nmsi` and magic `NMSI`. Its header identifies the format version, file length, normalization revision and section directory. Sections hold fixed-width candidate records, key-to-candidate references, the lookup structure and UTF-8 strings. The selected lookup representation is determined by the M2 comparisons. Offsets and counts use checked arithmetic; reject unknown required features, overlapping sections, invalid lengths, UTF-8, IDs, references or lookup order before use. A WASM ABI version and an index format version evolve independently.

The data manifest binds the format and normalization revisions, generation, and byte length and SHA-256 for every asset, including the optional query rules. It records the candidate input hash and, for extracted corpora, dictionary and policy identity. Keep immutable files under generation-specific URLs. Load a new generation within the byte limit, verify its contents and validate the index and rules, then adopt both together. A failed load leaves the active generation available. The atomic swap can briefly hold two generations, so the loader checks combined memory before adopting. An application whose budget cannot hold both generations uses an explicit reload. Hashes detect artifact mismatches. The manifest and update pointers must come from a trusted origin. Any CDN delivery of a document-derived index must use the same visibility scope as the source corpus.

The boundary exposes an opaque index handle, a bounded request carrying the original query, one optional derived query and `limit`, and an owned result containing ordered IDs, display text and edit distances. Add explicit disposal and typed, recoverable errors for unsupported inputs, validation and exhausted budgets. A result stays valid after later queries and index disposal. The TypeScript wrapper rejects invalid UTF-16 before passing strings to WASM. A failed query leaves the index reusable.

| Initial limit                        | Proposed v0.1 value                                                                                                                                          |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Results                              | Default 8, maximum 20                                                                                                                                        |
| Each raw or normalized query variant | Maximum 128 Unicode scalars, 512 UTF-8 bytes; at most 2 variants together                                                                                    |
| Build inputs                         | Maximum 256 normalized scalars and 1,024 UTF-8 bytes per key and display text, also 1,024 bytes before normalization; at most 8 alternate keys per candidate |
| Index and rules                      | Maximum 32 MiB decoded index, 2 MiB decoded query rules                                                                                                      |
| Runtime work                         | A single total work budget covers both variants; M2 sets its documented units and limit with adversarial fixtures                                            |
| Memory                               | 64 MiB target for peak app-owned index and search state per generation; loader measures and bounds overlapping staging separately                            |

Bounds apply before allocation or expansion where possible. Test decompressed bytes and observed memory separately from transfer size. If the engine cannot finish within its work limit, return a typed error rather than an unlabelled incomplete top-k. Package the native CLI, raw WASM, ABI metadata, matching wasm-bindgen toolchain identity, checksums and legal notices as tag Release Assets. The demo builds and checks the bindings using the pinned release. Keep browser code and dependency lockfiles in the demo. Require native/CLI/WASM output parity for a small shared corpus before acceptance.
