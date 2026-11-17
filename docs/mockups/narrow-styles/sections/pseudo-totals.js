// Section Pseudo totals (20b): a setting that shows or hides the Pseudo group of a rare or magic item, shown by
// default so that nothing changes for whoever leaves it. Hidden, a total isn't searched, and the lines it would
// count start as when no total counts them: checked on a magic item; on a rare, whose modifiers start unchecked,
// the base is left alone, and the price block asks for a modifier.
// Three places: A, a third checkbox row under Close with the game; B, the same row under a label of its own,
// "Search", to set the settings of the search apart from those of the window; C, no setting: a faint "Hide" on
// the right of the title of the group, which folds it and is remembered, "Show" bringing it back.
// D, the owner's idea on 2026-11-17: every group of the settings under a label, Always on top and Close with the
// game under "Game", then the hotkey, then the pseudo totals under "Search"; the height left aside, the settings
// being few. D chosen by the owner on 2026-11-17, with its texts.

const PT_TEXT = {
  en: { box: "Show the pseudo totals", help: "Resistances, life, mana and attributes added up, among the filters.",
    label: "Search", game: "Game", hide: "Hide", show: "Show" },
  fr: { box: "Afficher les totaux pseudo", help: "Résistances, vie, mana et attributs additionnés, parmi les filtres.",
    label: "Recherche", game: "Jeu", hide: "Masquer", show: "Afficher" },
};

// A and B: the settings of 20a2b, a row of the pseudo totals after Close with the game; `label` for B.
const ptSettings = ({ on = true, fr = false, label = false } = {}) => {
  const t = PT_TEXT[fr ? "fr" : "en"];
  const close = CG_TEXT[fr ? "fr" : "en"];
  const closeRow = `<div class="field app-check-field">${cgRow(false, close.box, close.help)}</div>`;
  const row = cgRow(on, t.box, t.help);
  const field = label
    ? `<div class="field"><span class="label">${t.label}</span>${row}</div>`
    : `<div class="field app-check-field">${row}</div>`;
  return cgWinD({ wide: true, on: false, fr }).replace(closeRow, `${closeRow}\n    ${field}`);
};

// D: Always on top and Close with the game under "Game", the pseudo totals under "Search" after the hotkey.
const ptSettingsD = ({ on = true, fr = false } = {}) => {
  const t = PT_TEXT[fr ? "fr" : "en"];
  const close = CG_TEXT[fr ? "fr" : "en"];
  const top = fr ? "Toujours au premier plan" : "Always on top";
  const topRow = `<div class="field app-check-field">${cgRow(false, top, close.topHelp)}</div>`;
  return cgWinD({ wide: true, on: false, fr })
    .replace(topRow, `<div class="field"><span class="label">${t.game}</span>${cgRow(false, top, close.topHelp)}</div>`)
    .replace(`<div class="app-kofi-line">`,
      `<div class="field"><span class="label">${t.label}</span>${cgRow(on, t.box, t.help)}</div>
    <div class="app-kofi-line">`);
};

// C: the title of the Pseudo group with its link, its rows left out while folded.
const ptFold = (rows, folded, fr) => {
  const t = PT_TEXT[fr ? "fr" : "en"];
  let inPseudo = false;
  return rows.flatMap(row => {
    if (row.group) inPseudo = row.group === "Pseudo";
    if (row.group === "Pseudo") return [{ group: `Pseudo</span><span class="app-group-toggle">${folded ? t.show : t.hide}` }];
    return inPseudo && folded ? [] : [row];
  });
};
// The ring without its Pseudo group (A or B unchecked).
const ptHidden = rows => {
  let inPseudo = false;
  return rows.filter(row => {
    if (row.group) inPseudo = row.group === "Pseudo";
    return !inPseudo;
  });
};
// The Breach ring: its three totals checked, its modifiers unchecked (17e3f).
const PT_RING = [RULE_BREACH, ruleBreachRows];

SECTIONS.unshift(["Pseudo totals (20b): D chosen, every group of the settings under a label", [
  ["Chosen · D · every group under a label: Game, Hotkey, Search", ptSettingsD()],
  ["Chosen · D · en français", ptSettingsD({ fr: true })],
  ["Rejected · A · a third checkbox row, under Close with the game (checked, the default)", ptSettings()],
  ["Rejected · A · en français", ptSettings({ fr: true })],
  ["Rejected · B · the same row under a label of its own, Search", ptSettings({ label: true })],
  ["Chosen · shown (the default): the rare ring as today", ruleWin(PT_RING[0], PT_RING[1], true)],
  ["Chosen · hidden: the ring searched by its base alone, no estimate until a modifier is checked", ruleWin(PT_RING[0], ptHidden(PT_RING[1]), true)],
  ["Rejected · C · no setting: Hide on the title of the group", ruleWin(PT_RING[0], ptFold(PT_RING[1], false), true)],
  ["Rejected · C · folded and remembered: Show brings it back", ruleWin(PT_RING[0], ptFold(PT_RING[1], true), true)],
]]);
