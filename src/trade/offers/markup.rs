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
/// `<if:MS>{ orageux}<elif:FS>{ orageuse}…<else>{…}` takes the branch of the first form set,
/// else its `<else>`, else its first branch. Seen in the answers of `/fetch` of
/// 2026-11-15, on the French and the English trade sites (a Russian seller's whisper on the
/// English one): the names of the English site have no markup. A markup that doesn't close is
/// kept as it is.
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
    text.strip_prefix("<<set:")?.split_once(">>")
}

/// The branch of the choice at the start of `text`,
/// `<if:MS>{ orageux}<elif:FS>{ orageuse}<else>{…}`, for the `forms` set, and the text after
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
    if let Some((otherwise, after)) = rest
        .strip_prefix("<else>{")
        .and_then(|tail| tail.split_once('}'))
    {
        chosen = chosen.or(Some(otherwise));
        rest = after;
    }
    Some((chosen.unwrap_or(first), rest))
}

/// The form, the words and the text after a branch, from `MS>{ orageux}…`: the text after
/// its `<if:` or its `<elif:`.
fn branch(text: &str) -> Option<(&str, &str, &str)> {
    let (form, after) = text.split_once('>')?;
    let (words, after) = after.strip_prefix('{')?.split_once('}')?;
    Some((form, words, after))
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
    fn the_first_form_set_chooses_the_branch() {
        let whisper = "хочу купить у вас <<set:NS>><<set:N>><<set:S>><if:MS>{Ветреный}<elif:FS>{Ветреная}<elif:NS>{Ветреное}<elif:MP>{Ветреные}<elif:FP>{Ветреные}<elif:NP>{Ветреные} заклинание";
        assert_eq!(
            with_grammar_resolved(whisper),
            "хочу купить у вас Ветреное заклинание"
        );
        assert_eq!(
            with_grammar_resolved("a <<set:FP>>b<if:MS>{c}<elif:FS>{d}<elif:FP>{e}<else>{f} g"),
            "a be g"
        );
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
            "ends with <",
        ] {
            assert_eq!(with_grammar_resolved(text), text);
        }
        assert_eq!(
            with_grammar_resolved("<if:MS>{a}<elif:FS>{b"),
            "a<elif:FS>{b"
        );
    }
}
