//! Translation of XML Schema regular expressions (pattern facets) into the
//! syntax of the `regex` crate.
//!
//! The grammar follows XML Schema Part 2 (REC-xmlschema-2-20041028),
//! Appendix F. A pattern is implicitly anchored at both ends, `^` and `$`
//! are normal characters, `.` is `[^\n\r]`, and the multi-character escapes
//! map to the classes the recommendation defines. The `regex` crate matches
//! in time linear in the value, so a pattern such as `(;\s*.+=.+)*` cannot
//! backtrack catastrophically on untrusted input.
//!
//! Block escapes (`\p{IsBasicLatin}`) have no counterpart in the `regex`
//! crate and are rejected, as is anything outside the grammar; the caller
//! treats a rejected pattern as not checkable.

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
use std::sync::{Mutex, OnceLock, PoisonError};

use regex::Regex;

/// Unicode general categories allowed in `\p{..}` (XSD Part 2, F.1.1).
const CATEGORIES: &[&str] = &[
    "L", "Lu", "Ll", "Lt", "Lm", "Lo", "M", "Mn", "Mc", "Me", "N", "Nd", "Nl", "No", "P", "Pc",
    "Pd", "Ps", "Pe", "Pi", "Pf", "Po", "Z", "Zs", "Zl", "Zp", "S", "Sm", "Sc", "Sk", "So", "C",
    "Cc", "Cf", "Co", "Cn",
];

/// `\i`: initial name characters. The recommendation names XML 1.0
/// `Letter | '_' | ':'`; this uses the `NameStartChar` ranges of XML 1.0
/// fifth edition, which differ from the older `Letter` table only in
/// characters outside ASCII that no schema in use relies on.
const NAME_START: &str = r":A-Z_a-z\x{C0}-\x{D6}\x{D8}-\x{F6}\x{F8}-\x{2FF}\x{370}-\x{37D}\x{37F}-\x{1FFF}\x{200C}-\x{200D}\x{2070}-\x{218F}\x{2C00}-\x{2FEF}\x{3001}-\x{D7FF}\x{F900}-\x{FDCF}\x{FDF0}-\x{FFFD}\x{10000}-\x{EFFFF}";

/// `\c`: name characters, `NameStartChar` plus the `NameChar` additions.
const NAME_EXTRA: &str = r"\-.0-9\x{B7}\x{300}-\x{36F}\x{203F}-\x{2040}";

/// Returns the compiled regex for an XSD pattern, or `None` when the
/// pattern cannot be translated. Each pattern string compiles once.
pub(crate) fn compiled(pattern: &str) -> Option<Regex> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<Regex>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock().unwrap_or_else(PoisonError::into_inner);
    map.entry(pattern.to_string())
        .or_insert_with(|| translate(pattern).ok().and_then(|re| Regex::new(&re).ok()))
        .clone()
}

/// Translates an XSD pattern into an anchored `regex` crate pattern.
pub(crate) fn translate(pattern: &str) -> Result<String, String> {
    let mut chars = pattern.chars().peekable();
    let mut out = String::from(r"\A(?:");
    let mut depth = 0usize;
    // Whether the previous token is an atom a quantifier may follow.
    let mut quantifiable = false;
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                depth += 1;
                out.push_str("(?:");
                quantifiable = false;
            }
            ')' => {
                depth = depth.checked_sub(1).ok_or("unbalanced ')'")?;
                out.push(')');
                quantifiable = true;
            }
            '|' => {
                out.push('|');
                quantifiable = false;
            }
            '*' | '+' | '?' | '{' => {
                if !quantifiable {
                    return Err(format!("quantifier '{c}' without atom"));
                }
                if c == '{' {
                    out.push_str(&quantity(&mut chars)?);
                } else {
                    out.push(c);
                }
                quantifiable = false;
            }
            '.' => {
                out.push_str(r"[^\n\r]");
                quantifiable = true;
            }
            '[' => {
                out.push_str(&char_class_expr(&mut chars)?);
                quantifiable = true;
            }
            ']' => return Err("unescaped ']'".to_string()),
            '\\' => {
                out.push_str(&escape(&mut chars)?.into_regex());
                quantifiable = true;
            }
            _ => {
                out.push_str(&regex::escape(c.encode_utf8(&mut [0; 4])));
                quantifiable = true;
            }
        }
    }
    if depth != 0 {
        return Err("unbalanced '('".to_string());
    }
    out.push_str(r")\z");
    Ok(out)
}

/// Parses `n}`, `n,}` or `n,m}` after `{` into a `regex` quantifier.
fn quantity(chars: &mut Peekable<Chars<'_>>) -> Result<String, String> {
    let mut body = String::new();
    loop {
        match chars.next() {
            Some('}') => break,
            Some(c) if c.is_ascii_digit() || c == ',' => body.push(c),
            _ => return Err("malformed quantity".to_string()),
        }
    }
    let valid = match body.split_once(',') {
        None => !body.is_empty(),
        Some((min, max)) => {
            !min.is_empty()
                && !max.contains(',')
                && (max.is_empty() || max.parse::<u64>().ok() >= min.parse::<u64>().ok())
        }
    };
    if !valid {
        return Err(format!("malformed quantity {{{body}}}"));
    }
    Ok(format!("{{{body}}}"))
}

/// A parsed escape: one character, or a set in `regex` class syntax.
enum Escape {
    Char(char),
    Class(String),
}

impl Escape {
    fn into_regex(self) -> String {
        match self {
            Escape::Char(c) => regex::escape(c.encode_utf8(&mut [0; 4])),
            Escape::Class(class) => class,
        }
    }
}

/// Parses the escape after `\`.
fn escape(chars: &mut Peekable<Chars<'_>>) -> Result<Escape, String> {
    let c = chars.next().ok_or("trailing backslash")?;
    Ok(match c {
        'n' => Escape::Char('\n'),
        'r' => Escape::Char('\r'),
        't' => Escape::Char('\t'),
        '\\' | '|' | '.' | '?' | '*' | '+' | '(' | ')' | '{' | '}' | '-' | '[' | ']' | '^' => {
            Escape::Char(c)
        }
        's' => Escape::Class(r"[\t\n\r ]".to_string()),
        'S' => Escape::Class(r"[^\t\n\r ]".to_string()),
        'd' => Escape::Class(r"\p{Nd}".to_string()),
        'D' => Escape::Class(r"\P{Nd}".to_string()),
        'w' => Escape::Class(r"[^\p{P}\p{Z}\p{C}]".to_string()),
        'W' => Escape::Class(r"[\p{P}\p{Z}\p{C}]".to_string()),
        'i' => Escape::Class(format!("[{NAME_START}]")),
        'I' => Escape::Class(format!("[^{NAME_START}]")),
        'c' => Escape::Class(format!("[{NAME_START}{NAME_EXTRA}]")),
        'C' => Escape::Class(format!("[^{NAME_START}{NAME_EXTRA}]")),
        'p' | 'P' => {
            if chars.next() != Some('{') {
                return Err(format!("\\{c} without '{{'"));
            }
            let mut name = String::new();
            loop {
                match chars.next() {
                    Some('}') => break,
                    Some(n) => name.push(n),
                    None => return Err("unterminated property".to_string()),
                }
            }
            if !CATEGORIES.contains(&name.as_str()) {
                return Err(format!("unsupported property {name}"));
            }
            Escape::Class(format!("\\{c}{{{name}}}"))
        }
        _ => return Err(format!("unknown escape \\{c}")),
    })
}

/// Parses a character class expression after its opening `[`, through the
/// closing `]`, including a trailing subtraction `-[...]`.
fn char_class_expr(chars: &mut Peekable<Chars<'_>>) -> Result<String, String> {
    let negated = chars.peek() == Some(&'^');
    if negated {
        chars.next();
    }
    let mut items = String::new();
    let mut subtracted = None;
    loop {
        let c = chars.next().ok_or("unterminated character class")?;
        let start = match c {
            ']' if items.is_empty() => return Err("empty character group".to_string()),
            ']' => break,
            '[' => return Err("'[' inside a character group".to_string()),
            '-' if chars.peek() == Some(&'[') => {
                if items.is_empty() {
                    return Err("subtraction from an empty group".to_string());
                }
                chars.next();
                subtracted = Some(char_class_expr(chars)?);
                if chars.next() != Some(']') {
                    return Err("subtraction must end the group".to_string());
                }
                break;
            }
            '\\' => escape(chars)?,
            _ => Escape::Char(c),
        };
        match start {
            Escape::Class(class) => items.push_str(&class),
            Escape::Char(first) => {
                let mut ahead = chars.clone();
                let is_range =
                    ahead.next() == Some('-') && !matches!(ahead.peek(), Some(']' | '[') | None);
                if is_range {
                    chars.next();
                    let last = match chars.next() {
                        Some('\\') => match escape(chars)? {
                            Escape::Char(last) => last,
                            Escape::Class(_) => return Err("class as range end".to_string()),
                        },
                        Some(last) => last,
                        None => return Err("unterminated range".to_string()),
                    };
                    if last < first {
                        return Err(format!("reversed range {first}-{last}"));
                    }
                    items.push_str(&class_char(first));
                    items.push('-');
                    items.push_str(&class_char(last));
                } else {
                    items.push_str(&class_char(first));
                }
            }
        }
    }
    let group = if negated {
        format!("[^{items}]")
    } else {
        format!("[{items}]")
    };
    Ok(match subtracted {
        Some(sub) => format!("[{group}--{sub}]"),
        None => group,
    })
}

/// A character inside a `regex` class, written as a code point escape so
/// no class metacharacter (`-`, `&`, `~`, `^`, `[`) can take effect.
fn class_char(c: char) -> String {
    format!("\\x{{{:X}}}", u32::from(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::unwrap_used)]
    fn matches(pattern: &str, value: &str) -> bool {
        Regex::new(&translate(pattern).unwrap())
            .unwrap()
            .is_match(value)
    }

    #[test]
    fn block_escape_is_rejected() {
        assert!(translate(r"\p{IsBasicLatin}+").is_err());
        assert!(compiled(r"\p{IsBasicLatin}+").is_none());
    }

    #[test]
    fn caret_and_dollar_are_literals() {
        assert!(matches("^a$", "^a$"));
        assert!(!matches("^a$", "a"));
        assert!(matches("[^^]", "a"));
        assert!(!matches("[^^]", "^"));
    }

    #[test]
    fn word_escape_excludes_underscore() {
        assert!(matches(r"\w+", "aä1+$"));
        assert!(!matches(r"\w", "_"));
        assert!(!matches(r"\w", "-"));
        assert!(matches(r"[\w]", "a"));
        assert!(!matches(r"[\w]", "_"));
    }

    #[test]
    fn anchoring_covers_alternation() {
        assert!(!matches("a|b", "ab"));
        assert!(matches("a|b", "b"));
    }

    #[test]
    fn subtraction_and_ranges() {
        assert!(matches("[a-z-[aeiou]]+", "xyz"));
        assert!(!matches("[a-z-[aeiou]]", "a"));
        assert!(matches("[+-]", "-"));
        assert!(matches("[-a]", "-"));
    }

    #[test]
    fn name_escapes() {
        assert!(matches(r"\i\c*", "_a-1.b"));
        assert!(!matches(r"\i", "1"));
    }

    #[test]
    fn malformed_patterns_are_rejected() {
        for bad in ["(a", "a)", "*a", "a{2", "a{3,1}", "[a", r"a\", r"\q", "a**"] {
            assert!(translate(bad).is_err(), "{bad}");
        }
    }
}
