use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

use crate::domain::{KeyValue, TranslationResult};

pub async fn translate(
    word: &str,
    source_language: &str,
    target_language: &str,
) -> Result<TranslationResult> {
    let encoded = urlencoding::encode(word.trim());
    let dicts = dicts_for_languages(source_language, target_language);
    let url = format!("https://dict.youdao.com/jsonapi?q={encoded}&dicts={dicts}");
    let response = Client::new().post(url).send().await?;
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

    let preferred_dictionary =
        if is_chinese_language(source_language) && is_english_language(target_language) {
            "ce"
        } else {
            "ec"
        };
    let fallback_dictionary = if preferred_dictionary == "ec" {
        "ce"
    } else {
        "ec"
    };

    if let Some(word) = first_dictionary_word(&json, preferred_dictionary)
        .or_else(|| first_dictionary_word(&json, fallback_dictionary))
    {
        us_phone = word
            .get("usphone")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        uk_phone = word
            .get("ukphone")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(trs) = word.get("trs").and_then(Value::as_array) {
            let meanings = trs
                .iter()
                .flat_map(|tr_obj| {
                    tr_obj
                        .get("tr")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                })
                .filter_map(|tr| {
                    tr.get("l")
                        .and_then(|l| l.get("i"))
                        .and_then(Value::as_array)
                })
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|meaning| !meaning.is_empty())
                .collect::<Vec<_>>();
            translated_text = meanings.join(",\n ");
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

fn dicts_for_languages(source_language: &str, target_language: &str) -> &'static str {
    if is_chinese_language(source_language) && is_english_language(target_language) {
        "%7B%22count%22%3A99%2C%22dicts%22%3A%5B%5B%22ce%22%2C%22fanyi%22%2C%22blng_sents_part%22%2C%22ec%22%2C%22rel_word%22%2C%22phrs%22%5D%5D%7D"
    } else {
        "%7B%22count%22%3A99%2C%22dicts%22%3A%5B%5B%22ec%22%2C%22fanyi%22%2C%22blng_sents_part%22%2C%22ce%22%2C%22rel_word%22%2C%22phrs%22%5D%5D%7D"
    }
}

fn first_dictionary_word<'a>(json: &'a Value, dictionary: &str) -> Option<&'a Value> {
    json.get(dictionary)
        .and_then(|v| v.get("word"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
}

fn is_chinese_language(language: &str) -> bool {
    let normalized = language.trim().to_lowercase();
    normalized.contains("chinese")
        || normalized.contains("中文")
        || normalized.contains("简体")
        || normalized.contains("繁體")
}

fn is_english_language(language: &str) -> bool {
    language.trim().eq_ignore_ascii_case("english")
}
