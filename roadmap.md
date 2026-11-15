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
- then a second refactoring pass (model: Opus 5.5), with the method of 17e3d: the code that changed for 1.0, what the reports of the first players pointed at (owner, 2026-11-11), and the section Code of [backlog.md](backlog.md).

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
- [x] **Urgent fix: the font sizes in the names of the offers** (#241, v0.1.1): the tooltip of an offer showed the `<<set:…>>` tags the trade site writes before a long name.
- [ ] **19. First retrospective** (model: Opus 5.5): right after the first release (owner, 2026-11-14: like the refactoring, better often than once and big; another project of the owner needs the template now, and the players' reports take time to come), the lessons of [docs/lessons.md](docs/lessons.md) sorted one by one, with the owner: into a project template of a Rust desktop application (to create, with the technologies chosen here: egui and eframe, X11 on Linux, Fluent, the Excalidraw wireframes and the HTML mock-ups, the CI and the releases, the agents, the skills, the settings that hide the unused skills and connectors), into the `CLAUDE.md`, the skills or the agents of this project, or dropped; the rule that writes the lessons goes into the template too. Only a command-line template and a static website template exist today. In four sub-steps (Claude Code's split, agreed by the owner on 2026-11-14), the template before the changes of this project, since another project waits for it:
  - [x] **19a1, the first half of the lessons sorted** (#237)
  - [x] **19a2, the second half sorted** (#238)
  - [ ] **19b, the template of a Rust desktop application**: a new repository, `sylent-dev/rust-desktop-template`, public and marked as a template like `rust-cli-template`, created from it (`gh repo create --template`) rather than by emptying this project; a rule changed in a `CLAUDE.md` goes into `rust-cli-template` too. One pull request of this project per sub-step of 19b, its box of each part checked once that part is merged in its template. In three sub-steps (Claude Code's split, agreed by the owner on 2026-11-15):
    - [x] **19b1, the repository and the skeleton** (rust-desktop-template#1)
    - [x] **19b2, the lessons into the desktop template** (#240; rust-desktop-template#3 to #6)
    - [ ] **19b3, the same rules into the other templates**: the lessons sorted for `rust-cli-template`, and for `astro-static-website-template` when they name the three templates. Documentation alone, no code review. The generic rules sorted "Both" go into `rust-cli-template` too, without what belongs to a desktop application (Clippy for Windows and macOS, the desktop probes, the screenshots of egui) (owner, 2026-11-15). One pull request of this project for the whole of 19b3. In three parts, one pull request of a template each (Claude Code's split, agreed by the owner on 2026-11-15), with a stop for the owner's word between two:
      - [ ] **19b3a, `rust-cli-template`: the rules**: `CLAUDE.md` (the section Lessons and the generic rules), `docs/code-quality.md`, `docs/lessons.md` and `docs/releases.md`, taken from the desktop template; the lesson of 2026-11-15 on a pull request of the version alone and the audit of a patch release, into `rust-cli-template` and, by a small pull request of its own, into the desktop template.
      - [ ] **19b3b, `rust-cli-template`: the agents, the skills and the roadmap**: the `security-auditor` agent (new), `code-reviewer` and `dependency-reviewer` brought level; the `html-mockups` skill brought level and `tidy-io-folders` added (neither `hand-test-guides` nor `egui-screenshots`); the Models section of `roadmap.md` (Opus 5.5 for every step but critical code), the quality review before the first release, the donation method and the two retrospectives.
      - [ ] **19b3c, `astro-static-website-template`**: the lessons that name the three templates (`CLAUDE.md`: the short note of the session state, the mutations, a notification isn't a signal, when to hold a retrospective; `roadmap.md`: the Models section and the quality review) and the two sorted for it alone (the hosting sized to the need, a DNS name created before it is queried).
  - [ ] **19c, the changes of this project**: the lessons sorted into this project's `CLAUDE.md`, its skills and its agents.
- [ ] **20. More settings** (model: Opus 5.5): after the first release, the settings the players may want: the entries of [backlog.md](backlog.md) (Features, Settings), chosen with the owner from what the players ask.
- [ ] **21. Automatic updates** (model: Fable 5.2): after the first release, if the application finds its players: the new version downloaded in the background and installed at the next launch, like Discord, with a signature checked before installing anything; Velopack (a real installer for Windows and macOS) or the `self_update` crate (the executable replaced), and Flathub for Linux, whose system updates it; to decide with the installers of the first release.
- [ ] **22. Polish** (model: Opus 5.5): the last touches, picked with the owner from [backlog.md](backlog.md): its sections Polish, Edge cases and Waiting on others, and the features of step 22.
- [ ] **23. Second retrospective** (model: Opus 5.5): right after the polish, the same sorting on the lessons written since the first one, with the players' reports as well (the automatic updates and the installers of step 21 above all, for the template).
- [ ] **24. Price fallback** (model: Fable 5.2): a plan B, after the polish, for when poe2scout can't be reached: the price of a currency or a unique from the trade site (its currency exchange, or the median of its cheapest offers), on a click only, with a search block of its own in the price block; the cached prices stay shown with their age until then. Within the [Trade site usage](docs/trade-site-usage.md) guardrails: never when an item is copied, never in the background, never a local copy of poe2scout (poe2scout reads the currency exchange API of Grinding Gear Games and the trade site from one server; every player doing it would be far more requests). Mock-up first.
- [ ] **25. Responsive layout and price history** (model: Opus 5.5): at the very end, after the price fallback (owner's choice on 2026-10-31): wireframes then mock-up of the medium and wide layouts with the visual style, then these layouts in the application, with the 7-day history chart in the wide layout.
