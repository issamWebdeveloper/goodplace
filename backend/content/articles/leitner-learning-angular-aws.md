+++
title = "Leitner-Learning : des flashcards intelligentes avec Angular et AWS"
slug = "leitner-learning-angular-aws"
excerpt = "Retour sur la conception de Leitner-Learning, une application de mémorisation par répétition espacée construite avec Angular, Cognito, Lambda et DynamoDB."
tags = ["Angular", "AWS", "Projet", "Serverless"]
cover_image = "/images/blog/leitner.png"
seo_description = "Conception de Leitner-Learning : méthode Leitner, signals Angular, authentification Cognito, API serverless Lambda et modélisation DynamoDB."
status = "published"
published_at = "2024-11-04T08:00:00Z"
+++

Apprendre du vocabulaire, des formules ou des commandes Linux, c'est surtout lutter contre l'oubli. Dans les années 1970, le journaliste allemand Sebastian Leitner a proposé une méthode d'une simplicité désarmante pour y parvenir : des cartes, quelques boîtes, et une règle de déplacement. **Leitner-Learning** est mon implémentation web de cette méthode, avec un frontend Angular et un backend entièrement serverless sur AWS.

Le code est disponible sur [GitHub](https://github.com/issamWebdeveloper/Leitner-learning).

## La méthode Leitner en deux minutes

Chaque carte possède une question au recto et une réponse au verso. Les cartes sont réparties dans des boîtes numérotées :

- toutes les nouvelles cartes commencent dans la **boîte 1** ;
- une bonne réponse fait passer la carte dans la boîte suivante ;
- une mauvaise réponse la renvoie **dans la boîte 1** ;
- plus le numéro de boîte est élevé, plus la révision est espacée.

On révise ainsi très souvent ce que l'on ne maîtrise pas, et de moins en moins ce que l'on connaît. C'est une forme simple de **répétition espacée**, le principe qui sous-tend la plupart des applications de mémorisation modernes.

## L'algorithme de planification

Le cœur de l'application tient en une fonction pure, facile à tester :

```ts
export const BOX_INTERVALS_DAYS = [1, 2, 4, 8, 16, 32] as const;
export const MAX_BOX = BOX_INTERVALS_DAYS.length;

export interface Card {
  id: string;
  front: string;
  back: string;
  box: number;          // 1..MAX_BOX
  nextReviewAt: string; // ISO 8601
}

export function review(card: Card, correct: boolean, now = new Date()): Card {
  const box = correct ? Math.min(card.box + 1, MAX_BOX) : 1;
  const next = new Date(now);
  next.setDate(next.getDate() + BOX_INTERVALS_DAYS[box - 1]);
  return { ...card, box, nextReviewAt: next.toISOString() };
}
```

La fonction ne modifie rien : elle renvoie une nouvelle carte. C'est ce qui permet de la couvrir exhaustivement par des tests unitaires, et de l'exécuter indifféremment côté client (pour un retour instantané) ou côté serveur (pour faire foi).

## Le frontend Angular

L'interface a été construite avec Angular 18, en composants standalone et avec les **signals** pour l'état de la session de révision. Une session, c'est une file de cartes à réviser, une carte courante et quelques statistiques dérivées :

```ts
@Injectable({ providedIn: 'root' })
export class ReviewSession {
  private readonly queue = signal<Card[]>([]);
  private readonly results = signal<{ id: string; correct: boolean }[]>([]);

  readonly current = computed(() => this.queue()[0] ?? null);
  readonly remaining = computed(() => this.queue().length);
  readonly score = computed(() => {
    const r = this.results();
    return r.length ? Math.round((r.filter(x => x.correct).length / r.length) * 100) : 0;
  });

  start(cards: Card[]) {
    this.queue.set(cards);
    this.results.set([]);
  }

  answer(correct: boolean) {
    const card = this.current();
    if (!card) return;
    this.results.update(r => [...r, { id: card.id, correct }]);
    this.queue.update(([, ...rest]) => rest);
  }
}
```

Aucun store global, aucune bibliothèque externe : `current`, `remaining` et `score` sont dérivés et ne peuvent pas se désynchroniser. Le template utilise la nouvelle syntaxe de contrôle de flux, plus lisible que les directives structurelles :

```html
@if (session.current(); as card) {
  <app-flashcard [card]="card" (answered)="session.answer($event)" />
  <p class="progress">{{ session.remaining() }} carte(s) restante(s)</p>
} @else {
  <app-session-summary [score]="session.score()" />
}
```

L'animation de retournement de carte est un simple `transform: rotateY(180deg)` avec `backface-visibility: hidden` : pas besoin de librairie d'animation pour un effet aussi ciblé.

## Un backend serverless sur AWS

Pour une application personnelle dont l'usage est très irrégulier (intense le soir, nul la journée), le serverless est idéal : on ne paie que ce qui est consommé.

### Authentification avec Cognito

Les comptes utilisateurs sont gérés par **Amazon Cognito** : inscription, vérification de l'email, mot de passe oublié, jetons JWT. Côté Angular, un intercepteur HTTP ajoute le jeton d'accès à chaque appel de l'API, et un guard fonctionnel protège les routes :

```ts
export const authGuard: CanActivateFn = async () => {
  const auth = inject(AuthService);
  return (await auth.isSignedIn()) || inject(Router).createUrlTree(['/connexion']);
};
```

Déléguer l'authentification à un service managé évite de réécrire, et surtout de mal écrire, la gestion des mots de passe.

### API avec Lambda et API Gateway

Chaque opération (lister les paquets de cartes, récupérer les cartes à réviser, enregistrer une réponse) est une fonction **Lambda** en TypeScript, exposée par **API Gateway** avec un autorisateur Cognito. L'identifiant de l'utilisateur est extrait du jeton validé par la passerelle : une fonction ne peut jamais lire les cartes d'un autre utilisateur, même si l'identifiant d'une carte est deviné.

### Modélisation DynamoDB

**DynamoDB** impose de concevoir le modèle à partir des requêtes, et non l'inverse. La requête la plus fréquente est : « quelles cartes de cet utilisateur doivent être révisées aujourd'hui ? ». La table utilise donc une clé de partition par utilisateur et un index secondaire trié par date de prochaine révision :

| Attribut | Exemple | Rôle |
| --- | --- | --- |
| `PK` | `USER#8c1f…` | Partition par utilisateur |
| `SK` | `DECK#vocab-en#CARD#42` | Paquet et carte |
| `GSI1PK` | `USER#8c1f…` | Partition de l'index |
| `GSI1SK` | `2024-11-05T07:00:00Z` | Tri par prochaine révision |

Récupérer les cartes du jour devient une seule requête `Query` sur l'index avec la condition `GSI1SK <= maintenant`, quel que soit le nombre total de cartes. Les images associées aux cartes sont stockées dans **S3** et servies via des URL signées.

## Suivre sa progression

Un tableau de bord affiche la répartition des cartes par boîte. C'est la visualisation la plus parlante de la méthode : au fil des semaines, la « masse » des cartes glisse de la boîte 1 vers les boîtes supérieures. Un graphique en barres en SVG pur, généré par un composant Angular d'une cinquantaine de lignes, suffit à la représenter.

## Ce que j'en retiens

- **Une logique métier pure** (la fonction `review`) rend tout le reste plus simple : tests, réutilisation, confiance.
- **Les signals** conviennent parfaitement à l'état local d'un parcours utilisateur comme une session de révision.
- **Le serverless** est pertinent pour un usage irrégulier, à condition de modéliser DynamoDB à partir des requêtes réelles.
- **Cognito** retire une grosse part de risque sur l'authentification, au prix d'une configuration initiale un peu verbeuse.

Les prochaines étapes envisagées : un mode hors ligne grâce à un service worker, et l'import de paquets de cartes depuis un fichier CSV.
