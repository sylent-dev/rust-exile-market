// Section Lines without tier (22d): why a line copied from the in-game trade has no tier.
// A line the application can't tell apart shows no tier, no bar and no word (owner, 2026-10-29), in Explicit, and
// the only note is "The tiers are found from the values.": a player may take the hole for a bug. Measured on the
// draw of `random_items.rs` (2026-11-18, a throwaway counter): 11,737 lines without tier of 154,013 (7.6 %), every
// one of them a doubt between several readings; 70 % are a line of a hybrid, 21 % a line that adds up two
// modifiers, 9 % a pure modifier. What the search knows of a doubt (a hybrid or pure modifiers, a sum read another
// way, a line the best reading leaves out) doesn't tell which rolled, and one line in ten has several of these at
// once: one text for every doubt, rather than a text per cause that would often be the wrong one.
// 1. A line in doubt (pure modifiers or their hybrid, a hybrid whose lines don't follow each other, a line that adds
//    up modifiers in several ways, two readings as good, a search that stopped at its bound). A, "Uncertain tier" in
//    gray where the tier goes, why in its tooltip; B, the same word, why once in the note under the stats, without
//    tooltip; C, the reason itself under each line, without tooltip; D, nothing on the lines, a second sentence in
//    the note. Recommended: B, the word seen without the pointer on the line it is about, in the place the eye looks
//    for the tier, and the reason said once (the owner chose the reason written rather than in a tooltip for the
//    uncertain prices, 22b); C repeats a long text on two to four lines; D is far from the lines, under
//    the fold on a long item.
// 2. A value no modifier of the game data holds (an item copied before a patch, a stat the tables don't know):
//    nothing today either. "No tier found", its tooltip saying why: rare, so not in the note.
// 3. A sanctified item, whose values can each have left their tier: no tier at all, and no note (owner, 2026-11-11).
//    A note under the stats, since every line is concerned.
// Left out: the readings a line in doubt may be ("T6 or hybrid T3", the backlog entry of a tier among several), the
// lines a doubt ties together named in a tooltip (the search would keep the parts of the item), and an item whose
// base the game data doesn't know (no tier, no note, as today).

const TL_TEXT = {
  en: { uncertain: "Uncertain tier", why: "Several modifiers fit this value.<br>Nothing in the copy tells which one.",
    noteWhy: "A tier is uncertain when several modifiers fit its value.", reason: "Fits several modifiers",
    noteOnly: "A line without tier fits several modifiers.", none: "No tier found", noneWhy: "No known modifier holds this value.",
    sanctified: "The values of a sanctified item can leave their tier: none is found." },
  fr: { uncertain: "Palier incertain", noteWhy: "Un palier est incertain quand plusieurs modificateurs correspondent à sa valeur.",
    reason: "Plusieurs modificateurs possibles", none: "Aucun palier trouvé",
    sanctified: "Les valeurs d'un objet sanctifié peuvent sortir de leur palier : aucun n'est trouvé." },
};

// A tiara copied from the in-game trade: its energy shield and its life are two pure modifiers (T6 and T8) or their
// hybrid (T3), two prefixes open either way.
const TL_TIARA = { rarity: "rare", name: "Chimeric Visage", base: "Ancestral Tiara", cls: "Helmets", tags: ["Q 20%", "2 sockets", "ilvl 82"],
  props: [["Energy Shield", "342"]], req: "Requires Level 80 · 115 Int", statsLabel: "Stats (6)", filters: "5 filters" };
// Its stats: `word` under each line in doubt, `notes` under the stats, and its cold resistance as `cold`.
const tlTiaraStats = ({ word, notes = [DEDUCED_NOTE.note], cold } = {}) => [
  ...itemFilters({ base: "Ancestral Tiara", level: { value: 82, check: false }, sockets: { value: 2, check: true } }),
  { group: "Prefixes", fill: "1 of 3 · 2 open" },
  { mod: "+69 to maximum Energy Shield", check: true, min: 62, tier: 1, range: "61–73", over: { share: .94 } },
  { group: "Suffixes", fill: cold ? "2 of 3 · 1 open" : "3 of 3" },
  { mod: "18% increased Rarity of Items found", check: true, min: 16, tier: 1, range: "15–18", over: { share: 1 } },
  { mod: "+43% to Fire Resistance", check: false, min: 39, tier: 1, range: "41–45", over: { share: .95 } },
  ...(cold ? [] : [{ mod: "+40% to Cold Resistance", check: false, min: 36, tier: 2, range: "36–40", over: { share: .87 } }]),
  { group: "Explicit" },
  { mod: "30% increased Energy Shield", check: true, min: 27, after: word },
  { mod: "+30 to maximum Life", check: false, min: 27, after: word && { ...word, hover: false } },
  ...(cold ? [cold] : []),
  ...notes.map(note => ({ note })),
];
const tlTiaraWin = options => win(topBar(), item({ ...TL_TIARA, stats: tlTiaraStats(options) }, true), price(RARE_ESTIMATE_BEFORE), foot());

// The same tiara in French.
const TL_TIARA_FR = { rarity: "rare", name: "Visage chimérique", base: "Tiare ancestrale", cls: "Casques", tags: ["Q 20%", "2 châsses", "niv. 82"],
  props: [["Bouclier d'énergie", "342"]], req: "Niveau requis 80 · 115 Int", statsLabel: "Statistiques (6)", filters: "5 filtres",
  tradeLabel: "Voir sur l'échange", tradeTitle: "Ouvrir la recherche sur le site d'échange" };
const tlTiaraStatsFr = (word, notes) => [
  { group: "Objet", min: true },
  { mod: "Base : Tiare ancestrale", check: true },
  { mod: "Niveau de l'objet : 82", check: false, min: 82 },
  { mod: "Châsses : 2", check: true, min: 2 },
  { group: "Préfixes", fill: "1 sur 3 · 2 libres" },
  { mod: "+69 au Bouclier d'énergie maximal", check: true, min: 62, tier: 1, range: "61–73", over: { share: .94 } },
  { group: "Suffixes", fill: "3 sur 3" },
  { mod: "18% d'Augmentation de la Rareté des Objets trouvés", check: true, min: 16, tier: 1, range: "15–18", over: { share: 1 } },
  { mod: "+43% à la Résistance au feu", check: false, min: 39, tier: 1, range: "41–45", over: { share: .95 } },
  { mod: "+40% à la Résistance au froid", check: false, min: 36, tier: 2, range: "36–40", over: { share: .87 } },
  { group: "Explicite" },
  { mod: "30% d'Augmentation du Bouclier d'énergie", check: true, min: 27, after: word },
  { mod: "+30 de Vie maximale", check: false, min: 27, after: word },
  ...notes.map(note => ({ note })),
];
const tlTiaraWinFr = (word, notes) => win(topBar(), item({ ...TL_TIARA_FR, stats: tlTiaraStatsFr(word, notes) }, true), price(RARE_ESTIMATE_BEFORE), foot());

// A fire resistance above its tier 1 (41–45), as a patch may leave it: no modifier holds it.
const tlNoTier = word => ({ mod: "+48% to Fire Resistance", check: false, min: 43, after: word });

// The owner's sanctified sceptre as the in-game trade copies it: no tier is found from its values.
const tlSceptreStats = (lines, base, level, group) => note => [
  { group: group[0], min: true },
  { mod: base, check: true },
  { mod: level, check: false, min: 79 },
  { group: group[1] },
  { mod: lines[0], check: false },
  { mod: lines[1], check: false, min: 67 },
  { mod: lines[2], check: false, min: 74, origin: "crafted", originLabel: group[2] },
  { mod: lines[3], check: false, min: 26 },
  { mod: lines[4], check: false, min: 45 },
  ...(note ? [{ note }] : []),
];
const tlSceptre = tlSceptreStats(["Allies in your Presence deal 4 to 60 added Attack Lightning Damage", "+74 to maximum Mana",
  "Allies in your Presence deal 82% increased Damage", "29% reduced Attribute Requirements", "50% increased Presence Area of Effect"],
  "Base: Rattling Sceptre", "Item level: 79", ["Item", "Explicit", "Crafted"]);
const tlSceptreFr = tlSceptreStats(["Les Alliés dans votre Aura infligent 4 à 60 Dégâts de foudre d'Attaque supplémentaires", "+74 de Mana maximal",
  "Les Alliés dans votre Aura infligent 82% de Dégâts supplémentaires", "29% de Réduction des Prérequis d'attributs",
  "50% d'Augmentation de la Zone d'effet de l'Aura"], "Base : Sceptre cliquetant", "Niveau de l'objet : 79", ["Objet", "Explicite", "Artisanat"]);
const TL_SCEPTRE = { ...SCEPTRE_SANCTIFIED, statsLabel: "Stats (5)", filters: "1 filter" };
const TL_SCEPTRE_FR = { ...TL_SCEPTRE, name: "Glas d'entropie", base: "Sceptre cliquetant", cls: "Sceptres", tags: ["Sanctifié", "niv. 79"],
  props: [["Esprit", "100"]], req: "Niveau requis 60", statsLabel: "Statistiques (5)", filters: "1 filtre",
  tradeLabel: "Voir sur l'échange", tradeTitle: "Ouvrir la recherche sur le site d'échange" };
const tlSceptreWin = (it, stats) => win(topBar(), item({ ...it, stats }, true), price(RARE_ESTIMATE_BEFORE), foot());

SECTIONS.unshift(["Lines without tier (22d): three questions, a recommendation for each", [
  ["1 · Today · two lines in doubt: no tier, no word", tlTiaraWin()],
  ["1 · Recommended · B · “Uncertain tier”, why once in the note", tlTiaraWin({ word: { label: TL_TEXT.en.uncertain }, notes: [DEDUCED_NOTE.note, TL_TEXT.en.noteWhy] })],
  ["1 · Recommended · B · en français", tlTiaraWinFr({ label: TL_TEXT.fr.uncertain }, [DEDUCED_NOTE_FR.note, TL_TEXT.fr.noteWhy])],
  ["1 · A · “Uncertain tier”, why in its tooltip (open on the first line)", tlTiaraWin({ word: { label: TL_TEXT.en.uncertain, tip: TL_TEXT.en.why, hover: true } })],
  ["1 · C · the reason under each line, no tooltip", tlTiaraWin({ word: { label: TL_TEXT.en.reason } })],
  ["1 · C · en français", tlTiaraWinFr({ label: TL_TEXT.fr.reason }, [DEDUCED_NOTE_FR.note])],
  ["1 · D · nothing on the lines, a second sentence in the note", tlTiaraWin({ notes: [DEDUCED_NOTE.note, TL_TEXT.en.noteOnly] })],
  ["2 · Today · a value no modifier holds: nothing", tlTiaraWin({ word: { label: TL_TEXT.en.uncertain }, notes: [DEDUCED_NOTE.note, TL_TEXT.en.noteWhy], cold: tlNoTier() })],
  ["2 · Recommended · “No tier found”, why in its tooltip (open)", tlTiaraWin({ word: { label: TL_TEXT.en.uncertain }, notes: [DEDUCED_NOTE.note, TL_TEXT.en.noteWhy],
    cold: tlNoTier({ label: TL_TEXT.en.none, tip: TL_TEXT.en.noneWhy, hover: true }) })],
  ["3 · Today · a sanctified item: no tier, no note", tlSceptreWin(TL_SCEPTRE, tlSceptre())],
  ["3 · Recommended · a note under the stats", tlSceptreWin(TL_SCEPTRE, tlSceptre(TL_TEXT.en.sanctified))],
  ["3 · Recommended · en français", tlSceptreWin(TL_SCEPTRE_FR, tlSceptreFr(TL_TEXT.fr.sanctified))],
]]);
