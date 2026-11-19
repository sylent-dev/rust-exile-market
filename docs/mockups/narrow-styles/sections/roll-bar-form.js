// Section Roll bar form (22h): one other form of the roll bar of a rare or magic modifier, kept for every
// player, no setting (owner, 2026-11-18). The forms drawn in steps 13e and 13j2, compared again with the bar
// of today on the same lines (the tiers of Ancestral Mail and Gemini Bow of the section Roll bar):
// 1, today: filled to the roll, from the lowest roll of the worst tier to the highest of tier 1 (13j2).
// 2, the zone searched (13e): the same bar, its fill replaced by the zone between the minimum and the maximum
//    of the range field, lighter, and the roll as a tick; it follows the field as the player types.
// 3, a segment per tier (13j2): one segment per tier of equal width, the worst on the left, the tiers under
//    the roll's full, its own filled to where the roll falls in it, those above empty.
// 4, the zone of its tier shaded (13j2): the bar of today, the zone of the roll's tier lighter behind the fill,
//    so that the room left in the tier shows.
// A line of two values ("Adds # to #") takes the average of their shares, like the bar of today.
// Chosen by the owner on 2026-11-19: 1, the bar of today, the easiest to read for a new player; the room left
// in the tier of 4 is already written by the range of the tier next to the bar.

// Where `value` falls between `low` and `high`, from 0 to 1.
const rbfShare = (value, low, high) => (high > low ? Math.min(1, Math.max(0, (value - low) / (high - low))) : 1);
// The lowest and the highest roll of value `index` over every tier.
const rbfBounds = (tiers, index) => [Math.min(...tiers.map(t => t[index][0])), Math.max(...tiers.map(t => t[index][1]))];
const rbfMean = shares => shares.reduce((sum, share) => sum + share, 0) / shares.length;
const pct = share => `${(share * 100).toFixed(1)}%`;

// The bar of a line in form `form`: `values` its rolls, `tiers` the best first (each the range of each value),
// `tier` the tier of the line (1 the best), `min` and `max` its range field (empty: no limit).
function rbfBar(form, values, tiers, tier, min, max) {
  const over = rbfMean(values.map((value, index) => rbfShare(value, ...rbfBounds(tiers, index))));
  if (form === 1) return `<span class="bar"><i style="width:${pct(over)}"></i></span>`;
  if (form === 2) {
    // The field holds the first value of the line; a line of two values has no field, so its zone is the
    // whole bar and only the tick shows.
    const [low, high] = rbfBounds(tiers, 0);
    const from = min === undefined || min === "" ? 0 : rbfShare(min, low, high);
    const to = max === undefined || max === "" ? 1 : rbfShare(max, low, high);
    const zone = min === undefined ? "" : `<b class="app-rbf-zone" style="left:${pct(from)};width:${pct(to - from)}"></b>`;
    return `<span class="bar app-rbf">${zone}<b class="app-rbf-tick" style="left:${pct(over)}"></b></span>`;
  }
  if (form === 3) {
    const own = tiers[tier - 1];
    const inTier = rbfMean(values.map((value, index) => rbfShare(value, own[index][0], own[index][1])));
    // Worst tier on the left: segment k is tier `tiers.length - k`.
    const segments = tiers.map((_, k) => {
      const t = tiers.length - k;
      const fill = t > tier ? 1 : t === tier ? inTier : 0;
      return `<b><i style="width:${pct(fill)}"></i></b>`;
    }).join("");
    return `<span class="bar app-rbf-segments">${segments}</span>`;
  }
  const own = tiers[tier - 1];
  const from = rbfMean(own.map(([low], index) => rbfShare(low, ...rbfBounds(tiers, index))));
  const to = rbfMean(own.map(([, high], index) => rbfShare(high, ...rbfBounds(tiers, index))));
  return `<span class="bar app-rbf"><b class="app-rbf-zone" style="left:${pct(from)};width:${pct(to - from)}"></b>`
    + `<i style="width:${pct(over)}"></i></span>`;
}

// The lines of the section Roll bar, their bar drawn in `form` (`over.html`, read by `rareBar`).
const rbfLine = (form, row, values, tiers, tier) => ({ ...row, over: { html: rbfBar(form, values, tiers, tier, row.min, row.max) } });
const rbfMailStats = form => mailStats().map(row => {
  const line = {
    "+86 to maximum Life": [[86], one(BODY_LIFE), 7],
    "62% increased Armour and Evasion": [[62], one(BODY_ARMOUR_EVASION), 5],
    "+98 to Armour": [[98], one(BODY_ARMOUR), 4],
    "+88 to Evasion Rating": [[88], one(BODY_EVASION), 4],
    "+41% to Cold Resistance": [[41], one(RESISTANCE), 1],
    "+23% to Lightning Resistance": [[23], one(RESISTANCE), 5],
  }[row.mod];
  return line ? rbfLine(form, row, ...line) : row;
});
const rbfBowStats = form => bowTierStats.map(row => {
  const line = {
    "152% increased Physical Damage": [[152], one(BOW_PHYSICAL), 3],
    "Adds 21 to 38 Physical Damage": [[21, 38], BOW_ADDED_PHYSICAL, 2],
    "Adds 3 to 140 Lightning Damage": [[3, 140], BOW_ADDED_LIGHTNING, 3],
    "+3 to Level of all Projectile Skills": [[3], one(BOW_PROJECTILE_LEVEL), 2],
    "14% increased Attack Speed": [[14], one(BOW_ATTACK_SPEED), 2],
  }[row.mod];
  return line ? rbfLine(form, row, ...line) : row;
});

const RBF_FORMS = [
  [1, "Chosen · 1 · today: filled to the roll over every tier"],
  [2, "Not chosen · 2 · the zone searched, the roll as a tick"],
  [3, "Not chosen · 3 · a segment per tier"],
  [4, "Not chosen · 4 · the zone of its tier shaded"],
];
SECTIONS.unshift(["Roll bar form (22h): one other form of the bar of a rare or magic modifier", [
  ...RBF_FORMS.map(([form, title]) => [`${title} · Ancestral Mail`,
    win(topBar(), item({ ...MAIL, stats: rbfMailStats(form) }, true), price(RARE_ESTIMATE_BEFORE), foot())]),
  ...RBF_FORMS.map(([form, title]) => [`${title} · Gemini Bow`,
    win(topBar(), item({ ...BOW_TIERS, stats: rbfBowStats(form) }, true), price(RARE_ESTIMATE_BEFORE), foot())]),
]]);
