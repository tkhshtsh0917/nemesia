use nemesia_builder::build;
use nemesia_engine::{reference, suggest, suggest_bounded};
use nemesia_index::{Candidate, binary::Index};

fn corpus() -> Vec<Candidate> {
    let mut values = vec![String::new()];
    for _ in 0..4 {
        values.extend(
            values
                .clone()
                .into_iter()
                .flat_map(|prefix| ['a', 'い', '🪐'].map(move |c| format!("{prefix}{c}"))),
        );
        values.sort();
        values.dedup();
    }
    values
        .into_iter()
        .filter(|v| !v.is_empty())
        .enumerate()
        .map(|(id, text)| Candidate {
            aliases: vec![format!("abc{text}"), text.chars().rev().collect()],
            id: id as u32,
            text,
            weight: (id % 7) as u32,
        })
        .collect()
}

#[test]
fn trie_agrees_with_exhaustive_scan() {
    let corpus = corpus();
    let bytes = build(&corpus).unwrap();
    let mut reversed = corpus.clone();
    reversed.reverse();
    assert_eq!(build(&reversed).unwrap(), bytes);
    let index = Index::open(bytes).unwrap();

    for value in &corpus {
        assert_eq!(
            suggest(&index, &value.text, Some("abc"), 7),
            reference(&corpus, &value.text, Some("abc"), 7)
        );
    }
    for raw in [
        "",
        "  ",
        "abc🪐あい",
        "abcあい🪐",
        "abcd🪐い",
        "abc🪐あ🪐",
        "ａｂｃ",
        "abc ",
        "🪐🪐🪐aいあa",
    ] {
        assert_eq!(
            suggest(&index, raw, None, 20),
            reference(&corpus, raw, None, 20)
        );
    }
    assert_eq!(
        suggest_bounded(&index, "abc", Some("い"), 7, 121),
        Err("query work limit exceeded")
    );
    assert_eq!(
        suggest(&index, "a", None, 7),
        reference(&corpus, "a", None, 7)
    );
}

#[test]
fn corrupt_layouts_fail_before_queries() {
    let mut corpus = corpus();
    corpus.truncate(4);
    let bytes = build(&corpus).unwrap();
    for end in 0..bytes.len() {
        assert!(Index::open(bytes[..end].to_vec()).is_err());
    }
    for i in 0..bytes.len() {
        let mut altered = bytes.clone();
        altered[i] = 0xff;
        if let Ok(index) = Index::open(altered) {
            let _ = suggest(&index, "abcあ🪐", Some("abc"), 8);
        }
    }
    let mut duplicate = corpus[0].clone();
    duplicate.text = "別".into();
    corpus.push(duplicate);
    assert!(build(&corpus).is_err());
}
