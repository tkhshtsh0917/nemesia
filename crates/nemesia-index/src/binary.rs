//! Validated access to a little-endian completion index.
use crate::{MAX_CANDIDATES, MAX_INDEX_BYTES, MAX_KEY, normalize};
use std::ops::Range;

/// Size of the NMSI v1 header in bytes.
pub const HEADER: usize = 32;
/// NMSI layout revision accepted by this reader.
pub const VERSION: u32 = 1;
/// Key normalization revision expected by this reader.
pub const NORMALIZATION: u32 = 1;

/// A structurally validated, immutable NMSI snapshot that owns its bytes.
#[derive(Debug)]
pub struct Index {
    bytes: Vec<u8>,
    rows: usize,
    nodes: usize,
    edges: usize,
    posts: usize,
    count: usize,
    node_count: usize,
}

fn word(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

impl Index {
    /// Validates section arithmetic, tree shape, and candidate references before serving queries.
    pub fn open(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if !(HEADER..=MAX_INDEX_BYTES).contains(&bytes.len())
            || &bytes[..4] != b"NMSI"
            || word(&bytes, 4) != VERSION as usize
            || word(&bytes, 8) != NORMALIZATION as usize
            || word(&bytes, 12) != bytes.len()
        {
            return Err("invalid index header");
        }
        let counts: [usize; 4] = std::array::from_fn(|i| word(&bytes, 16 + 4 * i));
        let [count, node_count, edge_count, post_count] = counts;
        if count > MAX_CANDIDATES
            || node_count == 0
            || edge_count != node_count - 1
            || post_count < count
            || post_count > count * (1 + crate::MAX_ALIASES)
        {
            return Err("invalid index counts");
        }
        let mut offsets = [0; 5];
        offsets[0] = HEADER;
        for (i, width) in [16, 16, 8, 8].into_iter().enumerate() {
            offsets[i + 1] = counts[i]
                .checked_mul(width)
                .and_then(|n| offsets[i].checked_add(n))
                .filter(|end| *end <= bytes.len())
                .ok_or("invalid index sections")?;
        }
        let [rows, nodes, edges, posts, _] = offsets;
        let index = Self {
            bytes,
            rows,
            nodes,
            edges,
            posts,
            count,
            node_count,
        };
        index.validate(edge_count, post_count, offsets[4])?;
        Ok(index)
    }

    fn validate(
        &self,
        edge_count: usize,
        post_count: usize,
        mut text_at: usize,
    ) -> Result<(), &'static str> {
        let mut previous_id = None;
        for i in 0..self.count {
            let at = self.rows + 16 * i;
            let id = word(&self.bytes, at);
            if previous_id.is_some_and(|prev| prev >= id) || word(&self.bytes, at + 8) != text_at {
                return Err("invalid candidate order or string offset");
            }
            previous_id = Some(id);
            let len = word(&self.bytes, at + 12);
            let text_bytes = self
                .bytes
                .get(text_at..)
                .and_then(|tail| tail.get(..len))
                .filter(|value| !value.is_empty() && value.len() <= 4 * MAX_KEY)
                .ok_or("invalid candidate string length")?;
            let text = std::str::from_utf8(text_bytes).map_err(|_| "invalid candidate UTF-8")?;
            if normalize(text, MAX_KEY, false)?.is_empty() {
                return Err("empty candidate key");
            }
            text_at += len;
        }
        if text_at != self.bytes.len() {
            return Err("invalid string section length");
        }

        let mut depths = vec![0u16; self.node_count];
        let mut canonical = vec![(false, 0u8); self.count];
        let (mut next_edge, mut next_post) = (0, 0);
        for node in 0..self.node_count {
            let at = self.nodes + 16 * node;
            let (e, el, p, pl) = (
                word(&self.bytes, at),
                word(&self.bytes, at + 4),
                word(&self.bytes, at + 8),
                word(&self.bytes, at + 12),
            );
            if e != next_edge
                || p != next_post
                || el > edge_count - next_edge
                || pl > post_count - next_post
                || (node == 0 && pl != 0)
                || (node > 0 && (depths[node] == 0 || (el == 0 && pl == 0)))
            {
                return Err("invalid trie ranges");
            }
            let mut previous = None;
            for (symbol, child) in self.children(node) {
                if child <= node
                    || child >= self.node_count
                    || depths[child] != 0
                    || previous.is_some_and(|prev| prev >= symbol)
                    || char::from_u32(symbol).is_none()
                {
                    return Err("invalid trie edge");
                }
                let depth = depths[node] + 1;
                if usize::from(depth) > MAX_KEY {
                    return Err("trie depth exceeds limit");
                }
                depths[child] = depth;
                previous = Some(symbol);
            }
            let mut previous = None;
            for (row, tier) in self.postings(node) {
                let Some((seen, aliases)) = canonical.get_mut(row) else {
                    return Err("invalid trie posting");
                };
                if previous.is_some_and(|prev| prev >= row)
                    || tier > 1
                    || (tier == 0 && *seen)
                    || (tier == 1 && usize::from(*aliases) == crate::MAX_ALIASES)
                {
                    return Err("invalid trie posting");
                }
                if tier == 0 {
                    *seen = true;
                } else {
                    *aliases += 1;
                }
                previous = Some(row);
            }
            next_edge += el;
            next_post += pl;
        }
        if next_edge != edge_count
            || next_post != post_count
            || canonical.iter().any(|(seen, _)| !seen)
        {
            return Err("incomplete trie");
        }
        Ok(())
    }

    /// Returns the number of display candidates, excluding aliases.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Returns whether the index contains no display candidates.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the ID, ranking weight, and display string for a row ordinal.
    ///
    /// Panics if the ordinal is outside `0..self.len()`.
    pub fn candidate(&self, row: usize) -> (u32, u32, &str) {
        assert!(row < self.count);
        let at = self.rows + 16 * row;
        let offset = word(&self.bytes, at + 8);
        let text = &self.bytes[offset..offset + word(&self.bytes, at + 12)];
        (
            word(&self.bytes, at) as u32,
            word(&self.bytes, at + 4) as u32,
            std::str::from_utf8(text).unwrap(),
        )
    }

    fn range(&self, node: usize, field: usize) -> Range<usize> {
        assert!(node < self.node_count);
        let at = self.nodes + 16 * node + field;
        let start = word(&self.bytes, at);
        start..start + word(&self.bytes, at + 4)
    }

    fn pair(&self, section: usize, i: usize) -> (usize, usize) {
        let offset = section + 8 * i;
        (word(&self.bytes, offset), word(&self.bytes, offset + 4))
    }

    /// Looks up a transition for one scalar from a valid trie node ordinal.
    pub fn child(&self, node: usize, c: char) -> Option<usize> {
        let range = self.range(node, 0);
        let data = &self.bytes[self.edges + 8 * range.start..self.edges + 8 * range.end];
        let needle = c as usize;
        let index = data
            .as_chunks::<8>()
            .0
            .partition_point(|entry| word(entry, 0) < needle);
        let entry = data.as_chunks::<8>().0.get(index)?;
        (word(entry, 0) == needle).then(|| word(entry, 4))
    }

    /// Iterates scalar and child ordinal pairs for a valid node.
    pub fn children(&self, node: usize) -> impl Iterator<Item = (u32, usize)> + '_ {
        self.range(node, 0).map(|i| {
            let (symbol, child) = self.pair(self.edges, i);
            (symbol as u32, child)
        })
    }

    /// Iterates candidate ordinal and key tier pairs for a valid node.
    pub fn postings(&self, node: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.range(node, 8).map(|i| self.pair(self.posts, i))
    }
}
