//! The blocks of the item shown, one under the other, and what they are drawn from: the
//! filters of its search, its estimated price and the reference of its suspicious offers.

use std::time::Duration;

use egui::{OpenUrl, Ui};

use crate::filters::{Filter, FilterKey, Filters};
use crate::i18n::Translator;
use crate::item::Item;
use crate::league::League;
use crate::offers_state::{Offers, OffersState};
use crate::price::Rates;
use crate::price::estimate::{self, Estimate, NoEstimate, Reference, Worth};
use crate::price::fixed::{FixedPrice, NoFixedPrice};
use crate::session::{ItemTrade, PricesState, Session, ShownItem};
use crate::settings::PseudoTotals;
use crate::trade::query::{SearchFilters, SearchKind};
use crate::ui::{
    ItemBlock, ItemBlockAction, LineFacts, OffersAction, OffersBlock, PriceBlock, ShownLeague,
    StatChecks, Stats,
};

/// The filters of the search of the item of `trade`, the item `shown`: as the player set
/// them, or else by default, its `pseudo_totals` shown or hidden by the settings.
pub(super) fn search_filters(
    trade: &ItemTrade,
    shown: &ShownItem,
    filters: &Filters,
    pseudo_totals: PseudoTotals,
) -> SearchFilters {
    let (_, search) = every_filter(trade, shown, filters, pseudo_totals);
    search
}

/// What the application knows of the modifier lines of the item `shown`.
fn line_facts(shown: &ShownItem) -> LineFacts<'_> {
    LineFacts {
        copied_lines: shown.lines(),
        line_stats: shown.stats(),
        tier_rolls: shown.tiers(),
    }
}

/// Every filter of the item of `trade`, the item `shown`, checked or not, as the player set
/// them or else by default, and the filters of its search they make ([`search_filters`]).
fn every_filter(
    trade: &ItemTrade,
    shown: &ShownItem,
    filters: &Filters,
    pseudo_totals: PseudoTotals,
) -> (Vec<(FilterKey, Filter)>, SearchFilters) {
    let item = trade.item();
    let stats = Stats::new(
        item,
        line_facts(shown),
        trade.filtered_base_type(),
        pseudo_totals,
    );
    let all = stats.filters(filters);
    let search = SearchFilters {
        states: filters.lit_states_of(item).collect(),
        ..shown.stats().search_filters(&all)
    };
    (all, search)
}

/// The blocks of the item shown, one under the other: the item with its filters, its price,
/// and its offers when it can be searched (`kind`).
pub(super) struct ItemView<'a> {
    pub(super) session: &'a Session,
    pub(super) shown: &'a ShownItem,
    pub(super) item: &'a Item,
    pub(super) offers: &'a Offers,
    pub(super) filters: &'a mut Filters,
    pub(super) stats_open: bool,
    pub(super) pseudo_totals: PseudoTotals,
    pub(super) prices: PricesState,
    pub(super) kind: Option<SearchKind>,
    /// The window has the bug report, which the hint of a failed search opens: the main
    /// window only.
    pub(super) has_bug_report: bool,
    /// How long ago the item was copied.
    pub(super) since_copy: Duration,
    pub(super) i18n: &'a Translator,
}

impl ItemView<'_> {
    /// Draws the blocks; what the player did in the item block and in the offers block. The
    /// Check on trade pill opens its link here.
    pub(super) fn show(self, ui: &mut Ui) -> (Option<ItemBlockAction>, Option<OffersAction>) {
        let Self {
            session,
            shown,
            item,
            offers,
            filters,
            stats_open,
            pseudo_totals,
            prices,
            kind,
            has_bug_report,
            since_copy,
            i18n,
        } = self;
        let trade = session.trade(item);
        let fixed_price = session.fixed_price(item);
        let worth = session.worth();
        let estimate_worth = estimate_worth(offers, item, &fixed_price, worth);
        let estimate = shown_estimate(offers, item, estimate_worth);
        // Without a fixed price, the estimate tells the suspicious offers.
        let reference =
            reference(&fixed_price, worth).or(estimate.ok().map(|estimate| estimate.reference));
        let action = ItemBlock::new(item, line_facts(shown), filters, i18n)
            // The pill shows only when its link can be opened: the league and the search known.
            .tradable(trade.is_tradable())
            .stats(stats_open, trade.filtered_base_type(), pseudo_totals)
            .show(ui);
        // With the filters as the player just left them.
        let (all, searched) = every_filter(&trade, shown, filters, pseudo_totals);
        let checks = StatChecks::of(&searched, shown.stats(), &all);
        if action == Some(ItemBlockAction::CheckOnTrade)
            && let Some(url) = trade.url(searched)
        {
            ui.ctx().open_url(OpenUrl::new_tab(url));
        }
        let price_block = PriceBlock {
            item,
            price: fixed_price,
            estimate,
            checks,
            league: ShownLeague {
                name: session.league().map(League::name),
                prices,
            },
            i18n,
        };
        let asks_to_search = price_block.asks_to_search();
        price_block.show(ui);
        let offers_action = if let Some(kind) = kind {
            OffersBlock {
                offers,
                kind,
                reference,
                estimate_worth,
                item,
                lines: shown.lines(),
                has_bug_report,
                search_called_since: asks_to_search.then_some(since_copy),
                i18n,
            }
            .show(ui)
        } else {
            if trade.must_identify() {
                OffersBlock::identify(ui, i18n);
            }
            None
        };
        (action, offers_action)
    }
}

/// The price of one item that tells a suspicious offer: the price of a fixed-name item, with
/// the `worth` of the currencies of its prices; none for a price of low reliability, a unique
/// on too few offers or an item of the currency exchange on too few trades, which would tell
/// honest offers. Both come from the prices of the league shown: the rates beside the fixed
/// price are those of `worth`, which also prices the other currencies.
fn reference<'a>(
    fixed_price: &Result<(FixedPrice, Rates), NoFixedPrice>,
    worth: Option<Worth<'a>>,
) -> Option<Reference<'a>> {
    let exalted = match fixed_price {
        Ok((FixedPrice::Currency(currency), _)) => currency.trusted_exalted()?,
        Ok((FixedPrice::Unique(unique), _)) => unique.trusted_exalted()?,
        Ok((FixedPrice::Uniques(_), _)) | Err(_) => return None,
    };
    Some(Reference {
        exalted,
        worth: worth?,
    })
}

/// The `worth` of the currencies with the prices of the league, when `item`, the item shown, is
/// priced by its `offers`: without a fixed price, and for a rare or magic item, once a search
/// compared some of its stats; none otherwise, the offers of any item of its base saying
/// nothing of its own price.
fn estimate_worth<'a>(
    offers: &Offers,
    item: &Item,
    fixed_price: &Result<(FixedPrice, Rates), NoFixedPrice>,
    worth: Option<Worth<'a>>,
) -> Option<Worth<'a>> {
    let Err(NoFixedPrice::NotFixedName) = fixed_price else {
        return None;
    };
    if item.is_priced_by_stats() && !offers.compared_stats() {
        return None;
    }
    worth
}

/// The estimated price of `item`, the item shown, from the first page of the `offers` of its
/// search, with the `estimate_worth` of the currencies when it is priced by its offers; or why
/// it has none.
fn shown_estimate<'a>(
    offers: &Offers,
    item: &Item,
    estimate_worth: Option<Worth<'a>>,
) -> Result<Estimate<'a>, NoEstimate> {
    let (Some(worth), OffersState::Items(items)) = (estimate_worth, offers.state()) else {
        return Err(NoEstimate::NotSearched);
    };
    let searched_catalyst = offers.searched_catalyst(item);
    // The next offers loaded don't move it.
    let first_page = items.offers.get(..items.first_page).unwrap_or_default();
    Estimate::of(first_page, items.total, worth, searched_catalyst)
        .ok_or_else(|| estimate::no_estimate_reason(first_page, worth, searched_catalyst))
}

#[cfg(test)]
mod tests;
