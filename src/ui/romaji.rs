//! Automatic Hepburn Romaji transliteration for Japanese lyrics using Lindera morphological analysis.
//!
//! Provides cached, zero-panic conversion of Kanji, Hiragana, and Katakana
//! to clean lowercase Hepburn romaji sub-lines with context-aware particle
//! and phonological handling.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;

static ROMAJI_CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);
static SEGMENTER: OnceLock<Option<Segmenter>> = OnceLock::new();

fn get_segmenter() -> Option<&'static Segmenter> {
    SEGMENTER
        .get_or_init(|| {
            let dict = load_dictionary("embedded://ipadic").ok()?;
            Some(Segmenter::new(Mode::Normal, dict, None))
        })
        .as_ref()
}

/// Returns true if the string contains Hiragana, Katakana, or CJK Kanji ideographs.
pub fn contains_japanese_script(s: &str) -> bool {
    s.chars().any(|c| matches!(c,
        '\u{3040}'..='\u{309F}'  // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A
    ))
}

/// Returns true ONLY if the lyric lines contain at least one Hiragana or Katakana character.
/// Rejects pure English, Korean Hangul, Chinese Hanzi-only, Spanish, etc.
pub fn track_has_japanese_lyrics(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        line.chars().any(|c| matches!(c,
            '\u{3040}'..='\u{309F}'  // Hiragana
            | '\u{30A0}'..='\u{30FF}' // Katakana
        ))
    })
}

/// Converts Hiragana to Katakana via Unicode scalar offset (+0x60).
fn to_katakana(s: &str) -> String {
    s.chars()
        .map(|c| {
            if ('\u{3041}'..='\u{3096}').contains(&c) {
                char::from_u32(c as u32 + 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

fn lookup_digraph(c1: char, c2: char) -> Option<&'static str> {
    match (c1, c2) {
        ('キ', 'ャ') => Some("kya"),
        ('キ', 'ュ') => Some("kyu"),
        ('キ', 'ョ') => Some("kyo"),
        ('シ', 'ャ') => Some("sha"),
        ('シ', 'ュ') => Some("shu"),
        ('シ', 'ェ') => Some("she"),
        ('シ', 'ョ') => Some("sho"),
        ('チ', 'ャ') => Some("cha"),
        ('チ', 'ュ') => Some("chu"),
        ('チ', 'ェ') => Some("che"),
        ('チ', 'ョ') => Some("cho"),
        ('ニ', 'ャ') => Some("nya"),
        ('ニ', 'ュ') => Some("nyu"),
        ('ニ', 'ョ') => Some("nyo"),
        ('ヒ', 'ャ') => Some("hya"),
        ('ヒ', 'ュ') => Some("hyu"),
        ('ヒ', 'ョ') => Some("hyo"),
        ('ミ', 'ャ') => Some("mya"),
        ('ミ', 'ュ') => Some("myu"),
        ('ミ', 'ョ') => Some("myo"),
        ('リ', 'ャ') => Some("rya"),
        ('リ', 'ュ') => Some("ryu"),
        ('リ', 'ョ') => Some("ryo"),
        ('ギ', 'ャ') => Some("gya"),
        ('ギ', 'ュ') => Some("gyu"),
        ('ギ', 'ョ') => Some("gyo"),
        ('ジ', 'ャ') => Some("ja"),
        ('ジ', 'ュ') => Some("ju"),
        ('ジ', 'ェ') => Some("je"),
        ('ジ', 'ョ') => Some("jo"),
        ('ヂ', 'ャ') => Some("ja"),
        ('ヂ', 'ュ') => Some("ju"),
        ('ヂ', 'ェ') => Some("je"),
        ('ヂ', 'ョ') => Some("jo"),
        ('ビ', 'ャ') => Some("bya"),
        ('ビ', 'ュ') => Some("byu"),
        ('ビ', 'ョ') => Some("byo"),
        ('ピ', 'ャ') => Some("pya"),
        ('ピ', 'ュ') => Some("pyu"),
        ('ピ', 'ョ') => Some("pyo"),
        ('フ', 'ァ') => Some("fa"),
        ('フ', 'ィ') => Some("fi"),
        ('フ', 'ェ') => Some("fe"),
        ('フ', 'ォ') => Some("fo"),
        ('フ', 'ュ') => Some("fyu"),
        ('ヴ', 'ァ') => Some("va"),
        ('ヴ', 'ィ') => Some("vi"),
        ('ヴ', 'ェ') => Some("ve"),
        ('ヴ', 'ォ') => Some("vo"),
        ('ヴ', 'ュ') => Some("vyu"),
        ('テ', 'ィ') => Some("ti"),
        ('テ', 'ュ') => Some("tyu"),
        ('デ', 'ィ') => Some("di"),
        ('デ', 'ュ') => Some("dyu"),
        ('ト', 'ゥ') => Some("tu"),
        ('ド', 'ゥ') => Some("du"),
        ('ツ', 'ァ') => Some("tsa"),
        ('ツ', 'ィ') => Some("tsi"),
        ('ツ', 'ェ') => Some("tse"),
        ('ツ', 'ォ') => Some("tso"),
        ('ウ', 'ィ') => Some("wi"),
        ('ウ', 'ェ') => Some("we"),
        ('ウ', 'ォ') => Some("wo"),
        ('ク', 'ァ') => Some("kwa"),
        ('ク', 'ィ') => Some("kwi"),
        ('ク', 'ェ') => Some("kwe"),
        ('ク', 'ォ') => Some("kwo"),
        ('グ', 'ァ') => Some("gwa"),
        _ => None,
    }
}

fn lookup_single(c: char) -> Option<&'static str> {
    match c {
        'ア' => Some("a"),
        'イ' => Some("i"),
        'ウ' => Some("u"),
        'エ' => Some("e"),
        'オ' => Some("o"),
        'カ' => Some("ka"),
        'キ' => Some("ki"),
        'ク' => Some("ku"),
        'ケ' => Some("ke"),
        'コ' => Some("ko"),
        'ガ' => Some("ga"),
        'ギ' => Some("gi"),
        'グ' => Some("gu"),
        'ゲ' => Some("ge"),
        'ゴ' => Some("go"),
        'サ' => Some("sa"),
        'シ' => Some("shi"),
        'ス' => Some("su"),
        'セ' => Some("se"),
        'ソ' => Some("so"),
        'ザ' => Some("za"),
        'ジ' => Some("ji"),
        'ズ' => Some("zu"),
        'ゼ' => Some("ze"),
        'ゾ' => Some("zo"),
        'タ' => Some("ta"),
        'チ' => Some("chi"),
        'ツ' => Some("tsu"),
        'テ' => Some("te"),
        'ト' => Some("to"),
        'ダ' => Some("da"),
        'ヂ' => Some("ji"),
        'ヅ' => Some("zu"),
        'デ' => Some("de"),
        'ド' => Some("do"),
        'ナ' => Some("na"),
        'ニ' => Some("ni"),
        'ヌ' => Some("nu"),
        'ネ' => Some("ne"),
        'ノ' => Some("no"),
        'ハ' => Some("ha"),
        'ヒ' => Some("hi"),
        'フ' => Some("fu"),
        'ヘ' => Some("he"),
        'ホ' => Some("ho"),
        'バ' => Some("ba"),
        'ビ' => Some("bi"),
        'ブ' => Some("bu"),
        'ベ' => Some("be"),
        'ボ' => Some("bo"),
        'パ' => Some("pa"),
        'ピ' => Some("pi"),
        'プ' => Some("pu"),
        'ペ' => Some("pe"),
        'ポ' => Some("po"),
        'マ' => Some("ma"),
        'ミ' => Some("mi"),
        'ム' => Some("mu"),
        'メ' => Some("me"),
        'モ' => Some("mo"),
        'ヤ' => Some("ya"),
        'ユ' => Some("yu"),
        'ヨ' => Some("yo"),
        'ラ' => Some("ra"),
        'リ' => Some("ri"),
        'ル' => Some("ru"),
        'レ' => Some("re"),
        'ロ' => Some("ro"),
        'ワ' => Some("wa"),
        'ヰ' => Some("wi"),
        'ヱ' => Some("we"),
        'ヲ' => Some("o"),
        'ヴ' => Some("vu"),
        'ァ' => Some("a"),
        'ィ' => Some("i"),
        'ゥ' => Some("u"),
        'ェ' => Some("e"),
        'ォ' => Some("o"),
        'ャ' => Some("ya"),
        'ュ' => Some("yu"),
        'ョ' => Some("yo"),
        'ヮ' => Some("wa"),
        'ヵ' => Some("ka"),
        'ヶ' => Some("ke"),
        _ => None,
    }
}

fn next_syllable_romaji(chars: &[char]) -> Option<&'static str> {
    if chars.len() >= 2 {
        if let Some(r) = lookup_digraph(chars[0], chars[1]) {
            return Some(r);
        }
    }
    if let Some(&c) = chars.first() {
        return lookup_single(c);
    }
    None
}

/// Transliterates Katakana string to Hepburn Romaji, handling sokuon, chōonpu, and n apostrophe.
fn katakana_to_romaji(kana: &str) -> String {
    let kata = to_katakana(kana);
    let chars: Vec<char> = kata.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(n * 3);
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // 1. Long vowel mark ー prolongs the preceding vowel
        if c == 'ー' {
            let last_vowel = out.chars().rev().find(|&ch| matches!(ch, 'a' | 'i' | 'u' | 'e' | 'o'));
            if let Some(v) = last_vowel {
                out.push(v);
            } else {
                out.push('-');
            }
            i += 1;
            continue;
        }

        // 2. Small ッ / っ sokuon gemination
        if c == 'ッ' {
            let next_romaji = next_syllable_romaji(&chars[i + 1..]);
            if let Some(nr) = next_romaji {
                if nr.starts_with("ch") {
                    out.push('t');
                } else if let Some(first_ch) = nr.chars().next() {
                    if first_ch.is_ascii_alphabetic() && !matches!(first_ch, 'a' | 'i' | 'u' | 'e' | 'o') {
                        out.push(first_ch);
                    }
                }
            }
            i += 1;
            continue;
        }

        // 3. ん / ン before vowels or y appends apostrophe
        if c == 'ン' {
            let next_romaji = next_syllable_romaji(&chars[i + 1..]);
            if let Some(nr) = next_romaji {
                if nr.starts_with(['a', 'i', 'u', 'e', 'o', 'y']) {
                    out.push_str("n'");
                } else {
                    out.push('n');
                }
            } else {
                out.push('n');
            }
            i += 1;
            continue;
        }

        // 4. Digraphs (2-character combinations)
        if i + 1 < n {
            if let Some(romaji) = lookup_digraph(c, chars[i + 1]) {
                out.push_str(romaji);
                i += 2;
                continue;
            }
        }

        // 5. Single kana
        if let Some(romaji) = lookup_single(c) {
            out.push_str(romaji);
            i += 1;
            continue;
        }

        // 6. Non-kana characters (punctuation, numbers, ascii)
        match c {
            '、' => out.push(','),
            '。' => out.push('.'),
            '「' | '」' | '『' | '』' => out.push('"'),
            '・' => out.push(' '),
            '〜' | '～' => out.push('~'),
            '！' => out.push('!'),
            '？' => out.push('?'),
            '（' => out.push('('),
            '）' => out.push(')'),
            '：' => out.push(':'),
            '；' => out.push(';'),
            _ => {
                if c.is_ascii() {
                    out.push(c.to_ascii_lowercase());
                } else {
                    out.push(c);
                }
            }
        }
        i += 1;
    }

    out
}

/// Cleans and collapses consecutive spaces in converted romaji, removing space before punctuation.
fn clean_romaji_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.trim().chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else if matches!(c, ',' | '.' | '!' | '?' | ':' | ';' | ')') && last_was_space {
            if result.ends_with(' ') {
                result.pop();
            }
            result.push(c);
            last_was_space = false;
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result
}

struct TransToken {
    surface: String,
    reading: String,
    pos0: String,
    pos1: String,
    pos2: String,
    particle_override: Option<&'static str>,
}

fn should_attach(prev: &TransToken, curr: &TransToken) -> bool {
    if curr.reading.is_empty() && curr.particle_override.is_none() {
        return true;
    }

    // Sokuon at end of previous reading MUST attach
    if prev.reading.ends_with('ッ') || prev.reading.ends_with('っ') {
        return true;
    }

    // Sokuon at start of current reading MUST attach
    if curr.reading.starts_with('ッ') || curr.reading.starts_with('っ') {
        return true;
    }

    // Auxiliary verbs (助動詞) attach (e.g. た, だ, だっ, ない, ます, です, う)
    if curr.pos0 == "助動詞" {
        return true;
    }

    // Conjunctive particles (助詞, 接続助詞) attach (e.g. て, で, ば, たら)
    if curr.pos0 == "助詞" && curr.pos1 == "接続助詞" {
        return true;
    }

    // Dependent verbs (動詞, 非自立) e.g. てる after 愛し, or after て/で
    if curr.pos0 == "動詞" && curr.pos1 == "非自立" {
        if curr.surface == "てる" || curr.surface == "ている" {
            return true;
        }
        if prev.surface == "て" || prev.surface == "で" {
            return true;
        }
    }

    // Suffixes attach (e.g. 回, 人)
    if curr.pos1 == "接尾" {
        return true;
    }

    // Numbers attach to numbers
    if curr.pos1 == "数" && prev.pos1 == "数" {
        return true;
    }

    // Closing punctuation attaches to preceding word
    if matches!(curr.surface.as_str(), "、" | "。" | "！" | "？" | "」" | "』" | "）" | "," | "." | "!" | "?" | ")") {
        return true;
    }

    false
}

fn emit_chunk(tokens: &[TransToken], chunk: &[usize]) -> String {
    if chunk.is_empty() {
        return String::new();
    }
    if chunk.len() == 1 {
        let tok = &tokens[chunk[0]];
        if let Some(part) = tok.particle_override {
            return part.to_string();
        }
    }
    let mut combined_reading = String::new();
    for &idx in chunk {
        let tok = &tokens[idx];
        if let Some(part) = tok.particle_override {
            combined_reading.push_str(part);
        } else {
            combined_reading.push_str(&tok.reading);
        }
    }
    katakana_to_romaji(&combined_reading)
}

/// Transliterates a line of text to lowercase Hepburn Romaji if it contains Japanese script.
/// Results are memoized in an in-memory cache for zero per-frame conversion overhead.
/// Returns `None` for non-Japanese lines or if the result is empty or identical to input.
pub fn to_romaji_if_japanese(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !contains_japanese_script(trimmed) {
        return None;
    }

    // Check thread-safe memoization cache first
    if let Ok(guard) = ROMAJI_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(cached) = cache.get(trimmed) {
                return cached.clone();
            }
        }
    }

    let segmenter = get_segmenter()?;
    let mut tokens = segmenter.segment(Cow::Borrowed(trimmed)).ok()?;

    let mut trans_tokens: Vec<TransToken> = Vec::with_capacity(tokens.len());
    for token in tokens.iter_mut() {
        let surface = token.surface.to_string();
        let details = token.details();
        let pos0 = details.first().copied().unwrap_or("*").to_string();
        let pos1 = details.get(1).copied().unwrap_or("*").to_string();
        let pos2 = details.get(2).copied().unwrap_or("*").to_string();
        let reading = details
            .get(7)
            .copied()
            .filter(|&r| r != "*")
            .unwrap_or(&surface)
            .to_string();

        trans_tokens.push(TransToken {
            surface,
            reading,
            pos0,
            pos1,
            pos2,
            particle_override: None,
        });
    }

    let len = trans_tokens.len();
    for i in 0..len {
        // J-Pop Override 1: 傍 -> soba (override IPADIC's bou/hata)
        if trans_tokens[i].surface == "傍" {
            trans_tokens[i].reading = "ソバ".to_string();
        }

        // J-Pop Override 2: 君 -> kimi (when not suffix to person's name)
        if trans_tokens[i].surface == "君" {
            let is_person_suffix = i > 0
                && trans_tokens[i - 1].pos0 == "名詞"
                && trans_tokens[i - 1].pos1 == "固有名詞"
                && trans_tokens[i - 1].pos2 == "人名";
            if !is_person_suffix {
                trans_tokens[i].reading = "キミ".to_string();
            }
        }

        // J-Pop Override 3: 愛してる / 愛している / 愛し
        if trans_tokens[i].surface == "愛してる" {
            trans_tokens[i].reading = "アイシテル".to_string();
        } else if trans_tokens[i].surface == "愛している" {
            trans_tokens[i].reading = "アイシテイル".to_string();
        } else if trans_tokens[i].surface == "愛し" && trans_tokens[i].reading == "イトシ" {
            trans_tokens[i].reading = "アイシ".to_string();
        }

        // J-Pop Override 4: 一人 (hitori) / 二人 (futari)
        if trans_tokens[i].surface == "一" && i + 1 < len && trans_tokens[i + 1].surface == "人" {
            trans_tokens[i].reading = "ヒトリ".to_string();
            trans_tokens[i + 1].reading = String::new();
        } else if trans_tokens[i].surface == "二" && i + 1 < len && trans_tokens[i + 1].surface == "人" {
            trans_tokens[i].reading = "フタリ".to_string();
            trans_tokens[i + 1].reading = String::new();
        }

        // J-Pop Override 5: 何百回 -> nanbyakkai
        if trans_tokens[i].surface == "何" && i + 1 < len && trans_tokens[i + 1].surface == "百" {
            if i + 2 < len && trans_tokens[i + 2].surface == "回" {
                trans_tokens[i + 1].reading = "ビャッ".to_string();
            } else {
                trans_tokens[i + 1].reading = "ビャク".to_string();
            }
        }

        // Grammatical Particles (助詞): は -> wa, へ -> e, を -> o
        if trans_tokens[i].pos0 == "助詞" {
            if trans_tokens[i].surface == "は" || trans_tokens[i].reading == "ハ" {
                trans_tokens[i].particle_override = Some("wa");
            } else if trans_tokens[i].surface == "へ" || trans_tokens[i].reading == "ヘ" {
                trans_tokens[i].particle_override = Some("e");
            } else if trans_tokens[i].surface == "を" || trans_tokens[i].reading == "ヲ" {
                trans_tokens[i].particle_override = Some("o");
            }
        }
    }

    // Group into bunsetsu words
    let mut words: Vec<String> = Vec::new();
    let mut current_chunk: Vec<usize> = Vec::new();

    for (idx, token) in trans_tokens.iter().enumerate() {
        if token.reading.is_empty() && token.particle_override.is_none() {
            continue;
        }

        if current_chunk.is_empty() {
            current_chunk.push(idx);
        } else {
            let prev_idx = *current_chunk.last().unwrap_or(&0);
            let prev_token = &trans_tokens[prev_idx];
            if should_attach(prev_token, token) {
                current_chunk.push(idx);
            } else {
                let word = emit_chunk(&trans_tokens, &current_chunk);
                if !word.is_empty() {
                    words.push(word);
                }
                current_chunk.clear();
                current_chunk.push(idx);
            }
        }
    }

    if !current_chunk.is_empty() {
        let word = emit_chunk(&trans_tokens, &current_chunk);
        if !word.is_empty() {
            words.push(word);
        }
    }

    let joined = words.join(" ");
    let cleaned = clean_romaji_spaces(&joined);

    let res = if cleaned.is_empty() || cleaned.eq_ignore_ascii_case(trimmed) {
        None
    } else {
        Some(cleaned)
    };

    // Store in cache
    if let Ok(mut guard) = ROMAJI_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(trimmed.to_string(), res.clone());
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_english_returns_none() {
        assert_eq!(to_romaji_if_japanese("Hello world, this is English lyrics."), None);
        assert_eq!(to_romaji_if_japanese("1 2 3 4 testing 123!"), None);
        assert_eq!(to_romaji_if_japanese(""), None);
        assert_eq!(to_romaji_if_japanese("   "), None);
    }

    #[test]
    fn test_hiragana_katakana_conversion() {
        let res = to_romaji_if_japanese("こんにちは");
        assert!(res.is_some(), "Hiragana must convert to romaji");
        let romaji = res.unwrap_or_default();
        assert!(romaji.contains("konnichi"), "Expected 'konnichi' in '{}'", romaji);

        let res_kata = to_romaji_if_japanese("サヨナラ");
        assert!(res_kata.is_some(), "Katakana must convert to romaji");
        let romaji_kata = res_kata.unwrap_or_default();
        assert!(romaji_kata.contains("sayonara"), "Expected 'sayonara' in '{}'", romaji_kata);
    }

    #[test]
    fn test_mixed_kanji_kana_conversion() {
        let res = to_romaji_if_japanese("残響散歌");
        assert!(res.is_some(), "Kanji phrase must convert to romaji");
        let romaji = res.unwrap_or_default();
        assert!(!romaji.is_empty());
        assert!(romaji.chars().all(|c| c.is_ascii() || c.is_whitespace() || c.is_ascii_punctuation()));

        let res_phrase = to_romaji_if_japanese("誰かが待っている「夜明け」へ");
        assert!(res_phrase.is_some(), "Mixed Kanji/Kana lyric line must convert");
        let romaji_phrase = res_phrase.unwrap_or_default();
        assert!(romaji_phrase.contains("yoake"), "Expected 'yoake' in '{}'", romaji_phrase);
    }

    #[test]
    fn test_memoization_cache() {
        let line = "桜の木の下で";
        let first = to_romaji_if_japanese(line);
        let second = to_romaji_if_japanese(line);
        assert_eq!(first, second);
        assert!(first.is_some());
    }

    #[test]
    fn test_jpop_lyrics_edge_cases() {
        // 1. "愛してるを並べてみて" -> contains "aishiteru" (not "itoshi")
        let res1 = to_romaji_if_japanese("愛してるを並べてみて");
        assert!(res1.is_some(), "Should transliterate Japanese lyric");
        let romaji1 = res1.unwrap_or_default();
        assert!(
            romaji1.contains("aishiteru"),
            "Expected 'aishiteru' in '{}'",
            romaji1
        );
        assert!(
            !romaji1.contains("itoshi"),
            "Should not contain 'itoshi' in '{}'",
            romaji1
        );

        // 2. "残ってしまうだろうけど" -> contains "nokotte" (not "nokotsu")
        let res2 = to_romaji_if_japanese("残ってしまうだろうけど");
        assert!(res2.is_some(), "Should transliterate Japanese lyric");
        let romaji2 = res2.unwrap_or_default();
        assert!(
            romaji2.contains("nokotte"),
            "Expected 'nokotte' in '{}'",
            romaji2
        );
        assert!(
            !romaji2.contains("nokotsu"),
            "Should not contain 'nokotsu' in '{}'",
            romaji2
        );

        // 3. "離れないで傍に居てくれたのは" -> contains "soba" (not "bou")
        let res3 = to_romaji_if_japanese("離れないで傍に居てくれたのは");
        assert!(res3.is_some(), "Should transliterate Japanese lyric");
        let romaji3 = res3.unwrap_or_default();
        assert!(
            romaji3.contains("soba"),
            "Expected 'soba' in '{}'",
            romaji3
        );
        assert!(
            !romaji3.contains("bou"),
            "Should not contain 'bou' in '{}'",
            romaji3
        );

        // 4. "結局君一人だったよね" -> contains "kimi" (not "kun")
        let res4 = to_romaji_if_japanese("結局君一人だったよね");
        assert!(res4.is_some(), "Should transliterate Japanese lyric");
        let romaji4 = res4.unwrap_or_default();
        assert!(
            romaji4.contains("kimi"),
            "Expected 'kimi' in '{}'",
            romaji4
        );
        assert!(
            !romaji4.contains("kun"),
            "Should not contain 'kun' in '{}'",
            romaji4
        );

        // 5. "何百回の夜を過ごしたって" -> contains "nanbyakkai" or "nanbyakukai", "yoru", "sugoshitatte"
        let res5 = to_romaji_if_japanese("何百回の夜を過ごしたって");
        assert!(res5.is_some(), "Should transliterate Japanese lyric");
        let romaji5 = res5.unwrap_or_default();
        assert!(
            romaji5.contains("nanbyakkai") || romaji5.contains("nanbyakukai"),
            "Expected 'nanbyakkai' or 'nanbyakukai' in '{}'",
            romaji5
        );
        assert!(
            romaji5.contains("yoru"),
            "Expected 'yoru' in '{}'",
            romaji5
        );
        assert!(
            romaji5.contains("sugoshitatte"),
            "Expected 'sugoshitatte' in '{}'",
            romaji5
        );
    }

    #[test]
    fn test_strict_language_rejection() {
        // English text
        let english_lines = ["Hello world, this is English.", "Second line of the song."];
        assert!(!track_has_japanese_lyrics(&english_lines));
        assert_eq!(to_romaji_if_japanese("Hello world, this is English."), None);

        // Korean text
        let korean_lines = ["사랑해요", "보고 싶어", "영원히 함께"];
        assert!(!track_has_japanese_lyrics(&korean_lines));
        assert_eq!(to_romaji_if_japanese("사랑해요"), None);

        // Chinese Hanzi text
        let chinese_lines = ["我爱你", "你是一只小鸭子", "明天会更好"];
        assert!(!track_has_japanese_lyrics(&chinese_lines));

        // Japanese text must pass track check
        let japanese_lines = ["愛してるを並べてみて", "君一人だったよね"];
        assert!(track_has_japanese_lyrics(&japanese_lines));
    }

    #[test]
    fn test_phonological_rules() {
        // Sokuon before ch -> tch
        let res_tch = to_romaji_if_japanese("こっち");
        assert!(res_tch.is_some());
        assert!(res_tch.unwrap_or_default().contains("kotchi"));

        // Long vowel mark ー
        let res_choon = to_romaji_if_japanese("コーヒー");
        assert!(res_choon.is_some());
        assert!(res_choon.unwrap_or_default().contains("koohii"));

        // n' before vowel
        let res_n = to_romaji_if_japanese("きんえん");
        assert!(res_n.is_some());
        assert!(res_n.unwrap_or_default().contains("kin'en"));

        // Particle は -> wa, へ -> e, を -> o
        let res_particles = to_romaji_if_japanese("空へ手を伸ばせば");
        assert!(res_particles.is_some());
        let romaji_p = res_particles.unwrap_or_default();
        assert!(romaji_p.contains(" e "), "Expected particle 'e' in '{}'", romaji_p);
        assert!(romaji_p.contains(" o "), "Expected particle 'o' in '{}'", romaji_p);
    }
}
