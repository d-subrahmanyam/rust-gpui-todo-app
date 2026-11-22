//! Markdown toolbar edits, computed as plain text operations so they can be
//! tested without a window. Offsets are UTF-8 byte offsets, matching the
//! input state's selection API.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdFormat {
    Bold,
    Italic,
    Code,
    Link,
    Heading,
    Bullet,
    Numbered,
    Quote,
}

/// Replace `range` of the original text with `text`, then select `select`
/// (offsets in the edited text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
    pub select: Range<usize>,
}

/// Compute the edit a toolbar button makes to `text` with selection `sel`.
pub fn apply(text: &str, sel: Range<usize>, f: MdFormat) -> Edit {
    match f {
        MdFormat::Bold => wrap(text, sel, "**", "**", "bold text"),
        MdFormat::Italic => wrap(text, sel, "_", "_", "italic text"),
        MdFormat::Code => wrap(text, sel, "`", "`", "code"),
        MdFormat::Link => link(text, sel),
        MdFormat::Heading => prefix_lines(text, sel, |_| "## ".into()),
        MdFormat::Bullet => prefix_lines(text, sel, |_| "- ".into()),
        MdFormat::Numbered => prefix_lines(text, sel, |i| format!("{}. ", i + 1)),
        MdFormat::Quote => prefix_lines(text, sel, |_| "> ".into()),
    }
}

fn wrap(text: &str, sel: Range<usize>, open: &str, close: &str, placeholder: &str) -> Edit {
    let inner = if sel.is_empty() {
        placeholder
    } else {
        &text[sel.clone()]
    };
    let start = sel.start + open.len();
    Edit {
        text: format!("{open}{inner}{close}"),
        select: start..start + inner.len(),
        range: sel,
    }
}

fn link(text: &str, sel: Range<usize>) -> Edit {
    const URL: &str = "https://";
    if sel.is_empty() {
        let start = sel.start + 1;
        return Edit {
            text: format!("[link text]({URL})"),
            select: start..start + "link text".len(),
            range: sel,
        };
    }
    let label = &text[sel.clone()];
    let url_start = sel.start + label.len() + 3; // "[" + label + "]("
    Edit {
        text: format!("[{label}]({URL})"),
        select: url_start..url_start + URL.len(),
        range: sel,
    }
}

/// Prefix every line touched by `sel`; if every line already starts with its
/// prefix, remove the prefixes instead (toggle).
fn prefix_lines(text: &str, sel: Range<usize>, prefix: impl Fn(usize) -> String) -> Edit {
    let start = text[..sel.start].rfind('\n').map_or(0, |i| i + 1);
    let end = text[sel.end..]
        .find('\n')
        .map_or(text.len(), |i| sel.end + i);
    let lines: Vec<&str> = text[start..end].split('\n').collect();
    let all_prefixed = lines
        .iter()
        .enumerate()
        .all(|(i, l)| l.starts_with(&prefix(i)));
    let new: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            if all_prefixed {
                l[prefix(i).len()..].to_string()
            } else {
                format!("{}{l}", prefix(i))
            }
        })
        .collect();
    let new = new.join("\n");
    Edit {
        select: start..start + new.len(),
        text: new,
        range: start..end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, sel: Range<usize>, f: MdFormat) -> (String, String) {
        let e = apply(text, sel, f);
        let mut out = text.to_string();
        out.replace_range(e.range.clone(), &e.text);
        let selected = out[e.select.clone()].to_string();
        (out, selected)
    }

    #[test]
    fn bold_wraps_selection_and_keeps_it_selected() {
        assert_eq!(
            run("say hi now", 4..6, MdFormat::Bold),
            ("say **hi** now".into(), "hi".into())
        );
    }

    #[test]
    fn bold_with_no_selection_inserts_placeholder_selected() {
        assert_eq!(
            run("ab", 1..1, MdFormat::Bold),
            ("a**bold text**b".into(), "bold text".into())
        );
    }

    #[test]
    fn italic_and_code_wrap() {
        assert_eq!(run("x", 0..1, MdFormat::Italic).0, "_x_");
        assert_eq!(run("x", 0..1, MdFormat::Code).0, "`x`");
    }

    #[test]
    fn link_wraps_text_and_selects_url() {
        assert_eq!(
            run("see docs", 4..8, MdFormat::Link),
            ("see [docs](https://)".into(), "https://".into())
        );
    }

    #[test]
    fn link_with_no_selection_selects_link_text() {
        assert_eq!(
            run("", 0..0, MdFormat::Link),
            ("[link text](https://)".into(), "link text".into())
        );
    }

    #[test]
    fn bullet_prefixes_every_selected_line() {
        let (out, _) = run("one\ntwo\nthree", 1..6, MdFormat::Bullet);
        assert_eq!(out, "- one\n- two\nthree");
    }

    #[test]
    fn bullet_toggles_off_when_all_lines_have_it() {
        let (out, _) = run("- one\n- two", 0..11, MdFormat::Bullet);
        assert_eq!(out, "one\ntwo");
    }

    #[test]
    fn numbered_counts_lines() {
        let (out, _) = run("a\nb\nc", 0..5, MdFormat::Numbered);
        assert_eq!(out, "1. a\n2. b\n3. c");
    }

    #[test]
    fn numbered_toggles_off() {
        let (out, _) = run("1. a\n2. b", 0..9, MdFormat::Numbered);
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn heading_and_quote_prefix_current_line_with_cursor_only() {
        assert_eq!(
            run("top\nmiddle\nend", 6..6, MdFormat::Heading).0,
            "top\n## middle\nend"
        );
        assert_eq!(run("top\nmiddle", 5..5, MdFormat::Quote).0, "top\n> middle");
    }

    #[test]
    fn line_formats_select_the_edited_block() {
        let (_, selected) = run("one\ntwo", 0..7, MdFormat::Bullet);
        assert_eq!(selected, "- one\n- two");
    }

    #[test]
    fn works_with_multibyte_text() {
        assert_eq!(run("héllo ✅", 0..6, MdFormat::Bold).0, "**héllo** ✅");
        assert_eq!(run("✅ done", 0..0, MdFormat::Bullet).0, "- ✅ done");
    }
}
