#![no_main]

use libfuzzer_sys::fuzz_target;
use nemesia_engine::suggest_bounded;
use nemesia_index::binary::Index;

fuzz_target!(|input: &[u8]| {
    if let Ok(index) = Index::open(input.to_vec()) {
        let _ = suggest_bounded(&index, "bi-t", Some("びーt"), 8, 10_000);
    }
});
