//! The markup of the texts of the trade site: the links of its words,
//! `[ItemRarity|Rarity of Items]`, and the grammar of the names it builds in each language,
//! `<<set:FS>> Longe<if:MS>{ orageux}<elif:FS>{ orageuse}`.

/// A text as the game shows it, without the markup of the trade site: `[ItemRarity|Rarity of
/// Items]` becomes `Rarity of Items`, `[Resistances]` becomes `Resistances`, and the grammar of
/// a name is resolved (`with_grammar_resolved`), the space the name of a rare item begins
/// with removed.
pub(super) fn plain_text(text: &str) -> String {
    let text = with_grammar_resolved(text);
    let mut plain = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some((before, after)) = rest.split_once('[') {
        plain.push_str(before);
        let Some((inside, tail)) = after.split_once(']') else {
            plain.push('[');
            rest = after;
            break;
        };
        plain.push_str(inside.rsplit_once('|').map_or(inside, |(_, shown)| shown));
        rest = tail;
    }
    plain.push_str(rest);
    plain.trim().to_owned()
}

/// A text with the grammar of its words chosen, as the game writes it. The trade site builds
/// the name of a rare item, in its own text and in the whisper of the offer, from words whose
/// form depends on the gender and the number of the first one: `<<set:F>><<set:S>><<set:FS>>`
/// gives the forms of that word (feminine, singular), and each choice
/// `<if:MS>{ orageux}<elif:FS>{ orageuse}…<else>{…}` takes its first branch whose form is
/// among the forms set (`FS`: ` orageuse`), else its `<else>`, else its first branch. Seen in
/// the answers of `/fetch` of 2026-11-15, on the French and the English trade sites (a Russian
/// seller's whisper on the English one), without an `<else>`, read in case a language writes
/// one: the names of the English site have no markup. A markup of another shape is kept as it
/// is: a form of other characters than capital letters, words holding a `<` or a `{`, a tag
/// that doesn't close. Each tag is given up at its first unexpected character, so that a
/// hostile text, a tag left open a thousand times, costs its length once, not its square
/// (the security audit of v0.1.2).
pub(super) fn with_grammar_resolved(text: &str) -> String {
    let mut forms = Vec::new();
    let mut resolved = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        resolved.push_str(&rest[..start]);
        let tail = &rest[start..];
        if let Some((form, after)) = form_set(tail) {
            forms.push(form);
            rest = after;
        } else if let Some((chosen, after)) = choice(tail, &forms) {
            resolved.push_str(chosen);
            rest = after;
        } else {
            resolved.push('<');
            rest = &tail[1..];
        }
    }
    resolved.push_str(rest);
    resolved
}

/// The form `<<set:FS>>` sets at the start of `text`, `FS`, and the text after it.
fn form_set(text: &str) -> Option<(&str, &str)> {
    let (form, after) = form(text.strip_prefix("<<set:")?)?;
    Some((form, after.strip_prefix(">>")?))
}

/// The words of the choice at the start of `text`,
/// `<if:MS>{ orageux}<elif:FS>{ orageuse}<else>{…}`: those of its first branch whose form is
/// among the `forms` set, else of its `<else>`, else of its first branch; and the text after
/// the choice.
fn choice<'a>(text: &'a str, forms: &[&str]) -> Option<(&'a str, &'a str)> {
    let (form, first, mut rest) = branch(text.strip_prefix("<if:")?)?;
    let mut chosen = forms.contains(&form).then_some(first);
    while let Some((form, words, after)) = rest.strip_prefix("<elif:").and_then(branch) {
        if chosen.is_none() && forms.contains(&form) {
            chosen = Some(words);
        }
        rest = after;
    }
    if let Some((otherwise, after)) = rest.strip_prefix("<else>{").and_then(words) {
        chosen = chosen.or(Some(otherwise));
        rest = after;
    }
    Some((chosen.unwrap_or(first), rest))
}

/// The form, the words and the text after a branch, from `MS>{ orageux}…`: the text after
/// its `<if:` or its `<elif:`.
fn branch(text: &str) -> Option<(&str, &str, &str)> {
    let (form, after) = form(text)?;
    let (words, after) = words(after.strip_prefix(">{")?)?;
    Some((form, words, after))
}

/// The form at the start of `text`, `FS` from `FS>>…`, and the text after it; none unless it
/// begins with a capital letter.
fn form(text: &str) -> Option<(&str, &str)> {
    let end = text
        .find(|letter: char| !letter.is_ascii_uppercase())
        .unwrap_or(text.len());
    text.split_at_checked(end)
        .filter(|(form, _)| !form.is_empty())
}

/// The words of a branch and the text after them, from ` orageux}…`; none if a `<` or a `{`
/// comes before its `}`.
fn words(text: &str) -> Option<(&str, &str)> {
    let end = text.find(['}', '<', '{'])?;
    text[end..]
        .strip_prefix('}')
        .map(|after| (&text[..end], after))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names of three rare items on the French trade site, from `/fetch` on 2026-11-15.
    const GLAS: &str = "<<set:M>><<set:S>><<set:MS>> Glas<if:MS>{ courroucé}<elif:FS>{ courroucée}<elif:NS>{NONEXISTENT}<elif:MP>{ courroucés}<elif:FP>{ courroucées}<elif:NP>{NONEXISTENT}";
    const LONGE: &str = "<<set:F>><<set:S>><<set:FS>> Longe<if:MS>{ orageux}<elif:FS>{ orageuse}<elif:NS>{NONEXISTENT}<elif:MP>{ orageux}<elif:FP>{ orageuses}<elif:NP>{NONEXISTENT}";
    const POTEAU: &str = "<<set:M>><<set:S>><<set:MS>> Poteau<if:MS>{\u{a0}vipérin}<elif:FS>{ vipérine}<elif:NS>{NONEXISTENT}<elif:MP>{ vipérins}<elif:FP>{ vipérines}<elif:NP>{NONEXISTENT}";

    #[test]
    fn the_links_of_the_trade_site_are_removed() {
        assert_eq!(
            plain_text("+10 to [Dexterity|Dexterity]"),
            "+10 to Dexterity"
        );
        assert_eq!(plain_text("[Resistances] and [A|B|C]"), "Resistances and C");
        assert_eq!(plain_text("an unclosed [bracket"), "an unclosed [bracket");
    }

    #[test]
    fn a_name_takes_the_words_of_the_gender_of_its_first_word() {
        assert_eq!(plain_text(GLAS), "Glas courroucé");
        assert_eq!(plain_text(LONGE), "Longe orageuse");
        assert_eq!(plain_text(POTEAU), "Poteau\u{a0}vipérin");
    }

    /// The forms set may come after the space that precedes the name, or with no space; and a
    /// form of a language that the French trade site never sets, the neuter of Russian.
    #[test]
    fn the_first_branch_whose_form_is_set_is_taken() {
        let whisper = "хочу купить у вас <<set:NS>><<set:N>><<set:S>><if:MS>{Ветреный}<elif:FS>{Ветреная}<elif:NS>{Ветреное}<elif:MP>{Ветреные}<elif:FP>{Ветреные}<elif:NP>{Ветреные} заклинание";
        assert_eq!(
            with_grammar_resolved(whisper),
            "хочу купить у вас Ветреное заклинание"
        );
        assert_eq!(
            with_grammar_resolved("a <<set:FP>>b<if:MS>{c}<elif:FS>{d}<elif:FP>{e}<else>{f} g"),
            "a be g"
        );
        assert_eq!(
            with_grammar_resolved("<<set:MS>><<set:FS>><if:MS>{a}<elif:FS>{b}"),
            "a"
        );
        assert_eq!(
            with_grammar_resolved("<<set:MS>><<set:FS>><if:FP>{a}<elif:FS>{b}<elif:MS>{c}"),
            "b"
        );
        assert_eq!(with_grammar_resolved("<<set:MS>><if:MS>{a}<else>{b}"), "a");
    }

    #[test]
    fn without_a_form_set_the_else_branch_then_the_first_one_is_taken() {
        assert_eq!(
            with_grammar_resolved("<if:MS>{Dentelé}<elif:FS>{Dentelée}<else>{Denteler}!"),
            "Denteler!"
        );
        assert_eq!(
            with_grammar_resolved("<if:MS>{Dentelé}<elif:FS>{Dentelée}!"),
            "Dentelé!"
        );
        assert_eq!(
            with_grammar_resolved("<<set:FS>><if:MS>{Dentelé}<elif:MP>{Dentelés}!"),
            "Dentelé!"
        );
    }

    /// A whisper keeps its spaces: the trade site writes one before the forms set too.
    #[test]
    fn a_whisper_keeps_its_spaces() {
        assert_eq!(
            with_grammar_resolved(&format!("acheter {GLAS} Grand gourdin élagué pour")),
            "acheter  Glas courroucé Grand gourdin élagué pour"
        );
    }

    #[test]
    fn a_markup_that_does_not_close_is_kept() {
        for text in [
            "a <<set:unclosed tag",
            "<<b>> stays",
            "a < b",
            "<if:MS without its words",
            "<if:MS>{unclosed words",
            "<if:MS> {spaced}",
            "<if:Ms>{lower case}",
            "<<set:>>empty form",
            "<if:MS>{words <b>}",
            "ends with <",
        ] {
            assert_eq!(with_grammar_resolved(text), text);
        }
        assert_eq!(
            with_grammar_resolved("<if:MS>{a}<elif:FS>{b"),
            "a<elif:FS>{b"
        );
    }

    /// A tag that closes far from where it opens would hide the text between: the price of a
    /// whisper (the security audit of v0.1.2).
    #[test]
    fn a_tag_never_hides_the_text_up_to_a_far_closing() {
        for text in [
            "@a Hi <<set:MS listed for 1 divine >> in Standard",
            "@a Hi <if:MS listed for 1 divine >{x} in Standard",
            "@a Hi <if:MS>{listed for <b> 1 divine} in Standard",
        ] {
            assert_eq!(with_grammar_resolved(text), text);
        }
    }

    /// The tags left open a hundred thousand times, about a megabyte: read in a few
    /// milliseconds, where the first reading of the grammar took about 20 seconds (the security
    /// audit of v0.1.2). The bound is loose, for a slow machine of the CI.
    #[test]
    fn a_text_of_open_tags_is_read_in_linear_time() {
        for open in ["<<set:", "<<set:MS", "<if:MS>{", "<if:MS>{a}<elif:FS>{"] {
            let text = open.repeat(1_000_000 / open.len());
            let started = std::time::Instant::now();

            with_grammar_resolved(&text);
            assert!(
                started.elapsed() < std::time::Duration::from_secs(2),
                "{open}: {:?}",
                started.elapsed()
            );
        }
    }
}
