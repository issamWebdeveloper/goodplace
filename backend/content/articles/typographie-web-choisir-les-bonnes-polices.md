+++
title = "Typographie web : choisir les bonnes polices"
slug = "typographie-web-choisir-les-bonnes-polices"
excerpt = "Guide pratique pour sélectionner et combiner les typographies digitales."
tags = ["Design", "CSS", "Angular", "Performance"]
cover_image = "/images/blog/typographie.png"
seo_description = "Choisir, combiner et charger efficacement des polices web : lisibilité, paires typographiques, polices variables, font-display et intégration dans un projet Angular."
status = "published"
published_at = "2023-04-20T08:00:00Z"
+++

On estime que plus de 90 % d'une interface web est constituée de texte. Menus, boutons, formulaires, messages d'erreur, contenus éditoriaux : l'utilisateur lit en permanence. La typographie n'est donc pas une décoration que l'on ajoute en fin de projet, c'est la matière première de l'interface. Une bonne police rend un produit crédible et confortable ; une mauvaise fatigue, ralentit la page et brouille la hiérarchie de l'information.

Ce guide rassemble les critères que j'applique sur mes projets front-end, de la sélection des familles jusqu'à leur chargement dans une application Angular.

## Commencer par l'usage, pas par le coup de cœur

Avant d'ouvrir un catalogue de polices, il faut répondre à trois questions simples :

1. **Quel type de contenu ?** Un tableau de bord dense en chiffres n'a pas les mêmes besoins qu'un blog aux longs paragraphes.
2. **Sur quels écrans ?** Mobile en extérieur, grand écran de bureau, terminal métier en basse résolution…
3. **Quelle personnalité ?** Institutionnelle, technique, chaleureuse, éditoriale.

Pour le texte courant, privilégiez une police conçue pour l'écran : une hauteur d'x généreuse, des contreformes ouvertes (l'intérieur du « e », du « a »), des caractères facilement distinguables. Le test classique consiste à écrire `Il1 O0 rn m` : si le « I » majuscule, le « l » minuscule et le chiffre « 1 » se confondent, la police posera problème dans un identifiant, un mot de passe ou un numéro de dossier.

## Serif, sans serif, monospace : le rôle de chaque famille

- **Sans serif** (Inter, Open Sans, Lato, Source Sans) : le choix par défaut des interfaces. Neutres, lisibles dans les petites tailles, elles s'effacent devant le contenu.
- **Serif** (Merriweather, Source Serif, Lora) : parfaites pour la lecture longue sur écran moderne et pour donner une touche éditoriale aux titres.
- **Monospace** (JetBrains Mono, Fira Code, Source Code Pro) : indispensables pour le code, les identifiants techniques et les données tabulaires.

## Combiner deux polices sans faire de faute de goût

La règle la plus sûre : **deux familles maximum**, et un contraste net entre elles. Deux sans serif trop proches donnent l'impression d'une erreur ; une sans serif géométrique pour les titres et une humaniste pour le texte créent au contraire une hiérarchie claire.

Quelques paires éprouvées :

| Titres | Texte | Ambiance |
| --- | --- | --- |
| Lato (900) | Open Sans | Moderne, dynamique |
| Playfair Display | Source Sans | Éditoriale, élégante |
| Montserrat | Merriweather | Contrastée, magazine |
| Inter | Inter | Sobre, orientée produit |

Utiliser une seule famille déclinée en plusieurs graisses est aussi une excellente option : c'est cohérent par construction, et plus léger à charger.

## Construire une échelle typographique

Plutôt que de choisir des tailles au hasard (15 px ici, 22 px là), définissez une **échelle modulaire** : une taille de base multipliée par un ratio constant. Avec une base de 16 px et un ratio de 1,25 (« tierce majeure »), on obtient 16, 20, 25, 31, 39 px. Chaque niveau de titre a ainsi une relation mathématique avec les autres.

En SCSS, cela tient en quelques lignes que l'on partage dans tout le projet Angular :

```scss
// src/styles/_typography.scss
$base-size: 1rem;
$ratio: 1.25;

@function step($n) {
  @return $base-size * math.pow($ratio, $n);
}

h1 { font-size: step(4); line-height: 1.1; }
h2 { font-size: step(3); line-height: 1.2; }
h3 { font-size: step(2); line-height: 1.3; }
body { font-size: step(0); line-height: 1.6; }
```

Pour que les titres s'adaptent à la largeur d'écran sans multiplier les media queries, `clamp()` est redoutable :

```css
h1 { font-size: clamp(2rem, 1.2rem + 3vw, 3.5rem); }
```

La taille ne descend jamais sous 2 rem, ne dépasse jamais 3,5 rem et varie de façon fluide entre les deux.

## Les réglages qui font la différence

Une bonne police mal réglée reste pénible à lire. Trois paramètres comptent plus que tous les autres :

- **La longueur de ligne** : entre 60 et 75 caractères. En CSS, `max-width: 68ch` sur le conteneur de texte règle la question.
- **L'interlignage** : autour de 1,5 à 1,7 pour le texte courant, plus serré (1,1 à 1,3) pour les grands titres.
- **Le contraste** : un gris trop clair sur fond blanc est à la mode mais échoue souvent au critère WCAG de 4,5:1. Visez un gris foncé (`#343434` plutôt que `#999`).

## Polices variables : une seule requête pour toutes les graisses

Une police variable regroupe dans un seul fichier un continuum de graisses (et parfois de largeurs ou d'inclinaisons). Là où un site charge classiquement quatre fichiers (400, 600, 700, 900), un unique fichier variable suffit :

```css
@font-face {
  font-family: 'Inter';
  src: url('/fonts/inter-var.woff2') format('woff2-variations');
  font-weight: 100 900;
  font-display: swap;
}
```

Le gain dépend du nombre de graisses utilisées : si vous n'en utilisez que deux, deux fichiers statiques sous-ensemblés restent parfois plus légers. Mesurez.

## Charger les polices sans dégrader la performance

Une police web est une ressource bloquante pour l'affichage du texte. Mal chargée, elle provoque un texte invisible (FOIT) ou un saut de mise en page quand elle remplace la police de secours. Les bonnes pratiques :

1. **Format WOFF2 uniquement** : il est supporté par tous les navigateurs modernes et compresse mieux que WOFF.
2. **`font-display: swap`** pour afficher immédiatement le texte avec la police système, puis basculer.
3. **Sous-ensembles** : ne chargez que les plages Unicode utiles (`unicode-range`). Pour un site francophone, le latin étendu suffit.
4. **Précharger la police critique**, celle du texte au-dessus de la ligne de flottaison :

```html
<link rel="preload" href="/fonts/open-sans-var.woff2" as="font" type="font/woff2" crossorigin>
```

5. **Limiter le nombre de fichiers** : au-delà de trois ou quatre, le coût réseau devient visible sur mobile.

## Et dans un projet Angular ?

Angular facilite déjà une partie du travail. Depuis la version 11, avec l'optimisation activée en production, le CLI **inline automatiquement la feuille de style Google Fonts** référencée dans `index.html` : une requête bloquante de moins au démarrage. Il suffit que l'option `optimization.fonts` soit active dans `angular.json` (c'est le cas par défaut en configuration de production).

Si vous préférez l'auto-hébergement — plus respectueux du RGPD, puisque aucune adresse IP n'est transmise à un tiers — les paquets `@fontsource` s'installent comme n'importe quelle dépendance :

```bash
npm install @fontsource/open-sans @fontsource/lato
```

puis dans `styles.scss` :

```scss
@import '@fontsource/open-sans/400.css';
@import '@fontsource/lato/900.css';
```

Les fichiers de police sont alors copiés et hashés par le build, et servis avec un cache long depuis votre propre domaine.

Dernier conseil : centralisez les familles dans des variables CSS (`--font-body`, `--font-heading`) définies sur `:root`. Changer de police devient une modification d'une ligne.

## En résumé

- Partez de l'usage et de la lisibilité, pas de l'effet visuel.
- Deux familles maximum, avec un vrai contraste de rôle.
- Une échelle modulaire et `clamp()` pour une hiérarchie cohérente et fluide.
- WOFF2, `font-display: swap`, sous-ensembles et préchargement de la police critique.
- Dans Angular : inlining automatique en production ou auto-hébergement via `@fontsource`.

La meilleure typographie est celle que l'on ne remarque pas : le lecteur avance dans le texte sans effort, et c'est exactement ce que l'on attend d'une interface.
