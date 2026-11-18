use nemesia_analyzer::{Analyzer, Document, Policy, Ranking, extract, query_rules};
use std::path::Path;

fn analyzer(policy: &Policy) -> Analyzer {
    Analyzer::open(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/system.dic"),
        policy,
    )
    .unwrap()
}
fn docs(entries: &[(&str, &str)]) -> Vec<Document> {
    entries
        .iter()
        .map(|(id, text)| Document {
            id: (*id).into(),
            text: (*text).into(),
        })
        .collect()
}

#[test]
fn sudachi_counts_a_term_once_per_document_and_preserves_readings() {
    let mut analyzer = analyzer(&Policy::default());
    let entries = [("B", "京都に"), ("A", "京都、京都")];
    let df = extract(&mut analyzer, docs(&entries), Ranking::Df).unwrap();
    let (row, stats) = df
        .rows
        .iter()
        .zip(&df.stats)
        .find(|(row, _)| row.text == "京都")
        .unwrap();
    assert_eq!((row.weight, stats.df, stats.cf), (2, 2, 3));
    assert!(!df.rows.iter().any(|row| row.text == "に"));
    assert!(row.aliases.iter().any(|value| value == "kyouto"));
    assert!(row.aliases.iter().any(|value| value == "きょうと"));

    let bm25 = extract(
        &mut analyzer,
        docs(&[entries[1], entries[0]]),
        Ranking::Bm25Sum,
    )
    .unwrap();
    let weighted = bm25.rows.iter().find(|value| value.id == row.id).unwrap();
    assert_eq!(weighted.weight, stats.bm25_sum);
    assert_eq!(weighted.weight, 440);
    assert!(extract(&mut analyzer, docs(&[entries[0], entries[0]]), Ranking::Df).is_err());
}

#[test]
fn explicit_filters_and_mixed_ime_rules_work() {
    let policy = Policy {
        stopwords: vec!["京都".into()],
        ..Policy::default()
    };
    let result = extract(
        &mut analyzer(&policy),
        docs(&[("1", "京都に")]),
        Ranking::Df,
    )
    .unwrap();
    assert!(result.rows.is_empty());
    assert_eq!(query_rules()["び"], "bi");
    assert_eq!(query_rules()["ビ"], "bi");
    assert_eq!(query_rules()["ー"], "-");
    assert_eq!(query_rules()["きょ"], "kyo");
}
