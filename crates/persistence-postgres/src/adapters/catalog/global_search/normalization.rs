pub(super) const COMPACT_SQL_PATTERN: &str =
    "[[:space:][:punct:]·•・，。！？：；（）【】《》“”‘’]+";
pub(super) const LATIN_FOLD_SOURCE: &str = "áàâäãåāăąçćčďđéèêëēėęěíìîïīįłñńóòôöõøōőřśšúùûüūůűýÿžźż";
pub(super) const LATIN_FOLD_TARGET: &str = "aaaaaaaaacccddeeeeeeeeiiiiiilnnoooooooorssuuuuuuuyyzzz";

pub(super) fn normalize_query(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(fold_latin_character)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn fold_latin_character(character: char) -> char {
    match character {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'ď' | 'đ' => 'd',
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'ī' | 'į' => 'i',
        'ł' => 'l',
        'ñ' | 'ń' => 'n',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'ō' | 'ő' => 'o',
        'ř' => 'r',
        'ś' | 'š' => 's',
        'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' => 'u',
        'ý' | 'ÿ' => 'y',
        'ž' | 'ź' | 'ż' => 'z',
        other => other,
    }
}

pub(super) fn compact_query(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punctuation_free_query_can_match_compact_name() {
        assert_eq!(normalize_query("马龙·索萨"), "马龙 索萨");
        assert_eq!(compact_query("马龙·索萨"), "马龙索萨");
        assert_eq!(compact_query("marlon-sousa"), "marlonsousa");
    }

    #[test]
    fn latin_diacritics_are_folded_for_search() {
        assert_eq!(normalize_query("São Tomé"), "sao tome");
        assert_eq!(normalize_query("Kovačić"), "kovacic");
    }
}
