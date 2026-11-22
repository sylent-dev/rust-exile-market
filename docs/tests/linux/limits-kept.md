# Les limites du site d'échange gardées d'un lancement à l'autre : les tests (22p4)

## Pourquoi

Exile Market garde maintenant dans son dossier de cache (`trade-limits.json`) ce qu'il sait des limites du site d'échange : ses requêtes, les fenêtres annoncées et l'heure de la dernière recherche. Fermer l'application et la relancer ne raccourcit donc plus les 10 secondes entre deux recherches. Fais les tests d'affilée, sans m'attendre. Un ✅ suffit quand ça marche ; détaille seulement ce qui cloche. Puis dis-moi « fait ».

## Avant

1. Ferme Exile Market et Exiled Exchange 2.
2. Dans Konsole (pas dans le terminal de VS Code : son bac à sable Flatpak n'a pas X11, la fenêtre ne s'ouvre pas), dans le dossier du projet, sur la branche à tester :

   ```
   cargo build --release
   ```

   puis lance l'application avec `target/release/exile-market` : c'est cette commande que tu relanceras pendant le test (flèche du haut, Entrée).

3. Lance le jeu en **plein écran fenêtré**, dans ton repaire, avec un objet rare dans l'inventaire.
4. Le journal est dans `~/.local/state/exile-market/exile-market.log`.

## Les tests

1. **Les 10 secondes après une relance** : **Ctrl+D** sur l'objet rare, puis clique sur la **loupe** : les offres arrivent. Aussitôt, **ferme Exile Market**, **relance-le** depuis Konsole, refais **Ctrl+D** sur le même objet et clique sur la **loupe**, le tout en moins de 10 secondes après le premier clic.
   ✅ Aucune offre n'arrive tout de suite : le bloc de recherche attend, avec son décompte, puis redevient cliquable.
   ✅ Dans le journal, au lancement, la ligne « Trade site: the limits kept in trade-limits.json, N requests counted » (2 pour une première recherche : la recherche et ses offres ; plus si tu avais déjà cherché dans les 6 dernières heures), puis au clic « Trade site: not sent, N s to wait ».
   Si tu as mis plus de 10 secondes, la recherche part normalement : recommence plus vite. La ligne du lancement, elle, doit être là dans tous les cas.

2. **Test libre** : ferme et relance à d'autres moments (pendant une recherche, après plusieurs recherches, après une longue pause), puis cherche.
   ✅ Rien d'étrange : note tout ce qui te surprend, surtout une attente longue que rien n'explique.

## Résultats

| Test | Résultat | Remarque |
|---|---|---|
| 1. Les 10 secondes après une relance | | |
| 2. Test libre | | |
