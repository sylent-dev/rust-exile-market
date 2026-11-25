# Roadmap

Implementation steps of the features described in [specs.md](specs.md). Each step is delivered in its own pull request, which checks its box.

## Reference

Read only by the steps that need them:

- [Technical choices](docs/technical-choices.md): the libraries, the services and the main designs, with their reasons.
- [Code quality](docs/code-quality.md): the rules of the code (names, functions, types, ownership, modules, patterns, tests, refactoring), read at the start of each step that changes code.
- [Releases](docs/releases.md): how to publish a version, and what its archives hold.
- [Images](docs/images.md): the game images the application may show, and the rules of Grinding Gear Games.
- [poe2scout usage](docs/poe2scout-usage.md): the limits of the price database and the rules the application follows.
- [Trade site usage](docs/trade-site-usage.md): the risk accepted by the owner, the limits and the guardrails.
- [Game data](docs/game-data.md): the stat ids and the modifier tiers, their sources and how they fit together.
- [Backlog](backlog.md): what is noted and not being done; read when the next work is chosen (steps 20 and 22).
- [Lessons](docs/lessons.md): what working on the project taught, kept for the retrospectives (steps 19 and 23).
- [Roadmap archive](docs/roadmap-archive.md): the detail of the steps done.
- Last refactoring pass: 17e3d, ended at `c527799` (#226, 2026-11-13); the next one reads the code changed since ([refactoring-pass skill](.claude/skills/refactoring-pass/SKILL.md)).

## Models

Each step to come names the Claude model recommended for its session (owner, 2026-11-10), so that the owner switches with `/model` before starting it; a new step or sub-step gets its model when it is written here. A judgment of the difficulty, not a measure: Fable 5.2 where a quiet mistake costs most (the matching of the game data, the guardrails of the trade site, what installs code on a player's machine); Opus 5.5 for every other step (mock-up, interface, refactoring, release, documentation), never Sonnet 5.5 (owner, 2026-11-15). The subagents (`code-reviewer`, `security-auditor`, `dependency-reviewer`) name no model and run on the model of the session, but a code review, whose model follows what it reads, not the session (owner, 2026-11-12, after the review of 17e3d4b4 and 17e3d4b5, tests alone, took 18 minutes and 271k tokens on Fable 5.2): Opus 5.5 for a pull request of tests alone or of moves alone, also in a session on Fable 5.2, a miss there leaving a weaker guard or a move to look at again, not a bug; Fable 5.2 for critical code (the matching of the game data, the guardrails of the trade site, what installs code on a player's machine); the model of the session otherwise. Claude Code gives `code-reviewer` its model when it launches it, and says it with the cost of the review: one agent, rather than a copy of it per model or a switch of the session by the owner (owner's choice on 2026-11-12); the notes of a review name at their top the model that wrote them. What Opus 5.5 misses that Fable 5.2 would find isn't measured: the line of a step says which model reviewed it. If a step proves harder than its model (a review that keeps finding medium recommendations, a design that doesn't settle), say so and propose to switch.

## Path of Exile 2 1.0

Path of Exile 2 leaves early access with its version 1.0 on 2026-12-11. The modifiers, their tiers and ranges, the trade site data and maybe the format of the copied text can change. After this release (model: Fable 5.2, the game data generated again and their matching checked):

- copy real items again (English and French) and rerun the parser tests;
- check the stat ids, the pseudo stats and the filters of the trade site, and the categories of poe2scout;
- see whether a macOS client comes with 1.0 (none on 2026-11-01: `"mac": false` on Steam for the app 2694490, while Path of Exile 1 has one; the players of a Mac run the Windows client through CrossOver or Whisky). If one does, the price-check hotkey on macOS (left out of step 14a2 by the owner on 2026-11-01): `global-hotkey` (Carbon) and the copy sent with CGEvent through `objc2-core-graphics`, already a dependency of `arboard`, whose functions are safe (`CGEvent::new_keyboard_event`, `set_flags`, `post`, Ctrl read with `CGEventSource::flags_state`); the copy of the game is Ctrl+C there too (`Ctrl + C` on every system in Exiled Exchange 2, which presses Ctrl again on a Mac); the Accessibility permission, needed to send keys, asked at the first Ctrl+D only (`CGPreflightPostEventAccess`, `CGRequestPostEventAccess`);
- check the names of the properties of a waystone (`waystone_properties` in `src/item/labels.rs`; French `Efficacité des Monstres` from Exiled Exchange 2, not seen in a copy yet, but written so by the French trade site on 2026-10-31), the same names among the `properties` of `/fetch` on both trade sites (read for the tooltip of an offer, 13k5c2) and their ids among the `map_filters` of the trade site (`/data/filters`; `map_rare_monsters` is "Monster Rarity" and `map_magic_monsters` "Monster Effectiveness", checked by two searches on 2026-10-31): an unknown name is logged (`ParseWarning::UnknownWaystoneProperty`);
- check the words a Time-Lost jewel writes before its lines (`radius_skills` in `src/item/labels.rs`, English and French): reworded, a line shows them whole, without its tag of the passive skills;
- check the properties of a gem in `/fetch` (`Level` type 5, the quality type 6, `gemSockets`) and the types of its `requirements` (62 to 65; the strength, type 63, not seen yet), read for the tooltip of an offer (13k5d2);
- check the tiers and ranges given by the advanced copy, and generate the game data again (stat ids and modifier tables; the checklist of the section "Generator" of [docs/game-data.md](docs/game-data.md)), then record again the tests on the tables of the game (the `game_tests.rs` files of `src/game_data/modifiers/`, the draw of `random_items.rs` among them; the `tests.rs` files, on tables written by hand, must pass as they are); then decide on the `divisor` of a line without a number, which no table holds (parser review of 17e3, recommendation 14);
- check the bases of the modifiers of the essences and of the alloys (`ESSENCES` and `ALLOYS` in `tools/game-data/src/modifiers/currencies.rs`) against Craft of Exile, poe2db and the trade site, and add the new ones;
- compare the crafts of the game (the crafted modifiers that aren't an alloy's, e.g. the `Motivating` of the owner's sceptre) with the lines of the alloys: crafts have no table, so a line marked crafted from the in-game trade that only an alloy holds is named "Alloy" (`Table::may_hold` in `src/game_data/modifiers/tables.rs`; no case known on 2026-10-31, second code review of 13k3d2); if a craft shares a line and a range with an alloy on a base, name no currency for that line.
- then a second refactoring pass (model: Opus 5.5), with the `refactoring-pass` skill, on the code changed since the last pass: the code that changed for 1.0, what the reports of the first players pointed at (owner, 2026-11-11), and the section Code of [backlog.md](backlog.md).

## Steps

- [x] **0. Mock-ups** (#4)
- [x] **1. Application window** (#5)
- [x] **2. Clipboard watching** (#7)
- [x] **3. Item parser** (#9)
- [x] **4. Interface language** (#10)
- [x] **5. Settings** (#11)
- [x] **6. Title bar** (#12, #13, #14)
- [x] **7. Price data** (#15, #16)
- [x] **8. French item names** (#17)
- [x] **9. Price display** (#18, #19)
- [x] **10. Item stats and filters** (#20, #23, #25, #26)
- [x] **11. Trade site client** (#28)
- [x] **12. Offers display** (#29, #30)
- [x] **13. Estimated prices** (fifty-four pull requests, #31 to #148)
- [x] **14. Overlay POC** (fifteen pull requests, #150 to #165)
- [x] **15. Update notice** (#166)
- [x] **16. Bug report** (#167, #168, #169)
- [x] **17. First release** (#170 to #233; v0.1.0 published on 2026-11-14)
- [x] **18. Donation method** (#234, #236)
- [x] **Urgent fix: the `<<set:…>>` tags in the names of the offers** (#241, v0.1.1): the tooltip of an offer showed the tags the trade site writes before the name of a rare item, taken for font sizes.
- [x] **Urgent fix: the grammar of the names of the offers** (#244, #245, v0.1.2; model: Opus 5.5): the `<<set:…>>` tags of #241 are the forms of the first word of a name (gender, number), which choose its next words, `<if:MS>{ courroucé}<elif:FS>{ courroucée}`, on the French trade site: the choices showed in the tooltip, after a space. Resolved in the name and in the whisper, checked on real answers of `/fetch` (French and English sites). One code review on Opus 5.5, 6 minutes and 49,000 tokens: one medium and four low, all applied. The security audit of the diff (Opus 5.5, 6 minutes, 64,000 tokens) found the reading quadratic on a tag left open (a megabyte in 21 s, from the trade site alone) and a tag closing far away hiding the text between, the price of a whisper: each tag given up at its first unexpected character, and the forms set kept once each (#245; one code review on Opus 5.5, 7 minutes and 61,000 tokens: one high, the forms set again and again still quadratic, and four low, applied but `split_at`, which `clippy.toml` forbids).
- [x] **19. First retrospective** (#237, #238, #240, #243, #247, #248; rust-desktop-template#1 to #9, rust-cli-template#29 to #34, ts-astro-static-website-template#18, #19)
- [ ] **20. More settings** (model: Opus 5.5): no player has asked for anything yet (no issue three days after v0.1.0, and most players are expected with Path of Exile 2 1.0), so in two parts (owner, 2026-11-17): first the settings that need no player report, then the polish of 22a to 22d below, then the pass of 1.0; then the entries of [backlog.md](backlog.md) (Features, Settings) the players ask for.
  - [x] **20a, 20b, the first part** (#249 to #252): closing with the game, the pseudo totals shown or hidden.
- [ ] **21. Automatic updates** (model: Fable 5.2): after the first release, if the application finds its players: the new version downloaded in the background and installed at the next launch, like Discord, with a signature checked before installing anything; Velopack (a real installer for Windows and macOS) or the `self_update` crate (the executable replaced), and Flathub for Linux, whose system updates it; to decide with the installers of the first release. The new version started once the old one has quit: the lock of 22m makes a launch while another runs quit with "already open".
- [ ] **22. Polish** (model: Opus 5.5): the last touches, picked with the owner from [backlog.md](backlog.md): its sections Polish, Edge cases and Waiting on others, and the features of step 22. 22a to 22d come before the pass of 1.0, right after 20b (owner, 2026-11-17: what needs no player report nor the data of 1.0, and surely brings value); each one mock-up first:
  - [x] **22a, the small texts of the item block** (#253)
  - [x] **22b, the uncertain prices** (#254)
  - [x] **22c, the error texts** (#255)
  - [x] **22d, why a line copied from the in-game trade has no tier** (#256)
  - 22e to 22h, the polish that needs neither a player report nor the data of 1.0 either, before a release v0.2.0 (owner, 2026-11-18); each one mock-up first, with a stop for the owner's approval of the mock-up before its code. 22e, the offers listed long ago faded, dropped by the owner on 2026-11-18 after its mock-up.
  - [x] **22f, the name of the orb under the pointer, and the class searched cut by "…"** (#257)
  - [x] **22g, the tier of a waystone searched between a minimum and a maximum** (#258)
  - [x] **22h, one other form of the roll bar** (#259)
  - [x] **22i, the unchecked checkboxes and the min–max fields in soft relief** (#262, #265)
  - 22j to 22m, the entries of [backlog.md](backlog.md) that depend on nothing (no real copy, no data of 1.0, no Windows tester), before the pass of 1.0 (owner, 2026-11-20):
  - [x] **22j, the runs of `main` no longer cancelled, and two checks of the security audit** (#266)
  - [x] **22k, the wait of a refusal of the trade site kept from one launch to the next** (#267)
  - [x] **22l, two modifiers of one type refused in the best reading of a copy from the in-game trade** (#268)
  - [x] **22m, a single Exile Market at once** (#269)
  - [x] **22n, the call to estimate before the search** (#271)
  - 22o and 22p, the last entries of [backlog.md](backlog.md) that depend on nothing, before a release v0.2.2 (owner, 2026-11-22):
  - [x] **22o, the dialog of the system in a module of its own, and the message of kdialog kept as written** (#272)
  - [x] **22p, the guardrails of the trade site kept from one launch to the next and shared by two Exile Market** (#273, #274)
  - 22q to 22t, before a release and the pass of 1.0 (owner, 2026-11-25): the three low of the security audit of v0.2.2, then the first requests of a player (issues #279, #280), and the touch of #277 last, since it waits for the logs of a probe run by its reporter:
  - [ ] **22q, three cases of the files of the cache that 22p reads** (model: Fable 5.2, with the review loop of critical code, a guardrail of the trade site; the security audit of v0.2.2, 2026-11-22, three low; about 20 lines and three tests). A `trade-limits.json` written by hand can hold requests marked counted that were never sent, which hide the requests of another tool from the counts of the trade site, the plainest form an `answered` before its `sent` (`Reading::hit` in `src/trade/limits/kept.rs`): at the reading, `sent` brought back to `answered` at the latest, and the sentence of `docs/trade-site-usage.md` that says a file written by hand is read on the safe side corrected. `write_atomically` (`src/files.rs`) writes every process to the same `<name>.new`, so two Exile Market writing one file at the same instant (only when the lock of 22m couldn't be taken) can leave a broken JSON and lose the wait: a temporary name of each process, its PID in it. The files of the cache read whole (`fs::read` in `src/files.rs`), with no bound nor a check that each is a plain file, before each request since 22p1: a named pipe in place of `trade-refusal.json` blocks the reading under the mutex of `Refusal`, and the launch too: `is_file()` checked, then the reading bounded with `take(max)`. Commits: the archive of the release v0.2.2 and these lines; `sent` brought back to `answered`, with its test and the doc; the temporary name of each process, with its test; the files of the cache read bounded, with its test.
  - [ ] **22r, the price-check hotkey over an empty cell closes the window over the game** (model: Opus 5.5; [issue #279](https://github.com/sylent-dev/rust-exile-market/issues/279), a player on a handheld PC under Windows, the ROG Ally, where closing the window takes a double tap outside it): over another item the hotkey keeps pricing it, so that a button mapped to Ctrl+D closes the window too, with no setting more. Mock-up first, with a stop for the owner's approval before its code; the split and its commits at its start. Left out for now: Esc closing it as an option (Exile Market would take Esc from the game while the window is open), and a Close button (`specs.md`, Closing).
  - [ ] **22s, the status of the sellers searched** (model: Opus 5.5; [issue #280](https://github.com/sylent-dev/rust-exile-market/issues/280)): the search asks `online` today (`src/trade/query.rs`), the sellers online or away; the trade site also offers Instant Buyout, Instant Buyout and In Person, In Person (Online in League) and Any. An offer bought at once is a real price even with its seller offline, so the default of every player may change, not only a setting. First a measure on the trade site, within [its guardrails](docs/trade-site-usage.md) (the ids of the options, the share of the offers bought at once, what their answer holds: the whisper, the status), then a mock-up (the default, maybe a choice, the **Copy whisper** pill and the dot of an offer bought at once), then the code; the split at its start.
  - [ ] **22t, a single tap outside the window over the game closes it on a Windows touchscreen** (model: Opus 5.5; [issue #277](https://github.com/sylent-dev/rust-exile-market/issues/277)): a tap and a long press outside it leave it open, a double tap closes it (the reporter, on a ROG Ally, 2026-11-25). The clicks are read every 20 ms, a button held at one reading and not at the one before (`src/overlay/clicks.rs`, `src/overlay/windows.rs`): Windows likely sends the press and the release of a tap at once, between two readings. Depends on: the logs of the probe of the branch `explore/windows-touch-probe` (never merged), asked of the reporter on 2026-11-25; last for that reason. The fix, chosen by the owner from the logs: a press seen between two readings if Windows reports it; otherwise another way to tell a tap, mock-up first. If the logs never come, the case goes to [backlog.md](backlog.md).
- [x] **Release v0.2.0** (#260, #261; v0.2.0 published on 2026-11-19)
- [x] **Release v0.2.1** (#270; v0.2.1 published on 2026-11-21)
- [x] **Release v0.2.2** (#275; v0.2.2 published on 2026-11-22)
- [ ] **23. Second retrospective** (model: Opus 5.5): much later, after the polish and the players' reports (owner, 2026-11-17), the same sorting on the lessons written since the first one, with the players' reports as well (the automatic updates and the installers of step 21 above all, for the template).
- [ ] **24. Price fallback** (model: Fable 5.2): a plan B, after the polish, for when poe2scout can't be reached: the price of a currency or a unique from the trade site (its currency exchange, or the median of its cheapest offers), on a click only, with a search block of its own in the price block; the cached prices stay shown with their age until then. Within the [Trade site usage](docs/trade-site-usage.md) guardrails: never when an item is copied, never in the background, never a local copy of poe2scout (poe2scout reads the currency exchange API of Grinding Gear Games and the trade site from one server; every player doing it would be far more requests). Mock-up first.
- [ ] **25. Responsive layout and price history** (model: Opus 5.5): at the very end, after the price fallback (owner's choice on 2026-10-31): wireframes then mock-up of the medium and wide layouts with the visual style, then these layouts in the application, with the 7-day history chart in the wide layout.
