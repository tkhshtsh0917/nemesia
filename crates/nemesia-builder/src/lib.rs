//! Compiles normalized candidate keys to a contiguous trie.
#![forbid(unsafe_code)]

use nemesia_index::binary::{HEADER, NORMALIZATION, VERSION};
use nemesia_index::{Candidate, MAX_ALIASES, MAX_CANDIDATES, MAX_INDEX_BYTES, MAX_KEY, normalize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Node {
    edges: BTreeMap<char, usize>,
    posts: Vec<(usize, usize)>,
}

fn encode(bytes: &mut Vec<u8>, words: impl IntoIterator<Item = usize>) {
    for word in words {
        bytes.extend((word as u32).to_le_bytes());
    }
}

pub fn build(input: &[Candidate]) -> Result<Vec<u8>, &'static str> {
    if input.len() > MAX_CANDIDATES {
        return Err("candidate limit exceeded");
    }
    let mut rows: Vec<_> = input.iter().collect();
    rows.sort_unstable_by_key(|row| row.id);
    let mut nodes = vec![Node::default()];
    let mut posts = 0;
    let mut strings = 0;
    let mut display = BTreeSet::new();

    for (i, row) in rows.iter().enumerate() {
        if row.aliases.len() > MAX_ALIASES
            || !display.insert(row.text.as_str())
            || (i > 0 && rows[i - 1].id == row.id)
        {
            return Err("duplicate candidate or alias count exceeded");
        }

        strings += row.text.len();
        let mut keys = BTreeSet::new();
        for (key_index, raw) in std::iter::once(&row.text).chain(&row.aliases).enumerate() {
            let key = normalize(raw, MAX_KEY, false)?;
            if key.is_empty() {
                return Err("empty match key");
            }
            if !keys.insert(key.clone()) {
                continue;
            }

            let mut node = 0;
            for symbol in key.chars() {
                node = if let Some(&child) = nodes[node].edges.get(&symbol) {
                    child
                } else {
                    let child = nodes.len();
                    if encoded_size(rows.len(), child + 1, posts, strings) > MAX_INDEX_BYTES {
                        return Err("index byte limit exceeded");
                    }
                    nodes.push(Node::default());
                    nodes[node].edges.insert(symbol, child);
                    child
                };
            }
            nodes[node].posts.push((i, usize::from(key_index != 0)));
            posts += 1;
        }
        if encoded_size(rows.len(), nodes.len(), posts, strings) > MAX_INDEX_BYTES {
            return Err("index byte limit exceeded");
        }
    }

    let size = encoded_size(rows.len(), nodes.len(), posts, strings);
    let mut out = Vec::with_capacity(size);
    out.extend(b"NMSI");
    encode(
        &mut out,
        [
            VERSION as usize,
            NORMALIZATION as usize,
            size,
            rows.len(),
            nodes.len(),
            nodes.len() - 1,
            posts,
        ],
    );
    let mut text = size - strings;
    for row in &rows {
        encode(
            &mut out,
            [row.id as usize, row.weight as usize, text, row.text.len()],
        );
        text += row.text.len();
    }
    let (mut edge, mut post) = (0, 0);
    for node in &nodes {
        encode(&mut out, [edge, node.edges.len(), post, node.posts.len()]);
        edge += node.edges.len();
        post += node.posts.len();
    }
    for node in &nodes {
        for (&c, &child) in &node.edges {
            encode(&mut out, [c as usize, child]);
        }
    }
    for node in &nodes {
        for &(row, tier) in &node.posts {
            encode(&mut out, [row, tier]);
        }
    }
    for row in rows {
        out.extend(row.text.as_bytes());
    }
    Ok(out)
}

fn encoded_size(rows: usize, nodes: usize, posts: usize, strings: usize) -> usize {
    HEADER + 16 * rows + 16 * nodes + 8 * (nodes - 1 + posts) + strings
}
