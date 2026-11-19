// Section Soft relief (22i): the unchecked checkboxes and the min–max fields in soft relief, neumorphism, on the
// color of the title bar and of the status bar (owner, 2026-11-19, a test of style): every checkbox (the filters,
// the settings) while unchecked and every min–max field (the filters, the tier of a waystone); the checked box
// stays as it is, already very visible. The lead follows the tab "Relief" of the header, the shadows the sliders
// Depth, Softness, Dark and Light; the leads are drawn in css/soft-relief.css, each one egui can draw.
// The board shows every lead side by side, whatever the tab.

const SR_LEADS = [
  [1, "1 today"], [2, "2 flat"], [3, "3 raised"], [4, "4 pressed in"], [5, "5 bevel"], [6, "6 pressed, line"],
];
const srField = (min, max = "", focused = false) =>
  `<span class="input app-range${focused ? " is-focused" : ""}"><input value="${min}" inputmode="decimal" aria-label="Minimum">` +
  `<i>–</i><input value="${max}" inputmode="decimal" aria-label="Maximum"></span>`;
const srBoard = () => `<div class="win"><div class="app-soft-board">
  <div class="app-soft-row app-soft-head"><b></b><span>Off</span><span>On</span><span>Empty</span><span>Filled</span><span>Typing</span></div>
  ${SR_LEADS.map(([lead, title]) => `<div class="app-soft app-soft-row" data-lead="${lead}"><b>${title}</b>` +
    `<div><span class="check"></span></div><div><span class="check on">${ICON.check(11)}</span></div>` +
    `<div>${srField("")}</div><div>${srField(80)}</div><div>${srField(11, 13, true)}</div></div>`).join("")}
</div></div>`;
// A window of the section, its lead after the tab "Relief".
const srWin = html => `<div class="app-soft">${html}</div>`;

SECTIONS.unshift(["Soft relief (22i): the unchecked checkboxes and the min–max fields, on the color of the bars", [
  ["Every lead · the states, whatever the tab", srBoard()],
  ["The lead of the tab · Ancestral Mail, its filters", srWin(win(topBar(), item({ ...MAIL, stats: mailStats() }, true), price(RARE_ESTIMATE_BEFORE), foot()))],
  ["The lead of the tab · a waystone, its tier 11 – 13", srWin(wtrWin(wtrStatsB(11, 13)))],
  ["The lead of the tab · the settings", srWin(ptSettingsD())],
]]);
