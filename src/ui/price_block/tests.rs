use super::*;
use crate::game_data::UniqueModifiers;
use crate::price::estimate::{Reference, Worth};
use crate::price::{CurrencyPrice, UniquePrice};
use std::time::Duration;

const RATES: Rates = Rates {
    chaos: 75.0,
    divine: 750.0,
};
const WORTH: Worth = Worth {
    rates: RATES,
    currencies: &[],
};
const KNOWN: PricesState = PricesState::Known {
    age: Duration::ZERO,
    failure: None,
};
/// The complete prices of Standard.
const STANDARD: ShownLeague = ShownLeague {
    name: Some("Standard"),
    prices: KNOWN,
};
const DOWNLOADING: PricesState = PricesState::Downloading { progress: 0.5 };

fn notice<'a>(
    price: &'a Result<(FixedPrice<'a>, Rates), NoFixedPrice>,
    prices: PricesState,
    league: Option<&'a str>,
) -> PriceView<'a> {
    view(
        price,
        None,
        Rarity::Currency,
        ShownLeague {
            name: league,
            prices,
        },
    )
}

#[test]
fn a_rare_item_has_no_market_price() {
    let price = Err(NoFixedPrice::NotFixedName);

    assert_eq!(
        view(&price, None, Rarity::Magic, STANDARD),
        PriceView::Notice(Notice::NoMarketPrice(Rarity::Magic))
    );
}

#[test]
fn a_rare_or_magic_item_asks_for_a_stat_to_estimate_it_and_says_nothing_when_none_can_be() {
    use NoEstimate::NotSearched;
    use StatChecks::{Checked, NoneToCheck, Unchecked};

    // A rare or magic item.
    assert_eq!(
        estimate_hint(true, Checked, NotSearched),
        Some(Message::SearchToEstimate)
    );
    assert_eq!(
        estimate_hint(true, Unchecked, NotSearched),
        Some(Message::CheckToEstimate)
    );
    assert_eq!(estimate_hint(true, NoneToCheck, NotSearched), None);
    // A normal item, a gem or a waystone: priced by its base, its quality, its level or
    // its tier, not by modifiers.
    for checks in [Checked, Unchecked, NoneToCheck] {
        assert_eq!(
            estimate_hint(false, checks, NotSearched),
            Some(Message::SearchToEstimate),
            "{checks:?}"
        );
    }
}

#[test]
fn after_a_search_without_estimate_the_hint_says_why_or_leaves_it_to_the_offers() {
    use NoEstimate::{CurrenciesNotCounted, NothingCounted};
    use StatChecks::{Checked, Unchecked};

    for is_priced_by_stats in [true, false] {
        assert_eq!(
            estimate_hint(is_priced_by_stats, Checked, CurrenciesNotCounted),
            Some(Message::NoOfferCountedToEstimate)
        );
        assert_eq!(
            estimate_hint(is_priced_by_stats, Checked, NothingCounted),
            None
        );
    }
    // Its stats unchecked since: what to do first.
    assert_eq!(
        estimate_hint(true, Unchecked, CurrenciesNotCounted),
        Some(Message::CheckToEstimate)
    );
}

#[test]
fn a_skill_gem_has_no_market_price_either() {
    let price = Err(NoFixedPrice::NotFixedName);

    assert_eq!(
        view(&price, None, Rarity::Gem, STANDARD),
        PriceView::Notice(Notice::NoMarketPrice(Rarity::Gem))
    );
}

/// An estimate of `exalted` Exalted Orbs from `counted` offers, from `cheapest` to `dearest`.
fn estimate(exalted: f64, counted: usize, (cheapest, dearest): (f64, f64)) -> Estimate<'static> {
    Estimate {
        exalted,
        counted,
        cheapest,
        dearest,
        total: 18,
        reference: Reference {
            exalted: 1925.0,
            worth: WORTH,
        },
        searched_catalyst: None,
    }
}

#[test]
fn once_its_offers_are_searched_a_rare_item_has_an_estimated_price() {
    let estimate = estimate(1680.0, 5, (1500.0, 2250.0));

    assert_eq!(
        view(
            &Err(NoFixedPrice::NotFixedName),
            Some(estimate),
            Rarity::Rare,
            STANDARD
        ),
        PriceView::Estimated(estimate)
    );
}

#[test]
fn the_range_of_an_estimate_is_in_the_currency_of_its_median() {
    let written = |exalted, ends| {
        written_range(&estimate(exalted, 3, ends))
            .map(|range| (range.cheapest, range.dearest, range.currency))
    };
    let range = |cheapest: &str, dearest: &str, currency| {
        Some((cheapest.to_owned(), dearest.to_owned(), currency))
    };

    // Under one Divine Orb, in Exalted Orbs.
    assert_eq!(written(95.0, (40.0, 300.0)), range("40", "300", "ex"));
    // From one Divine Orb, in Divine Orbs, the cheapest too though it is worth less.
    assert_eq!(written(900.0, (600.0, 1500.0)), range("0.8", "2", "div"));
    // The dearest reaches one Divine Orb, the median doesn't: in Exalted Orbs.
    assert_eq!(written(700.0, (600.0, 1500.0)), range("600", "1.5K", "ex"));
}

#[test]
fn a_range_whose_ends_are_written_the_same_isnt_shown() {
    // One offer.
    assert_eq!(written_range(&estimate(1500.0, 1, (1500.0, 1500.0))), None);
    // 2.001 and 2.004 div are both written 2.
    assert_eq!(written_range(&estimate(1501.0, 2, (1500.75, 1503.0))), None);
}

#[test]
fn an_estimate_from_too_few_offers_has_a_low_reliability() {
    let low = |view: PriceView| low_reliability(&view);
    let estimated = |counted| PriceView::Estimated(estimate(1680.0, counted, (1500.0, 2250.0)));

    assert_eq!(low(estimated(4)), Some(LowReliability::SimilarOffers(4)));
    assert_eq!(low(estimated(5)), None);
    assert_eq!(
        low(PriceView::Notice(Notice::NoMarketPrice(Rarity::Rare))),
        None
    );
}

#[test]
fn an_item_missing_from_the_complete_prices_has_no_price_found() {
    let price = Err(NoFixedPrice::NotPriced);

    assert_eq!(
        notice(&price, KNOWN, Some("Standard")),
        PriceView::Notice(Notice::NoPriceFound("Standard"))
    );
    // Its category may still come.
    assert_eq!(
        notice(&price, DOWNLOADING, Some("Standard")),
        PriceView::Notice(Notice::PricesDownloading(Some("Standard")))
    );
    assert_eq!(
        notice(
            &price,
            PricesState::Unavailable(Failure::Unreachable),
            Some("Standard")
        ),
        PriceView::Notice(Notice::PricesUnavailable(Failure::Unreachable))
    );
}

#[test]
fn before_the_prices_the_block_says_they_are_coming() {
    let price = Err(NoFixedPrice::PricesUnknown);

    assert_eq!(
        notice(&price, DOWNLOADING, None),
        PriceView::Notice(Notice::PricesDownloading(None))
    );
    assert_eq!(
        notice(&price, PricesState::Unavailable(Failure::Unreachable), None),
        PriceView::Notice(Notice::PricesUnavailable(Failure::Unreachable))
    );
    assert_eq!(
        notice(&Err(NoFixedPrice::NamesUnknown), KNOWN, Some("Standard")),
        PriceView::Notice(Notice::FrenchNamesDownloading)
    );
}

#[test]
fn the_price_is_written_in_each_currency() {
    assert_eq!(in_currency(9000.0, Orb::Divine, RATES).0, "12");
    assert_eq!(in_currency(9000.0, Orb::Chaos, RATES).0, "120");
    assert_eq!(in_currency(9000.0, Orb::Exalted, RATES).0, "9K");
}

#[test]
fn an_amount_under_a_tenth_of_divine_or_chaos_orb_is_hidden() {
    let shown =
        |exalted, currency| shown_amount(exalted, currency, RATES).map(|(amount, _)| amount);
    // 0.1 div, 1 c: all shown.
    assert_eq!(shown(75.0, Orb::Divine).as_deref(), Some("0.1"));
    // 0.08 div, 0.8 c: the Divine Orbs hidden.
    assert_eq!(shown(60.0, Orb::Divine), None);
    assert_eq!(shown(60.0, Orb::Chaos).as_deref(), Some("0.8"));
    // 0.0013 div, 0.013 c: only the Exalted Orbs.
    assert_eq!(shown(1.0, Orb::Divine), None);
    assert_eq!(shown(1.0, Orb::Chaos), None);
    assert_eq!(shown(1.0, Orb::Exalted).as_deref(), Some("1"));
    assert_eq!(shown(0.004, Orb::Exalted).as_deref(), Some("<0.01"));
}

#[test]
fn the_threshold_reads_the_amount_as_written() {
    // 0.0953 div, written 0.1: shown; 0.0947 div, written 0.09: hidden.
    assert!(shown_amount(71.5, Orb::Divine, RATES).is_some());
    assert!(shown_amount(71.0, Orb::Divine, RATES).is_none());
}

#[test]
fn the_unit_price_leaves_out_the_small_amounts() {
    assert_eq!(unit_amounts(20.0, RATES), "0.27 c · 20 ex");
    assert_eq!(unit_amounts(187.5, RATES), "0.25 div · 2.5 c · 188 ex");
}

#[test]
fn a_row_of_a_list_is_in_divine_orbs_from_one_divine_orb() {
    assert_eq!(list_price(3000.0, RATES), ("4".to_owned(), Orb::Divine));
    assert_eq!(list_price(640.0, RATES), ("640".to_owned(), Orb::Exalted));
}

#[test]
fn a_change_is_rounded_to_a_whole_percent() {
    assert_eq!(rounded_percent(0.044), 4);
    assert_eq!(rounded_percent(0.106), 11);
}

#[test]
fn a_fixed_price_on_too_little_has_a_low_reliability_and_says_on_what() {
    let price = |quantity| Price {
        exalted: 120.0,
        quantity,
        history: Vec::new(),
    };
    let currency = |quantity| CurrencyPrice {
        name: "Lesser Charging Rune".to_owned(),
        exchange_id: "lesser-charging-rune".to_owned(),
        price: price(quantity),
    };
    let unique = |quantity| UniquePrice {
        name: "Headhunter".to_owned(),
        base_type: "Heavy Belt".to_owned(),
        price: price(quantity),
        modifiers: UniqueModifiers::default(),
    };
    let low = |fixed: FixedPrice| {
        low_reliability(&view(&Ok((fixed, RATES)), None, Rarity::Unique, STANDARD))
    };

    assert_eq!(
        low(FixedPrice::Currency(&currency(9))),
        Some(LowReliability::Trades(9))
    );
    assert_eq!(low(FixedPrice::Currency(&currency(10))), None);
    assert_eq!(
        low(FixedPrice::Unique(&unique(4))),
        Some(LowReliability::Offers(4))
    );
    assert_eq!(low(FixedPrice::Unique(&unique(5))), None);
}

#[test]
fn the_tooltip_of_an_orb_names_it_in_the_language_of_the_interface() {
    assert_eq!(orb_name(Orb::Divine, Language::English), "Divine Orb");
    assert_eq!(orb_name(Orb::Chaos, Language::French), "Orbe du chaos");
    assert_eq!(orb_name(Orb::Exalted, Language::French), "Orbe exalté");
}

#[test]
fn the_tooltip_of_an_orb_writes_what_one_is_worth_in_the_others() {
    let rates = Rates {
        chaos: 70.4,
        divine: 712.4,
    };
    let english = Translator::new(Language::English);
    let rate = |currency| orb_rate(currency, rates, &english);

    assert_eq!(
        [rate(Orb::Divine), rate(Orb::Chaos), rate(Orb::Exalted)],
        [
            "1 = 712 Exalted Orbs",
            "1 = 70 Exalted Orbs",
            "712 = 1 Divine Orb"
        ]
    );
    let french = Translator::new(Language::French);
    assert_eq!(orb_rate(Orb::Exalted, rates, &french), "712 = 1 Orbe divin");
}

#[test]
fn the_plural_of_a_rate_follows_the_amount_written() {
    let chaos = |chaos: f64, language| {
        let rates = Rates {
            chaos,
            divine: 712.0,
        };
        orb_rate(Orb::Chaos, rates, &Translator::new(language))
    };

    assert_eq!(chaos(1.0, Language::English), "1 = 1 Exalted Orb");
    assert_eq!(chaos(1.5, Language::English), "1 = 1.5 Exalted Orbs");
    // French puts a number under 2 in the singular; 1.996 is written 2, in the plural.
    assert_eq!(chaos(1.5, Language::French), "1 = 1.5 Orbe exalté");
    assert_eq!(chaos(1.996, Language::French), "1 = 2 Orbes exaltés");
}
