# Cadrage technique – Mod usine NeoForge (0.1 · âge de pierre)

Oct 10, 2026 · @tom

## Objectif de la 0.1

La 0.1.0 livre l'âge de pierre complet, jouable en multijoueur sur un serveur dédié Linux : 2 à 3 heures de progression, de l'arrivée sans rien jusqu'à la première loupe de fer. Le contenu de jeu est décrit dans Cadrage gameplay – Âge de pierre, ce document dit comment le construire.

Toute la logique vit dans un cœur Rust indépendant de Minecraft, pour pouvoir le rebrancher un jour sur Pumpkin. La 0.1 pose aussi tout le socle technique dont les âges suivants dépendront.

**Dans le périmètre 0.1**

- Socle : cœur Rust, pont JNI, mod Kotlin, sauvegarde, contenu en TOML, CI et déploiement sur le serveur de test.
- Équipes et HUB : kit de campement, 4 niveaux de campement, rôles et permissions.
- Progression : quêtes principales et optionnelles, écran des plans révélés au fil du campement.
- Fabrication primitive : récolte au sol, assemblage en main, mini-jeux de taille et de façonnage, postes custom.
- Chaleur : feu de camp, four à fosse, four à métaux, soufflet, fonte et coulée en moules.
- Multiblocs avec plan fantôme : four à fosse et four à métaux.
- Monde : minerais du mod, morceaux visibles en surface, vanilla neutralisé, animaux qui ripostent.
- Simulation multithreadée, modes exact et agrégé, appliqués d'abord aux fours.

**Hors périmètre 0.1**

- Âge du fer et cinétique (0.2), électricité et tapis (0.3), électronique (0.4).
- Mode solo et builds Windows et macOS.
- Compatibilité avec d'autres mods, échanges entre équipes, placement automatique des multiblocs.

## Décisions validées

Ces choix ont été tranchés du 10 au 12 octobre 2026 et servent de base à tout le reste.

| Sujet | Décision | Conséquence |
| --- | --- | --- |
| Plateforme | Mod NeoForge, cœur en Rust | Rendu complet, mod requis côté client |
| Source de vérité | Rust détient tout l'état simulé | Kotlin n'affiche que ce que Rust renvoie |
| Liaison Kotlin ↔ Rust | JNI natif, zéro `unsafe` hors frontière JNI | Une cible Linux x86\_64 en V1 |
| Multithreading | Maximisé | Îlots en parallèle, résultat identique au séquentiel |
| Couche Minecraft | Kotlin | Kotlin for Forge comme dépendance |
| Simulation | Au rythme de Minecraft, calcul par débits | Mode exact près des joueurs, mode agrégé ailleurs |
| Activité des usines | Politique choisie au HUB, « équipe connectée » par défaut | Aucun calcul quand l'équipe est absente |
| Énergie | Système propre au mod | Compatibilité Forge Energy reportée |
| Sauvegarde | Rust sérialise, Kotlin range dans le monde | Une sauvegarde du monde contient les usines |
| Simulations | Une seule pour tout le serveur, toutes dimensions confondues | Équipes, HUB et quêtes traversent les dimensions |
| Panne globale | Simulation gelée, jamais sauvegardée, reprise automatique désactivable | Au pire, retour à la dernière copie saine |
| Contenu | Fichiers TOML dans le cœur | Tout ce qui peut être configurable l'est |
| Rechargement | Les valeurs du contenu se rechargent par `/reload` | Un nouvel item ou bloc demande un redémarrage |
| Cible de jeu | Serveur dédié multijoueur | Pas de solo en V1 |
| Équipes | 1 équipe par joueur, HUB déplaçable, 4 rôles | Rôles extensibles par la config |
| Tapis | Animés, en 0.3 avec l'électricité | Le plus gros chantier de rendu |
| Dépôt | Monorepo, 3 mods (moteur, monde, règles primitives) + modpack | Un seul build, une seule CI, une version commune |
| Code | En anglais, un test par fonction | Couverture vérifiée en CI |
| Versions | 0.0.1 au départ, patch par issue, mineure par jalon | V1 = 0.1.0, V2 = 0.2.0 |

## Architecture

Le projet est découpé en trois couches, et les dépendances ne vont que vers le bas : le cœur ne connaît ni Minecraft, ni JNI, ni Pumpkin.

&#91;embedded content: architecture · 3 couches, 1 port futur\]

Règles qui en découlent :

- `factory-core` n'importe aucun crate lié à JNI, à Minecraft ou au réseau. Il compile aussi en `wasm32`, ce que la CI vérifie.
- `factory-jni` ne contient aucune règle de jeu : uniquement de la traduction, la gestion des panics et le parallélisme.
- La couche Kotlin ne prend aucune décision de gameplay. Si une règle se retrouve en Kotlin, c'est un bug d'architecture.

**Découpage en mods**

Le projet est livré en trois mods et un modpack. Le moteur apporte les systèmes, les deux autres mods façonnent l'expérience de l'âge de pierre, et peuvent être retirés pour utiliser les machines dans un monde vanilla.

| Mod | Contenu | Langages |
| --- | --- | --- |
| `factory` (le moteur) | Cœur Rust, JNI, équipes, HUB, quêtes, plans, mini-jeux, postes, chaleur, multiblocs, machines, contenu par défaut en TOML | Rust + Kotlin |
| `factory-world` | Génération : placement des minerais, morceaux en surface, contenu des coffres de structures, hauteur du monde | Kotlin + datapack |
| `factory-primitive` | Règles de l'expérience : vanilla neutralisé, pas de récolte à mains nues, riposte des animaux, Nether et End bloqués | Kotlin + datapack |
| Le modpack | Config, surcharges de contenu, assemblage des trois mods | Données uniquement |

- **Seul le moteur déclare des blocs et des items**, à partir du TOML. `factory-world` ne fait que placer les minerais du moteur, `factory-primitive` ne fait que restreindre. Aucune dépendance entre ces deux-là.
- `factory-world` et `factory-primitive` dépendent du moteur, jamais l'inverse. Le moteur fonctionne seul.
- Le moteur expose aux autres mods une petite API Kotlin : lire l'âge atteint par une équipe, savoir si un item ou un bloc est débloqué. C'est ce qui permet à `factory-primitive` de rouvrir le vanilla au bon âge.
- Un futur mod de compatibilité (Forge Energy, inventaires d'autres mods) suivra le même modèle.
- Un seul dépôt, un sous-projet Gradle par mod, **une seule version commune** et une seule release pour les trois.

## Le contrat entre le cœur et les plateformes

Le cœur expose une seule interface : il reçoit une liste de **commandes** et renvoie une liste d'**effets**. Toute plateforme (NeoForge aujourd'hui, Pumpkin demain) se résume à traduire ses événements en commandes et à appliquer les effets.

```rust
pub trait Simulation {
    /// Applique les commandes reçues depuis le dernier appel.
    fn submit(&mut self, cmds: &[Command]) -> Vec<CommandResult>;
    /// Avance d'un tick et renvoie ce qui a changé.
    fn step(&mut self) -> Vec<Effect>;
    /// Lecture seule, pour les GUI et le rendu.
    fn query(&self, q: &Query) -> QueryResult;
    fn save(&self) -> Vec<u8>;
    fn load(bytes: &[u8]) -> Result<Self, LoadError> where Self: Sized;
}
```

**Commandes entrantes (exemples)**

| Commande | Champs | Rejetée si |
| --- | --- | --- |
| `PlaceMachine` | position, type, rotation, joueur | joueur sans droit, palier non débloqué |
| `BreakMachine` | position, joueur | machine d'une autre équipe |
| `TransferItems` | source, destination, item, quantité, joueur | pas de place, pas le droit, item absent de la source |
| `PlaceConveyor` / `PlaceCable` | position, orientation, joueur | réseau d'une autre équipe |
| `PlaceHub` / `BreakHub` | position, joueur | l'équipe a déjà un HUB posé, joueur pas chef |
| `InviteToTeam` / `LeaveTeam` / `SetRole` | joueur(s), rôle | droit insuffisant, joueur déjà dans une équipe |
| `SetActivityPolicy` | équipe, politique, joueur | politique non autorisée par le serveur |
| `ExternalInventoryChanged` | position d'un coffre vanilla, contenu | — |

**Effets sortants (exemples)**

| Effet | Ce que la plateforme en fait |
| --- | --- |
| `MachineStateChanged` | Met à jour le bloc (allumé, en panne, sans énergie) |
| `ItemsMovedOnConveyor` | Envoie les positions aux clients qui regardent ce tapis |
| `PushToExternal` | Remplit un coffre vanilla, mis en file si le chunk est déchargé |
| `GridTripped` | Coupe un réseau électrique et prévient l'équipe |
| `DropItems` | Fait apparaître des items au sol (machine cassée) |
| `MachineFaulted` | Affiche l'état « en erreur » après une panic attrapée |

Règles du contrat :

- Les positions sont des `(dimension, x, y, z)` en entiers, les items des identifiants texte (`factory:bauxite`), les joueurs des UUID. Aucun type Minecraft ne traverse.
- Chaque commande est validée par le cœur : permissions, place disponible, palier débloqué. La plateforme ne fait aucune vérification de gameplay.
- Les types du contrat sont versionnés avec le cœur. Ajouter un champ est permis, en retirer un demande une nouvelle version du contrat.

## Modèle de simulation

La simulation suit le rythme de Minecraft (20 ticks par seconde) mais calcule par débits plutôt que machine par machine : une chaîne stable se résume à « liste d'entrées par seconde → liste de sorties par seconde », même avec 100 machines au milieu.

**Deux modes de calcul**

| Mode | Quand | Comment |
| --- | --- | --- |
| Exact | Chunk chargé et joueur qui interagit avec l'îlot | Chaque machine avance tick par tick, vrais tirages aléatoires |
| Agrégé | Tout le reste | L'îlot est réduit à un bilan de débits, probabilités remplacées par leur moyenne |

**Calcul par débits**

- Chaque recette devient un débit : x énergie + x items par seconde → x items par seconde.
- Les machines enchaînées sont fusionnées en une seule chaîne, avec une liste de départ et une liste de fin.
- Pour chaque îlot, le cœur construit un arbre des flux et calcule le débit réel en remontant jusqu'à l'étape qui bloque (manque d'énergie, manque d'entrée, sortie pleine). Un blocage se propage ainsi naturellement à tout ce qui en dépend.
- L'arbre n'est recalculé que lorsqu'il change : pose, casse, changement de recette ou d'énergie disponible, sortie qui se remplit ou se vide.
- Les débits sont des fractions exactes (numérateur et dénominateur entiers) avec des accumulateurs : 0,3 item par tick donne bien 6 items par seconde, sans dérive ni flottants.

**Probabilités**

- Une recette peut avoir des sorties probabilistes, par exemple 20 % de chance d'un sous-produit.
- En mode exact, vrai tirage. En mode agrégé, la moyenne : 5 machines à 1 craft par seconde avec 20 % de chance donnent 1 item par seconde.
- Le générateur aléatoire est seedé par îlot : en jeu le résultat est imprévisible, en test il est reproductible.
- Le passage d'un mode à l'autre conserve les accumulateurs : aucun item créé ni perdu à la bascule.

**Quand une usine tourne**

- Chaque îlot n'est simulé que si sa politique d'activité le permet. C'est un enum `ActivityPolicy` : `Always`, `TeamOnline` (au moins un membre connecté), `Paused`.
- Le serveur fixe la politique par défaut et celles autorisées dans `config.toml`. L'équipe choisit au HUB parmi celles-ci, et peut tout mettre en pause.
- Défaut proposé : `TeamOnline`, sans rattrapage du temps passé hors ligne.
- Les déclencheurs qui réveillent un îlot (commande, changement d'énergie, sortie libérée, connexion d'un membre) sont eux aussi un enum, pour qu'un îlot en pause ou stable ne coûte rien.
- Les effets qui touchent le monde (coffre vanilla dans un chunk déchargé) restent en file, avec une taille maximale par position.

**Multithreading**

- Objectif : exploiter tous les cœurs disponibles. Le pool de threads prend par défaut le nombre de cœurs de la machine (`sim.threads`).
- Les îlots indépendants sont simulés en parallèle. À l'intérieur d'un gros îlot, le recalcul de l'arbre des flux et les machines en mode exact sont parallélisés aussi.
- Un tick se déroule en trois temps : commandes (séquentiel, rapide), îlots (parallèle), fusion des effets triés par îlot (séquentiel).
- Le parallélisme ne change jamais le résultat : même état final qu'en séquentiel, vérifié par les tests.
- Si un tick dépasse son budget (`sim.tick_budget_ms`), les îlots agrégés les moins urgents passent au tick suivant plutôt que de ralentir le serveur.

## Couche JNI

La couche JNI reste volontairement minuscule : peu de fonctions, des tableaux d'octets en entrée et en sortie, aucune panic qui sorte vers la JVM, et pratiquement aucun code `unsafe`.

**Points d'entrée**

| Fonction | Entrée | Sortie |
| --- | --- | --- |
| `init` | contenu TOML, config | identifiant de simulation |
| `load` | contenu TOML, config, octets de sauvegarde | identifiant de simulation |
| `save` | — | octets de sauvegarde |
| `submit` | lot de commandes sérialisé | résultats sérialisés |
| `tick` | — | lot d'effets sérialisé |
| `query` | requête sérialisée | réponse sérialisée |
| `reload` | contenu TOML, config | erreurs de validation |
| `shutdown` | identifiant | — |

- Un appel par lot, jamais par machine : `submit` puis `tick` une fois par tick serveur, ce qui limite le coût des traversées JNI.
- Sérialisation binaire compacte (postcard) des types du contrat. Côté Kotlin, un test d'aller-retour garantit que les deux côtés lisent la même chose.
- La JVM ne reçoit jamais de pointeur vers l'état Rust, seulement un identifiant numérique. Le cœur garde ses simulations dans un registre protégé par un verrou, ce qui évite le code `unsafe` qu'imposerait un pointeur brut.
- Une seule simulation pour tout le serveur, toutes dimensions confondues : équipes, HUB et quêtes les traversent, et les îlots tournent déjà en parallèle. Le registre en accepte plusieurs, ce qui sert aux tests.
- Les identifiants de simulation sont des entiers 64 bits croissants, jamais réutilisés : un identifiant périmé lève une exception au lieu de viser une autre simulation.
- `load` reçoit le contenu avec la sauvegarde : une simulation n'existe jamais sans son contenu, et le cœur vérifie dès le chargement la sauvegarde contre lui.
- Le contenu et la config traversent en un seul tableau d'octets postcard, chaque fichier avec son chemin : les erreurs citent `fichier:ligne`, et un nouveau fichier ne change pas la signature JNI.

**Zéro `unsafe`**

- `factory-core` : `#![forbid(unsafe_code)]`, aucune exception possible.
- `factory-jni` : `#![deny(unsafe_code)]`, avec une seule dérogation dans un petit module `exports`, pour ce que JNI impose (les fonctions exportées sous un nom fixe, `#[unsafe(no_mangle)]` en Rust 2024). Toute la logique passe par l'API safe du crate `jni`.
- Toute nouvelle dérogation exige une justification en commentaire et ta validation explicite dans la PR.
- La CI audite aussi les dépendances (vulnérabilités connues, licences).

**Erreurs et panics**

- Chaque point d'entrée est enveloppé dans `catch_unwind`. Une panic devient une exception Kotlin `FactoryCoreException` avec le message et l'îlot concerné.
- Profil de compilation avec `panic = "unwind"`, jamais `abort`.
- Une panic pendant la simulation d'un îlot met **cet îlot** en état `Faulted`, journalise, et laisse tourner le reste. Un admin le relance par commande une fois le bug corrigé.
- Une panic hors de tout îlot (commandes, fusion des effets, sauvegarde) peut laisser **toute la simulation** à moitié modifiée. Elle est alors gelée :
  - elle n'est plus jamais sauvegardée, pour ne pas écraser la dernière bonne sauvegarde ;
  - Kotlin cesse de la faire avancer et refuse les interactions avec les blocs du mod (poser, casser, ouvrir), avec un message au joueur, pour que le monde ne diverge pas du cœur ;
  - les opérateurs sont prévenus dans le chat et la console.
- Reprise automatique, activée par défaut (`sim.auto_recover`) : le cœur garde en mémoire une copie saine de son état (`sim.snapshot_interval_ticks`) et les lots de commandes reçus depuis. Sur une panne globale, il jette l'état gelé, recharge la copie et rejoue les lots, sans le dernier : la simulation étant déterministe, on retrouve l'état d'avant la panne. Les commandes du tick fautif reviennent refusées et Kotlin les annule.
- Si le rejeu panique à son tour, le cœur repart de la copie seule. Prix accepté : le cœur revient en arrière, pas les inventaires des joueurs, donc un item sorti d'une machine dans l'intervalle peut être dupliqué, un item déposé perdu.
- Au-delà de `sim.auto_recover_max_per_hour` reprises dans l'heure, ou si la reprise est désactivée, la simulation reste gelée jusqu'à `/satiscraft admin recover` ou au redémarrage, qui repart de la dernière sauvegarde sur disque.

**Threads**

- La couche Kotlin appelle le cœur uniquement depuis le thread serveur.
- Le pool de threads appartient au cœur. Ses threads ne rappellent jamais la JVM.
- Cible de build V1 : `x86_64-unknown-linux-gnu` uniquement, la bibliothèque `.so` étant embarquée dans le jar du mod.

## Couche NeoForge (Kotlin)

La couche Kotlin enregistre ce que Minecraft doit connaître et affiche l'état renvoyé par le cœur, sans jamais stocker ni décider elle-même.

**Enregistrement**

- Blocs, items et onglet créatif sont générés à partir du contenu TOML au démarrage. Ajouter une machine ne demande donc pas de code Kotlin, sauf rendu particulier.
- Les machines ont un block entity minimal qui ne contient que leur position : tout l'état est dans le cœur.
- Minerais et nœuds de ressources sont de vrais blocs, placés par la génération de monde NeoForge (features et biome modifiers en datapack).

**GUI**

- Les machines utilisent un **inventaire custom** : un conteneur « proxy » qui ne stocke rien. Lire un slot revient à interroger le cœur, modifier un slot revient à envoyer une commande.
- Tous les déplacements (clic, shift-clic, glisser, touches numériques, double-clic, jet d'item) passent par une seule commande `TransferItems`. Le cœur la valide et l'applique d'un bloc : l'item quitte la source et arrive à la destination dans la même opération, ou rien ne bouge.
- La logique de clic par défaut de Minecraft (notamment le shift-clic) est la source classique des duplications. Elle est entièrement remplacée, pas surchargée au cas par cas.
- Le client n'affiche qu'une copie, resynchronisée après chaque commande. S'il désynchronise, le serveur a toujours raison.
- Un menu générique sert toutes les machines (slots d'entrée, de sortie, jauge d'énergie, progression), son agencement vient du TOML.
- Un test de non-duplication rejoue des milliers de clics aléatoires et vérifie que le total d'items ne change jamais.

**Tapis animés (0.3)**

- La bande qui défile : une texture animée par fichier `.mcmeta`, aucun travail d'animation.
- Les modèles : quelques pavés faits dans Blockbench (tapis droit, virage, entrée de machine).
- Les items : un renderer de block entity dessine chaque item à sa position sur la voie, interpolée entre deux pas pour un mouvement fluide.
- Synchronisation : le serveur envoie aux clients proches les items entrés et sortis de chaque segment, et le client extrapole le reste à vitesse connue. Il n'y a pas d'envoi de position à chaque pas.
- Objectif de performance à mesurer tôt : 2 000 items visibles sans chute d'images notable.

**Synchronisation client**

- Un paquet réseau par type d'effet visible, envoyé seulement aux joueurs qui ont le chunk chargé.
- Le client ne fait tourner aucune simulation : il affiche et interpole.

## Systèmes de jeu de la 0.1

Chaque système de l'âge de pierre suit la même règle : la décision dans le cœur Rust, l'affichage dans Kotlin, les valeurs en TOML.

| Système | Cœur Rust | Couche Kotlin |
| --- | --- | --- |
| Quêtes et plans | Objectifs, progression par équipe, déblocages | Écran des quêtes et des plans, ouvert par une touche dédiée |
| Niveaux de campement | Niveau porté par l'équipe, conditions d'amélioration | Modèle du HUB qui change selon le niveau |
| Mini-jeux de taille et de façonnage | Motifs, validation du résultat, tirage d'échec | Grille interactive côté client |
| Assemblage en main | Recettes d'assemblage, validation | Détection du clic droit sur l'item posé au sol |
| Postes custom | Recettes de chaque poste | Blocs et menus |
| Chaleur des fours | Température, combustible, recettes à seuil | Jauge, particules, état visuel du four |
| Multiblocs | Gabarits, validation, état formé | Plan fantôme posé, rendu des blocs manquants |
| Monde | — | factory-world : minerais, morceaux en surface, contenu des coffres de structures |
| Vanilla neutralisé | — | factory-primitive : retrait des recettes, blocage de l'établi, du four, du commerce, du Nether et de l'End |
| Faune | — | factory-primitive : riposte ajoutée aux animaux vanilla |

**Points d'attention**

- Mini-jeux : le client affiche la grille et envoie la suite des coups. Le cœur rejoue la séquence et décide du résultat, y compris un silex qui casse au mauvais endroit. Un client modifié ne peut donc pas tricher.
- Assemblage en main : un item posé au sol est une entité que Minecraft peut faire disparaître. L'assemblage se fait d'un coup au clic, sans état intermédiaire qui dure.
- Chaleur : premier vrai usage des deux modes de simulation. Un four près d'un joueur chauffe tick par tick, un four éloigné passe en bilan agrégé de combustible et de production.
- Plans fantômes : le gabarit est envoyé au client, qui dessine les blocs manquants en transparence. La validation reste côté serveur.
- Vanilla neutralisé : une liste en config dit ce qui est bloqué (recettes, blocs, dimensions) et jusqu'à quel âge, pour pouvoir rouvrir chaque élément au bon moment.

**Ajouts au contrat**

- Commandes : `SubmitMinigame`, `AssembleInHand`, `PlaceBlueprint`, `DeliverToHub`, `UpgradeCamp`, `FuelFurnace`.
- Effets : `QuestProgressed`, `PlansUnlocked`, `CampLevelChanged`, `MultiblockFormed`, `HeatChanged`, `MinigameResolved`.

## Équipes, HUB et permissions

Chaque joueur appartient à au plus une équipe. L'équipe possède un HUB, sa progression et tous les blocs du mod posés par ses membres.

**Cycle de vie d'une équipe**

- Chaque joueur reçoit un kit de campement à sa première connexion. Le poser crée son équipe et en fait le chef. S'il rejoint plutôt une équipe existante, le kit peut disparaître ou être modifié selon son contenu. Une équipe a un seul HUB posé à la fois. Le chef peut le casser et le reposer ailleurs : l'équipe et sa progression sont stockées dans l'équipe, pas dans le bloc, donc rien n'est perdu.
- Le chef et les officiers invitent, le joueur invité accepte. Un joueur déjà dans une équipe doit la quitter avant d'en rejoindre une autre.
- Quand un joueur quitte l'équipe, ses machines **restent à l'équipe**.
- Si le chef part, la direction passe au membre de rang le plus élevé, puis au plus ancien à rang égal.
- Si le dernier membre part, tout ce que possédait l'équipe passe en **wilderness** : sans propriétaire, la simulation de ces îlots est mise en pause, et n'importe quel joueur peut casser ces blocs.

**Rangs et droits**

| Action | Chef | Officier | Membre | Hors équipe |
| --- | --- | --- | --- | --- |
| Utiliser une machine, ouvrir sa GUI | oui | oui | oui | non |
| Poser, casser, brancher | oui | oui | oui | non |
| Débloquer un palier au HUB | oui | oui | non | non |
| Inviter, exclure | oui | oui | non | non |
| Promouvoir, rétrograder | oui | non | non | non |
| Casser le HUB | oui | non | non | non |

Les rôles et leurs droits sont définis dans \`teams.toml\` sous forme de liste. La V1 livre ces 4 rôles, mais le système en accepte d'autres : ajouter un rôle « admin » avec ses propres permissions sera une simple modification de config.

**Règles techniques**

- Chaque machine, câble et segment de tapis porte l'identifiant de son équipe. Le cœur vérifie le droit à chaque commande.
- Un câble ou un tapis ne peut pas relier deux équipes : la pose est refusée.
- La protection ne couvre que les blocs du mod. La protection de terrain est laissée à un mod de claims.
- Les échanges entre équipes viendront plus tard, par une machine dédiée.

## Persistance et migrations

Le cœur sérialise tout son état en un blob binaire versionné, que la couche Kotlin range dans la sauvegarde du monde. Une sauvegarde du monde contient donc toujours les usines qui vont avec.

**Format**

- En-tête : magic `FCTR`, numéro de version du format, version du mod, somme de contrôle.
- Corps : l'état sérialisé (postcard), découpé par dimension.
- Stocké via `SavedData` de NeoForge, dans le dossier `data/` du monde, à chaque sauvegarde automatique du serveur.
- Écriture atomique (fichier temporaire puis renommage) pour ne jamais laisser une sauvegarde à moitié écrite.

**Migrations**

- Chaque changement de structure incrémente la version du format et ajoute une fonction `migrate_vN_to_vN+1`.
- Au chargement, le cœur enchaîne les migrations jusqu'à la version courante.
- Une version plus récente que celle du mod est refusée au chargement, pour ne pas écraser des données.
- Chaque version publiée laisse une sauvegarde de référence dans `fixtures/`. La CI vérifie qu'elles se chargent toutes.

**Identifiants figés**

- Un identifiant (`factory:bauxite`, `factory:smelter`) ne se renomme ni ne se supprime jamais. Pour retirer un contenu, on le marque `deprecated` et on prévoit sa conversion.
- Les noms affichés, textures, recettes et valeurs d'équilibrage peuvent changer librement.
- La CI compare la liste des identifiants à celle de la version précédente et échoue si l'un d'eux a disparu.

## Contenu configurable (TOML)

Tout le contenu de jeu se décrit en fichiers TOML embarqués dans le mod, et un serveur peut les surcharger depuis son dossier `config/factory/`. Le cœur valide l'ensemble au démarrage et refuse de démarrer sur une erreur, avec un message qui donne le fichier et la ligne.

**Organisation**

```
# moteur (factory)
content/
  items.toml        # items, ressources, outils, blocs de minerai
  stations.toml     # postes custom et leurs recettes
  minigames.toml    # motifs de taille (silex, pierre) et de façonnage (argile)
  assembly.toml     # assemblages en main
  heat.toml         # fours, combustibles, températures
  multiblocks.toml  # gabarits des multiblocs
  quests.toml       # quêtes principales et optionnelles
  progression.toml  # niveaux de campement et plans révélés
  teams.toml        # rôles et droits
  machines.toml     # machines (à partir de la 0.2)
  recipes.toml      # recettes des machines
  logistics.toml    # tapis, câbles (à partir de la 0.3)

# factory-world
worldgen.toml       # placement des minerais, morceaux en surface

# factory-primitive
vanilla.toml        # ce qui est bloqué, et jusqu'à quel âge

# serveur
config.toml         # threads, budget de tick, politiques d'activité, reprise après panne
```

**Exemple**

```toml
[[machines]]
id = "factory:smelter"
name = "Four électrique"
power_draw = 200           # milli-unités par tick
inputs = 1
outputs = 1
unlock = "tier_1"

[[recipes]]
id = "factory:iron_ingot"
machine = "factory:smelter"
ticks = 40                 # 2 secondes
inputs = [{ item = "factory:iron_ore", count = 1 }]
outputs = [
  { item = "factory:iron_ingot", count = 1 },
  { item = "factory:slag", count = 1, chance = 0.20 },
]
```

**Règles**

- Toute valeur d'équilibrage vit dans ces fichiers, jamais en dur dans le code.
- Les durées sont en ticks et les énergies en milli-unités. Les chances sont écrites en décimal pour la lisibilité, puis converties en fractions entières au chargement.
- Les textures et modèles restent dans le resource pack du mod, référencés par l'identifiant.

**Rechargement à chaud**

- Le `/reload` de Minecraft, réservé aux opérateurs, recharge aussi le contenu du moteur sans redémarrer le serveur, pour équilibrer sans couper la partie.
- Tout ce qui est valeur se recharge : durées, quantités, débits, recettes des postes, chances, récompenses de quêtes, budget de tick.
- Ce qui touche aux registres de Minecraft, figés au démarrage, demande un redémarrage : un item ou un bloc nouveau est refusé avec ce message. `sim.threads` aussi.
- Tout ou rien : le cœur valide l'ensemble du nouveau contenu. À la moindre erreur, l'ancien reste en place et l'opérateur reçoit `fichier:ligne`.
- Une machine en plein cycle le termine avec les anciennes valeurs ; le cycle suivant prend les nouvelles.
- Le contenu appartient à la simulation, jamais à un état global, pour pouvoir être remplacé.
- Pour l'instant, seul le contenu du moteur se recharge : `worldgen.toml` et `vanilla.toml` restent lus au démarrage.

## Dépôt, build, tests et déploiement

Un seul dépôt git, une seule commande de build (`./gradlew build`) qui compile le Rust puis le mod, et un agent qui ne peut déployer que sur le serveur de test.

**Arborescence**

```
factory/
  Cargo.toml                # workspace Rust
  crates/
    factory-core/           # logique pure + contenu par défaut (content/)
    factory-jni/            # pont JNI
  mods/
    factory/                # le moteur (Kotlin), embarque la .so
    factory-world/          # génération du monde
    factory-primitive/      # règles de l'âge de pierre, vanilla neutralisé
  modpack/                  # config et surcharges de contenu
  fixtures/                 # sauvegardes de référence par version
  scripts/deploy-test.sh    # envoi vers le serveur de test
  CLAUDE.md                 # règles pour l'agent
```

**Build**

- Une tâche Gradle lance `cargo build --release -p factory-jni` et copie la `.so` dans les ressources du jar du moteur. Les trois mods sont construits ensemble et portent la même version.
- Les versions de Minecraft, NeoForge, Kotlin for Forge et Rust sont épinglées (`rust-toolchain.toml`, propriétés Gradle). Une montée de version est une tâche à part.

**Tests**

| Niveau | Outil | Ce qui est vérifié |
| --- | --- | --- |
| Cœur | `cargo test` | Recettes, énergie, tapis, équipes, à partir de scénarios commandes → effets |
| Contenu | `cargo test` | Tous les TOML se chargent, pas de recette orpheline ni d'id inconnu |
| Sauvegarde | `cargo test` | Chaque fixture se charge et migre |
| Déterminisme | `cargo test` | Même scénario rejoué en séquentiel et en parallèle donne le même état |
| Contrat | Gradle | Aller-retour de sérialisation Kotlin ↔ Rust |
| En jeu | GameTests NeoForge | Poser une machine, l'alimenter, récupérer la sortie |

**CI**

À chaque PR : `cargo fmt --check`, `clippy`, tous les tests ci-dessus, compilation de `factory-core` en `wasm32` pour garantir la portabilité vers Pumpkin, et contrôle des identifiants figés.

**Déploiement**

- L'agent travaille sur une VM Proxmox dédiée, sans accès au nœud Pterodactyl.
- `deploy-test.sh` utilise une clé API client d'un utilisateur Pterodactyl qui n'a accès qu'au serveur de test : envoi des trois jars, puis redémarrage.
- La prod est mise à jour à la main, après validation sur le serveur de test et sauvegarde du monde.
- Les versions sont taguées (`v1.0.0`) et chaque tag produit un jar en artefact de CI.

## Conventions de code et versions

Tout le code est en anglais, chaque fonction a ses tests, et la version monte à chaque issue mergée.

**Langue**

- Noms, commentaires, messages de commit et documentation technique du dépôt (`CLAUDE.md`, README) en anglais.
- Les textes affichés aux joueurs passent par les fichiers de langue de Minecraft (`fr_fr`, `en_us`), jamais en dur.

**Tests**

- Toute fonction a au moins un test, y compris les fonctions privées non triviales. Une PR qui ajoute une fonction sans test est refusée.
- Rust : tests unitaires dans chaque module, tests de scénarios commandes → effets, et tests de propriétés (`proptest`) sur les invariants : aucun item créé ni perdu, énergie conservée, résultat identique en séquentiel et en parallèle.
- Kotlin : tests unitaires JUnit pour la traduction, un GameTest NeoForge par bloc et par menu.
- Couverture mesurée en CI (`cargo-llvm-cov`), avec un seuil minimal qui fait échouer la PR s'il baisse.

**Versions**

| Changement | Montée | Exemple |
| --- | --- | --- |
| Issue mergée : correctif, contenu, petite fonctionnalité | patch | 0.0.1 → 0.0.2 |
| Jalon terminé (une « V ») | mineure | 0.0.14 → 0.1.0 |
| Ta décision que le mod est complet | majeure | 0.x → 1.0.0 |

- Démarrage en 0.0.1. « V1 » = 0.1.0, « V2 » = 0.2.0 : un jalon GitHub par V, chaque issue rattachée à son jalon.
- La version est montée dans la PR elle-même, un tag git est posé au merge, et le changelog est généré depuis les titres de PR.
- Le numéro de format de sauvegarde et celui du contrat évoluent indépendamment de la version du mod.

## Points ouverts et reportés

**À trancher avant de coder**

- [ ] Version exacte de Minecraft et de NeoForge : la plus récente stable au démarrage, à vérifier ensemble.
- [x] Un identifiant de simulation pour tout le serveur, ou un par dimension : un seul pour tout le serveur (12 octobre 2026).
- [ ] Comportement exact de la wilderness : pause des îlots et casse libre, ou reprise possible par une autre équipe.
- [ ] Ce que font les fours et machines d'une équipe pendant que son HUB est démonté.
- [ ] Ce qui compte comme « interagir » pour passer un îlot en mode exact (GUI ouverte, distance au joueur).
- [ ] Seuil de couverture de tests minimal.
- [ ] Cinétique en 0.2 : dépendre de Create, dont le réseau vit en Java et ne tourne que dans les chunks chargés, ou écrire un cinétique maison dans le cœur, compatible avec le mode agrégé.
- [ ] Électricité en 0.3 : modèle en tension et intensité à définir.

**Reporté après la 0.1**

- 0.2 : âge du fer, cinétique (force et vitesse), premières machines automatisées.
- 0.3 : électricité, câbles, tapis animés.
- 0.4 : électronique et logique de pilotage de la production.
- Mode solo et builds Windows, macOS Intel et ARM.
- Compatibilité Forge Energy et inventaires des autres mods, dans le mod ou dans un mod de compatibilité séparé.
- Placement automatique des multiblocs.
- Pentes, ascenseurs et séparateurs de tapis.
- Échanges entre équipes par une machine dédiée.
- Nouveaux animaux, avec leurs modèles et animations.
- Hauteur du monde, niveau de la mer et rivières qui coulent.
- Portage vers Pumpkin si le support des mods arrive.
