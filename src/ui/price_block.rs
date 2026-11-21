//! The price block, on band A under the item block: the price of a fixed-name item in the
//! three currencies, with its stack, its offers and its trend; the estimated price of the
//! other items, once their similar offers are searched, or else what to do to get one; the
//! possible uniques of an unidentified item; or, without a price, why.
//!
//! What the block shows is decided apart from the drawing ([`view`]), and tested.

use egui::{
    Align, Color32, Frame, Image, ImageSource, Label, Layout, Rect, Response, RichText, Sense, Ui,
    Vec2, pos2, vec2,
};

use super::images;
use super::theme::{color, size, text};
use super::widgets;
use crate::exchange_currencies;
use crate::filters::{Filter, FilterKey};
use crate::format;
use crate::i18n::{Message, Translator, WrittenRange};
use crate::item::Item;
use crate::language::Language;
use crate::market::Failure;
use crate::price::Orb;
use crate::price::estimate::{COUNTED, Estimate, NoEstimate};
use crate::price::fixed::{Candidate, FixedPrice, NoFixedPrice};
use crate::price::{Direction, Price, Rates};
use crate::session::PricesState;
use crate::trade::query::SearchFilters;
use crate::trade::stat_filters::LineStats;

/// The space above and under the block.
const PADDING_Y: f32 = 12.0;
/// The space under the title row.
const TITLE_GAP: f32 = 8.0;
/// The space above the lines under the amounts.
const LINE_GAP: f32 = 9.0;
/// How much of an orb shows where an estimate will go, as the mock-up fades it (35 %).
const PENDING_ORB_ALPHA: u8 = 90;
/// The orbs beside the amounts, and in a list.
const ORB_SIZE: f32 = 24.0;
const LIST_ORB_SIZE: f32 = 18.0;
/// The space between an orb and its amount, beside the amounts and in a list.
const ORB_GAP: f32 = 7.0;
const LIST_ORB_GAP: f32 = 6.0;
/// The space above the list of the possible uniques.
const LIST_GAP: f32 = 6.0;
/// The space above and under the text of a row of the list, and its height.
const ROW_PADDING_Y: f32 = 4.0;
const ROW_HEIGHT: f32 = LIST_ORB_SIZE + 2.0 * ROW_PADDING_Y;
/// The least space between the name of a possible unique and its price.
const NAME_GAP: f32 = 12.0;
/// Under this many Divine or Chaos Orbs, an amount is too small to matter and isn't shown.
const SMALL_AMOUNT: f64 = 0.1;

/// What the price block shows.
#[derive(Debug, Clone, PartialEq)]
enum PriceView<'a> {
    /// The price of a fixed-name item; of a unique, with its offers.
    Priced(MarketPrice<'a>),
    /// The price of a rare, magic or normal item, from its similar offers.
    Estimated(Estimate<'a>),
    /// A rare, magic or normal item, or a skill gem, priced from similar offers, before an
    /// estimate: what to do to get one.
    ToEstimate,
    /// The uniques an unidentified item can be.
    Candidates {
        candidates: &'a [Candidate<'a>],
        rates: Rates,
    },
    /// Why there is no price.
    Notice(Notice<'a>),
}

/// The price of a fixed-name item, with the rates of the prices it is read in.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MarketPrice<'a> {
    price: &'a Price,
    rates: Rates,
    of: Fixed,
}

/// What a fixed price is the price of, and whether it stands on too little to be sure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fixed {
    /// A unique, on fewer than [`crate::price::MIN_OFFERS`] offers when `is_thin`.
    Unique { is_thin: bool },
    /// An item of the currency exchange, on fewer than [`crate::price::MIN_TRADES`] trades
    /// when `is_thin`.
    Exchange { is_thin: bool },
}

/// Why a price stands on too little to be sure of it, with the count it stands on; nothing is
/// said of a reliable price, which a price of the trade site can't promise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowReliability {
    /// A unique on fewer than [`crate::price::MIN_OFFERS`] offers.
    Offers(u32),
    /// An item of the currency exchange on fewer than [`crate::price::MIN_TRADES`] trades.
    Trades(u32),
    /// An estimate from fewer than [`COUNTED`] offers, the suspicious ones left out.
    SimilarOffers(usize),
}

/// Why the price of `view` has a low reliability, if it has.
fn low_reliability(view: &PriceView) -> Option<LowReliability> {
    match *view {
        PriceView::Priced(MarketPrice { price, of, .. }) => match of {
            Fixed::Unique { is_thin: true } => Some(LowReliability::Offers(price.quantity)),
            Fixed::Exchange { is_thin: true } => Some(LowReliability::Trades(price.quantity)),
            Fixed::Unique { is_thin: false } | Fixed::Exchange { is_thin: false } => None,
        },
        PriceView::Estimated(estimate) => {
            (estimate.counted < COUNTED).then_some(LowReliability::SimilarOffers(estimate.counted))
        }
        PriceView::ToEstimate | PriceView::Candidates { .. } | PriceView::Notice(_) => None,
    }
}

/// The range of the offers counted in `estimate`, written in one currency, as a row of a list
/// ([`list_currency`]) chosen by the estimate; none from one offer, nor when both ends are
/// written the same.
fn written_range(estimate: &Estimate) -> Option<WrittenRange> {
    let rates = estimate.reference.worth.rates;
    let currency = list_currency(estimate.exalted, rates);
    let (cheapest, abbreviation) = in_currency(estimate.cheapest, currency, rates);
    let (dearest, _) = in_currency(estimate.dearest, currency, rates);
    (cheapest != dearest).then_some(WrittenRange {
        cheapest,
        dearest,
        currency: abbreviation,
    })
}

/// Whether the filters of the item, as the player left them, compare its stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatChecks {
    /// A filter that compares the item is checked: a line of a modifier, a pseudo total or
    /// a DPS beyond what its base reaches, a property of a waystone or of a gem.
    Checked,
    /// Some can be checked, none is.
    Unchecked,
    /// None can be: an unidentified item, or lines the trade site can't search.
    NoneToCheck,
}

impl StatChecks {
    /// The checks of `search`, the filters of the search made of `all` the filters of the item,
    /// checked or not, whose lines have the `stats`: by the one rule of the estimate
    /// ([`SearchFilters::compares_stats`]), on the filters as they are, then on all of them
    /// checked.
    #[must_use]
    pub fn of(search: &SearchFilters, stats: &LineStats, all: &[(FilterKey, Filter)]) -> Self {
        if search.compares_stats() {
            return Self::Checked;
        }
        let everything: Vec<(FilterKey, Filter)> = all
            .iter()
            .map(|(key, filter)| {
                let filter = Filter {
                    checked: true,
                    ..filter.clone()
                };
                (*key, filter)
            })
            .collect();
        if stats.search_filters(&everything).compares_stats() {
            Self::Unchecked
        } else {
            Self::NoneToCheck
        }
    }
}

/// What the line under the dashes of an item without estimate asks or says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EstimateHint {
    /// Search the offers: the search block beats for it.
    Search,
    /// Check a stat that sets the item apart first.
    CheckStat,
    /// No offer found is paid in a currency the estimate counts.
    NoCountedCurrency,
}

impl EstimateHint {
    fn message(self) -> Message {
        match self {
            Self::Search => Message::SearchToEstimate,
            Self::CheckStat => Message::CheckToEstimate,
            Self::NoCountedCurrency => Message::NoOfferCountedToEstimate,
        }
    }
}

/// The line under the dashes of an item without estimate, given the `checks` of its
/// filters and why it has `no_estimate`: an item priced by its stats ([`Item::is_priced_by_stats`]) is
/// estimated only from a search that compares some, so that it asks for one to be checked, and
/// says nothing when none can be (the offers of any item of its base would say nothing of its
/// price); once searched, why no offer is counted, or nothing when the offers block says it.
fn estimate_hint(
    is_priced_by_stats: bool,
    checks: StatChecks,
    no_estimate: NoEstimate,
) -> Option<EstimateHint> {
    if is_priced_by_stats {
        match checks {
            StatChecks::Checked => {}
            StatChecks::Unchecked => return Some(EstimateHint::CheckStat),
            StatChecks::NoneToCheck => return None,
        }
    }
    match no_estimate {
        NoEstimate::NotSearched => Some(EstimateHint::Search),
        NoEstimate::NothingCounted => None,
        NoEstimate::CurrenciesNotCounted => Some(EstimateHint::NoCountedCurrency),
    }
}

/// Why the price block shows no price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notice<'a> {
    /// The prices of the league are downloading; the league once it is known.
    PricesDownloading(Option<&'a str>),
    /// No price of the league, and the price database can't be read for this reason.
    PricesUnavailable(Failure),
    FrenchNamesDownloading,
    /// Missing from the complete prices of the league.
    NoPriceFound(&'a str),
}

/// The league shown: its name, once it is known, and where its prices stand.
#[derive(Debug, Clone, Copy)]
pub struct ShownLeague<'a> {
    pub name: Option<&'a str>,
    pub prices: PricesState,
}

/// What the block shows for the fixed `price` of an item, or else its `estimate`, given the
/// prices of the `league` shown.
fn view<'a>(
    price: &'a Result<(FixedPrice<'a>, Rates), NoFixedPrice>,
    estimate: Option<Estimate<'a>>,
    league: ShownLeague<'a>,
) -> PriceView<'a> {
    let ShownLeague { name, prices } = league;
    let downloading = match prices {
        PricesState::Unavailable(failure) => Notice::PricesUnavailable(failure),
        PricesState::Downloading { .. } | PricesState::Known { .. } => {
            Notice::PricesDownloading(name)
        }
    };
    match price {
        Ok((FixedPrice::Currency(currency), rates)) => PriceView::Priced(MarketPrice {
            price: &currency.price,
            rates: *rates,
            of: Fixed::Exchange {
                is_thin: currency.is_thin(),
            },
        }),
        Ok((FixedPrice::Unique(unique), rates)) => PriceView::Priced(MarketPrice {
            price: &unique.price,
            rates: *rates,
            of: Fixed::Unique {
                is_thin: unique.is_thin(),
            },
        }),
        Ok((FixedPrice::Uniques(candidates), rates)) => PriceView::Candidates {
            candidates,
            rates: *rates,
        },
        Err(NoFixedPrice::NotFixedName) => match estimate {
            Some(estimate) => PriceView::Estimated(estimate),
            None => PriceView::ToEstimate,
        },
        Err(NoFixedPrice::NamesUnknown) => PriceView::Notice(Notice::FrenchNamesDownloading),
        Err(NoFixedPrice::PricesUnknown) => PriceView::Notice(downloading),
        Err(NoFixedPrice::NotPriced) => PriceView::Notice(match (prices, name) {
            // Complete prices: the item isn't in them.
            (PricesState::Known { .. }, Some(league)) => Notice::NoPriceFound(league),
            // Its category may still come.
            _ => downloading,
        }),
    }
}

/// The block of an item: its fixed price, or why it has none; for an item without a fixed
/// price, its estimate from its similar offers, or else what to do to get one.
pub struct PriceBlock<'a> {
    pub item: &'a Item,
    /// The fixed price of the item with the rates of its prices, or why it has none.
    pub price: Result<(FixedPrice<'a>, Rates), NoFixedPrice>,
    /// The estimated price of an item without a fixed price, from its similar offers, or why
    /// it has none.
    pub estimate: Result<Estimate<'a>, NoEstimate>,
    /// Whether the filters of the item, as the player left them, compare its stats.
    pub checks: StatChecks,
    pub league: ShownLeague<'a>,
    pub i18n: &'a Translator,
}

impl PriceBlock<'_> {
    /// Whether the block asks to search the offers to estimate the item, before its search:
    /// the search block beats for it.
    #[must_use]
    pub fn asks_to_search(&self) -> bool {
        view(&self.price, self.estimate.ok(), self.league) == PriceView::ToEstimate
            && self.estimate_hint() == Some(EstimateHint::Search)
    }

    /// The line under the dashes of an item without estimate ([`estimate_hint`]).
    fn estimate_hint(&self) -> Option<EstimateHint> {
        let Err(no_estimate) = self.estimate else {
            return None;
        };
        estimate_hint(self.item.is_priced_by_stats(), self.checks, no_estimate)
    }

    /// Draws the block, the whole width of `ui`.
    pub fn show(self, ui: &mut Ui) {
        let view = view(&self.price, self.estimate.ok(), self.league);
        Frame::NONE
            .fill(color::BAND_A)
            .inner_margin(vec2(size::BAND_PADDING, PADDING_Y))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                self.title_row(ui, &view);
                ui.add_space(TITLE_GAP);
                match view {
                    PriceView::Priced(market_price) => self.priced(ui, market_price),
                    PriceView::Estimated(estimate) => self.estimated(ui, estimate),
                    PriceView::ToEstimate => self.to_estimate(ui),
                    PriceView::Candidates { candidates, rates } => {
                        self.candidates(ui, candidates, rates);
                    }
                    PriceView::Notice(notice) => self.notice(ui, notice),
                }
            });
    }

    /// The title, and on its right the offers of a unique or of an estimate, or the size of
    /// a stack; under it, why the price has a low reliability.
    fn title_row(&self, ui: &mut Ui, view: &PriceView) {
        let title = match view {
            PriceView::Estimated(_) | PriceView::ToEstimate => Message::EstimatedPriceLabel,
            PriceView::Priced(_) | PriceView::Candidates { .. } | PriceView::Notice(_) => {
                Message::PriceLabel
            }
        };
        let label = widgets::small_title_line(ui, &self.i18n.text(title));
        let caption = match *view {
            PriceView::Priced(MarketPrice {
                price,
                of: Fixed::Unique { .. },
                ..
            }) => Some(self.i18n.offers(price.quantity)),
            PriceView::Priced(MarketPrice {
                of: Fixed::Exchange { .. },
                ..
            }) => self.stack_size().map(|size| self.i18n.stack_of(size)),
            PriceView::Estimated(estimate) => Some(self.i18n.similar(estimate.total)),
            PriceView::ToEstimate | PriceView::Candidates { .. } | PriceView::Notice(_) => None,
        };
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(label.size(), Sense::hover());
            ui.painter().galley(rect.min, label, color::WEAK);
            if let Some(caption) = caption {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(caption).font(text::tag()).color(color::WEAK));
                });
            }
        });
        if let Some(reliability) = low_reliability(view) {
            let i18n = self.i18n;
            let reason = match reliability {
                LowReliability::Offers(count) => i18n.low_reliability_offers(count),
                LowReliability::Trades(count) => i18n.low_reliability_trades(count),
                LowReliability::SimilarOffers(counted) => i18n.low_reliability_similar(counted),
            };
            ui.add_space(size::TITLE_LINE_GAP);
            ui.add(
                Label::new(
                    RichText::new(reason)
                        .font(text::small())
                        .color(color::WARNING),
                )
                .wrap(),
            );
        }
    }

    /// The size of a copied stack of more than one item.
    fn stack_size(&self) -> Option<u32> {
        self.item
            .stack()
            .map(|stack| stack.size)
            .filter(|&size| size > 1)
    }

    /// The price in the three currencies, the value of the whole stack when there is one,
    /// then the unit price and the trend.
    fn priced(&self, ui: &mut Ui, market_price: MarketPrice) {
        let MarketPrice { price, rates, .. } = market_price;
        let stack = self.stack_size();
        amounts(
            ui,
            price.exalted * f64::from(stack.unwrap_or(1)),
            rates,
            self.i18n,
        );
        if stack.is_some() {
            ui.add_space(LINE_GAP);
            line(
                ui,
                &self.i18n.unit_price(&unit_amounts(price.exalted, rates)),
                color::WEAK,
            );
        }
        if let Some(trend) = price.trend() {
            let direction = trend.direction();
            let tint = match direction {
                Direction::Rise => color::RISE,
                Direction::Fall => color::FALL,
                Direction::Stable => color::WEAK,
            };
            let text = self
                .i18n
                .trend(direction, rounded_percent(trend.change.abs()), trend.days);
            ui.add_space(LINE_GAP);
            line(ui, &text, tint);
        }
    }

    /// The estimated price in the three currencies, then how it is made, with the range of its
    /// offers.
    fn estimated(&self, ui: &mut Ui, estimate: Estimate) {
        amounts(
            ui,
            estimate.exalted,
            estimate.reference.worth.rates,
            self.i18n,
        );
        let counted = estimate.counted;
        let range = written_range(&estimate);
        let note = if counted == COUNTED {
            self.i18n
                .estimate_median_cheapest(counted, estimate.searched_catalyst, range.as_ref())
        } else {
            self.i18n
                .estimate_median(counted, estimate.searched_catalyst, range.as_ref())
        };
        ui.add_space(LINE_GAP);
        line(ui, &note, color::WEAK);
    }

    /// Faint dashes where the estimate will go, then what to do to get one, or why the search
    /// gave none.
    fn to_estimate(&self, ui: &mut Ui) {
        pending_amounts(ui);
        if let Some(hint) = self.estimate_hint() {
            ui.add_space(LINE_GAP);
            line(ui, &self.i18n.text(hint.message()), color::WEAK);
        }
    }

    /// The uniques an unidentified item can be, each with its price in its own currency and the
    /// tooltip of its orb ([`orb_tooltip`]); a name too long is cut with "…", and shown whole
    /// under the pointer.
    fn candidates(&self, ui: &mut Ui, candidates: &[Candidate], rates: Rates) {
        message(ui, &self.i18n.text(Message::IdentifyToPrice), color::TEXT);
        ui.add_space(LINE_GAP);
        line(ui, &self.i18n.text(Message::PossibleUniques), color::WEAK);
        ui.add_space(LIST_GAP);
        let language = self.item.language();
        for (index, candidate) in candidates.iter().enumerate() {
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::hover());
            // The rows go to the edges of the band.
            let row = rect.expand2(vec2(size::BAND_PADDING, 0.0));
            if index % 2 == 0 {
                ui.painter().rect_filled(row, 0.0, color::ROW_VEIL);
            }
            let listed = candidate
                .price
                .map(|unique| list_price(unique.price.exalted, rates));
            let currency = listed.as_ref().map(|&(_, currency)| currency);
            let price = self.candidate_price(ui, rect, listed);
            let is_price_hovered = currency.is_some_and(|currency| {
                let id = response.id.with("price");
                orb_tooltip(
                    ui.interact(price, id, Sense::hover()),
                    currency,
                    rates,
                    self.i18n,
                )
                .hovered()
            });
            let name = &candidate.names.of(language).name;
            let name_width = (rect.width() - price.width() - NAME_GAP).max(0.0);
            let galley = widgets::truncated_line(ui, name.as_str(), text::body(), name_width);
            let cut = galley.elided;
            let top = rect.center().y - galley.size().y / 2.0;
            ui.painter()
                .galley(pos2(rect.left(), top), galley, color::UNIQUE);
            // Not over the price, which has a tooltip of its own.
            if cut && !is_price_hovered {
                response.on_hover_text(name.as_str());
            }
        }
    }

    /// Draws the price of a possible unique at the right of its `row`, as [`list_price`] writes
    /// it, or that it has no offers; the part of the row it takes, the whole height of the row.
    fn candidate_price(&self, ui: &Ui, row: Rect, listed: Option<(String, Orb)>) -> Rect {
        let painter = ui.painter();
        let Some((amount, currency)) = listed else {
            let label = self.i18n.text(Message::NoOffers);
            let galley = widgets::text_line(ui, label, text::tag());
            let size = galley.size();
            painter.galley(
                pos2(row.right() - size.x, row.center().y - size.y / 2.0),
                galley,
                color::WEAK,
            );
            return price_part(row, size.x);
        };
        let orb_rect = Rect::from_center_size(
            pos2(row.right() - LIST_ORB_SIZE / 2.0, row.center().y),
            Vec2::splat(LIST_ORB_SIZE),
        );
        orb(currency, LIST_ORB_SIZE).paint_at(ui, orb_rect);
        let galley = widgets::text_line(ui, amount, text::list_amount());
        let size = galley.size();
        let left = orb_rect.left() - LIST_ORB_GAP - size.x;
        painter.galley(
            pos2(left, row.center().y - size.y / 2.0),
            galley,
            color::TEXT,
        );
        price_part(row, row.right() - left)
    }

    /// Why the item has no price.
    fn notice(&self, ui: &mut Ui, notice: Notice) {
        let i18n = self.i18n;
        let text = match notice {
            Notice::PricesDownloading(Some(league)) => i18n.prices_downloading(league),
            Notice::PricesDownloading(None) => i18n.text(Message::PricesDownloadingNoLeague),
            Notice::PricesUnavailable(failure) => i18n.prices_unavailable(failure),
            Notice::FrenchNamesDownloading => i18n.text(Message::FrenchNamesDownloading),
            Notice::NoPriceFound(league) => i18n.no_price_found(league),
        };
        message(ui, &text, color::TEXT);
    }
}

/// The amounts in the three currencies, side by side, each with its orb, a third of the width
/// each, and its tooltip ([`orb_tooltip`]). An amount too small to matter is left out, and the
/// currencies after it move to the left.
fn amounts(ui: &mut Ui, exalted: f64, rates: Rates, i18n: &Translator) {
    let width = ui.available_width() / 3.0;
    ui.horizontal(|ui| {
        for currency in Orb::ALL {
            let Some((amount, _)) = shown_amount(exalted, currency, rates) else {
                continue;
            };
            let amount = RichText::new(amount).color(color::TEXT);
            let shown = amount_cell(ui, width, orb(currency, ORB_SIZE), amount);
            orb_tooltip(shown, currency, rates, i18n);
        }
    });
}

/// Where the amounts of an estimate will go: each orb faint, a dash in place of its amount.
fn pending_amounts(ui: &mut Ui) {
    let width = ui.available_width() / 3.0;
    let faded = Color32::from_white_alpha(PENDING_ORB_ALPHA);
    ui.horizontal(|ui| {
        for currency in Orb::ALL {
            let icon = orb(currency, ORB_SIZE).tint(faded);
            amount_cell(ui, width, icon, RichText::new("—").color(color::FAINT));
        }
    });
}

/// An `orb` and its `amount`, in the font of the prices, in a cell `width` across, whatever the
/// length of the amount; the part they take.
fn amount_cell(ui: &mut Ui, width: f32, orb: Image, amount: RichText) -> Response {
    ui.allocate_ui_with_layout(
        vec2(width, ORB_SIZE),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.spacing_mut().item_spacing.x = ORB_GAP;
            let icon = ui.add(orb);
            let amount = ui.label(amount.font(text::price()));
            icon.union(amount)
        },
    )
    .inner
}

/// Under the pointer over an orb or its amount, the name of the orb in the language of the
/// interface, then in gray what one is worth in the others ([`orb_rate`]).
fn orb_tooltip(response: Response, currency: Orb, rates: Rates, i18n: &Translator) -> Response {
    response.on_hover_ui(|ui| {
        ui.label(RichText::new(orb_name(currency, i18n.language())).color(color::TEXT));
        ui.label(RichText::new(orb_rate(currency, rates, i18n)).color(color::WEAK));
    })
}

/// The name of `currency` in `language`, as the trade site writes it: "Divine Orb", "Orbe
/// divin".
fn orb_name(currency: Orb, language: Language) -> &'static str {
    exchange_currencies::shown(currency.id())
        .expect("the three orbs are among the currencies of the exchange")
        .name(language)
}

/// What one orb of `currency` is worth in the others, by the `rates` every price of the block
/// is converted with: a Divine or a Chaos Orb in Exalted Orbs, an Exalted Orb by how many make
/// a Divine Orb; each amount as [`written_rate`] writes it.
fn orb_rate(currency: Orb, rates: Rates, i18n: &Translator) -> String {
    match currency {
        Orb::Divine => i18n.orb_rate_in_exalted(&written_rate(rates.divine)),
        Orb::Chaos => i18n.orb_rate_in_exalted(&written_rate(rates.chaos)),
        Orb::Exalted => i18n.orb_rate_for_divine(&written_rate(rates.divine)),
    }
}

/// A rate in Exalted Orbs, whole from 10 and with two decimals at most under it, never cut
/// short with `K` as the prices are: a rate is read for its precision ("1049", not "1K").
fn written_rate(exalted: f64) -> String {
    let decimals = if exalted >= 10.0 { 0 } else { 2 };
    format::number(format::rounded(exalted, decimals))
}

/// The unit price of a stack on one line, e.g. `0.27 c · 20 ex`, without the amounts too
/// small to matter.
fn unit_amounts(exalted: f64, rates: Rates) -> String {
    Orb::ALL
        .into_iter()
        .filter_map(|currency| shown_amount(exalted, currency, rates))
        .map(|(amount, abbreviation)| format!("{amount} {abbreviation}"))
        .collect::<Vec<_>>()
        .join(text::SEPARATOR)
}

/// `exalted` Exalted Orbs in `currency`, written as [`in_currency`] does, when worth showing:
/// always in Exalted Orbs, the smallest currency; in Divine or Chaos Orbs, when the amount as
/// written, rounded to two decimals, reaches [`SMALL_AMOUNT`].
fn shown_amount(exalted: f64, currency: Orb, rates: Rates) -> Option<(String, &'static str)> {
    let written = (value_in(exalted, currency, rates) * 100.0).round() / 100.0;
    (currency == Orb::Exalted || written >= SMALL_AMOUNT)
        .then(|| in_currency(exalted, currency, rates))
}

/// `exalted` Exalted Orbs in `currency`, written, with the abbreviation of the currency, the
/// same in every language.
fn in_currency(exalted: f64, currency: Orb, rates: Rates) -> (String, &'static str) {
    let abbreviation = match currency {
        Orb::Divine => "div",
        Orb::Chaos => "c",
        Orb::Exalted => "ex",
    };
    (
        format::amount(value_in(exalted, currency, rates), currency),
        abbreviation,
    )
}

/// `exalted` Exalted Orbs in `currency`.
fn value_in(exalted: f64, currency: Orb, rates: Rates) -> f64 {
    match currency {
        Orb::Divine => exalted / rates.divine,
        Orb::Chaos => exalted / rates.chaos,
        Orb::Exalted => exalted,
    }
}

/// The part of a `row` of a list its price takes, `width` across on its right.
fn price_part(row: Rect, width: f32) -> Rect {
    Rect::from_min_max(pos2(row.right() - width, row.top()), row.max)
}

/// The price of a row of a list, in one currency ([`list_currency`]).
fn list_price(exalted: f64, rates: Rates) -> (String, Orb) {
    let currency = list_currency(exalted, rates);
    (in_currency(exalted, currency, rates).0, currency)
}

/// The one currency of `exalted` Exalted Orbs in a list or a range: Divine Orbs from one
/// Divine Orb, Exalted Orbs under it.
fn list_currency(exalted: f64, rates: Rates) -> Orb {
    if exalted >= rates.divine {
        Orb::Divine
    } else {
        Orb::Exalted
    }
}

/// The orb of `currency`, `size` across.
pub(super) fn orb(currency: Orb, size: f32) -> Image<'static> {
    let source: ImageSource<'static> = match currency {
        Orb::Divine => images::divine_orb(),
        Orb::Chaos => images::chaos_orb(),
        Orb::Exalted => images::exalted_orb(),
    };
    Image::new(source).fit_to_exact_size(Vec2::splat(size))
}

/// A change in percent, rounded, e.g. 4 for `0.04`.
fn rounded_percent(change: f64) -> u32 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a positive change, capped"
    )]
    let percent = (change * 100.0).round().clamp(0.0, 1e6) as u32;
    percent
}

/// A line under the amounts, in `tint`.
fn line(ui: &mut Ui, text: &str, tint: Color32) {
    ui.add(Label::new(RichText::new(text).font(text::item_line()).color(tint)).wrap());
}

/// A message in place of the price, in `tint`.
fn message(ui: &mut Ui, text: &str, tint: Color32) {
    ui.add(Label::new(RichText::new(text).font(text::message()).color(tint)).wrap());
}

#[cfg(test)]
mod tests;
