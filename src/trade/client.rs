//! The client of the trade site API: the searches of items and their offers, and the searches
//! of the currency exchange, within the limits the trade site announces.
//!
//! It never sends a request its limits don't allow, and lets 10 seconds pass at least between
//! two searches, whatever the item: what the limits know and the time of the last search are
//! kept in the cache directory after each answer, so that a new launch still counts them. It
//! sends no session cookie: the searches work without an account. The rules of its use are in
//! `docs/trade-site-usage.md`.

use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;

use super::limits::{Hit, Limits, Policy};
use super::offers::{ApiExchange, ApiFetch, ApiSearch, ExchangeOffer, ItemOffer};
use super::query::{self, ItemQuery, SearchKind};
use super::refusal::Refusal;
use super::transport::{Answer, Attempt, Guarded, Network, Request, Transport};
use crate::http::{self, HttpError};
use crate::language::Language;
use crate::league::League;

/// The time between two searches, at least.
pub const SEARCH_INTERVAL: Duration = Duration::from_secs(10);
/// The offers one fetch gives at most.
pub const FETCH_SIZE: usize = 10;

/// Why the trade site gave no answer.
#[derive(Debug)]
pub enum TradeError {
    /// The limits don't allow the request before this time: it wasn't sent.
    Wait(Instant),
    /// The trade site refused the request with this status (HTTP 429, or 403): nothing is sent
    /// before `until`.
    Refused { until: Instant, status: u16 },
    /// No answer, an error status, or an answer that isn't the expected one.
    Http(HttpError),
}

impl fmt::Display for TradeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = |until: &Instant| until.saturating_duration_since(Instant::now()).as_secs();
        match self {
            Self::Wait(until) => write!(formatter, "not sent, {} s to wait", seconds(until)),
            Self::Refused { until, status } => write!(
                formatter,
                "refused with the status {status}, {} s to wait",
                seconds(until)
            ),
            Self::Http(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for TradeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Wait(_) | Self::Refused { .. } => None,
        }
    }
}

impl From<HttpError> for TradeError {
    fn from(error: HttpError) -> Self {
        Self::Http(error)
    }
}

/// A search of items: the ids of its offers, the cheapest first, to fetch 10 at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSearch {
    pub id: String,
    pub offers: Vec<String>,
    /// The offers found, more than the ids given when there are many.
    pub total: u64,
}

/// A search of the currency exchange, with its offers.
#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeSearch {
    pub offers: Vec<ExchangeOffer>,
    pub total: u64,
}

/// The client of the trade site.
pub struct TradeClient<T = Network> {
    transport: Guarded<T>,
    /// The limits of the trade site, and when the last search left.
    limits: Limits,
    /// The log already says that an answer told no limits.
    told_no_limits: bool,
}

impl TradeClient {
    /// The client of the trade site, which counts the requests the last launch kept in
    /// `cache_directory`, and keeps its own there.
    #[must_use]
    pub fn new(refusal: Refusal, cache_directory: Option<PathBuf>) -> Self {
        Self::with_guarded(Guarded::network(refusal), Limits::kept_in(cache_directory))
    }
}

impl<T: Transport> TradeClient<T> {
    /// The client of the fake trade site `transport`, for the tests.
    #[cfg(test)]
    pub(super) fn with_transport(transport: T, refusal: Refusal) -> Self {
        Self::with_guarded(Guarded::new(transport, refusal), Limits::default())
    }

    /// The client of the trade site behind `transport`, which knows `limits` of it.
    fn with_guarded(transport: Guarded<T>, limits: Limits) -> Self {
        Self {
            transport,
            limits,
            told_no_limits: false,
        }
    }

    /// When a search of `kind` may be sent, `now` at the earliest: 10 s after the last
    /// search, of the last launch too, once its limits allow it; for items, once they also
    /// allow the fetch of its first offers.
    #[must_use]
    pub fn next_search(&self, kind: SearchKind, now: Instant) -> Instant {
        let interval = self
            .limits
            .since_last_search(now)
            .map_or(now, |since| now + SEARCH_INTERVAL.saturating_sub(since));
        let policies: &[Policy] = match kind {
            SearchKind::Items => &[Policy::Search, Policy::Fetch],
            SearchKind::Exchange => &[Policy::Exchange],
        };
        policies
            .iter()
            .map(|&policy| self.ready_at(policy, now))
            .fold(interval, Instant::max)
    }

    /// When a request of `policy` may be sent: once its limits allow it, and after the wait
    /// of a refusal, whichever request of the application the trade site refused.
    fn ready_at(&self, policy: Policy, now: Instant) -> Instant {
        let ready = self.limits.ready_at(policy, now);
        self.transport
            .refused_until(now)
            .map_or(ready, |until| ready.max(until))
    }

    /// When the next offers of a search may be fetched.
    #[must_use]
    pub fn next_fetch(&self, now: Instant) -> Instant {
        self.ready_at(Policy::Fetch, now)
    }

    /// Searches `query` in `league`, on the trade site of `language`.
    pub fn search_items(
        &mut self,
        language: Language,
        league: &League,
        query: &ItemQuery,
    ) -> Result<ItemSearch, TradeError> {
        self.check_search(SearchKind::Items)?;
        let request = Request {
            url: format!(
                "{}/api/trade2/search/poe2/{}",
                super::host(language),
                http::encode(league.id())
            ),
            body: Some(query.to_json().to_string()),
        };
        let answer: ApiSearch = self.send(Policy::Search, &request)?;
        Ok(ItemSearch {
            total: answer.total(),
            id: answer.id,
            offers: answer.result,
        })
    }

    /// The offers `ids` of the search `search`, 10 at most, in their order.
    pub fn fetch(
        &mut self,
        language: Language,
        search: &str,
        ids: &[String],
    ) -> Result<Vec<ItemOffer>, TradeError> {
        let ids = ids.get(..FETCH_SIZE).unwrap_or(ids);
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let request = Request {
            url: format!(
                "{}/api/trade2/fetch/{}?query={}&realm=poe2",
                super::host(language),
                ids.iter()
                    .map(|id| http::encode(id))
                    .collect::<Vec<_>>()
                    .join(","),
                http::encode(search)
            ),
            body: None,
        };
        let answer: ApiFetch = self.send(Policy::Fetch, &request)?;
        Ok(answer.offers(language))
    }

    /// Searches the offers of the item of the currency exchange `want` in `league`, on the
    /// trade site of `language`.
    pub fn search_exchange(
        &mut self,
        language: Language,
        league: &League,
        want: &str,
    ) -> Result<ExchangeSearch, TradeError> {
        self.check_search(SearchKind::Exchange)?;
        let request = Request {
            url: format!(
                "{}/api/trade2/exchange/poe2/{}",
                super::host(language),
                http::encode(league.id())
            ),
            body: Some(query::exchange_search(want).to_string()),
        };
        let answer: ApiExchange = self.send(Policy::Exchange, &request)?;
        let total = answer.total;
        let offers = answer.offers();
        Ok(ExchangeSearch {
            // Without the count of the trade site, at least the offers it gave.
            total: total.unwrap_or(offers.len() as u64),
            offers,
        })
    }

    /// Counts a search of `kind`, unless it must wait.
    fn check_search(&mut self, kind: SearchKind) -> Result<(), TradeError> {
        let now = Instant::now();
        let ready = self.next_search(kind, now);
        if ready > now {
            return Err(TradeError::Wait(ready));
        }
        self.limits.searched(now);
        Ok(())
    }

    /// Sends `request` of `policy`, when its limits allow it, and reads its answer.
    fn send<A: DeserializeOwned>(
        &mut self,
        policy: Policy,
        request: &Request,
    ) -> Result<A, TradeError> {
        let sent = Instant::now();
        let ready = self.ready_at(policy, sent);
        if ready > sent {
            return Err(TradeError::Wait(ready));
        }
        let attempt = self.transport.send(request);
        let answer = self.count(policy, sent, attempt);
        // Whatever came: the next launch counts this request and this search too.
        self.limits.keep();
        Ok(answer?.into_json()?)
    }

    /// Counts the request of `policy` sent at `sent`, now that `attempt` tells what came of
    /// it, with the limits its answer tells; gives the answer, unless the trade site refused.
    fn count(
        &mut self,
        policy: Policy,
        sent: Instant,
        attempt: Attempt,
    ) -> Result<Answer, TradeError> {
        // Counted once answered, with both times: nothing else is sent meanwhile
        // (`&mut self`). Only a success that tells the limits was counted for sure by the
        // trade site: a request that didn't reach it, or that it refused, may not be.
        let now = Instant::now();
        let hit = |counted| Hit {
            sent,
            answered: now,
            counted,
        };
        let answer = match attempt {
            // A refusal met meanwhile: by the download of the item names, or by another
            // Exile Market, whose wait the guard just read in its file.
            Attempt::Waiting(until) => return Err(TradeError::Wait(until)),
            Attempt::Unanswered(error) => {
                self.limits.sent(policy, hit(false));
                return Err(HttpError::Request(error).into());
            }
            Attempt::Refused { answer, .. } => {
                self.limits.sent(policy, hit(false));
                self.limits.answered(policy, &answer.limits, now);
                return Err(TradeError::Refused {
                    // When a request of this policy may leave again: after the wait the
                    // refusal started for every request, counted once by the guard, and
                    // after what the limits it told still ask of this policy.
                    until: self.ready_at(policy, now),
                    status: answer.status,
                });
            }
            Attempt::Answered(answer) => answer,
        };
        let counted = answer.is_success() && !answer.limits.rules.is_empty();
        self.limits.sent(policy, hit(counted));
        if answer.is_success() && !counted && !self.told_no_limits {
            // Its headers renamed, maybe: only the 10 seconds between two searches and the
            // wait of a refusal hold then. Once: every answer would say the same.
            log::warn!("Trade site: an answer told no limits, the known ones are kept");
            self.told_no_limits = true;
        }
        self.limits.answered(policy, &answer.limits, now);
        Ok(answer)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::files::TestDirectory;
    use crate::trade::kept_wait::tests::assert_about;
    use crate::trade::limits::LimitHeaders;
    use crate::trade::query::{RarityFilter, SearchedBase, StateFilters};
    use crate::trade::transport::tests::{FakeSite, limits};

    pub const SEARCH: &str = r#"{"id": "Q1", "complexity": 4, "total": 13,
        "result": ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m"]}"#;

    pub fn ring() -> ItemQuery {
        ItemQuery {
            name: Some("Andvarius".to_owned()),
            base: SearchedBase::Type("Gold Ring".to_owned()),
            properties: Vec::new(),
            rarity: Some(RarityFilter::Unique),
            states: StateFilters::default(),
            stats: Vec::new(),
        }
    }

    #[test]
    fn a_search_is_posted_to_the_trade_site_of_the_language_of_the_item() {
        let site = FakeSite::default();
        site.answer(200, LimitHeaders::default(), SEARCH);
        let mut client = TradeClient::with_transport(site.clone(), Refusal::default());

        let search = client
            .search_items(
                Language::French,
                &League::new("HC Forbidden Rites"),
                &ring(),
            )
            .expect("an answer");

        assert_eq!(search.id, "Q1");
        assert_eq!(search.offers.len(), 13);
        assert_eq!(search.total, 13);
        let request = &site.requests()[0];
        assert_eq!(
            request.url,
            "https://fr.pathofexile.com/api/trade2/search/poe2/HC%20Forbidden%20Rites"
        );
        assert_eq!(
            request.body.as_deref(),
            Some(ring().to_json().to_string().as_str())
        );
    }

    #[test]
    fn ten_offers_at_most_are_fetched_by_their_ids() {
        let site = FakeSite::default();
        site.answer(200, LimitHeaders::default(), r#"{"result": []}"#);
        let mut client = TradeClient::with_transport(site.clone(), Refusal::default());
        let ids: Vec<String> = ('a'..='m').map(String::from).collect();

        client
            .fetch(Language::English, "Q1", &ids)
            .expect("an answer");

        assert_eq!(
            site.requests()[0],
            Request {
                url: "https://www.pathofexile.com/api/trade2/fetch/a,b,c,d,e,f,g,h,i,j?query=Q1&realm=poe2"
                    .to_owned(),
                body: None,
            }
        );
        assert!(
            client.fetch(Language::English, "Q1", &[]).is_ok(),
            "nothing to fetch, nothing sent"
        );
        assert_eq!(site.requests().len(), 1);
    }

    #[test]
    fn a_second_search_waits_ten_seconds_whatever_the_item() {
        let site = FakeSite::default();
        site.answer(200, LimitHeaders::default(), SEARCH);
        let mut client = TradeClient::with_transport(site.clone(), Refusal::default());
        let league = League::new("Standard");
        client
            .search_items(Language::English, &league, &ring())
            .expect("an answer");

        let exchange = client.search_exchange(Language::English, &league, "divine");

        assert!(matches!(exchange, Err(TradeError::Wait(_))));
        assert_eq!(site.requests().len(), 1, "not sent");
        let now = Instant::now();
        let next = client.next_search(SearchKind::Exchange, now);
        assert!(next + Duration::from_secs(1) > now + SEARCH_INTERVAL);
    }

    /// The client of an application launched with the cache directory `directory`.
    fn launched_with(directory: &TestDirectory, site: FakeSite) -> TradeClient<FakeSite> {
        TradeClient::with_guarded(
            Guarded::new(site, Refusal::default()),
            Limits::kept_in(Some(directory.0.clone())),
        )
    }

    /// Closing the application and launching it again doesn't shorten the 10 seconds.
    #[test]
    fn a_search_makes_the_next_launch_wait_its_ten_seconds() {
        let directory = TestDirectory::new("client-search-kept");
        let site = FakeSite::default();
        site.answer(200, LimitHeaders::default(), SEARCH);
        let league = League::new("Standard");
        let searched = Instant::now();
        launched_with(&directory, site)
            .search_items(Language::English, &league, &ring())
            .expect("an answer");

        let site = FakeSite::default();
        let mut next_launch = launched_with(&directory, site.clone());

        let next = next_launch.next_search(SearchKind::Exchange, Instant::now());
        assert_about(Some(next), searched + SEARCH_INTERVAL);
        let search = next_launch.search_items(Language::English, &league, &ring());
        assert!(matches!(search, Err(TradeError::Wait(_))), "{search:?}");
        assert!(site.requests().is_empty(), "not sent");
    }

    /// Each one written to the file: a request without an answer, an answer, an error status.
    #[test]
    fn every_request_is_counted_by_the_next_launch() {
        let fetch_limits = |state| limits("trade-fetch-request-limit", "4:300:10", state);
        let directory = TestDirectory::new("client-requests-kept");
        let ids = ["a".to_owned()];
        // Nothing to answer the first request: the connection fails.
        let _ = launched_with(&directory, FakeSite::default()).fetch(Language::English, "Q1", &ids);
        let site = FakeSite::default();
        site.answer(200, fetch_limits("1:300:0"), r#"{"result": []}"#);
        site.answer(400, fetch_limits("2:300:0"), "{}");
        let mut client = launched_with(&directory, site);
        client.fetch(Language::English, "Q1", &ids).expect("offers");
        let _ = client.fetch(Language::English, "Q1", &ids);

        let next_launch = launched_with(&directory, FakeSite::default());

        // 4 per 300 s: the three requests, and the one of another tool the trade site
        // counted beyond the only request it counted for sure.
        let now = Instant::now();
        assert!(next_launch.next_fetch(now) > now + Duration::from_secs(290));
    }

    #[test]
    fn a_request_the_limits_dont_allow_isnt_sent() {
        let site = FakeSite::default();
        site.answer(
            200,
            limits("trade-fetch-request-limit", "1:4:10", "1:4:0"),
            r#"{"result": []}"#,
        );
        let mut client = TradeClient::with_transport(site.clone(), Refusal::default());
        let ids = ["a".to_owned()];
        client
            .fetch(Language::English, "Q1", &ids)
            .expect("an answer");

        let second = client.fetch(Language::English, "Q1", &ids);

        assert!(matches!(second, Err(TradeError::Wait(_))));
        assert_eq!(site.requests().len(), 1);
        // A search of items waits for its fetch too.
        let now = Instant::now();
        assert!(client.next_search(SearchKind::Items, now) > now);
        assert_eq!(client.next_search(SearchKind::Exchange, now), now);
    }

    #[test]
    fn a_refusal_waits_for_its_delay_without_retrying() {
        let site = FakeSite::default();
        let mut headers = limits("trade-search-request-limit", "5:10:60", "6:10:60");
        headers.retry_after = Some("60".to_owned());
        site.answer(429, headers, "");
        let mut client = TradeClient::with_transport(site.clone(), Refusal::default());

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        assert!(until > Instant::now() + Duration::from_secs(55));
        assert_eq!(site.requests().len(), 1);
        assert!(client.next_search(SearchKind::Items, Instant::now()) >= until);
    }

    /// The code review of 22k: the wait shown to the player was counted apart from the one
    /// every request shares, which ended with the delay when the restriction was longer.
    #[test]
    fn the_wait_a_refusal_shows_is_the_one_every_request_shares() {
        use crate::trade::kept_wait::tests::assert_about;

        let site = FakeSite::default();
        let mut headers = limits("trade-search-request-limit", "5:10:60", "6:10:300");
        headers.retry_after = Some("60".to_owned());
        site.answer(429, headers, "");
        let refusal = Refusal::default();
        let mut client = TradeClient::with_transport(site, refusal.clone());
        let start = Instant::now();

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        assert!(until >= start + Duration::from_mins(5), "{until:?}");
        assert_about(refusal.until(start), until);
    }

    /// The same for a refusal that tells no limits: the wait shown is the shared one, which
    /// the policy no longer counts by itself.
    #[test]
    fn a_refusal_without_limits_shows_the_wait_every_request_shares() {
        let site = FakeSite::default();
        site.answer(403, crate::trade::transport::tests::retry_after("600"), "");
        let refusal = Refusal::default();
        let mut client = TradeClient::with_transport(site, refusal.clone());
        let start = Instant::now();

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        assert_eq!(Some(until), refusal.until(start));
        assert!(until >= start + Duration::from_mins(10), "{until:?}");
    }

    /// The limits a refusal tells are read too: a window still full once its wait is over
    /// (600 searches per 6 h are refused for an hour only, measured on 2026-10-26; the state
    /// is written by hand) lets no search of its policy leave before it has room.
    #[test]
    fn the_windows_a_refusal_tells_still_count_after_its_wait() {
        let site = FakeSite::default();
        let mut headers = limits(
            "trade-search-request-limit",
            "600:21600:3600",
            "601:21600:3600",
        );
        headers.retry_after = Some("3600".to_owned());
        site.answer(429, headers, "");
        let refusal = Refusal::default();
        let mut client = TradeClient::with_transport(site, refusal.clone());
        let start = Instant::now();

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        // Every request waits an hour; the searches of items, the six hours of their window.
        let shared = refusal.until(start).expect("a shared wait");
        assert!(shared < start + Duration::from_hours(2), "{shared:?}");
        assert!(until >= start + Duration::from_hours(6), "{until:?}");
        assert!(client.next_search(SearchKind::Items, start) >= until);
        assert!(client.next_search(SearchKind::Exchange, start) < start + Duration::from_hours(2));
    }

    /// The security audit of 2026-11-10: the download of the item names shares this wait.
    #[test]
    fn a_refusal_makes_every_request_to_the_trade_site_wait() {
        for status in [429, 403] {
            let site = FakeSite::default();
            let mut headers = limits("trade-search-request-limit", "5:10:60", "6:10:60");
            headers.retry_after = Some("1800".to_owned());
            site.answer(status, headers, "");
            let refusal = Refusal::default();
            let mut client = TradeClient::with_transport(site.clone(), refusal.clone());
            let start = Instant::now();

            let search = client.search_items(Language::English, &League::new("Standard"), &ring());

            assert!(
                matches!(search, Err(TradeError::Refused { .. })),
                "{search:?}"
            );
            let until = refusal.until(start).expect("a shared wait");
            assert!(until >= start + Duration::from_mins(30), "{status}");
            // The other policies wait too: the same site refused the same address.
            assert!(client.next_search(SearchKind::Exchange, start) >= until);
            assert!(client.next_fetch(start) >= until);
            assert_eq!(site.requests().len(), 1, "{status}");
        }
    }

    #[test]
    fn a_refusal_without_delay_waits_a_minute() {
        let site = FakeSite::default();
        site.answer(403, LimitHeaders::default(), "");
        let refusal = Refusal::default();
        let mut client = TradeClient::with_transport(site, refusal.clone());
        let start = Instant::now();

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        assert!(until >= start + Duration::from_mins(1));
        assert!(
            refusal
                .until(start)
                .is_some_and(|wait| wait >= start + Duration::from_mins(1))
        );
    }

    #[test]
    fn nothing_is_sent_during_the_wait_of_refused_item_names() {
        let site = FakeSite::default();
        let refusal = Refusal::default();
        let until = Instant::now() + Duration::from_hours(2);
        refusal.extend(until);
        let mut client = TradeClient::with_transport(site.clone(), refusal);
        let league = League::new("Standard");

        let items = client.search_items(Language::English, &league, &ring());
        let exchange = client.search_exchange(Language::English, &league, "divine");
        let fetch = client.fetch(Language::English, "Q1", &["a".to_owned()]);

        assert!(matches!(items, Err(TradeError::Wait(wait)) if wait == until));
        assert!(matches!(exchange, Err(TradeError::Wait(wait)) if wait == until));
        assert!(matches!(fetch, Err(TradeError::Wait(wait)) if wait == until));
        assert_eq!(site.requests(), []);
    }

    #[test]
    fn a_refusal_whose_body_cant_be_read_waits_all_the_same() {
        let site = FakeSite::default();
        let mut headers = limits("trade-search-request-limit", "5:10:60", "6:10:60");
        headers.retry_after = Some("120".to_owned());
        site.answers.lock().unwrap().push_back(Answer {
            status: 429,
            limits: headers,
            body: Err(ureq::Error::ConnectionFailed),
        });
        let mut client = TradeClient::with_transport(site, Refusal::default());

        let search = client.search_items(Language::English, &League::new("Standard"), &ring());

        let Err(TradeError::Refused { until, .. }) = search else {
            panic!("refused: {search:?}");
        };
        assert!(until > Instant::now() + Duration::from_secs(115));
    }

    #[test]
    fn the_limits_of_an_answer_whose_body_cant_be_read_are_kept() {
        let site = FakeSite::default();
        site.answers.lock().unwrap().push_back(Answer {
            status: 200,
            limits: limits("trade-fetch-request-limit", "1:4:10", "1:4:0"),
            body: Err(ureq::Error::ConnectionFailed),
        });
        let mut client = TradeClient::with_transport(site, Refusal::default());

        let fetch = client.fetch(Language::English, "Q1", &["a".to_owned()]);

        assert!(matches!(
            fetch,
            Err(TradeError::Http(HttpError::Request(_)))
        ));
        assert!(client.next_fetch(Instant::now()) > Instant::now());
    }

    #[test]
    fn an_error_status_or_an_unexpected_answer_is_an_error() {
        let site = FakeSite::default();
        site.answer(400, LimitHeaders::default(), r#"{"error": {"code": 2}}"#);
        site.answer(200, LimitHeaders::default(), r#"{"result": "?"}"#);
        let mut client = TradeClient::with_transport(site, Refusal::default());
        let ids = ["a".to_owned()];

        assert!(matches!(
            client.fetch(Language::English, "Q1", &ids),
            Err(TradeError::Http(HttpError::Request(
                ureq::Error::StatusCode(400)
            )))
        ));
        assert!(matches!(
            client.fetch(Language::English, "Q1", &ids),
            Err(TradeError::Http(HttpError::Json(_)))
        ));
    }

    /// Told once: every answer would say the same.
    #[test]
    fn a_success_without_limits_is_noticed_and_one_with_limits_isnt() {
        let site = FakeSite::default();
        site.answer(
            200,
            limits("trade-fetch-request-limit", "12:4:10", "1:4:0"),
            r#"{"result": []}"#,
        );
        site.answer(400, LimitHeaders::default(), "{}");
        site.answer(200, LimitHeaders::default(), r#"{"result": []}"#);
        let mut client = TradeClient::with_transport(site, Refusal::default());
        let ids = ["a".to_owned()];

        client.fetch(Language::English, "Q1", &ids).expect("offers");
        assert!(!client.told_no_limits, "its limits were told");
        let _ = client.fetch(Language::English, "Q1", &ids);
        assert!(!client.told_no_limits, "an error status isn't a success");
        client.fetch(Language::English, "Q1", &ids).expect("offers");
        assert!(client.told_no_limits);
    }
}
