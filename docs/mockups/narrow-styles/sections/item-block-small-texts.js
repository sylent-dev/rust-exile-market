// Section Small texts of the item block (22a): three small texts that read as bugs or wrap too much.
// 1. The DPS of a weapon is the game's, with its quality, modifiers and runes: a tooltip under the pointer says it,
//    on each DPS, not to take it for the DPS of the base.
// 2. "Check a modifier to estimate this item." stays after checking an implicit, a rune, an enchantment or a total
//    its base reaches, which don't compare the item (any item of the base may pass them), and reads as a bug since
//    the game calls an implicit a modifier. Three leads: A names what compares, B keeps the text with a tooltip that
//    says what doesn't, C says what the stat must do (recommended: short, true of every case, the DPS and the
//    totals too).
// 3. The text of a filter without range field (the base, a rune, an enchantment, a fixed implicit, a line without
//    number) over the column of the range field too, so that it wraps less, the French above all (owner, 13k1); the
//    texts of the other rows stay aligned on their field.
// The owner followed the recommendations on 2026-11-17: the tooltip of the DPS, C, the text over the range column.

const ST_TEXT = {
  en: { dps: "With its quality, modifiers and runes", rare: "No market price for rare items.",
    today: "Check a modifier to estimate this item.",
    a: "Check a prefix, a suffix or a total to estimate this item.",
    bTip: "An implicit, a rune or an enchantment doesn't set the item apart: any item of its base may have it.",
    c: "Check a stat that sets this item apart to estimate it." },
  fr: { dps: "Avec sa qualité, ses modificateurs et ses runes", rare: "Pas de prix de marché pour les objets rares.",
    today: "Cochez un modificateur pour estimer cet objet.",
    a: "Cochez un préfixe, un suffixe ou un total pour estimer cet objet.",
    bTip: "Un implicite, une rune ou un enchantement ne distingue pas l'objet : tout objet de sa base peut l'avoir.",
    c: "Cochez une statistique qui distingue cet objet pour l'estimer." },
};

// The item block of `html` with the tooltip of each DPS, the first one open (`open`).
const stDps = (html, fr, open = true) => {
  let first = open;
  return html.replace(/<div><span class="">((?:Physical|Elemental|Chaos) DPS|DPS (?:physique|élémentaire|de chaos))<\/span><b>([^<]*)<\/b><\/div>/g,
    (_, label, value) => {
      const hovered = first ? " is-hovered" : "";
      first = false;
      return `<div class="app-prop-tip${hovered}"><span class="">${label}</span><b>${value}</b><span class="app-tip">${ST_TEXT[fr ? "fr" : "en"].dps}</span></div>`;
    });
};

// The ring whose resistance is its implicit (17e3f), the player having checked that implicit: it doesn't compare.
const stRingRows = implicitRingRows.map(row => row.mod === "+8% to all Elemental Resistances" ? { ...row, check: true } : row);
const stRingFr = { ...IMPLICIT_RING, fr: true, name: "Spire du vent", base: "Anneau prismatique", cls: "Anneaux", tags: ["niv. 80"],
  req: "Niveau requis 60", statsLabel: "Statistiques (3)", tradeLabel: "Voir sur l'échange", tradeTitle: "Ouvrir la recherche sur le site d'échange" };
const stRingRowsFr = [
  { group: "Objet", min: true },
  { mod: "Base : Anneau prismatique", check: true },
  { mod: "Niveau de l'objet : 80", check: false, min: 80 },
  { group: "Pseudo" },
  { mod: "+24% de Résistance élémentaire totale", check: false, min: 21 },
  { group: "Implicite" },
  { mod: "+8% à toutes les Résistances élémentaires", check: true, min: 7, range: "7–10", share: .33 },
  { group: "Préfixes", fill: "1 sur 3 · 2 libres" },
  { mod: "+125 au Score d'Évasion", check: false, min: 112, tier: 4, range: "108–141", over: { share: .52 } },
  { group: "Suffixes", fill: "1 sur 3 · 2 libres" },
  { mod: "16% d'Augmentation de la Rareté des Objets trouvés", check: false, min: 14, tier: 1, range: "15–18", over: { share: .83 } },
];
// The ring with its price block saying `note`, its tooltip `tip` open under it when given.
const stRing = (note, { fr = false, tip } = {}) => {
  const t = ST_TEXT[fr ? "fr" : "en"];
  const it = fr ? stRingFr : IMPLICIT_RING;
  const rows = fr ? stRingRowsFr : stRingRows;
  const count = rows.filter(row => row.check).length;
  const block = price({ lines: [t.rare, note] }, fr ? "fr" : "en");
  const withTip = tip ? block.replace(`<div class="msg muted">${note}</div>`,
    `<div class="msg muted app-msg-tip is-hovered">${note}<span class="app-tip">${tip}</span></div>`) : block;
  return win(topBar(), item({ ...it, filters: fr ? `${count} filtres` : `${count} filters`, stats: rows }, true),
    withTip, offers({ state: "idle" }, fr ? "fr" : "en"), foot(fr ? "poe2scout il y a 12min · GGG trade il y a 2min" : undefined));
};

// The French bow of the in-game trade (13k2e): its runes and its base, without range field, over the range column.
const stOverColumn = rows => rows.map(row => row.check !== undefined && row.min === undefined && row.mod ? { ...row, cls: "app-no-field" } : row);
const stBow = (rows, dpsTip = false) => {
  const html = item({ ...BOW_TRADE_FR, stats: rows }, true);
  return win(topBar(), dpsTip ? stDps(html, true) : html, price(BEFORE_FR, "fr"), foot("poe2scout il y a 12min · GGG trade il y a 2min"));
};

SECTIONS.unshift(["Small texts of the item block (22a): the DPS, the note of the price block, the filters without range field", [
  ["1 · Chosen · The DPS: its tooltip under the pointer, on each DPS", win(topBar(), stDps(item(BOW), false), price(RARE_ESTIMATE_BEFORE), foot())],
  ["1 · Chosen · The DPS · en français", stBow(bowTradeStatsFr, true)],
  ["2 · The note · Today: the implicit checked, still \"Check a modifier\"", stRing(ST_TEXT.en.today)],
  ["2 · The note · Rejected · A · names what compares (not the DPS of a weapon)", stRing(ST_TEXT.en.a)],
  ["2 · The note · Rejected · B · today's text, a tooltip saying what doesn't compare", stRing(ST_TEXT.en.today, { tip: ST_TEXT.en.bTip })],
  ["2 · The note · Chosen · C · what the stat must do", stRing(ST_TEXT.en.c)],
  ["2 · The note · Chosen · C · en français", stRing(ST_TEXT.fr.c, { fr: true })],
  ["3 · Filters without range field · Today: the range column kept, the runes wrap", stBow(bowTradeStatsFr)],
  ["3 · Filters without range field · Chosen: over the range column", stBow(stOverColumn(bowTradeStatsFr))],
]]);
