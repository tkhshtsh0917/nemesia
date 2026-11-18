//! Read-only WebAssembly API over the native index and matcher.
#![forbid(unsafe_code)]

use nemesia_engine::suggest;
use nemesia_index::binary::Index;
use wasm_bindgen::prelude::*;

/// The JS binding contract version. This is separate from the NMSI version.
#[wasm_bindgen]
pub fn abi_version() -> u32 {
    1
}

/// A reusable index handle for the generated JavaScript bindings.
/// Dispose of it with the JavaScript `free()` method when done.
#[wasm_bindgen]
pub struct CompletionIndex(Index);

#[wasm_bindgen]
impl CompletionIndex {
    /// Validates and takes ownership of the index bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<CompletionIndex, JsError> {
        Index::open(bytes).map(Self).map_err(JsError::new)
    }

    /// Returns a JSON array of ranked completions encoded as an owned string.
    ///
    /// Pass one optional derived spelling. Invalid input, limits outside
    /// `1..=20`, and work exhaustion are surfaced as JavaScript errors.
    /// Results stay valid after later requests and disposal of the handle.
    pub fn complete(
        &self,
        query: &str,
        derived: Option<String>,
        limit: u32,
    ) -> Result<String, JsError> {
        let hits =
            suggest(&self.0, query, derived.as_deref(), limit as usize).map_err(JsError::new)?;
        serde_json::to_string(&hits).map_err(|error| JsError::new(&error.to_string()))
    }
}
