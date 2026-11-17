# Fermer avec le jeu (20a2b) : les tests

## Pourquoi

Les réglages ont une nouvelle case, « Close with the game » : cochée, Exile Market se ferme quand Path of Exile 2 se ferme, mais seulement une fois le jeu vu ouvert, pour que lancer l'outil avant le jeu ne le ferme pas. Le jeu est dit fermé quand sa fenêtre a disparu 5 secondes de suite. Les tests vérifient que l'application se ferme quand il faut, et seulement quand il faut. Fais-les d'affilée ; un ✅ suffit quand ça marche, détaille seulement ce qui cloche.

Pour lire le journal, garde un second terminal ouvert :

```
tail -f ~/.local/state/exile-market/exile-market.log
```

## Avant

1. Ferme Exile Market et le jeu.
2. Dans Konsole (pas dans le terminal de VS Code : son bac à sable Flatpak n'a pas X11), dans le dossier du projet, sur la branche `feat/close-with-game` :

   ```
   cargo run --release
   ```

## Les tests

1. **Cochée, l'outil lancé avant le jeu** : ouvre les réglages (l'engrenage) et coche **Close with the game**, puis attends 10 secondes sans lancer le jeu. Lance ensuite le jeu par Steam et va dans ton repaire. Déplace et redimensionne un peu la fenêtre d'Exile Market, puis quitte le jeu (menu, Quitter vers le bureau).
   ✅ Sous la case, la ligne pâle « Exile Market quits when Path of Exile 2 closes. » ; l'application reste ouverte tant que le jeu n'est pas lancé. Au plus 5 secondes après la fermeture du jeu, elle se ferme, et le journal finit par « Path of Exile 2 closed », « Closing with the game », puis « Exile Market closed ». Relancée, elle s'ouvre à la place et à la taille où tu l'avais laissée, la case toujours cochée.

2. **Décochée** : relance le jeu, décoche la case, puis quitte le jeu.
   ✅ Exile Market reste ouverte ; le journal dit « Path of Exile 2 closed », sans « Closing with the game ».

3. **Cochée pendant que le jeu tourne** : ferme Exile Market, lance le jeu, puis relance Exile Market (`cargo run --release`), coche la case et quitte le jeu.
   ✅ Exile Market se ferme au plus 5 secondes après le jeu.

4. **Un changement du mode d'affichage** : case cochée, relance le jeu, puis, dans ses options graphiques, passe du **plein écran fenêtré** à **fenêtré**, puis reviens au plein écran fenêtré.
   ✅ Exile Market reste ouverte ; si le journal écrit « Path of Exile 2 closed », note-le dans la remarque, avec le temps que le jeu a mis à rouvrir sa fenêtre.

5. **Tes propres scénarios** : ce que tu veux essayer d'autre (en français, un retour au menu des personnages, un crash du jeu…).

## Résultats

| Test | Résultat | Remarque |
|---|---|---|
| 1. Cochée, l'outil lancé avant le jeu | | |
| 2. Décochée | | |
| 3. Cochée pendant que le jeu tourne | | |
| 4. Un changement du mode d'affichage | | |
| 5. Tes propres scénarios | | |
