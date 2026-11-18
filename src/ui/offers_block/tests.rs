use super::*;
use crate::language::Language;
use crate::offers_state::SearchedBy;
use crate::trade::offers::fixtures::{exchange_offer, item_offer};
use crate::trade::search::{Rest, WaitCause};

#[test]
fn no_offer_is_said_only_when_none_is_left_to_fetch() {
    assert!(no_offer(&[], More::None));
    assert!(
        !no_offer(&[], More::Chevron),
        "a page left out for its currency: the next ones may be shown"
    );
    assert!(!no_offer(&[], More::Loading));
    let counting = Counting {
        reference: None,
        estimate_worth: None,
        searched_catalyst: None,
    };
    let rows = item_rows(
        &[item_offer(1.0, "divine")],
        counting,
        SystemTime::UNIX_EPOCH,
    );
    assert!(!no_offer(&rows, More::None));
}

#[test]
fn the_offers_of_items_show_the_chevron_while_some_are_left_and_the_spinner_while_they_come() {
    let mut items = ItemOffers {
        offers: vec![item_offer(1.0, "divine")],
        total: 12,
        first_page: 1,
        rest: Rest::new(Language::English, "Q2", &["x"]),
        next: NextOffers::NotAwaited,
        searched_by: SearchedBy::default(),
        searched: SystemTime::UNIX_EPOCH,
    };
    assert_eq!(More::of_items(&items), More::Chevron);

    items.next = NextOffers::Loading;
    assert_eq!(More::of_items(&items), More::Loading);

    items.next = NextOffers::Failed(Failure::Unreachable);
    assert_eq!(More::of_items(&items), More::Chevron, "to ask them again");

    items.next = NextOffers::Loading;

    items.rest = Rest::new(Language::English, "Q2", &[]);
    assert_eq!(More::of_items(&items), More::Loading, "the last ones come");

    items.next = NextOffers::NotAwaited;
    assert_eq!(More::of_items(&items), More::None);
}

#[test]
fn the_offers_of_the_exchange_show_the_chevron_until_all_are_shown() {
    let offers = vec![exchange_offer(); 3];
    let rows = exchange_rows(&offers, None, SystemTime::UNIX_EPOCH);

    let first = List::of_exchange(&rows[..2], &offers, 3, SystemTime::UNIX_EPOCH);
    let all = List::of_exchange(&rows, &offers, 3, SystemTime::UNIX_EPOCH);

    assert_eq!(first.more, More::Chevron);
    assert_eq!(all.more, More::None);
}

#[test]
fn a_refusal_that_names_no_limit_says_so_and_what_to_do_while_its_wait_runs() {
    let i18n = Translator::new(Language::English);
    let minute = Duration::from_mins(1);
    let line = |cause, wait| {
        let ((line, color), hint) = limited_lines(&i18n, cause, wait);
        (line, color, hint.map(|hint| hint.text))
    };

    assert_eq!(
        line(WaitCause::Forbidden, minute),
        (
            "Refused by the trade site: next search in 1min.".to_owned(),
            color::WARNING,
            Some("If it happens again, report it.".to_owned())
        )
    );
    for limit in [WaitCause::Limits, WaitCause::TooManyRequests] {
        assert_eq!(
            line(limit, minute),
            (
                "Trade site limit: next search in 1min.".to_owned(),
                color::WARNING,
                None
            )
        );
    }
    // The wait over, the line before the first search.
    assert_eq!(
        line(WaitCause::Forbidden, Duration::ZERO),
        (
            "One search per click, within the limits of the trade site.".to_owned(),
            color::FAINT,
            None
        )
    );
}
