// Section Orb names (22f): the name of an orb under the pointer in the price block, so that a new player knows which
// orb they see (owner, 2026-11-06); the Greater and Perfect Orbs share the icon of their orb. In the language of the
// item, as the names of the currencies in the offers. The zone under the pointer: the icon and its amount.
// 1. What the tooltip says. A, the name alone; B, the name, then what one orb is worth in the others, from the rates
//    of the price database the block already uses ("1 = 70 Exalted Orbs"): the rate a player looks up elsewhere
//    today. Recommended: B.
// 2. The list of the possible uniques of an unidentified item: the same tooltip over the orb of a price.
// 3. The class searched under an unchecked base, cut with "…" past its column: its whole text under the pointer, as
//    the other lines cut. No class known is cut today ("Masses à deux mains", the longest, fits): a class longer than
//    any known, invented to show it.

const ON_TEXT = {
  en: { div: "Divine Orb", chaos: "Chaos Orb", ex: "Exalted Orb",
    rate: { div: "1 = 700 Exalted Orbs", chaos: "1 = 70 Exalted Orbs", ex: "700 = 1 Divine Orb" } },
  fr: { div: "Orbe divin", chaos: "Orbe du chaos", ex: "Orbe exalté",
    rate: { div: "1 = 700 Orbes exaltés", chaos: "1 = 70 Orbes exaltés", ex: "700 = 1 Orbe divin" } },
};

const onTip = (key, lead, lang) => lead === "A" ? `<span class="app-tip"><span>${ON_TEXT[lang][key]}</span></span>`
  : `<span class="app-tip"><span>${ON_TEXT[lang][key]}</span><span>${ON_TEXT[lang].rate[key]}</span></span>`;

// The price block with the orb `hovered` (div, chaos or ex) open: each amount and its icon a zone of its own.
const onPrice = (html, hovered, lead, lang = "en") => {
  let index = 0;
  const keys = ["div", "chaos", "ex"];
  return html.replace(/<div class="price">/g, () => {
    const key = keys[index++];
    return `<div class="price app-orb-hover ${key === hovered ? "is-hovered" : ""}">${onTip(key, lead, lang)}`;
  });
};

const onBow = (hovered, lead, lang = "en") => {
  const fr = lang === "fr";
  return win(topBar(), item(fr ? { ...BOW_RARE, cls: "Arcs" } : BOW_RARE),
    onPrice(price(fr ? { estimate: true, values: ["2.4", "24", "1.7K"], offers: "18 similaires", note: "Médiane des 5 offres similaires les moins chères, de 2 à 3 div." }
      : { estimate: true, values: ["2.4", "24", "1.7K"], offers: "18 similar", note: "Median of the 5 cheapest similar offers, 2 to 3 div." }, lang), hovered, lead, lang),
    offers({ state: "idle" }, lang), foot(fr ? "poe2scout il y a 12min · GGG trade il y a 2min" : undefined));
};

// 2. The unidentified amulet, the price of its first possible unique hovered.
const onCandidates = lead => {
  const html = price({ candidates: [["Astramentis", "4.0", "div"], ["Hinekora's Sight", "640", "ex"], ["Fixation of Yix", "95", "ex"], ["Strugglescream", null]] });
  const open = html.replace(/<span>4\.0 (<img[^>]*>)<\/span>/, `<span class="app-orb-hover is-hovered">4.0 $1${onTip("div", lead, "en")}</span>`);
  return win(topBar(), item({ rarity: "unique", name: "Stellar Amulet", cls: "Amulets", tags: ["Unidentified", "ilvl 74"], req: "Requires Level 60", statsLabel: "Stats (1)" }),
    open, offers({ lines: ["Identify the item to search its offers."] }), foot());
};

// 3. The bow, its base unchecked, the class searched cut.
const ON_LONG_CLASS = "Class searched: Two Hand Maces and Ancient Quarterstaves";
const onClassCut = hovered => win(topBar(), item({ ...BOW_RARE, filters: "3 filters", stats: [
  { group: "Item", min: true },
  { mod: `Base: Gemini Bow<span class="app-sub-cut ${hovered ? "is-hovered" : ""}"><span class="app-sub">${ON_LONG_CLASS}</span><span class="app-tip"><span>${ON_LONG_CLASS}</span></span></span>`, check: false },
  { mod: "Item level: 81", check: false, min: 81 },
  { mod: "Sockets: 2", check: false, min: 2 },
  ...BOW_FILTERS.slice(4),
] }, true), foot());

SECTIONS.unshift(["Orb names (22f): the name of an orb under the pointer in the price block, and the class searched cut by \"…\"", [
  ["1 · A · the name alone · the Divine Orb", onBow("div", "A")],
  ["1 · B · the name and its rate · the Divine Orb · recommended", onBow("div", "B")],
  ["1 · B · the Exalted Orb", onBow("ex", "B")],
  ["1 · B · en français, the Chaos Orb", onBow("chaos", "B", "fr")],
  ["2 · B · the possible uniques of an unidentified item", onCandidates("B")],
  ["3 · the class searched cut, its whole text under the pointer (an invented class)", onClassCut(true)],
]]);
