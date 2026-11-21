//! The identifiers of the messages of `locales/`: the messages without arguments, the variants
//! of [`Message`], and those with arguments, each read by its own method of
//! [`Translator`](super::Translator).

/// Declares [`Message`], its list for the tests and the identifier of each message in the
/// `.ftl` files, from one table, so that a message without arguments is declared once.
macro_rules! messages {
    ($($(#[$attribute:meta])* $variant:ident => $id:literal,)*) => {
        /// A message of the interface without arguments.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Message {
            $($(#[$attribute])* $variant,)*
        }

        impl Message {
            /// Every message, for the tests.
            #[cfg(test)]
            pub(super) const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// Its identifier in the `.ftl` files.
            pub(super) fn id(self) -> &'static str {
                match self {
                    $(Self::$variant => $id,)*
                }
            }
        }
    };
}

messages! {
    LeagueModeSoftcore => "league-mode-softcore",
    LeagueModeHardcore => "league-mode-hardcore",
    LeagueLoading => "league-loading",
    NewLeagueSwitch => "new-league-switch",
    UpdateDownload => "update-download",
    HomeTooltip => "home-tooltip",
    SettingsTooltip => "settings-tooltip",
    BugReportTooltip => "bug-report-tooltip",
    MinimizeTooltip => "minimize-tooltip",
    CloseTooltip => "close-tooltip",
    StatusPricesUnavailable => "status-prices-unavailable",
    StatusPricesDownloading => "status-prices-downloading",
    StatusTradeIdle => "status-trade-idle",
    StatusKeyboard => "status-keyboard",
    StatusKeyboardSingleScreen => "status-keyboard-single-screen",
    EmptyCopy => "empty-copy",
    EmptyPrice => "empty-price",
    EmptyDisplayMode => "empty-display-mode",
    ClipboardUnreadable => "clipboard-unreadable",
    PricesUnavailable => "prices-unavailable",
    PricesUnreadableRetrying => "prices-unreadable-retrying",
    ItemUnreadable => "item-unreadable",
    ItemUnreadableHint => "item-unreadable-hint",
    SettingsTitle => "settings-title",
    SettingsLanguage => "settings-language",
    LanguageSystem => "language-system",
    LanguageEnglish => "language-english",
    LanguageFrench => "language-french",
    SettingsAlwaysOnTop => "settings-always-on-top",
    SettingsAlwaysOnTopHelp => "settings-always-on-top-help",
    SettingsCloseWithGame => "settings-close-with-game",
    SettingsCloseWithGameHelp => "settings-close-with-game-help",
    SettingsGame => "settings-game",
    SettingsSearch => "settings-search",
    SettingsPseudoTotals => "settings-pseudo-totals",
    SettingsPseudoTotalsHelp => "settings-pseudo-totals-help",
    SettingsHotkey => "settings-hotkey",
    SettingsHotkeyPriceCheck => "settings-hotkey-price-check",
    SettingsHotkeyHelp => "settings-hotkey-help",
    SettingsHotkeyChange => "settings-hotkey-change",
    SettingsHotkeyDefault => "settings-hotkey-default",
    KeyShift => "key-shift",
    HotkeyTakenChoose => "hotkey-taken-choose",
    DonationLink => "donation-link",
    Disclaimer => "disclaimer",
    BugReportTitle => "bug-report-title",
    BugReportIntro => "bug-report-intro",
    BugReportSent => "bug-report-sent",
    BugReportLog => "bug-report-log",
    BugReportLogNote => "bug-report-log-note",
    BugReportThisSession => "bug-report-this-session",
    BugReportPreviousSession => "bug-report-previous-session",
    BugReportNoLog => "bug-report-no-log",
    BugReportJoinUnreadable => "bug-report-join-unreadable",
    BugReportOpen => "bug-report-open",
    BugReportCopy => "bug-report-copy",
    BugReportCopied => "bug-report-copied",
    BugReportCopyFailed => "bug-report-copy-failed",
    BugReportAccount => "bug-report-account",
    CheckOnTrade => "check-on-trade",
    TagCorrupted => "tag-corrupted",
    TagTwiceCorrupted => "tag-twice-corrupted",
    TagSanctified => "tag-sanctified",
    TagFractured => "tag-fractured",
    TagMirrored => "tag-mirrored",
    TagUnidentified => "tag-unidentified",
    /// The tooltip of a state tag, gray or lit.
    StateCorruptedOrNot => "state-corrupted-or-not",
    StateCorruptedOnly => "state-corrupted-only",
    StateTwiceCorruptedOnly => "state-twice-corrupted-only",
    StateSanctifiedOrNot => "state-sanctified-or-not",
    StateSanctifiedOnly => "state-sanctified-only",
    StateFracturedOrNot => "state-fractured-or-not",
    StateFracturedOnly => "state-fractured-only",
    StateMirroredOrNot => "state-mirrored-or-not",
    StateMirroredOnly => "state-mirrored-only",
    PriceLabel => "price-label",
    EstimatedPriceLabel => "estimated-price-label",
    PossibleUniques => "possible-uniques",
    IdentifyToPrice => "identify-to-price",
    NoOffers => "no-offers",
    SearchToEstimate => "search-to-estimate",
    CheckToEstimate => "check-to-estimate",
    NoOfferCountedToEstimate => "no-offer-counted-to-estimate",
    NoMarketPriceRare => "no-market-price-rare",
    NoMarketPriceSkillGem => "no-market-price-skill-gem",
    NoMarketPriceMagic => "no-market-price-magic",
    NoMarketPriceNormal => "no-market-price-normal",
    FrenchNamesDownloading => "french-names-downloading",
    PricesDownloadingNoLeague => "prices-downloading-no-league",
    HideStats => "hide-stats",
    GroupImplicit => "group-implicit",
    GroupPrefixes => "group-prefixes",
    GroupSuffixes => "group-suffixes",
    GroupExplicit => "group-explicit",
    DpsPhysical => "dps-physical",
    DpsElemental => "dps-elemental",
    DpsChaos => "dps-chaos",
    DpsTooltip => "dps-tooltip",
    DefenceArmour => "defence-armour",
    DefenceEvasion => "defence-evasion",
    DefenceEnergyShield => "defence-energy-shield",
    DefenceBlock => "defence-block",
    DefenceRunicWard => "defence-runic-ward",
    /// The properties of a waystone in the item block, two of them written short, their name
    /// of the game in their tooltip (`…Full`).
    WaystoneRevives => "waystone-revives",
    WaystoneRevivesFull => "waystone-revives-full",
    WaystoneItemRarity => "waystone-item-rarity",
    WaystonePackSize => "waystone-pack-size",
    WaystoneMonsterRarity => "waystone-monster-rarity",
    WaystoneMonsterEffectiveness => "waystone-monster-effectiveness",
    WaystoneDropChance => "waystone-drop-chance",
    WaystoneDropChanceFull => "waystone-drop-chance-full",
    GroupEnchant => "group-enchant",
    GroupRunes => "group-runes",
    GroupUnique => "group-unique",
    GroupItem => "group-item",
    GroupWeapon => "group-weapon",
    GroupWaystone => "group-waystone",
    GroupGem => "group-gem",
    GroupPseudo => "group-pseudo",
    ShowFilters => "show-filters",
    HideFilters => "hide-filters",
    FilterRange => "filter-range",
    FilterLowerBetter => "filter-lower-better",
    FilterUnsearchable => "filter-unsearchable",
    FilterUnsearchableHint => "filter-unsearchable-hint",
    /// The tooltip of the filter of the quality of a catalyst.
    FilterAnyQualityKind => "filter-any-quality-kind",
    OriginDesecrated => "origin-desecrated",
    OriginFractured => "origin-fractured",
    OriginCrafted => "origin-crafted",
    OriginVaal => "origin-vaal",
    /// The currency that adds a modifier whose tier the game doesn't write, in place of its
    /// origin.
    OriginEssence => "origin-essence",
    OriginAlloy => "origin-alloy",
    /// The line of a modifier not revealed yet, and the note under the stats of its item.
    UnrevealedPrefix => "unrevealed-prefix",
    UnrevealedSuffix => "unrevealed-suffix",
    UnrevealedNote => "unrevealed-note",
    /// The note under the stats of a rare or magic item copied from the in-game trade, and its
    /// second sentence once a line is in doubt.
    TradeCopyNote => "trade-copy-note",
    UncertainTierNote => "uncertain-tier-note",
    /// The note under the stats of a sanctified item copied from the in-game trade, in place
    /// of the one above.
    SanctifiedNote => "sanctified-note",
    /// The note under the stats of a unique copied from the in-game trade.
    UniqueRollsNote => "unique-rolls-note",
    /// After the range of a line of a Time-Lost jewel, the passive skills in its radius it is
    /// granted to.
    RadiusSmall => "radius-small",
    RadiusNotable => "radius-notable",
    /// In place of the tier of a line copied from the in-game trade that has none, why: a line
    /// in doubt, or a value no modifier holds, with its tooltip.
    TierUncertain => "tier-uncertain",
    TierNotFound => "tier-not-found",
    TierNotFoundWhy => "tier-not-found-why",
    /// After the range of a tier found from the values, the affix it is the tier of.
    AffixPrefix => "affix-prefix",
    AffixSuffix => "affix-suffix",
    /// The tooltip of the suffix of a line that adds up a prefix and a suffix.
    AddedUpWithPrefix => "added-up-with-prefix",
    /// After the range of the part of a hybrid in a line that adds up two modifiers, and of
    /// its other lines; with its affix when the other part is of the other affix.
    Hybrid => "hybrid",
    HybridPrefix => "hybrid-prefix",
    HybridSuffix => "hybrid-suffix",
    OffersLabel => "offers-label",
    OffersSearchHint => "offers-search-hint",
    OffersSearchTooltip => "offers-search-tooltip",
    OffersSearching => "offers-searching",
    OffersColumnPrice => "offers-column-price",
    OffersColumnPerUnit => "offers-column-per-unit",
    OffersColumnStock => "offers-column-stock",
    OffersColumnListed => "offers-column-listed",
    OffersAgainTooltip => "offers-again-tooltip",
    OffersBusyTooltip => "offers-busy-tooltip",
    OffersNone => "offers-none",
    OffersNoneHint => "offers-none-hint",
    OffersUnreachable => "offers-unreachable",
    /// The trade site turned the search down with an error.
    OffersRefused => "offers-refused",
    /// The answer of the trade site can't be read.
    OffersUnreadable => "offers-unreadable",
    OffersIdentify => "offers-identify",
    OffersMoreTooltip => "offers-more-tooltip",
    OffersSuspicious => "offers-suspicious",
    /// Under the name of an offer of another kind of quality, while the quality is searched.
    OffersOtherQuality => "offers-other-quality",
    OfferNotCounted => "offer-not-counted",
    SellerOnline => "seller-online",
    SellerAway => "seller-away",
    SellerOffline => "seller-offline",
    CopyWhisper => "copy-whisper",
    WhisperCopied => "whisper-copied",
    WhisperCopyFailed => "whisper-copy-failed",
}

/// The identifiers of the messages with arguments, each read by its own method.
pub(super) const STATUS_BAR: &str = "status-bar";
pub(super) const LEAGUE_TOOLTIP: &str = "league-tooltip";
pub(super) const STATUS_MADE_BY: &str = "status-made-by";
pub(super) const DONATION_INTRO: &str = "donation-intro";
pub(super) const NEW_LEAGUE: &str = "new-league";
pub(super) const UPDATE_AVAILABLE: &str = "update-available";
pub(super) const STATUS_UPDATE: &str = "status-update";
pub(super) const HOTKEY_TAKEN: &str = "hotkey-taken";
pub(super) const HOTKEY_TAKEN_SETTINGS: &str = "hotkey-taken-settings";
pub(super) const BUG_REPORT_JOIN_ITEM: &str = "bug-report-join-item";
pub(super) const BUG_REPORT_LOG_SESSION: &str = "bug-report-log-session";
pub(super) const WINDOW_ERROR_DISPLAY: &str = "window-error-display";
pub(super) const WINDOW_ERROR_DISPLAY_FIX: &str = "window-error-display-fix";
pub(super) const WINDOW_ERROR_OPENGL: &str = "window-error-opengl";
pub(super) const WINDOW_ERROR_OPENGL_FIX: &str = "window-error-opengl-fix";
pub(super) const WINDOW_ERROR_OTHER: &str = "window-error-other";
pub(super) const WINDOW_ERROR_OTHER_FIX: &str = "window-error-other-fix";
pub(super) const WINDOW_ERROR_LOG: &str = "window-error-log";
pub(super) const WINDOW_ERROR_DETAILS: &str = "window-error-details";
pub(super) const ALREADY_OPEN: &str = "already-open";
pub(super) const ALREADY_OPEN_WHERE: &str = "already-open-where";
pub(super) const STATUS_PRICES: &str = "status-prices";
pub(super) const STATUS_PRICES_OFFLINE: &str = "status-prices-offline";
pub(super) const STATUS_PRICES_UNREADABLE: &str = "status-prices-unreadable";
pub(super) const PRICES_UNREACHABLE: &str = "prices-unreachable";
pub(super) const PRICES_UNREADABLE: &str = "prices-unreadable";
pub(super) const AGE_NOW: &str = "age-now";
pub(super) const AGE_MINUTES: &str = "age-minutes";
pub(super) const AGE_HOURS: &str = "age-hours";
pub(super) const AGE_DAYS: &str = "age-days";
pub(super) const WAIT_SECONDS: &str = "wait-seconds";
pub(super) const WAIT_MINUTES: &str = "wait-minutes";
pub(super) const DOWNLOAD_TITLE: &str = "download-title";
pub(super) const TAG_QUALITY: &str = "tag-quality";
pub(super) const TAG_SOCKETS: &str = "tag-sockets";
pub(super) const TAG_ITEM_LEVEL: &str = "tag-item-level";
pub(super) const TAG_STACK: &str = "tag-stack";
pub(super) const GEM_LEVEL: &str = "gem-level";
pub(super) const GRANTED_SKILL: &str = "granted-skill";
pub(super) const REQUIRES: &str = "requires";
pub(super) const REQUIREMENT_LEVEL: &str = "requirement-level";
pub(super) const REQUIREMENT_STRENGTH: &str = "requirement-strength";
pub(super) const REQUIREMENT_DEXTERITY: &str = "requirement-dexterity";
pub(super) const REQUIREMENT_INTELLIGENCE: &str = "requirement-intelligence";
pub(super) const OFFERS: &str = "offers";
pub(super) const STACK_OF: &str = "stack-of";
pub(super) const UNIT_PRICE: &str = "unit-price";
pub(super) const ORB_RATE_IN_EXALTED: &str = "orb-rate-in-exalted";
pub(super) const ORB_RATE_FOR_DIVINE: &str = "orb-rate-for-divine";
pub(super) const TREND_RISE: &str = "trend-rise";
pub(super) const TREND_FALL: &str = "trend-fall";
pub(super) const TREND_STABLE: &str = "trend-stable";
pub(super) const LOW_RELIABILITY_OFFERS: &str = "low-reliability-offers";
pub(super) const LOW_RELIABILITY_TRADES: &str = "low-reliability-trades";
pub(super) const SIMILAR: &str = "similar";
pub(super) const LOW_RELIABILITY_SIMILAR: &str = "low-reliability-similar";
pub(super) const ESTIMATE_NOTE: &str = "estimate-note";
pub(super) const ESTIMATE_NOTE_RANGE: &str = "estimate-note-range";

/// The two messages of the note of an estimate: `plain`, and `of_kind` with the kind of the
/// quality of a catalyst searched.
#[derive(Clone, Copy)]
pub(super) struct EstimateNote {
    pub(super) plain: &'static str,
    pub(super) of_kind: &'static str,
}

pub(super) const MEDIAN: EstimateNote = EstimateNote {
    plain: "estimate-median",
    of_kind: "estimate-median-of",
};
pub(super) const MEDIAN_CHEAPEST: EstimateNote = EstimateNote {
    plain: "estimate-median-cheapest",
    of_kind: "estimate-median-cheapest-of",
};
pub(super) const QUALITY_OF_KIND: &str = "quality-of-kind";
pub(super) const OFFER_QUALITY_KIND: &str = "offer-quality-kind";
pub(super) const PRICES_DOWNLOADING: &str = "prices-downloading";
pub(super) const NO_PRICE_FOUND: &str = "no-price-found";
pub(super) const SHOW_STATS: &str = "show-stats";
pub(super) const AFFIX_FILL: &str = "affix-fill";
pub(super) const AFFIX_FILL_OPEN: &str = "affix-fill-open";
pub(super) const AFFIX_FILL_UP_TO: &str = "affix-fill-up-to";
pub(super) const LORD_PREFIX: &str = "lord-prefix";
pub(super) const LORD_SUFFIX: &str = "lord-suffix";
pub(super) const ACTIVE_FILTERS: &str = "active-filters";
pub(super) const FILTER_ROLLS: &str = "filter-rolls";
pub(super) const HYBRID_WITH: &str = "hybrid-with";
pub(super) const HYBRID_LINES_AND: &str = "hybrid-lines-and";
pub(super) const HYBRID_PART: &str = "hybrid-part";
pub(super) const FILTER_BASE_TYPE: &str = "filter-base-type";
pub(super) const FILTER_ITEM_CLASS: &str = "filter-item-class";
pub(super) const FILTER_ITEM_LEVEL: &str = "filter-item-level";
pub(super) const FILTER_SOCKETS: &str = "filter-sockets";
pub(super) const FILTER_GEM_LEVEL: &str = "filter-gem-level";
pub(super) const FILTER_QUALITY: &str = "filter-quality";
pub(super) const FILTER_CATALYST_QUALITY: &str = "filter-catalyst-quality";
pub(super) const FILTER_REQUIRED_LEVEL: &str = "filter-required-level";
pub(super) const FILTER_PHYSICAL_DPS: &str = "filter-physical-dps";
pub(super) const FILTER_ELEMENTAL_DPS: &str = "filter-elemental-dps";
pub(super) const FILTER_WAYSTONE_TIER: &str = "filter-waystone-tier";
pub(super) const FILTER_WAYSTONE_REVIVES: &str = "filter-waystone-revives";
pub(super) const FILTER_WAYSTONE_ITEM_RARITY: &str = "filter-waystone-item-rarity";
pub(super) const FILTER_WAYSTONE_PACK_SIZE: &str = "filter-waystone-pack-size";
pub(super) const FILTER_WAYSTONE_MONSTER_RARITY: &str = "filter-waystone-monster-rarity";
pub(super) const FILTER_WAYSTONE_MONSTER_EFFECTIVENESS: &str =
    "filter-waystone-monster-effectiveness";
pub(super) const FILTER_WAYSTONE_DROP_CHANCE: &str = "filter-waystone-drop-chance";
pub(super) const PSEUDO_ELEMENTAL_RESISTANCE: &str = "pseudo-elemental-resistance";
pub(super) const PSEUDO_CHAOS_RESISTANCE: &str = "pseudo-chaos-resistance";
pub(super) const PSEUDO_LIFE: &str = "pseudo-life";
pub(super) const PSEUDO_MANA: &str = "pseudo-mana";
pub(super) const PSEUDO_STRENGTH: &str = "pseudo-strength";
pub(super) const PSEUDO_DEXTERITY: &str = "pseudo-dexterity";
pub(super) const PSEUDO_INTELLIGENCE: &str = "pseudo-intelligence";
pub(super) const PSEUDO_ALL_ATTRIBUTES: &str = "pseudo-all-attributes";
pub(super) const PSEUDO_UNREVEALED_MODIFIERS: &str = "pseudo-unrevealed-modifiers";
pub(super) const STATUS_TRADE: &str = "status-trade";
pub(super) const STATUS_TRADE_WAIT: &str = "status-trade-wait";
pub(super) const OFFERS_SEARCHED: &str = "offers-searched";
pub(super) const OFFERS_COUNT: &str = "offers-count";
pub(super) const OFFERS_NEXT_SEARCH_TOOLTIP: &str = "offers-next-search-tooltip";
pub(super) const OFFERS_LIMIT: &str = "offers-limit";
pub(super) const OFFERS_FORBIDDEN: &str = "offers-forbidden";
pub(super) const OFFERS_FORBIDDEN_HINT: &str = "offers-forbidden-hint";
pub(super) const OFFERS_REFUSED_HINT: &str = "offers-refused-hint";
pub(super) const OFFERS_UNREADABLE_HINT: &str = "offers-unreadable-hint";
/// The words of the two hints above that open the bug report, inside a sentence and first.
pub(super) const OFFERS_REPORT: &str = "offers-report";
pub(super) const OFFERS_REPORT_START: &str = "offers-report-start";
/// The sentences under the bar of the download of the prices, one drawn at random per
/// download.
pub(super) const DOWNLOAD_QUOTES: [&str; 10] = [
    "download-quote-1",
    "download-quote-2",
    "download-quote-3",
    "download-quote-4",
    "download-quote-5",
    "download-quote-6",
    "download-quote-7",
    "download-quote-8",
    "download-quote-9",
    "download-quote-10",
];
/// The number of sentences of the download of the prices.
pub const DOWNLOAD_QUOTE_COUNT: usize = DOWNLOAD_QUOTES.len();
