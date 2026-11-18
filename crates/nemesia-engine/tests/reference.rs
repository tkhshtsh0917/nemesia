use nemesia_engine::reference;
use nemesia_index::Candidate;

fn candidate(id: u32, text: &str, weight: u32, aliases: &[&str]) -> Candidate {
    Candidate {
        id,
        text: text.into(),
        weight,
        aliases: aliases.iter().map(|value| (*value).into()).collect(),
    }
}

#[test]
fn only_leading_prefixes_match_and_untyped_suffix_is_free() {
    let rows = [
        candidate(1, "possibility", 99, &[]),
        candidate(2, "ｂｉ-toruzu", 0, &[]),
        candidate(3, "Tokyo Station", 0, &[]),
        candidate(4, "Newark", 0, &[]),
        candidate(5, "New York", 0, &[]),
    ];
    assert_eq!(reference(&rows, "bi", None, 8).unwrap()[0].id, 2);
    assert_eq!(reference(&rows, "toky st", None, 8).unwrap()[0].distance, 1);
    assert_eq!(
        reference(&rows, "new ", None, 8)
            .unwrap()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [5]
    );
    assert!(reference(&rows, "　", None, 8).unwrap().is_empty());
}

#[test]
fn quality_precedes_weight_and_a_candidate_is_emitted_once() {
    let rows = [
        candidate(9, "びーt", 0, &["bi-t"]),
        candidate(7, "ビートルズ", 5, &["bitoluzu", "bi-toruzu", "bi-t"]),
        candidate(3, "bi-toruzu", 2, &[]),
        candidate(4, "bi-torus", 2, &[]),
    ];
    let ids: Vec<_> = reference(&rows, "びーt", Some("bi-t"), 8)
        .unwrap()
        .into_iter()
        .map(|hit| hit.id)
        .collect();
    assert_eq!(ids, [9, 3, 4, 7]);
}

#[test]
fn scalar_distance_counts_emoji_as_one_edit() {
    let rows = [candidate(1, "星🪐空", 0, &[])];
    assert_eq!(reference(&rows, "星⭐空", None, 8).unwrap()[0].distance, 1);
    assert!(reference(&rows, &"a".repeat(129), None, 8).is_err());
    assert!(reference(&rows, "星", None, 0).is_err());
    assert_eq!(reference(&rows, "星", None, 8).unwrap()[0].id, 1);
}
