# Interface texts in English, the reference language.
# Every message must also exist in fr.ftl.

## Title bar

# The mode of the league, said only by the color of its block and in its tooltip.
league-mode-softcore = Softcore
league-mode-hardcore = Hardcore
# Tooltip of the league block: its mode, then what a click does.
league-tooltip = { $mode } · Change the league
# In place of the league, before the leagues arrive at the first launch.
league-loading = Loading…
# Bubble under the league block, when a new league has started: its name, then a pill.
new-league = { $league } has started.
new-league-switch = Switch
# Bubble under the league block, when a newer version of the application is out: its version, then a pill
# that opens the page of the release.
update-available = { $app } { $version } is available.
update-download = Download
# Tooltip of the emblem, which goes back to the item.
home-tooltip = Home
settings-tooltip = Settings
# Tooltip of the bug, left of the gear, which opens the bug report.
bug-report-tooltip = Report a bug
minimize-tooltip = Minimize
close-tooltip = Close

## Status bar

status-bar = { $prices } · { $trade }
# No price of the league, and the price database can't be reached or read.
status-prices-unavailable = poe2scout: no prices yet
# During the first download of the prices of a league.
status-prices-downloading = poe2scout: downloading prices…
# The age of the prices, e.g. "12min ago".
status-prices = poe2scout { $age }
status-prices-offline = poe2scout { $age } (offline)
# The price database answers something the application can't read.
status-prices-unreadable = poe2scout { $age } (unreadable)
status-trade-idle = GGG trade: no search yet
# The age of the last search of the trade site, e.g. "2min ago".
status-trade = GGG trade { $age }
# During a long wait before the next search, e.g. "4min".
status-trade-wait = GGG trade: next search in { $wait }
# While the settings are open, after the version (and a newer one, if out): the author.
status-made-by = Made by { $author }
# While the settings are open and a newer version is out, after the running version: a link to its page.
status-update = { $version } available
# In the window over the game, while a min or max field is edited: the keys no longer reach the game.
status-keyboard = Keys go to this window · Enter or Esc: back to the game
# In the window over the game, the whole time it is open on a single screen, where it becomes the active window so that the game lets the mouse go: the keys no longer reach the game.
status-keyboard-single-screen = Keys go here · Esc or click outside: back to the game

## Window before the first item

# Followed by the keys "Ctrl + D" ("Ctrl + C" on macOS, without hotkey, or when the hotkey doesn't copy).
empty-copy = Copy an item in the game with
empty-price = The price shows up here.
# In faint text under the invitation: the game must not be in exclusive fullscreen.
empty-display-mode = Play in windowed fullscreen: in fullscreen, the game lets no application copy the item or show its price over it.
# Under the invitation, in the color of the warnings, when the key of the hotkey is taken by another application: $hotkey its keys (Ctrl+D), in bold. The next sentence on a line of its own.
hotkey-taken = { $hotkey } is taken by another application.
hotkey-taken-choose = Choose another key in the settings.

## Copied item that can't be read

item-unreadable = This item couldn't be read.
item-unreadable-hint = Copy it again, or copy another item.

## Download of the prices

# In place of the invitation to copy an item, while no price of the league is known.
download-title = Downloading the prices of { $league }
# Under the bar, one drawn at random per download.
download-quote-1 = Counting the Exalted Orbs one by one…
download-quote-2 = Haggling with the merchants of Wraeclast…
download-quote-3 = Polishing the Divine Orbs…
download-quote-4 = Checking that no one sells a Mirror for 1 Chaos…
download-quote-5 = Ignoring the "still available?" whispers…
download-quote-6 = Sorting the stash tabs. Again…
download-quote-7 = Converting Chaos into Exalted, the math checks out…
download-quote-8 = Looking for the seller who went AFK in their hideout…
download-quote-9 = One request per second: poe2scout deserves respect…
download-quote-10 = Spotting the bait offers…

## Ages, rounded down to their largest unit

age-now = just now
age-minutes = { $minutes }min ago
age-hours = { $hours }h ago
age-days = { $days }d ago

## Waits, rounded up

# E.g. "8s", "4min".
wait-seconds = { $seconds }s
wait-minutes = { $minutes }min

## Warnings

clipboard-unreadable = The clipboard can't be read.
# The age of the prices, e.g. "3h ago".
prices-unreachable = The price database can't be reached. Last updated { $age }.
prices-unavailable = Prices unavailable, retrying…
# The price database answers something the application can't read: it may have changed.
prices-unreadable = The answer of the price database can't be read. Last updated { $age }.
prices-unreadable-retrying = Prices can't be read, retrying…

## Item block

# Opens the search for the item on the trade site, in the browser.
check-on-trade = Check on trade
# The tags after the item class.
tag-corrupted = Corrupted
tag-twice-corrupted = Twice Corrupted
tag-sanctified = Sanctified
tag-fractured = Fractured
tag-mirrored = Mirrored
tag-unidentified = Unidentified
# The tooltip of a state tag, a filter of the search: gray, the offers with or without that state;
# lit by a click, the offers of that state only. Twice Corrupted gray reads "Corrupted or not".
state-corrupted-or-not = Corrupted or not
state-corrupted-only = Corrupted offers only
state-twice-corrupted-only = Twice corrupted offers only
state-sanctified-or-not = Sanctified or not
state-sanctified-only = Sanctified offers only
state-fractured-or-not = Fractured or not
state-fractured-only = Fractured offers only
state-mirrored-or-not = Mirrored or not
state-mirrored-only = Mirrored offers only
tag-quality = Q { $value }%
tag-sockets = { $value ->
    [one] { $value } socket
   *[other] { $value } sockets
}
tag-item-level = ilvl { $value }
tag-stack = Stack { $size }/{ $max }
# The level of a gem, after its item class under its name: "Skill Gems · Level 2"; a no-break space keeps the word with its number.
gem-level = Level { $value }
# The main values of the base, each followed by its number in bold: the DPS of a weapon,
# the defences of an armour piece or a shield.
dps-physical = Physical DPS
dps-elemental = Elemental DPS
dps-chaos = Chaos DPS
# The tooltip of each DPS: the value of the game, not the DPS of the base.
dps-tooltip = With its quality, modifiers and runes
defence-armour = Armour
defence-evasion = Evasion
defence-energy-shield = Energy Shield
defence-block = Block
defence-runic-ward = Runic Ward
# The properties of a waystone, as the game names them, each followed by its value in bold;
# the -full names, in the tooltip of a name written short, are those of the game.
waystone-revives = Revives
waystone-revives-full = Revives Available
waystone-item-rarity = Item Rarity
waystone-pack-size = Pack Size
waystone-monster-rarity = Monster Rarity
waystone-monster-effectiveness = Monster Effectiveness
waystone-drop-chance = Waystone Drop Chance
waystone-drop-chance-full = Waystone Drop Chance
# The summary of an item that grants skills, e.g. "Level 18 Sigil of Power", joined by " · ".
granted-skill = Grants Skill: { $skill }
# The requirements line, its parts joined by " · ": "Requires Level 65 · 145 Dex".
requires = Requires { $parts }
requirement-level = Level { $value }
requirement-strength = { $value } Str
requirement-dexterity = { $value } Dex
requirement-intelligence = { $value } Int

## Stats of the item

# The pill under the item block, and once unfolded.
show-stats = { $count ->
    [one] Show { $count } stat
   *[other] Show { $count } stats
}
hide-stats = Hide stats
# The small titles of the stats.
group-enchant = Enchant
group-runes = Runes
group-implicit = Implicit
group-prefixes = Prefixes
group-suffixes = Suffixes
group-explicit = Explicit
# The modifiers of a unique, no prefix nor suffix.
group-unique = Unique
# The filters of the base type, the item level, the sockets, the quality and the required level of a rare, magic or normal item.
group-item = Item
group-weapon = Weapon
group-waystone = Waystone
# The filters of the level, the quality and the sockets of a skill gem.
group-gem = Gem
# The totals the trade site adds up from several modifiers.
group-pseudo = Pseudo
# Beside Prefixes and Suffixes: how many the item has, of the most it can have.
affix-fill = { $filled } of { $max }
affix-fill-open = { $filled } of { $max } · { $open } open
# The same while a modifier of the item has no affix known (a line in doubt, a prefix or a suffix): it takes a place the count doesn't show, so the open places are a most.
affix-fill-up-to = { $filled } of { $max } · up to { $open } open
# The pill of an item without modifiers, which has filters only, and once unfolded.
show-filters = Show filters
hide-filters = Hide filters
# Beside the pill: the filters checked for the search.
active-filters = { $count ->
    [one] { $count } filter
   *[other] { $count } filters
}
# On the right of the first small title, above the range fields of the filters.
filter-range = min – max
# The tooltip of a range field: the rolls it accepts, then for a line better lower, in gray.
filter-rolls = Rolls { $low } to { $high }
filter-lower-better = Lower is better
# The filters of the properties of an item, with their value.
filter-base-type = Base: { $base }
# Under the base type unchecked: the item class of the copied item, searched in its place.
filter-item-class = Class searched: { $class }
filter-item-level = Item level: { $value }
filter-sockets = Sockets: { $value }
# The level of a gem.
filter-gem-level = Level: { $value }
filter-quality = Quality: { $value }%
# The quality of a catalyst on a ring, an amulet or a jewel, short with its kind; a kind the application doesn't know keeps the words of the game ($words).
filter-catalyst-quality = { $kind ->
        [life] Life quality
        [mana] Mana quality
        [defence] Defence quality
        [physical] Physical quality
        [fire] Fire quality
        [cold] Cold quality
        [lightning] Lightning quality
        [chaos] Chaos quality
        [attack] Attack quality
        [caster] Caster quality
        [speed] Speed quality
        [attribute] Attribute quality
        [minion] Minion quality
       *[other] Quality ({ $words })
    }: { $value }%
filter-required-level = Required level: { $value }
filter-physical-dps = Physical DPS: { $value }
filter-elemental-dps = Elemental DPS: { $value }
# The filters of the tier and the properties of a waystone; $value has its sign for a percentage ("+11").
filter-waystone-tier = Tier: { $value }
filter-waystone-revives = Revives available: { $value }
filter-waystone-item-rarity = Item rarity: { $value }%
filter-waystone-pack-size = Pack size: { $value }%
filter-waystone-monster-rarity = Monster rarity: { $value }%
filter-waystone-monster-effectiveness = Monster effectiveness: { $value }%
filter-waystone-drop-chance = Waystone drop chance: { $value }%
# The filters of the pseudo totals, with their value.
pseudo-elemental-resistance = +{ $value }% total Elemental Resistance
pseudo-chaos-resistance = +{ $value }% total to Chaos Resistance
pseudo-life = +{ $value } total maximum Life
pseudo-mana = +{ $value } total maximum Mana
pseudo-strength = +{ $value } total to Strength
pseudo-dexterity = +{ $value } total to Dexterity
pseudo-intelligence = +{ $value } total to Intelligence
pseudo-all-attributes = +{ $value } total to all Attributes
# In the Item group: the number of unrevealed modifiers searched.
pseudo-unrevealed-modifiers = Unrevealed modifiers: { $value }
# The tooltip of a line the trade site can't search (a modifier added by a patch), on two lines.
filter-unsearchable = The trade site can't search this line.
filter-unsearchable-hint = The offers may not have it.
# The tooltip of the filter of the quality of a catalyst: the trade site doesn't tell the kinds apart.
filter-any-quality-kind = The trade site searches any kind of quality: the offers of another kind aren't counted in the estimate.
# After the range of the tier of a modifier: how it was added to the item, in its color.
origin-desecrated = Desecrated
origin-fractured = Fractured
origin-crafted = Crafted
# The Vaal modifier of a corrupted unique: the word of the game, the same in every language.
origin-vaal = Vaal
# In place of the origin: the currency that added a modifier whose tier the game doesn't write.
origin-essence = Essence
origin-alloy = Alloy
# A desecrated modifier not revealed yet, in its group; and the note under the stats of its item.
unrevealed-prefix = Unrevealed desecrated prefix
unrevealed-suffix = Unrevealed desecrated suffix
unrevealed-note = An unrevealed modifier can change the item: its other stats start unchecked.
# Under the stats of a rare or magic item copied from the trade of the game, without the affixes and tiers of the advanced copy.
trade-copy-note = The tiers are found from the values.
# Its second sentence, on a line of its own, once a line is in doubt ("Uncertain tier").
uncertain-tier-note = A tier is uncertain when several modifiers fit its value.
# In place of the tier of a line copied from the trade of the game, small and gray: several modifiers fit its value and nothing tells which one rolled.
tier-uncertain = Uncertain tier
# The same for a value no modifier of the game data holds (an item copied before a patch), and its tooltip.
tier-not-found = No tier found
tier-not-found-why = No known modifier holds this value.
# Under the stats of a sanctified item copied from the trade of the game, in place of the note of the tiers: the sanctification moves each value, maybe out of its tier, so no tier is searched.
sanctified-note = The values of a sanctified item can leave their tier: none is found.
# Under the stats of a unique copied from the trade of the game, without the ranges of the advanced copy: its rolls come from the price database.
unique-rolls-note = The rolls are those of poe2scout.
# After the range of a line of a Time-Lost jewel, on a small gray block: the passive skills in its radius that receive the line. The line is shown without the words of the game that name them.
radius-small = Small
radius-notable = Notable
# After the range of a tier found from the values, when the line can be a prefix or a suffix: the affix of that tier, in gray.
affix-prefix = Prefix
affix-suffix = Suffix
# The same for a modifier of a lord of the Abyss (Amanamu, Kurgal, Ulaman), of one tier, so that its row without tier number reads as "the only one": "Amanamu Prefix".
lord-prefix = { $lord } Prefix
lord-suffix = { $lord } Suffix
# The tooltip of "Suffix" under a line that adds up a prefix and a suffix of the same stat, the prefix written above it.
added-up-with-prefix = Added up with the prefix on this line
# After the range of the part of a hybrid in a line that adds up two modifiers of the same stat, and under its other lines, in gray; with its affix when the other part is of the other affix (a hybrid prefix added up with a pure suffix).
hybrid = Hybrid
hybrid-prefix = Hybrid prefix
hybrid-suffix = Hybrid suffix
# The tooltip of "Hybrid" on the line that adds up the part of a hybrid: the other lines of the hybrid, e.g. "With +195 to Accuracy Rating".
hybrid-with = With { $lines }
# Two other lines of a hybrid in the tooltip above: "+38 to Armour and +35 to Evasion Rating".
hybrid-lines-and = { $first } and { $second }
# The tooltip of "Hybrid" under the other lines of the hybrid: the range of its part on the line that adds it up, then that line, e.g. "With 75–79 of 258% increased Physical Damage".
hybrid-part = With { $ranges } of { $line }

## Price block

price-label = Price
# The price of a rare, magic or normal item, from the offers of similar items.
estimated-price-label = Estimated price
# On the right of the price title.
offers = { $count ->
    [one] { $count } offer
   *[other] { $count } offers
}
stack-of = Stack of { $size }
# Under the price of a stack: "Unit price: 0.25 div · 2.49 c · 182 ex".
unit-price = Unit price: { $prices }
# The tooltip of an orb of the price block, under its name: what one Divine or Chaos Orb is worth in Exalted
# Orbs, from the rates of the price database ($amount; $count, the same number, for the plural).
orb-rate-in-exalted = 1 = { $amount } { $count ->
    [one] Exalted Orb
   *[other] Exalted Orbs
}
# The same under the name of the Exalted Orb: how many make a Divine Orb.
orb-rate-for-divine = { $amount } = 1 Divine Orb
# The change first, with a real minus sign.
trend-rise = +{ $percent }% over the last { $days } days
trend-fall = −{ $percent }% over the last { $days } days
trend-stable = Stable over the last { $days } days
# Under the title of the price of a unique on fewer than 5 offers, in yellow.
low-reliability-offers = { $count ->
    [0] Low reliability: no offer lately
    [one] Low reliability: { $count } offer
   *[other] Low reliability: { $count } offers
}
# Under the title of the price of an item of the currency exchange on fewer than 10 trades.
low-reliability-trades = { $count ->
    [0] Low reliability: no trade lately
    [one] Low reliability: { $count } trade
   *[other] Low reliability: { $count } trades
}
# On the right of the title of an estimated price: the similar offers found.
similar = { $count ->
    [one] { $count } similar
   *[other] { $count } similar
}
# Under the title of an estimated price from fewer than 5 offers, the suspicious ones left out.
low-reliability-similar = { $count ->
    [one] Low reliability: { $count } similar offer
   *[other] Low reliability: { $count } similar offers
}
# Under an estimated price: how it is made, without its final period (estimate-note).
estimate-median = { $count ->
    [one] Price of the only similar offer
   *[other] Median of the { $count } similar offers
}
estimate-median-cheapest = Median of the { $count } cheapest similar offers
# The same, when the quality of a catalyst is searched: the offers of another kind of quality ($quality, quality-of-kind) aren't counted.
estimate-median-of = { $count ->
    [one] Price of the only similar offer of { $quality }
   *[other] Median of the { $count } similar offers of { $quality }
}
estimate-median-cheapest-of = Median of the { $count } cheapest similar offers of { $quality }
# The note of an estimate ($note, one of the four above); with the range of the offers counted, from
# the cheapest to the dearest, in one currency ($currency: div or ex): "…, 2 to 3 div."
estimate-note = { $note }.
estimate-note-range = { $note }, { $cheapest } to { $dearest } { $currency }.
# The kind of the quality of a catalyst, within a sentence.
quality-of-kind = { $kind ->
        [life] life quality
        [mana] mana quality
        [defence] defence quality
        [physical] physical quality
        [fire] fire quality
        [cold] cold quality
        [lightning] lightning quality
        [chaos] chaos quality
        [attack] attack quality
        [caster] caster quality
        [speed] speed quality
        [attribute] attribute quality
        [minion] minion quality
       *[other] quality
    }
# An unidentified unique whose base type has several uniques.
identify-to-price = Identify the item to get its price.
possible-uniques = Possible uniques
# In place of the price of a possible unique without offers.
no-offers = No offers
# Rare, magic and normal items, priced from the offers of similar items.
no-market-price-rare = No market price for rare items.
# A skill gem, which no price database lists, priced from the offers of similar gems.
no-market-price-skill-gem = No market price for skill gems.
no-market-price-magic = No market price for magic items.
no-market-price-normal = No market price for normal items.
search-to-estimate = Search similar offers to estimate it.
# A rare or magic item whose filters compare none of its stats: the offers of any item of its
# base would say nothing of its price.
check-to-estimate = Check a stat that sets this item apart to estimate it.
# After a search whose offers none is counted, some paid in a currency without price, or priced on fewer than 10 trades.
no-offer-counted-to-estimate = No similar offer is paid in a currency with a reliable price.
# A fixed-name item whose price isn't known yet, or missing.
prices-downloading = The prices of { $league } are downloading…
# Before the league is known, at the first launch.
prices-downloading-no-league = Downloading the prices…
french-names-downloading = The French item names are downloading…
no-price-found = No price found for this item in { $league }.

## Offers block

offers-label = Offers
# Before the first search of the item, beside the search block.
offers-search-hint = One search per click, within the limits of the trade site.
offers-search-tooltip = Search the offers
offers-searching = Searching the trade site…
# Under the title, e.g. "searched 2min ago".
offers-searched = searched { $age }
# On the right of the title: the offers shown, of those found.
offers-count = { $shown } of { $total } online
# The titles of the columns of the offers.
offers-column-price = Price
offers-column-per-unit = Per unit
offers-column-stock = Stock
offers-column-listed = Listed
offers-again-tooltip = Search again
# The tooltip of the dark block while the answer of the last search or of more offers is
# awaited.
offers-busy-tooltip = Waiting for the answer of the trade site
# The tooltip of the countdown before the next search, e.g. "8s".
offers-next-search-tooltip = Next search in { $wait }
# Under the title, during a long wait, e.g. "4min".
offers-limit = Trade site limit: next search in { $wait }.
# During the wait of a refusal of the trade site that names no limit (HTTP 403), e.g. "1min", then what to do under it;
# $report is offers-report.
offers-forbidden = Refused by the trade site: next search in { $wait }.
offers-forbidden-hint = If it happens again, { $report }.
offers-none = No offer online with these filters.
offers-none-hint = Uncheck a stat or lower a minimum, then search again.
offers-unreachable = The trade site can't be reached.
# A search the trade site turned down with an error (a stat renamed by a patch, most often), then what to do under it;
# $report is offers-report, a link that opens the bug report.
offers-refused = The trade site refused this search.
offers-refused-hint = Uncheck a filter and search again. If it happens again, { $report }.
offers-report = report it
# An answer of the trade site the application can't read, then what to do under it; $report is offers-report-start.
offers-unreadable = The answer of the trade site can't be read.
offers-unreadable-hint = The game may have changed. { $report }, so that Exile Market can be fixed.
offers-report-start = Report it
offers-identify = Identify the item to search its offers.
# The double chevron under the list.
offers-more-tooltip = Show more offers
# The tooltip of an offer: a price far below the market, the seller status, and the pill
# that copies the whisper message of the trade site.
offers-suspicious = Be careful: far below the market price.
# Under the name of an offer whose quality of a catalyst is of another kind than the item's, while its quality is searched.
offers-other-quality = Another kind of quality: not counted in the estimate.
# After the name of the currency of an offer the estimate doesn't count, in the list: a currency without price, or priced on fewer than 10 trades.
offer-not-counted = not counted
# After the price of such an offer, in the list: its kind of quality, written as the filter, so that it reads as a quality and not as a name.
offer-quality-kind = { $kind ->
        [life] Life quality
        [mana] Mana quality
        [defence] Defence quality
        [physical] Physical quality
        [fire] Fire quality
        [cold] Cold quality
        [lightning] Lightning quality
        [chaos] Chaos quality
        [attack] Attack quality
        [caster] Caster quality
        [speed] Speed quality
        [attribute] Attribute quality
        [minion] Minion quality
       *[other] Other quality
    }
seller-online = Online
seller-away = Away
seller-offline = Offline
copy-whisper = Copy whisper
whisper-copied = Copied
# The system refused the text: a click on the pill tries again.
whisper-copy-failed = Not copied, try again

## Settings

settings-title = Settings
# Label above the choices of the language.
settings-language = Language
# The language of the system, when the interface has it.
language-system = System
# Each language is named in itself, so that a player finds theirs in any language.
language-english = English
language-french = Français
settings-always-on-top = Always on top
# The line under Always on top, what it does.
settings-always-on-top-help = Keeps the window above the game.
# Label above Always on top and Close with the game.
settings-game = Game
settings-close-with-game = Close with the game
# The line under Close with the game, what it does.
settings-close-with-game-help = Exile Market quits when Path of Exile 2 closes.
# Label above the setting of the pseudo totals, after the hotkey.
settings-search = Search
# The pseudo totals of a rare or magic item shown among its filters, or neither shown nor searched.
settings-pseudo-totals = Show the pseudo totals
# The line under Show the pseudo totals, what it does.
settings-pseudo-totals-help = Resistances, life, mana and attributes added up, among the filters.
# Label above the price-check hotkey.
settings-hotkey = Hotkey
# The hotkey turned on or off; its keys are drawn after it, in a button that opens the list of the keys.
settings-hotkey-price-check = Price check in the game
settings-hotkey-help = Copies the item under the cursor and shows its price over the game.
# The tooltip of the button of the keys of the hotkey.
settings-hotkey-change = Change the key
# On the right of the default keys, in the list of the keys of the hotkey.
settings-hotkey-default = Default
# The key of the hotkey taken by another application, in the settings: "hotkey-taken" first, then this sentence on a line of its own. $copy the keys of the copy of the game (Ctrl+C), in bold.
hotkey-taken-settings = Choose another key, or copy with { $copy }.
# The name of the Shift key, in the keys of the hotkey (Ctrl+Shift+D).
key-shift = Shift
# Above the disclaimer of the settings, then the link to the donation page.
donation-intro = { $app } is free.
# The link to the donation page on Ko-fi, after "donation-intro".
donation-link = Support it on Ko-fi
# Required by Grinding Gear Games for the tools of the community.
disclaimer = This product isn't affiliated with or endorsed by Grinding Gear Games in any way.

## Bug report

# The preview of the bug report, in place of the content of the window. The report itself is written in English,
# the language of the issues, whatever the language of the interface.
bug-report-title = Report a bug
bug-report-intro = The report opens on GitHub with the text below. Describe what happened there, then send it.
# Label above the summary of the report: the version, the system, the languages, the league and the settings.
bug-report-sent = Sent with the report
# Label above the last lines of the log.
bug-report-log = Log
# Under the lines of the log. "another window" stays in English: the report writes it so.
bug-report-log-note = Its last lines. The other windows are written “another window”.
# Two pills joined on the right of the label of the log, that choose the log sent: the one of this session, or of the
# previous session, chosen at once after a crash.
bug-report-this-session = This session
bug-report-previous-session = The previous one
# What a screen reader says of one of these pills, $session its word.
bug-report-log-session = Log: { $session }
# In place of the lines of the log, when it can't be read.
bug-report-no-log = The log can't be read.
# Checkbox that joins the item shown to the report, $name its name, in bold after the colon; a long name is cut with "…".
bug-report-join-item = Join the item shown: { $name }
# Checkbox that joins the text of an item that couldn't be read.
bug-report-join-unreadable = Join the item that couldn't be read
# The main action: the issue form of GitHub opened in the browser, its fields filled.
bug-report-open = Open on GitHub
# Copies the whole log of the session chosen, to paste into the issue; "Copied" after a click, until the preview closes or the other log, not copied yet, is chosen.
bug-report-copy = Copy the log
bug-report-copied = Copied
# The system refused the log: a click on the pill tries again.
bug-report-copy-failed = Not copied, try again
bug-report-account = A GitHub account is needed.

## Window that can't open

# A dialog of the system when the window of the application can't open, its paragraphs in this order: what failed,
# what to do, the log (another cause only), then the error itself, in English. $app is the name of the application.
# No X11 display (Linux): neither X11 nor XWayland.
window-error-display = { $app } can't open its window: it needs X11, or XWayland on a Wayland desktop.
window-error-display-fix = Install XWayland (the package xwayland on most distributions), then launch { $app } again.
# No OpenGL 2.1, the least the application draws with.
window-error-opengl = { $app } can't open its window: it needs OpenGL 2.1 or later, which the graphics driver doesn't provide.
window-error-opengl-fix = Update the graphics driver, then launch { $app } again.
# Any other cause, to report: $issues is the address of the issues of the project on GitHub.
window-error-other = { $app } can't open its window.
window-error-other-fix = Report it on GitHub with its log, so that it can be fixed: { $issues }
# The path of the log file.
window-error-log = Log: { $path }
# The error, in English.
window-error-details = Details: { $error }
