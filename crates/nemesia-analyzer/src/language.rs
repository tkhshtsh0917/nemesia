use crate::Fallible;
use nemesia_index::{MAX_KEY, normalize};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::{path::Path, rc::Rc};
use sudachi::analysis::{Mode, mlist::MorphemeList, stateful_tokenizer::StatefulTokenizer};
use sudachi::config::{Config, PathResolver};
use sudachi::dic::dictionary::JapaneseDictionary;
use sudachi::prelude::MorphemeView;

use wana_kana::ConvertJapanese;

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
/// Server-side tokenizer settings, filters, and reviewed alternate readings.
pub struct Policy {
    pub split_mode: char,
    pub keep_unknown: bool,
    pub stopwords: Vec<String>,
    pub stoptags: Vec<Vec<String>>,
    pub ranking: crate::Ranking,
    pub aliases: BTreeMap<String, Vec<String>>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            split_mode: 'B',
            keep_unknown: false,
            stopwords: vec![
                "こと".into(),
                "もの".into(),
                "する".into(),
                "ある".into(),
                "いる".into(),
            ],
            stoptags: vec![
                vec!["名詞".into(), "数詞".into()],
                vec!["動詞".into(), "非自立可能".into()],
            ],
            ranking: crate::Ranking::Df,
            aliases: BTreeMap::new(),
        }
    }
}

#[derive(Default, Serialize)]
pub struct Exclusions {
    pub pos: u64,
    pub unknown: u64,
    pub stop: u64,
    pub empty: u64,
}

pub(crate) struct Token {
    pub key: String,
    pub display: String,
    pub aliases: BTreeSet<String>,
}

/// A reusable Sudachi tokenizer backed by one native system dictionary.
pub struct Analyzer {
    tokenizer: StatefulTokenizer<Rc<JapaneseDictionary>>,
    morphemes: MorphemeList<Rc<JapaneseDictionary>>,
    keep_unknown: bool,
    stopwords: BTreeSet<String>,
    stoptags: Vec<Vec<String>>,
    reviewed: BTreeMap<String, BTreeSet<String>>,
}

impl Analyzer {
    /// Opens an explicit system dictionary with embedded Sudachi settings.
    pub fn open(dictionary: &Path, policy: &Policy) -> Fallible<Self> {
        let mode = match policy.split_mode {
            'A' => Mode::A,
            'B' => Mode::B,
            'C' => Mode::C,
            _ => return Err("invalid split mode".into()),
        };
        if policy.stopwords.len() > 10_000
            || policy.stoptags.len() > 128
            || policy.stoptags.iter().any(|tag| {
                tag.is_empty() || tag.len() > 6 || tag.iter().any(|v| v.is_empty() || v.len() > 128)
            })
        {
            return Err("invalid stopword or stoptag policy".into());
        }

        let reviewed = reviewed_aliases(&policy.aliases)?;
        let stopwords = policy
            .stopwords
            .iter()
            .map(|s| normalize(s, MAX_KEY, false))
            .collect::<Result<_, _>>()?;
        let config = Config::minimal_at(PathResolver::from_embedded()).with_system_dic(dictionary);
        let dictionary = Rc::new(JapaneseDictionary::from_cfg(&config)?);
        Ok(Self {
            tokenizer: StatefulTokenizer::new(dictionary.clone(), mode),
            morphemes: MorphemeList::empty(dictionary),
            keep_unknown: policy.keep_unknown,
            stopwords,
            stoptags: policy.stoptags.clone(),
            reviewed,
        })
    }

    pub(crate) fn reviewed(&self, key: &str) -> impl Iterator<Item = &String> {
        self.reviewed.get(key).into_iter().flatten()
    }

    pub(crate) fn tokens(&mut self, text: &str, counts: &mut Exclusions) -> Fallible<Vec<Token>> {
        if text.len() > 49_149 || text.trim().is_empty() {
            return Err("invalid Sudachi source length".into());
        }
        self.tokenizer.reset().push_str(text);
        self.tokenizer.do_tokenize()?;
        self.morphemes.collect_results(&mut self.tokenizer)?;

        let mut out = Vec::new();
        for m in self.morphemes.iter() {
            let pos = m.part_of_speech();
            let noun = pos[0] == "名詞";
            if !noun && pos[0] != "動詞" && pos[0] != "形容詞" {
                counts.pos += 1;
                continue;
            }
            if m.is_oov() && !self.keep_unknown {
                counts.unknown += 1;
                continue;
            }
            let text = if noun {
                m.normalized_form()
            } else {
                m.dictionary_form()
            };
            let key = normalize(text, MAX_KEY, false)?;
            if key.is_empty() {
                counts.empty += 1;
                continue;
            }
            let surface = m.surface();
            if self.stopwords.contains(&key)
                || self
                    .stopwords
                    .contains(&normalize(surface.as_ref(), MAX_KEY, false)?)
                || self.stoptags.iter().any(|tag| pos.starts_with(tag))
            {
                counts.stop += 1;
                continue;
            }
            let mut aliases = BTreeSet::new();
            let lemma = if noun || m.is_oov() {
                None
            } else {
                m.dictionary_form_morpheme().ok()
            };
            let reading = lemma
                .as_ref()
                .map_or_else(|| m.reading_form(), |base| base.reading_form());
            for alias in [
                surface.to_string(),
                reading.to_owned(),
                reading.to_hiragana(),
                reading
                    .split('ー')
                    .map(ConvertJapanese::to_hiragana)
                    .collect::<Vec<_>>()
                    .join("ー"),
                reading.to_romaji(),
                long_mark_romaji(reading),
            ] {
                let alias = normalize(&alias, MAX_KEY, false)?;
                if !alias.is_empty() && alias != key && alias != "*" {
                    aliases.insert(alias);
                }
            }
            out.push(Token {
                key,
                display: text.to_owned(),
                aliases,
            });
        }
        Ok(out)
    }
}

fn reviewed_aliases(
    input: &BTreeMap<String, Vec<String>>,
) -> Fallible<BTreeMap<String, BTreeSet<String>>> {
    if input.len() > 10_000 {
        return Err("too many alias overrides".into());
    }
    let mut out = BTreeMap::new();
    for (raw, aliases) in input {
        let key = normalize(raw, MAX_KEY, false)?;
        let normalized = aliases
            .iter()
            .map(|s| normalize(s, MAX_KEY, false))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if key.is_empty()
            || aliases.len() > nemesia_index::MAX_ALIASES
            || normalized.contains("")
            || normalized.contains(&key)
            || out.insert(key, normalized).is_some()
        {
            return Err("invalid alias override".into());
        }
    }
    Ok(out)
}

fn long_mark_romaji(reading: &str) -> String {
    reading
        .split('ー')
        .map(ConvertJapanese::to_romaji)
        .collect::<Vec<_>>()
        .join("-")
}

/// Prepares a bounded kana-to-Latin replacement table for mixed IME input.
pub fn query_rules() -> BTreeMap<String, String> {
    let mut rules = BTreeMap::from([("ー".to_owned(), "-".to_owned())]);
    for c in '\u{3041}'..='\u{3096}' {
        for suffix in ["", "ゃ", "ゅ", "ょ", "ぁ", "ぃ", "ぅ", "ぇ", "ぉ"] {
            let kana = format!("{c}{suffix}");
            let romaji = kana.to_romaji();
            if !romaji.is_empty()
                && romaji.bytes().all(|b| b.is_ascii_lowercase() || b == b'\'')
                && (suffix.is_empty()
                    || romaji != format!("{}{}", c.to_string().to_romaji(), suffix.to_romaji()))
            {
                rules.insert(kana.to_katakana(), romaji.clone());
                rules.insert(kana, romaji);
            }
        }
    }
    rules
}
