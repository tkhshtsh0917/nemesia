//! Bounded exact and fuzzy traversal over a validated index.
use crate::{Quality, Rank, Suggestion, budget, queries};
use nemesia_index::{MAX_QUERY, binary::Index};

pub const WORK_LIMIT: usize = 12_000_000;
const INF: u8 = 255;

struct Search<'a> {
    index: &'a Index,
    best: Vec<Option<Quality>>,
    work: usize,
    variant: u8,
}

impl Search<'_> {
    fn charge(&mut self, units: usize) -> Result<(), &'static str> {
        self.work = self
            .work
            .checked_sub(units)
            .ok_or("query work limit exceeded")?;
        Ok(())
    }

    fn record(&mut self, node: usize, distance: u8) -> Result<(), &'static str> {
        for (row, tier) in self.index.postings(node) {
            self.charge(1)?;
            let quality = Quality(distance, self.variant, tier as u8);
            self.best[row] = Some(self.best[row].map_or(quality, |old| old.min(quality)));
        }
        Ok(())
    }

    fn descendants(&mut self, node: usize, distance: u8) -> Result<(), &'static str> {
        self.charge(1)?;
        self.record(node, distance)?;
        for (_, child) in self.index.children(node) {
            self.descendants(child, distance)?;
        }
        Ok(())
    }

    fn fuzzy(
        &mut self,
        node: usize,
        q: &[char],
        k: u8,
        previous: &Row,
        best: u8,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if best <= k {
            self.record(node, best)?;
        }
        if previous.depth >= q.len() + usize::from(k) || best == 0 {
            if best <= k {
                for (_, child) in self.index.children(node) {
                    self.descendants(child, best)?;
                }
            }
            return Ok(());
        }
        for (symbol, child) in self.index.children(node) {
            let row = previous.next(q, char::from_u32(symbol).unwrap(), k);
            self.charge(1 + row.width())?;
            let best = best.min(row.get(q.len()));
            if best <= k || row.min() <= k {
                self.fuzzy(child, q, k, &row, best)?;
            }
        }
        Ok(())
    }
}

/// A diagonal band contains every edit path within the declared budget.
struct Row {
    cells: [u8; 5],
    start: usize,
    end: usize,
    depth: usize,
}

impl Row {
    fn new(depth: usize, count: usize, k: u8) -> Self {
        let width = usize::from(k);
        Self {
            cells: [INF; 5],
            start: depth.saturating_sub(width),
            end: (depth + width).min(count),
            depth,
        }
    }

    fn width(&self) -> usize {
        self.end + 1 - self.start
    }

    fn get(&self, i: usize) -> u8 {
        if (self.start..=self.end).contains(&i) {
            self.cells[i - self.start]
        } else {
            INF
        }
    }

    fn min(&self) -> u8 {
        *self.cells.iter().min().unwrap()
    }

    fn next(&self, query: &[char], c: char, k: u8) -> Self {
        let mut row = Self::new(self.depth + 1, query.len(), k);
        for i in row.start..=row.end {
            row.cells[i - row.start] = if i == 0 {
                row.depth as u8
            } else {
                self.get(i - 1)
                    .saturating_add(u8::from(query[i - 1] != c))
                    .min(self.get(i).saturating_add(1))
                    .min(row.get(i - 1).saturating_add(1))
            };
        }
        row
    }
}

/// Evaluates both variants against one snapshot, then selects the exact top k.
pub fn suggest(
    index: &Index,
    raw: &str,
    derived: Option<&str>,
    limit: usize,
) -> Result<Vec<Suggestion>, &'static str> {
    suggest_bounded(index, raw, derived, limit, WORK_LIMIT)
}

/// Evaluates a query under an explicit budget shared by both variants.
pub fn suggest_bounded(
    index: &Index,
    raw: &str,
    derived: Option<&str>,
    limit: usize,
    work: usize,
) -> Result<Vec<Suggestion>, &'static str> {
    let variants = queries(raw, derived, limit)?;
    if variants.is_empty() || index.is_empty() {
        return Ok(Vec::new());
    }
    let mut search = Search {
        index,
        best: vec![None; index.len()],
        work,
        variant: 0,
    };
    search.charge(index.len())?;

    for (variant, key) in variants.iter().enumerate() {
        search.variant = variant as u8;
        let k = budget(key);
        if k == 0 {
            let mut node = Some(0);
            for c in key.chars() {
                search.charge(1)?;
                node = node.and_then(|at| index.child(at, c));
            }
            if let Some(node) = node {
                search.descendants(node, 0)?;
            }
        } else {
            let mut buffer = ['\0'; MAX_QUERY];
            let mut len = 0;
            for (i, c) in key.chars().enumerate() {
                buffer[i] = c;
                len = i + 1;
            }
            let q = &buffer[..len];
            let mut row = Row::new(0, q.len(), k);
            for i in 0..=row.end {
                row.cells[i] = i as u8;
            }
            search.fuzzy(0, q, k, &row, INF)?;
        }
    }

    let mut ranked = std::collections::BinaryHeap::with_capacity(limit + 1);
    for (row, quality) in search.best.into_iter().enumerate() {
        let Some(quality) = quality else {
            continue;
        };
        let (id, weight, _) = index.candidate(row);
        ranked.push((
            Rank {
                quality,
                weight,
                id,
            }
            .key(),
            row,
        ));
        if ranked.len() > limit {
            ranked.pop();
        }
    }
    Ok(ranked
        .into_sorted_vec()
        .into_iter()
        .map(|((quality, _, id), row)| Suggestion {
            id,
            text: index.candidate(row).2.to_owned(),
            distance: quality.0,
        })
        .collect())
}
