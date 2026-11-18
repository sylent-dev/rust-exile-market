# Palier d'une pierre de téléportation entre un min et un max : les tests

## Pourquoi

Le palier d'une pierre de téléportation est maintenant un filtre à part, « Tier: 12 », en tête du groupe **Waystone**, de 12 à 12 par défaut : tu peux l'élargir (« 11 – ») pour comparer plus d'offres quand le marché d'un palier est maigre. La recherche passe par la catégorie des pierres et le palier, plus par la base. Fais les tests d'affilée, sans m'attendre. Un ✅ suffit quand ça marche ; détaille seulement ce qui cloche. Puis dis-moi « fait ».

## Avant

1. Ferme Exile Market et Exiled Exchange 2.
2. Lance, dans Konsole (pas dans le terminal de VS Code : son bac à sable Flatpak n'a pas X11, la fenêtre ne s'ouvre pas) :

   ```
   cargo run --release
   ```

   dans le dossier du projet, sur la branche à tester.

3. Lance le jeu en **plein écran fenêtré**, dans ton repaire, avec une pierre de téléportation rare dans l'inventaire, de palier 2 à 15.

## Les tests

1. **Le palier élargi** : **Ctrl+D** sur la pierre, **« Show N stats »**.
   ✅ Pas de ligne « Base: Waystone » ; en tête du groupe **Waystone**, « Tier: N » coché, son champ « N – N », la barre et « TN » dessous, dans la couleur du palier.
   Tape `N-1` dans le **min** et vide le **max** (« 11 – » pour une T12), puis clique sur la **loupe**.
   ✅ Les offres arrivent ; au survol, au moins une est d'un autre palier que N (« … · T11 » pour une T12, ou au-dessus), aucune en dessous du min. Si toutes sont en TN, note-le : le marché est peut-être trop maigre.
   Puis clique sur **« Check on trade »**.
   ✅ Le site d'échange s'ouvre sur la catégorie des pierres de téléportation, le filtre « Waystone Tier » à min `N-1`, sans max.

2. **Test libre** : prix quelques autres pierres (une normale, une magique, une corrompue si tu en as), en jouant avec le palier.
   ✅ Rien d'étrange : note tout ce qui te surprend.

## Résultats

| Test | Résultat | Remarque |
|---|---|---|
| 1. Le palier élargi | | |
| 2. Test libre | | |
