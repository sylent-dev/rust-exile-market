// Section Waystone tier range (22g): the tier of a waystone searched between a minimum and a maximum (owner,
// 2026-11-01), "12 – 12" by default so that the search stays the same, widened by the player ("11 –"): the market
// of one tier is thin (32 offers online for T11–T12 in Forbidden Rites on 2026-11-18, 5 for T12 alone).
// Checked on the trade site on 2026-11-18: `map_tier` (`map_filters`) takes a minimum and a maximum together, and
// "12 – 12" among the category `map.waystone` finds as many offers as the base type `Waystone (Tier 12)`: a
// waystone can always be searched by its category and its tier, one search, no request more.
// Two leads:
// A, the range field on the row of the base, as planned: "Base: Waystone", "12 – 12", the bar of the tier under
//    it. Unchecked, the base searches every tier, as an empty range does: two ways of saying the same thing.
// B, a row "Tier: 12" of its own at the top of the Waystone group (French "Palier : 12"), with the bar of the
//    tier under it, and no row of the base: checked or not, it would search the same category. The tier is then
//    a filter like the other properties of the waystone, `map_filters` all of them (Claude Code's recommendation).
// Chosen by the owner on 2026-11-18: B.
// An estimate over several tiers mixes their values: the player's choice; the tier of each offer is in its
// tooltip ("Waystone · T11").

// The row of the tier, its range field from `min` to `max` (empty: no limit).
const wtrRow = (label, min, max, extra = {}) => ({ ...tierBaseRow(label, 12), min, max, ...extra });

// The Waystone group of the owner's waystone, its item rarity and drop chance checked.
const WTR_PROPERTIES = [
  { mod: "Revives available: 3", check: false, min: 3 },
  { mod: "Item rarity: +11%", check: true, min: 9 },
  { mod: "Monster rarity: +25%", check: false, min: 22 },
  { mod: "Monster effectiveness: +13%", check: false, min: 11 },
  { mod: "Waystone drop chance: +50%", check: true, min: 45 },
];
// The modifiers under the groups, unchecked.
const WTR_MODIFIERS = waystonePropertyStats([1, 4]).slice(waystonePropertyStats([1, 4]).findIndex(row => row.group === "Prefixes"));

// A: the range on the row of the base.
const wtrStatsA = (min, max, checked = true) => [
  { group: "Item", min: true },
  wtrRow("Base: Waystone", min, max, { check: checked }),
  { mod: "Item level: 80", check: false, min: 80 },
  { group: "Waystone" },
  ...WTR_PROPERTIES,
  ...WTR_MODIFIERS,
];
// B: a row of its own at the top of the Waystone group, no row of the base.
const wtrStatsB = (min, max) => [
  { group: "Item", min: true },
  { mod: "Item level: 80", check: false, min: 80 },
  { group: "Waystone" },
  wtrRow("Tier: 12", min, max),
  ...WTR_PROPERTIES,
  ...WTR_MODIFIERS,
];
const wtrStatsBFr = (min, max) => waystonePropertyStatsFr
  .filter(row => !row.mod?.startsWith("Base"))
  .flatMap(row => row.group === "Pierre de téléportation" ? [row, wtrRow("Palier : 12", min, max)] : [row]);

const wtrWin = (stats, filters = "3 filters") => waystoneWin({ ...WAYSTONE_PROPS, filters }, stats);

SECTIONS.unshift(["Waystone tier range (22g): the tier of a waystone searched between a minimum and a maximum", [
  ["Not chosen · A · the range on the row of the base · by default, 12 – 12", wtrWin(wtrStatsA(12, 12))],
  ["Not chosen · A · widened by the player, 11 –", wtrWin(wtrStatsA(11, ""))],
  ["Not chosen · A · the base unchecked: every tier, as an empty range", wtrWin(wtrStatsA(12, 12, false), "2 filters")],
  ["Chosen · B · a row Tier of its own, no row of the base · by default, 12 – 12", wtrWin(wtrStatsB(12, 12))],
  ["Chosen · B · widened by the player, 11 –", wtrWin(wtrStatsB(11, ""))],
  ["Chosen · B · en français, 11 – 13", win(topBar(), item({ ...WAYSTONE_PROPS_FR, stats: wtrStatsBFr(11, 13) }, true), price(RARE_ESTIMATE_BEFORE), foot())],
]]);
