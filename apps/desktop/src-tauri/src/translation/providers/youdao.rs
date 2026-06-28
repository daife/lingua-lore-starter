use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::domain::{KeyValue, TranslationResult};

pub async fn translate(
    word: &str,
    source_language: &str,
    target_language: &str,
) -> Result<TranslationResult> {
    let encoded = urlencoding::encode(word.trim());
    let dicts = dicts_for_languages(source_language, target_language);
    let le = le_for_languages(source_language, target_language);
    let url = format!("https://dict.youdao.com/jsonapi?q={encoded}&le={le}&dicts={dicts}");
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let response = client.post(url).send().await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(anyhow!("Youdao request failed with {status}"));
    }
    parse_youdao_response(word, &text, source_language, target_language)
}

fn parse_youdao_response(
    source_text: &str,
    json_input: &str,
    source_language: &str,
    target_language: &str,
) -> Result<TranslationResult> {
    let json: Value = serde_json::from_str(json_input)?;
    let mut translated_text = String::new();
    let mut us_phone = String::new();
    let mut uk_phone = String::new();
    let mut related_words = Vec::new();
    let mut phrases = Vec::new();
    let mut examples = Vec::new();

    let dictionaries = dictionary_codes(source_language, target_language);
    if dictionaries.iter().any(|dictionary| *dictionary == "newhh") {
        if let Some(result) = parse_newhh_dictionary(&json) {
            translated_text = result.translated_text;
            us_phone = result.phone;
        }
    }

    if translated_text.trim().is_empty() {
        if let Some(word) = dictionaries
            .iter()
            .find_map(|dictionary| first_dictionary_word(&json, dictionary))
        {
            us_phone = first_string(word, &["usphone", "phone", "pinyin"]);
            uk_phone = first_string(word, &["ukphone"]);
            let meanings = collect_word_meanings(word);
            translated_text = meanings.join(",\n ");
        }
    }

    if us_phone.is_empty() && uk_phone.is_empty() {
        if let Some((fallback_us_phone, fallback_uk_phone)) = first_phonetics(&json, &dictionaries)
        {
            us_phone = fallback_us_phone;
            uk_phone = fallback_uk_phone;
        }
    }

    if let Some(rels) = json
        .get("rel_word")
        .and_then(|v| v.get("rels"))
        .and_then(Value::as_array)
    {
        for word_obj in rels
            .iter()
            .flat_map(|rel| {
                rel.get("rel")
                    .and_then(|r| r.get("words"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .take(20)
        {
            if let Some(key) = word_obj.get("word").and_then(Value::as_str) {
                related_words.push(KeyValue {
                    key: key.to_string(),
                    value: word_obj
                        .get("tran")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                });
            }
        }
    }

    if let Some(items) = json
        .get("phrs")
        .and_then(|v| v.get("phrs"))
        .and_then(Value::as_array)
    {
        for item in items.iter().take(5) {
            let phrase = item
                .get("phr")
                .and_then(|p| p.get("headword"))
                .and_then(|h| h.get("l"))
                .and_then(|l| l.get("i"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let meaning = item
                .get("phr")
                .and_then(|p| p.get("trs"))
                .and_then(Value::as_array)
                .and_then(|trs| trs.first())
                .and_then(|tr| tr.get("tr"))
                .and_then(|tr| tr.get("l"))
                .and_then(|l| l.get("i"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if !phrase.is_empty() {
                phrases.push(KeyValue {
                    key: phrase.to_string(),
                    value: meaning.to_string(),
                });
            }
        }
    }

    if let Some(sentence_pairs) = json
        .get("blng_sents_part")
        .and_then(|v| v.get("sentence-pair"))
        .and_then(Value::as_array)
    {
        for pair in sentence_pairs.iter().take(2) {
            let eng = pair
                .get("sentence-eng")
                .and_then(Value::as_str)
                .unwrap_or("")
                .replace("<b>", "[")
                .replace("</b>", "]");
            let trans = pair
                .get("sentence-translation")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !eng.is_empty() || !trans.is_empty() {
                examples.push(format!("{eng} - {trans}"));
            }
        }
    }

    if translated_text.trim().is_empty() {
        translated_text = json
            .get("fanyi")
            .and_then(|f| f.get("tran"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }

    Ok(TranslationResult {
        source_text: source_text.to_string(),
        translated_text,
        us_phone,
        uk_phone,
        related_words,
        phrases,
        example_sentences: examples.join(",\n "),
        provider: "youdao_public".to_string(),
    })
}

fn dicts_for_languages(source_language: &str, target_language: &str) -> String {
    let dicts = dictionary_codes(source_language, target_language);
    let dicts_json = serde_json::json!({
        "count": 10,
        "dicts": [dicts]
    });
    urlencoding::encode(&dicts_json.to_string()).into_owned()
}

fn le_for_languages(source_language: &str, target_language: &str) -> &'static str {
    match (
        language_kind(source_language),
        language_kind(target_language),
    ) {
        (LanguageKind::Japanese, _) | (_, LanguageKind::Japanese) => "ja",
        (LanguageKind::Korean, _) | (_, LanguageKind::Korean) => "ko",
        (LanguageKind::English, _) | (_, LanguageKind::English) => "en",
        _ => "auto",
    }
}

fn dictionary_codes(source_language: &str, target_language: &str) -> Vec<&'static str> {
    match (
        language_kind(source_language),
        language_kind(target_language),
    ) {
        (LanguageKind::Chinese, LanguageKind::English) => vec!["ce", "ec"],
        (LanguageKind::English, LanguageKind::Chinese) => vec!["ec", "ce"],
        (LanguageKind::Japanese, LanguageKind::Chinese) => vec!["jc"],
        (LanguageKind::Chinese, LanguageKind::Japanese) => vec!["cj"],
        (LanguageKind::Korean, LanguageKind::Chinese) => vec!["kc"],
        (LanguageKind::Chinese, LanguageKind::Korean) => vec!["ck"],
        (LanguageKind::English, LanguageKind::English) => vec!["ee", "ec"],
        (LanguageKind::Chinese, LanguageKind::Chinese) => vec!["newhh", "yw", "ce"],
        _ => vec!["ec", "ce"],
    }
}

fn first_dictionary_word<'a>(json: &'a Value, dictionary: &str) -> Option<&'a Value> {
    let word = json.get(dictionary).and_then(|v| v.get("word"))?;
    if let Some(words) = word.as_array() {
        return words.first();
    }
    word.as_object().map(|_| word)
}

fn first_phonetics(json: &Value, dictionaries: &[&str]) -> Option<(String, String)> {
    dictionaries
        .iter()
        .filter_map(|dictionary| first_dictionary_word(json, dictionary))
        .filter_map(|word| {
            let us_phone = first_string(word, &["usphone", "phone", "pinyin"]);
            let uk_phone = first_string(word, &["ukphone"]);
            if us_phone.is_empty() && uk_phone.is_empty() {
                None
            } else {
                Some((us_phone, uk_phone))
            }
        })
        .next()
}

struct NewhhParseResult {
    translated_text: String,
    phone: String,
}

fn parse_newhh_dictionary(json: &Value) -> Option<NewhhParseResult> {
    let entries = json
        .get("newhh")
        .and_then(|v| v.get("dataList"))
        .and_then(Value::as_array)?;
    let mut phone = String::new();
    let mut meanings = Vec::new();
    for entry in entries.iter().take(3) {
        let pinyin = entry.get("pinyin").and_then(Value::as_str).unwrap_or("");
        if phone.is_empty() {
            phone = pinyin.to_string();
        }
        let senses = entry.get("sense").and_then(Value::as_array);
        let mut sense_texts = Vec::new();
        if let Some(senses) = senses {
            for sense in senses.iter().take(5) {
                let category = sense.get("cat").and_then(Value::as_str).unwrap_or("");
                let definitions = sense
                    .get("def")
                    .and_then(Value::as_array)
                    .map(|defs| {
                        defs.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join("；")
                    })
                    .unwrap_or_default();
                if definitions.is_empty() {
                    continue;
                }
                if category.is_empty() {
                    sense_texts.push(definitions);
                } else {
                    sense_texts.push(format!("{category}: {definitions}"));
                }
            }
        }
        if !sense_texts.is_empty() {
            if pinyin.is_empty() {
                meanings.push(sense_texts.join("；"));
            } else {
                meanings.push(format!("{pinyin} {}", sense_texts.join("；")));
            }
        }
    }
    if meanings.is_empty() {
        return None;
    }
    Some(NewhhParseResult {
        translated_text: meanings.join(",\n "),
        phone,
    })
}

fn collect_word_meanings(word: &Value) -> Vec<String> {
    let mut meanings = Vec::new();
    if let Some(trs) = word.get("trs").and_then(Value::as_array) {
        for tr_obj in trs {
            let pos = tr_obj.get("pos").and_then(Value::as_str).unwrap_or("");
            if let Some(translations) = tr_obj.get("tr").and_then(Value::as_array) {
                for translation in translations {
                    let meaning = translation
                        .get("l")
                        .and_then(|l| l.get("i"))
                        .map(flatten_translation_value)
                        .unwrap_or_default();
                    if meaning.is_empty() {
                        continue;
                    }
                    if pos.is_empty() || meaning.starts_with(pos) {
                        meanings.push(meaning);
                    } else {
                        meanings.push(format!("{pos} {meaning}"));
                    }
                }
            }
        }
    }
    meanings
}

fn first_string(value: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .unwrap_or("")
        .to_string()
}

fn flatten_translation_value(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.trim().to_string();
    }
    if let Some(items) = value.as_array() {
        return flatten_translation_items(items);
    }
    String::new()
}

fn flatten_translation_items(items: &[Value]) -> String {
    items
        .iter()
        .filter_map(translation_item_text)
        .collect::<Vec<_>>()
        .join(", ")
        .trim()
        .to_string()
}

fn translation_item_text(item: &Value) -> Option<String> {
    if let Some(text) = item.as_str() {
        return Some(text.trim().to_string());
    }
    item.get("#text")
        .and_then(Value::as_str)
        .map(|text| text.trim().to_string())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LanguageKind {
    Chinese,
    English,
    Japanese,
    Korean,
    Other,
}

fn language_kind(language: &str) -> LanguageKind {
    if is_chinese_language(language) {
        return LanguageKind::Chinese;
    }
    if language.trim().eq_ignore_ascii_case("english") {
        return LanguageKind::English;
    }
    let normalized = language.trim().to_lowercase();
    if normalized.contains("日本") || normalized.contains("japanese") || normalized == "ja" {
        return LanguageKind::Japanese;
    }
    if normalized.contains("한국") || normalized.contains("korean") || normalized == "ko" {
        return LanguageKind::Korean;
    }
    LanguageKind::Other
}

fn is_chinese_language(language: &str) -> bool {
    let normalized = language.trim().to_lowercase();
    normalized.contains("chinese")
        || normalized.contains("中文")
        || normalized.contains("简体")
        || normalized.contains("繁體")
        || normalized == "zh-chs"
}
