// Section Already open (22m): a second launch of Exile Market while it already runs quits with a message, rather
// than bringing the first window forward, which would need the two to talk (owner, 2026-11-20). The second launch
// can't draw anything of its own before quitting either: the dialog of the system of 17a (css/system.css), in the
// language of the settings. A, chosen by the owner on 2026-11-21: the sign of information, since nothing failed, and
// where to look for the window, the real question of the player; its color is the system's. Rejected: B, the error
// sign of 17a and one sentence; C, A with the hotkey, which may be turned off or changed.

const AO_TEXTS = {
  A: {
    en: ["Exile Market is already open.", "Its window may be minimized, or behind the game: look for it in the taskbar."],
    fr: ["Exile Market est déjà ouvert.", "Sa fenêtre est peut-être réduite, ou derrière le jeu : cherchez-la dans la barre des tâches."],
  },
  B: { en: ["Exile Market is already open."] },
  C: {
    en: ["Exile Market is already open.", "Its window may be minimized, or behind the game: look for it in the taskbar.",
      "In the game, Ctrl+D shows the item under the pointer over the game."],
  },
};
const aoDialog = (lead, lang, sign) => `<div class="sys-dialog"><div class="sys-dialog-title">Exile Market</div>
  <div class="sys-dialog-body"><span class="sys-dialog-sign${sign === "i" ? " is-info" : ""}">${sign}</span>
    <div class="sys-dialog-text">${AO_TEXTS[lead][lang].map(line => `<p>${line}</p>`).join("")}</div></div>
  <div class="sys-dialog-buttons"><span>OK</span></div></div>`;

SECTIONS.unshift(["Already open (22m): the dialog of the system at a second launch, which then quits", [
  ["Chosen · A · the sign of information, and where the window is", aoDialog("A", "en", "i")],
  ["Chosen · A · en français", aoDialog("A", "fr", "i")],
  ["Rejected · B · the error sign of 17a, one sentence", aoDialog("B", "en", "!")],
  ["Rejected · C · A and the hotkey, which may be off or changed", aoDialog("C", "en", "i")],
]]);
