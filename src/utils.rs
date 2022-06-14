/// Create a speakable sentence from a string.
/// This handles the following:
/// - Replacing symbols with their spoken equivalent
/// - Replacing diacritics with their spoken equivalent
/// - Speaking common operations like [i] as "index at i"
///
pub fn string_to_speakable_tokens(text: &str, cursor_position: Option<usize>) -> String {
    let replace_map = vec![
        ("<=", "less-than-or-equal-to"),
        (">=", "greater-than-or-equal-to"),
        ("<>", "not-equal-to"),
        ("<<", "left-shift"),
        (">>", "right-shift"),
        ("[", "square-bracket index at"),
        ("]", "close bracket"),
        ("(", "open paren"),
        (")", "close paren"),
        ("{", "open curly brace"),
        ("}", "close curly brace"),
        ("<", "open angle bracket"),
        (">", "close angle bracket"),
        (".", "dot"),
        ("&", "ref"),
        ("!", "bang"),
        ("#", "hash"),
        ("$", "dollarsign"),
        ("%", "percent"),
        ("^", "caret"),
        ("*", "asterisk"),
        ("++", "plus-plus"),
        ("--", "minus-minus"),
        ("+=", "plus-equals"),
        ("-=", "minus-equals"),
        ("+", "plus"),
        ("-", "minus"),
        ("=", "equals"),
        ("\\", "backslash"),
        ("|", "pipe"),
        ("/", "slash"),
        ("```", "triple-backtick"),
        ("`", "backtick"),
        ("'", "single-quote"),
        (",", "comma"),
        (";", "semicolon"),
        (":", "colon"),
        ("\"", "double-quote"),
        ("?", "question-mark"),
        ("_", "underscore"),
        ("~", "tilde"),
        ("@", "at-sign"),
        ("€", "euro"),
        ("£", "pound"),
        ("¥", "yen"),
    ];

    let mut text_copy = text.clone().to_string();
    for (symbol, replacement) in replace_map {
        text_copy = text_copy
            .replace(symbol, format!(" {} ", replacement).as_str())
            .to_string();
    }

    return text_copy.to_string();
}
