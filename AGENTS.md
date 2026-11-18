# Nemesia contributor instructions

- This repository owns Rust, native extraction, indexing, and the WASM ABI. Browser code belongs to nemesia-demo.
- Run rustfmt, Clippy with warnings denied, and locked workspace tests. Add regression coverage for changed observable behavior.
- Keep code and tests grouped by responsibility. Describe behavior and relevant constraints in comments, and preserve upstream notices.
- Write English source docs and update their faithful Japanese counterparts in the same change. Keep private paths, credentials and owner operations out of the repository.
