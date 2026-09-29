+++
title = "ANSAAB : concevoir une plateforme généalogique autour de GEDCOM X"
slug = "ansaab-genealogie-gedcomx"
excerpt = "Pourquoi bâtir une application d'arbres généalogiques sur la spécification GEDCOM X, et comment structurer son modèle de données et son interface Angular."
tags = ["Architecture", "Angular", "Projet", "Données"]
cover_image = "/images/blog/ansaab.png"
seo_description = "Conception d'ANSAAB, application généalogique basée sur GEDCOM X : modèle de données, sources, contrôle de cohérence et visualisation d'arbres en Angular."
status = "published"
published_at = "2024-12-09T08:00:00Z"
+++

« Ansab » signifie « lignées » en arabe. **ANSAAB** est un projet que je mène pour offrir une plateforme complète de création, d'exploration et de partage d'arbres généalogiques. Avant d'écrire la moindre ligne d'interface, une décision structurante s'imposait : le format des données. J'ai choisi de m'appuyer sur **GEDCOM X**, la spécification ouverte portée par FamilySearch. Cet article explique ce choix et la conception qui en découle.

La présentation du projet est disponible sur [GitHub](https://github.com/issamWebdeveloper/ansaab).

## Pourquoi un standard plutôt qu'un modèle maison ?

La généalogie a un problème historique d'interopérabilité. Le format GEDCOM 5.5, créé dans les années 1980, reste le format d'échange le plus répandu, mais il est ambigu, textuel et interprété différemment par chaque logiciel. Importer un arbre d'une application à une autre se solde souvent par des pertes d'informations.

GEDCOM X a été conçu pour corriger cela :

- un **modèle conceptuel** clairement documenté, sérialisable en JSON et en XML ;
- une place centrale donnée aux **sources** et aux preuves, pas seulement aux conclusions ;
- une extensibilité propre, sans détourner les champs existants.

Mon expérience dans l'e-santé, où j'ai travaillé avec la norme FHIR pour l'interopérabilité des données médicales, m'a convaincu de la valeur d'un modèle standard : il impose une rigueur dès le départ et ouvre la porte aux échanges avec d'autres systèmes.

## Le modèle GEDCOM X en bref

Les principales entités du modèle :

| Entité | Rôle |
| --- | --- |
| `Person` | Un individu, avec ses noms, son genre et ses faits |
| `Relationship` | Un lien entre deux personnes : couple ou parent-enfant |
| `Fact` | Un événement ou une caractéristique : naissance, mariage, profession… |
| `PlaceDescription` | Un lieu, avec ses coordonnées et sa hiérarchie |
| `SourceDescription` | La description d'une source : acte, recensement, photo |
| `Agent` | Une personne ou organisation qui contribue aux données |

Point essentiel : **il n'existe pas d'entité « Famille »**. Une famille est déduite des relations. Un enfant est relié à chacun de ses parents par une relation `ParentChild`, et les parents entre eux par une relation `Couple`. Ce choix, qui peut surprendre, colle bien mieux à la réalité des familles recomposées, des adoptions ou des données incomplètes.

En TypeScript, une version simplifiée des types du frontend ressemble à ceci :

```ts
export interface Person {
  id: string;
  names: { nameForms: { fullText: string }[] }[];
  gender?: { type: 'http://gedcomx.org/Male' | 'http://gedcomx.org/Female' | 'http://gedcomx.org/Unknown' };
  facts?: Fact[];
  sources?: SourceReference[];
}

export interface Relationship {
  id: string;
  type: 'http://gedcomx.org/Couple' | 'http://gedcomx.org/ParentChild';
  person1: { resource: string }; // parent, dans une relation ParentChild
  person2: { resource: string }; // enfant
  facts?: Fact[];
}

export interface Fact {
  type: string;               // ex. http://gedcomx.org/Birth
  date?: { original?: string; formal?: string };
  place?: { original?: string; description?: string };
}
```

La distinction entre `original` (la date telle qu'écrite dans la source : « vers 1890 », « an XII ») et `formal` (la forme normalisée, exploitable par la machine) est précieuse : on ne perd jamais l'information d'origine.

## Des sources au centre

Un arbre généalogique n'a de valeur que s'il est vérifiable. Dans ANSAAB, chaque fait peut être rattaché à une ou plusieurs sources : un scan d'acte d'état civil, une photographie, un témoignage. L'interface met en évidence les faits **non sourcés**, pour inciter à les documenter plutôt que de laisser circuler des hypothèses comme des certitudes.

## Contrôler la cohérence des données

Les erreurs sont inévitables dans un arbre de plusieurs centaines de personnes : doublons, dates inversées, relations impossibles. ANSAAB prévoit un module d'analyse qui applique des règles simples et explicites :

```ts
type Rule = (p: Person, tree: Tree) => Warning[];

export const parentTooYoung: Rule = (child, tree) =>
  tree.parentsOf(child).flatMap(parent => {
    const gap = yearsBetween(birthOf(parent), birthOf(child));
    return gap !== null && gap < 13
      ? [{ level: 'warning', message: `${nameOf(parent)} aurait eu ${gap} ans à la naissance de ${nameOf(child)}` }]
      : [];
  });
```

Chaque règle est une fonction pure et indépendante : âge improbable au décès, naissance après la mort d'un parent, mariage avant la naissance, doublons probables (même nom, dates proches, mêmes parents). On en ajoute une nouvelle sans toucher aux autres, et chacune est testable isolément.

## Visualiser un arbre dans Angular

Afficher un arbre généalogique est plus difficile qu'il n'y paraît : ce n'est pas un arbre au sens informatique, mais un graphe (un enfant a deux parents, une personne peut avoir plusieurs conjoints). Les choix retenus :

- **Plusieurs vues** plutôt qu'une seule vue « parfaite » : ascendance (vers les ancêtres), descendance, et une vue centrée sur une personne avec ses proches.
- **Un rendu SVG** généré par des composants Angular, avec un calcul de positions séparé du rendu. La mise en page est une fonction pure qui transforme les personnes et relations en coordonnées ; le composant ne fait qu'afficher.
- **Un chargement progressif** : seules les générations visibles sont chargées, les autres le sont à la demande quand l'utilisateur déploie une branche.

```html
<svg [attr.viewBox]="viewBox()">
  @for (link of layout().links; track link.id) {
    <path class="link" [attr.d]="link.path" />
  }
  @for (node of layout().nodes; track node.person.id) {
    <g app-person-node [node]="node" (select)="focus(node.person.id)" />
  }
</svg>
```

`layout` est un signal calculé à partir de la personne sélectionnée et du nombre de générations affichées : changer de focus recalcule la mise en page, et Angular ne met à jour que les nœuds concernés grâce au `track`.

## Collaboration et confidentialité

La généalogie touche à des données personnelles, parfois sensibles, de personnes vivantes. Le projet prévoit dès la conception :

- des **permissions par arbre** : propriétaire, contributeur, lecteur ;
- le **masquage automatique des personnes vivantes** pour les visiteurs non autorisés ;
- l'export complet des données au format GEDCOM X, pour que l'utilisateur reste maître de son travail.

## Les cartes de migration

Les lieux GEDCOM X peuvent porter des coordonnées géographiques. En les exploitant, on peut tracer sur une carte les déplacements d'une lignée au fil des générations : lieux de naissance, de mariage, de décès. C'est souvent la fonctionnalité qui fait le plus réagir les familles, car elle raconte une histoire que les tableaux de dates ne montrent pas.

## En résumé

- S'appuyer sur un **standard ouvert** (GEDCOM X) garantit la rigueur du modèle et l'interopérabilité.
- Les **relations** remplacent la notion de famille, ce qui colle mieux à la réalité.
- Les **sources** et les **règles de cohérence** sont des fonctionnalités de premier plan, pas des options.
- Côté Angular, séparer **calcul de mise en page** et **rendu SVG** rend la visualisation testable et performante.

ANSAAB est un projet au long cours ; je partagerai ici les prochaines étapes de son développement.
