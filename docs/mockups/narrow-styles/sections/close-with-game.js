// Section Close with the game (20a1): a checkbox of the settings, unchecked by default (owner, 2026-11-06):
// Exile Market quits when Path of Exile 2 closes, only once the game was seen running, so that launching the tool
// before the game doesn't close it; a game that crashes or closes for a patch closes it too (owner, 2026-11-17).
// Three places drawn on the settings of today (the Ko-fi line of 18b): A, a second checkbox under "Always on top",
// the two behaviors of the window together; B, A with a faint line under it that says when it acts; C, a field of
// its own under the hotkey, "Game", with the same line, the two rows that follow the game together. In B and C,
// Always on top gets a faint line too, short, the modes of the game left to the specifications (owner, 2026-11-17).
// D, the owner's idea on 2026-11-17: B with each checkbox alone on the left, its line right under its title, and
// the rows closer; the line of the hotkey either stops before its key (D) or runs under it (D, wide). D wide chosen
// by the owner on 2026-11-17.

const CG_TEXT = {
  en: { box: "Close with the game", help: "Exile Market quits when Path of Exile 2 closes.", label: "Game",
    topHelp: "Keeps the window above the game." },
  fr: { box: "Fermer avec le jeu", help: "Exile Market se ferme quand Path of Exile 2 se ferme.", label: "Jeu",
    topHelp: "Garde la fenêtre au-dessus du jeu." },
};
const cgBox = (on, text) =>
  `<div class="toggle"><span class="check ${on ? "on" : ""}">${on ? ICON.check(11) : ""}</span>${text}</div>`;
const cgHelp = text => `<span class="app-hotkey-note">${text}</span>`;

// `place` "a", "b" or "c"; `on` the box checked.
const cgWin = ({ place = "a", on = false, fr = false } = {}) => {
  const t = CG_TEXT[fr ? "fr" : "en"];
  // The help of the hotkey in French too, which the settings of `hlWin` leave in English.
  const html = kofiB({ chosen: true, fr }).replace(HK_HELP,
    fr ? "Copie l'objet sous le curseur et affiche son prix par-dessus le jeu." : HK_HELP);
  const top = fr ? "Toujours au premier plan" : "Always on top";
  const afterTop = `<div class="toggle"><span class="check"></span>${top}</div></div>`;
  // B and C: Always on top gets its line too (owner, 2026-11-17), so that every checkbox says what it does.
  const topLine = `<div class="toggle"><span class="check"></span>${top}</div>${cgHelp(t.topHelp)}</div>`;
  if (place === "c") {
    return html.replace(afterTop, topLine).replace(`<div class="app-kofi-line">`,
      `<div class="field"><span class="label">${t.label}</span>${cgBox(on, t.box)}${cgHelp(t.help)}</div>
    <div class="app-kofi-line">`);
  }
  if (place === "b") {
    return html.replace(afterTop, `${topLine}
    <div class="field">${cgBox(on, t.box)}${cgHelp(t.help)}</div>`);
  }
  return html.replace(afterTop, `${afterTop}
    <div class="field">${cgBox(on, t.box)}</div>`);
};

// D: a checkbox row, its line under its title; `key` the key button of the hotkey on the right of the title.
const cgRow = (on, title, help, { key = "", wide = false } = {}) =>
  `<div class="app-check-row ${wide ? "is-wide" : ""}"><span class="check ${on ? "on" : ""}">${on ? ICON.check(11) : ""}</span>` +
  `<span>${title}</span>${key}${cgHelp(help)}</div>`;
const cgWinD = ({ on = true, fr = false, wide = false } = {}) => {
  const t = CG_TEXT[fr ? "fr" : "en"];
  const top = fr ? "Toujours au premier plan" : "Always on top";
  const hkHelp = fr ? "Copie l'objet sous le curseur et affiche son prix par-dessus le jeu." : HK_HELP;
  return kofiB({ chosen: true, fr })
    .replace(`<div class="field"><div class="toggle"><span class="check"></span>${top}</div></div>`,
      `<div class="field app-check-field">${cgRow(false, top, t.topHelp)}</div>
    <div class="field app-check-field">${cgRow(on, t.box, t.help)}</div>`)
    .replace(/<div class="app-hotkey-row"><div class="toggle">(<span class="check[^"]*">[\s\S]*?<\/span>)([^<]*)<\/div>([\s\S]*?)<\/div>\s*<span class="app-hotkey-note[^"]*">[^<]*<\/span>/,
      (_, box, title, key) => cgRow(true, title, hkHelp, { key, wide }));
};

SECTIONS.unshift(["Close with the game (20a1): D wide chosen, each line under its title, the rows closer", [
  ["Chosen · D wide · the line of the hotkey runs under its key", cgWinD({ wide: true })],
  ["Chosen · D wide · en français", cgWinD({ wide: true, fr: true })],
  ["Chosen · D wide · unchecked (the default)", cgWinD({ wide: true, on: false })],
  ["Rejected · D · the line of the hotkey stops before its key, on two lines", cgWinD()],
  ["Rejected · A · under Always on top, no line", cgWin({ place: "a", on: true })],
  ["Rejected · B · A with a faint line under each box", cgWin({ place: "b", on: true })],
  ["Rejected · C · a field of its own, Game, under the hotkey", cgWin({ place: "c", on: true })],
]]);
