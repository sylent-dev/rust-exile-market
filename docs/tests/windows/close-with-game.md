# Fermer avec le jeu, sous Windows : les tests (20a2b)

Pour le testeur Windows. Les réglages ont une nouvelle case, « Close with the game » : cochée, Exile Market se ferme quand Path of Exile 2 se ferme, mais seulement une fois le jeu vu ouvert, pour que lancer l'outil avant le jeu ne le ferme pas. Le jeu est dit fermé quand ses fenêtres ont disparu 5 secondes de suite (Exile Market les relit chaque seconde, donc au plus 6 secondes après le jeu). Fais les tests d'affilée. Un ✅ suffit quand ça marche ; détaille seulement ce qui cloche. Renvoie ensuite le tableau et le fichier de log.

## Avant

1. Ferme Exile Market et le jeu.
2. Télécharge la nouvelle archive Windows `exile-market-windows-x64.zip` (lien donné par le propriétaire ; il faut être connecté à un compte GitHub pour la télécharger), décompresse-la, puis lance `exile-market.exe`. Si Windows SmartScreen bloque : « Informations complémentaires », puis « Exécuter quand même ».
3. Le log est dans `%LOCALAPPDATA%\exile-market\data\exile-market.log` (colle ce chemin dans la barre d'adresse de l'Explorateur). Il est écrit au fur et à mesure, et remplacé à chaque lancement d'Exile Market (le précédent devient `exile-market.previous.log`) : lis-le **avant** de relancer l'application, ou ouvre le précédent.

## Les tests

1. **Cochée, l'outil lancé avant le jeu** : ouvre les réglages (l'engrenage) et coche **Close with the game**, puis attends 10 secondes sans lancer le jeu. Lance ensuite Path of Exile 2 en **plein écran fenêtré** et va dans ton repaire. Quitte le jeu (menu, Quitter vers le bureau), puis, aussitôt, déplace et redimensionne un peu la fenêtre d'Exile Market avant qu'elle se ferme.
   ✅ Sous la case, la ligne pâle « Exile Market quits when Path of Exile 2 closes. » ; l'application reste ouverte tant que le jeu n'est pas lancé. Au plus 6 secondes après la fermeture du jeu, elle se ferme, et le log finit par « Path of Exile 2 closed », « Closing with the game », puis « Exile Market closed » (lis le log maintenant). Relancée, elle s'ouvre à la place et à la taille où tu l'avais laissée, la case toujours cochée.

2. **Décochée** : relance le jeu, décoche la case, puis quitte le jeu.
   ✅ Exile Market reste ouverte ; le log dit « Path of Exile 2 closed », sans « Closing with the game ».

3. **Cochée pendant que le jeu tourne** : ferme Exile Market, lance le jeu, puis relance `exile-market.exe`, coche la case et quitte le jeu.
   ✅ Exile Market se ferme au plus 6 secondes après le jeu.

4. **Un changement du mode d'affichage** : case cochée, relance le jeu, puis, dans ses options graphiques, passe du **plein écran fenêtré** à **fenêtré**, puis reviens au plein écran fenêtré.
   ✅ Exile Market reste ouverte ; si le log écrit « Path of Exile 2 closed », note-le dans la remarque.

5. **Tes propres scénarios** : ce que tu veux essayer d'autre (en français, un retour au menu des personnages, un crash du jeu…).

## Résultats

| Test | Résultat | Remarque |
|---|---|---|
| 1. Cochée, l'outil lancé avant le jeu | | |
| 2. Décochée | | |
| 3. Cochée pendant que le jeu tourne | | |
| 4. Un changement du mode d'affichage | | |
| 5. Tes propres scénarios | | |
