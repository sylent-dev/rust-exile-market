// Section Uncertain prices (22b): what the price block says of a price on few trades or offers, beyond its warning.
// The warnings are there already ("Only 3 similar offers: the estimate is uncertain", 13i; "Only 3 trades: the price
// is uncertain", 17e3c, the amounts of a thin currency kept as they are).
// 1. How sure an estimate is. A, today: the warning under 5 offers, the note of the median. B, the range of the
//    offers counted at the end of the note, also with 5 offers (a narrow range tells a price to trust), in Divine Orbs
//    from one Divine Orb at the median and in Exalted Orbs under it, as the price of a possible unique. C, a mark of
//    reliability right of the title, low, medium or high (PoE Overlay II shows "a reliability rating", computed by a
//    closed model). The owner chose on 2026-11-17 a mark of low reliability only, nothing when the price is reliable,
//    so that it promises nothing; on 2026-11-18, "Low reliability" in yellow on a small line under the title, as
//    "searched 2min ago" under the title of the offers. Drawn in place of the warning line, the warning in its
//    tooltip, with the range of B, for the three kinds of price.
// 2. An offer paid in a currency the estimate doesn't count (priced on fewer than 10 trades, or without price: a
//    Wombgift) is shown today as any other. A, faint, why in the tooltip of its icon; B, faint, "not counted" after
//    its name, as the kind of an offer of another quality (13k4b). Recommended: B, seen without the pointer.
// 3. After a search whose offers none is counted, the price block still says "Search similar offers to estimate it.".
//    Its own words; after a search without any offer, no second line (the offers block says what to do).
// 4. The price of a unique on fewer than 5 offers, told uncertain, still tells the suspicious offers: an honest offer
//    under a quarter of a stale price is struck through. Recommended: none suspicious then, as a currency on fewer
//    than 10 trades (17e3c): the rule "a price the application calls uncertain doesn't judge the offers".

const UP_TEXT = {
  en: { uncertain3: "Only 3 similar offers: the estimate is uncertain", median3: "Median of the 3 similar offers.",
    median3Range: "Median of the 3 similar offers, 40 to 300 ex.", median5: "Median of the 5 cheapest similar offers.",
    median5Range: "Median of the 5 cheapest similar offers, 2 to 3 div.", notCounted: "not counted",
    tipThin: "Not counted: its price stands on 1 trade", tipNone: "Not counted: this currency has no price",
    noneCounted: "No similar offer is paid in a currency with a reliable price.",
    low: "Low reliability", uniqueWarning: "Only 3 offers: the price is uncertain", thin: "Only 1 trade: the price is uncertain" },
  fr: { uncertain3: "Seulement 3 offres similaires : l'estimation est incertaine", median3Range: "Médiane des 3 offres similaires, de 40 à 300 ex.",
    median5Range: "Médiane des 5 offres similaires les moins chères, de 2 à 3 div.", notCounted: "non comptée",
    noneCounted: "Aucune offre similaire n'est payée dans une devise au prix fiable.", low: "Fiabilité faible" },
};

// The magic ring of three similar offers, far apart: 40, 95 and 300 ex, the median 95 ex.
const UP_RING_ROWS = (fr = false) => [
  { v: "40", k: "ex", when: fr ? "il y a 2j" : "2d ago" }, { v: "95", k: "ex", when: fr ? "il y a 5h" : "5h ago" },
  { v: "300", k: "ex", when: fr ? "il y a 1h" : "1h ago" }];
const UP_RING_FR = { ...MAGIC, name: "Anneau de saphir scintillant du lynx", cls: "Anneaux", tags: ["niv. 68"], req: "Niveau requis 44",
  statsLabel: "Statistiques (3)", filters: "2 filtres" };
const upRing = (p, { fr = false } = {}) => win(topBar(), item(fr ? UP_RING_FR : MAGIC),
  price({ estimate: true, values: ["0.14", "1.4", "95"], offers: fr ? "3 similaires" : "3 similar", ...p }, fr ? "fr" : "en"),
  offers(fr ? { count: "3 sur 3 en ligne", columns: ["Prix", "Publiée"], age: "recherche il y a 2min", rows: UP_RING_ROWS(true) }
    : { count: "3 of 3 online", columns: ["Price", "Listed"], rows: UP_RING_ROWS() }, fr ? "fr" : "en"),
  foot(fr ? "poe2scout il y a 12min · GGG trade il y a 2min" : undefined));

// The rare bow of five offers counted, close: 2 to 3 div, the median 2.4 div; the bait at 300 ex left out.
const UP_BOW_ROWS = [
  { v: "300", k: "ex", when: "1h ago", sus: true }, { v: "2", k: "div", when: "20min ago" }, { v: "2.2", k: "div", when: "4h ago" },
  { v: "2.4", k: "div", when: "1d ago", away: true }, { v: "2.5", k: "div", when: "2h ago" }, { v: "3", k: "div", when: "6h ago" }];
const upBow = (p, rows = UP_BOW_ROWS) => win(topBar(), item(BOW_RARE),
  price({ estimate: true, values: ["2.4", "24", "1.7K"], offers: "18 similar", ...p }),
  offers({ count: "10 of 18 online", columns: ["Price", "Listed"], rows }), foot());

// The mark of low reliability on a small line under the title, as "searched 2min ago" under the title of the offers
// (owner, 2026-11-18), `tip` in its tooltip, open when `hovered`.
const lowMark = (html, words, tip, hovered = false) => html.replace(/<span class="label">([^<]*)<\/span>/,
  `<span class="title-stack"><span class="label">$1</span><span class="age warn app-reliability${hovered ? " is-hovered" : ""}">${words}<span class="app-tip">${tip}</span></span></span>`);

// The other lead (2026-11-18): the reason written in the small line, no tooltip, nothing hidden under the pointer.
const lowLine = (html, words) => html.replace(/<span class="label">([^<]*)<\/span>/,
  `<span class="title-stack"><span class="label">$1</span><span class="age warn">${words}</span></span>`);
// The bow of 6 similar offers, 2 of them suspicious: the line counts the 4 of the median, the title the 6 found.
const UP_BOW_FOUR = [
  { v: "300", k: "ex", when: "1h ago", sus: true }, { v: "350", k: "ex", when: "3h ago", sus: true }, { v: "2", k: "div", when: "20min ago" },
  { v: "2.2", k: "div", when: "4h ago" }, { v: "2.4", k: "div", when: "1d ago", away: true }, { v: "2.5", k: "div", when: "2h ago" }];

// 2. The bow, two of its offers paid in a currency the estimate doesn't count: a Lesser Charging Rune priced on one
// trade (40,156 ex), and a Banded Wombgift, without price.
const UP_NOT_COUNTED = [
  { v: "2", k: "div", when: "20min ago", name: "Divine Orb" },
  { v: "1", k: "lesser-charging-rune", when: "35min ago", name: "Lesser Charging Rune", thin: true },
  { v: "2.2", k: "div", when: "4h ago", name: "Divine Orb" },
  { v: "3", k: "banded-wombgift", when: "1h ago", name: "Banded Wombgift", none: true },
  { v: "2.4", k: "div", when: "1d ago", away: true, name: "Divine Orb" },
  { v: "2.5", k: "div", when: "2h ago", name: "Divine Orb" },
  { v: "3", k: "div", when: "6h ago", name: "Divine Orb" }];
const UP_NOT_COUNTED_FR = [
  { v: "2", k: "div", when: "il y a 20min", name: "Orbe divin" },
  { v: "1", k: "lesser-charging-rune", when: "il y a 35min", name: "Rune de charge inférieure", thin: true },
  { v: "2.2", k: "div", when: "il y a 4h", name: "Orbe divin" },
  { v: "3", k: "banded-wombgift", when: "il y a 1h", name: "Matrice cerclée", none: true },
  { v: "2.4", k: "div", when: "il y a 1j", away: true, name: "Orbe divin" },
  { v: "2.5", k: "div", when: "il y a 2h", name: "Orbe divin" },
  { v: "3", k: "div", when: "il y a 6h", name: "Orbe divin" }];
// A: faint, the tooltip of the icon of the row `hovered` open. B: faint, "not counted" after the name.
const upTagged = (rows, option, { hovered, fr = false } = {}) => rows.map((r, i) => {
  if (!r.thin && !r.none) return r;
  if (option === "today") return r;
  if (option === "A") return { ...r, faint: true, tip: r.thin ? UP_TEXT.en.tipThin : UP_TEXT.en.tipNone, tipShown: i === hovered };
  return { ...r, kindTag: UP_TEXT[fr ? "fr" : "en"].notCounted };
});
const upNotCounted = (option, { hovered, fr = false } = {}) => {
  const rows = upTagged(fr ? UP_NOT_COUNTED_FR : UP_NOT_COUNTED, option, { hovered, fr });
  const block = offers(fr ? { count: "10 sur 18 en ligne", columns: ["Prix", "Publiée"], age: "recherche il y a 2min", rows }
    : { count: "10 of 18 online", columns: ["Price", "Listed"], rows }, fr ? "fr" : "en");
  const p = fr ? { estimate: true, values: ["2.4", "24", "1.7K"], offers: "18 similaires", note: "Médiane des 5 offres similaires les moins chères." }
    : { estimate: true, values: ["2.4", "24", "1.7K"], offers: "18 similar", note: UP_TEXT.en.median5 };
  return win(topBar(), item(fr ? { ...BOW_RARE, cls: "Arcs" } : BOW_RARE), price(p, fr ? "fr" : "en"), block,
    foot(fr ? "poe2scout il y a 12min · GGG trade il y a 2min" : undefined));
};

// 3. A search whose two offers are both paid in a currency the estimate doesn't count, then one without any offer.
const upNoneCounted = (second, fr = false) => {
  const rows = upTagged((fr ? UP_NOT_COUNTED_FR : UP_NOT_COUNTED).filter(r => r.thin || r.none), "B", { fr });
  const lines = fr ? ["Pas de prix de marché pour les objets rares.", second] : ["No market price for rare items.", second];
  return win(topBar(), item(fr ? { ...BOW_RARE, cls: "Arcs" } : BOW_RARE), price({ lines: lines.filter(Boolean) }, fr ? "fr" : "en"),
    offers(fr ? { count: "2 sur 2 en ligne", columns: ["Prix", "Publiée"], age: "recherche il y a 2min", rows }
      : { count: "2 of 2 online", columns: ["Price", "Listed"], rows }, fr ? "fr" : "en"),
    foot(fr ? "poe2scout il y a 12min · GGG trade il y a 2min" : undefined));
};
const upNoOffer = second => win(topBar(), item(BOW_RARE), price({ lines: ["No market price for rare items.", second].filter(Boolean) }),
  offers({ lines: ["No offer online with these filters. Uncheck a stat or widen a range, then search again."] }), foot());

// 4. A unique priced on 3 offers at 150 ex, the market fallen since: its offers now ask 30 to 140 ex.
const UP_UNIQUE = { rarity: "unique", name: "Ryslatha's Coil", base: "Wide Belt", cls: "Belts", tags: ["ilvl 74"], req: "Requires Level 55",
  statsLabel: "Stats (4)", filters: "3 filters" };
const upUnique = (suspicious, mark = false) => win(topBar(), item(UP_UNIQUE),
  (mark ? html => lowMark(html, UP_TEXT.en.low, UP_TEXT.en.uniqueWarning) : html => html)(
    price({ values: ["0.21", "2.1", "150"], offers: "3 offers", trend: "Stable over the last 7 days", stable: true,
      warning: mark ? undefined : UP_TEXT.en.uniqueWarning })),
  offers({ count: "4 of 4 online", columns: ["Price", "Listed"], rows: [
    { v: "30", k: "ex", when: "40min ago", sus: suspicious }, { v: "35", k: "ex", when: "2h ago", sus: suspicious },
    { v: "90", k: "ex", when: "1d ago", away: true }, { v: "140", k: "ex", when: "3d ago" }] }), foot());

SECTIONS.unshift(["Uncertain prices (22b): how sure an estimate is, the offers not counted, the suspicious offers of an uncertain price", [
  ["1 · How sure · Recommended · one line · the reason in the line, no tooltip", lowLine(upRing({ note: UP_TEXT.en.median3Range }), "Low reliability: 3 similar offers")],
  ["1 · How sure · Recommended · one line · 6 found, 2 suspicious: the line counts the 4 of the median", lowLine(upBow({ values: ["2.3", "23", "1.6K"], offers: "6 similar",
    note: "Median of the 4 similar offers, 2 to 2.5 div." }, UP_BOW_FOUR).replace("10 of 18 online", "6 of 6 online"), "Low reliability: 4 similar offers")],
  ["1 · How sure · Recommended · one line · en français", lowLine(upRing({ note: UP_TEXT.fr.median3Range }, { fr: true }), "Fiabilité faible : 3 offres similaires")],
  ["1 · How sure · Recommended · one line · a unique on 3 offers", lowLine(upUnique(false, false).replace(/<div class="line warn">[^<]*<\/div>/, ""), "Low reliability: 3 offers")],
  ["1 · How sure · Recommended · one line · a currency on 1 trade", thinWin(RUNE, lowLine(price(RUNE_PRICE), "Low reliability: 1 trade"), offers(RUNE_OFFERS()))],
  ["1 · How sure · Recommended · one line · a currency without trade lately", thinWin(RUNE, lowLine(price(RUNE_PRICE), "Low reliability: no trade lately"), offers(RUNE_OFFERS()))],
  ["1 · How sure · A · Today: the warning under 5 offers", upRing({ warning: UP_TEXT.en.uncertain3, note: UP_TEXT.en.median3 })],
  ["1 · How sure · A · Today: 5 offers, no warning", upBow({ note: UP_TEXT.en.median5 })],
  ["1 · How sure · B · the range of the offers counted, 3 offers far apart", upRing({ warning: UP_TEXT.en.uncertain3, note: UP_TEXT.en.median3Range })],
  ["1 · How sure · B · 5 offers close: a narrow range", upBow({ note: UP_TEXT.en.median5Range })],
  ["1 · How sure · Tooltip lead · the mark of low reliability, the range of B, the warning in its tooltip", lowMark(upRing({ note: UP_TEXT.en.median3Range }), UP_TEXT.en.low, UP_TEXT.en.uncertain3, true)],
  ["1 · How sure · Tooltip lead · 5 offers close: no mark", upBow({ note: UP_TEXT.en.median5Range })],
  ["1 · How sure · Tooltip lead · en français", lowMark(upRing({ note: UP_TEXT.fr.median3Range }, { fr: true }), UP_TEXT.fr.low, UP_TEXT.fr.uncertain3)],
  ["1 · How sure · Tooltip lead · a unique on 3 offers", upUnique(false, true)],
  ["1 · How sure · Tooltip lead · a currency on 1 trade", thinWin(RUNE, lowMark(price(RUNE_PRICE), UP_TEXT.en.low, UP_TEXT.en.thin), offers(RUNE_OFFERS()))],
  ["2 · Offers not counted · Today: shown as any other", upNotCounted("today")],
  ["2 · Offers not counted · A · faint, why in the tooltip of the icon", upNotCounted("A", { hovered: 1 })],
  ["2 · Offers not counted · Recommended · B · faint, \"not counted\" after the name", upNotCounted("B")],
  ["2 · Offers not counted · Recommended · B · en français", upNotCounted("B", { fr: true })],
  ["3 · No offer counted · Today: \"Search similar offers\" after the search", upNoneCounted("Search similar offers to estimate it.")],
  ["3 · No offer counted · Recommended: its own words", upNoneCounted(UP_TEXT.en.noneCounted)],
  ["3 · No offer counted · Recommended · en français", upNoneCounted(UP_TEXT.fr.noneCounted, true)],
  ["3 · No offer at all · Today: \"Search similar offers\" after the search", upNoOffer("Search similar offers to estimate it.")],
  ["3 · No offer at all · Recommended: no second line, the offers block says what to do", upNoOffer()],
  ["4 · An uncertain unique · Today: its honest offers struck through", upUnique(true)],
  ["4 · An uncertain unique · Recommended: none suspicious, as a thin currency", upUnique(false, true)],
]]);
