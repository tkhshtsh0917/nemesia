//! Read-only WebAssembly API over the native index and matcher.
#![forbid(unsafe_code)]

use nemesia_engine::suggest;
use nemesia_index::binary::Index;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn abi_version() -> u32 {
    1
}

#[wasm_bindgen]
pub struct CompletionIndex(Index);

#[wasm_bindgen]
impl CompletionIndex {
    /// Validates and takes ownership of the index bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<CompletionIndex, JsError> {
        Index::open(bytes).map(Self).map_err(JsError::new)
    }

    /// Returns a string owned by the JS caller and independent of future requests.
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
