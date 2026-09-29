+++
title = "simpleCV : un éditeur de CV full-stack avec Angular et AWS Amplify Gen 2"
slug = "angular-amplify-graphql-simplecv"
excerpt = "Comment Angular, AWS Amplify Gen 2 et GraphQL permettent de construire rapidement un éditeur de CV en ligne sécurisé, du schéma de données jusqu'à l'export PDF."
tags = ["Angular", "AWS", "GraphQL", "Projet"]
cover_image = "/images/blog/simplecv.png"
seo_description = "Construire simpleCV avec Angular et AWS Amplify Gen 2 : schéma de données TypeScript, AppSync GraphQL, authentification Cognito, éditeur réactif et export PDF."
status = "published"
published_at = "2025-01-20T08:00:00Z"
+++

**simpleCV** est un projet personnel né d'un constat banal : mettre à jour son CV reste une corvée. Les traitements de texte cassent la mise en page à chaque modification, et les services en ligne enferment souvent les données derrière un abonnement. L'objectif : un éditeur simple, plusieurs modèles de mise en page, un export PDF propre, et des données stockées de façon sécurisée.

Pour le construire, je suis parti du starter officiel **Angular + AWS Amplify**, dont j'ai publié un fork sur [GitHub](https://github.com/issamWebdeveloper/amplify-angular). Cet article présente l'architecture et les choix qui m'ont fait gagner le plus de temps.

## Pourquoi Amplify Gen 2 ?

Amplify existe depuis longtemps, mais sa deuxième génération change l'expérience développeur : **tout le backend est décrit en TypeScript**, dans le même dépôt que le frontend. Plus de CLI interactive qui génère des fichiers opaques : le schéma de données, l'authentification et les règles d'accès sont du code versionné, relu en merge request comme le reste.

Derrière, Amplify provisionne les services AWS habituels :

- **Cognito** pour les comptes utilisateurs ;
- **AppSync** pour l'API GraphQL ;
- **DynamoDB** pour le stockage ;
- **S3** pour les fichiers (photos de profil).

Chaque développeur dispose en plus d'un **bac à sable cloud** personnel (`npx ampx sandbox`), déployé en quelques secondes à chaque sauvegarde.

## Le schéma de données

Un CV est un document structuré. Je me suis inspiré du format **JSON Resume**, un standard ouvert qui découpe un CV en sections (`basics`, `work`, `education`, `skills`…). Côté Amplify, le schéma tient dans un seul fichier :

```ts
// amplify/data/resource.ts
import { a, defineData, type ClientSchema } from '@aws-amplify/backend';

const schema = a.schema({
  Resume: a
    .model({
      title: a.string().required(),
      template: a.enum(['classic', 'modern', 'compact']),
      accentColor: a.string(),
      content: a.json().required(), // document au format JSON Resume
    })
    .authorization(allow => [allow.owner()]),
});

export type Schema = ClientSchema<typeof schema>;
export const data = defineData({ schema, authorizationModes: { defaultAuthorizationMode: 'userPool' } });
```

La règle `allow.owner()` est essentielle : AppSync ajoute automatiquement l'identifiant du propriétaire à chaque enregistrement et filtre les requêtes en conséquence. Un utilisateur ne peut ni lire ni modifier le CV d'un autre, sans une seule ligne de code d'autorisation côté client.

Stocker le contenu dans un champ JSON plutôt que dans une dizaine de modèles relationnels est un choix assumé : un CV est lu et écrit d'un bloc, on n'interroge jamais « toutes les expériences de tous les utilisateurs ». Le document reste ainsi exportable tel quel au format JSON Resume.

## Un client GraphQL typé de bout en bout

Côté Angular, le client généré à partir du type `Schema` est entièrement typé. Pas de requêtes GraphQL écrites à la main, pas de types dupliqués :

```ts
import { generateClient } from 'aws-amplify/data';
import type { Schema } from '../../amplify/data/resource';

@Injectable({ providedIn: 'root' })
export class ResumeStore {
  private client = generateClient<Schema>();
  readonly resumes = signal<Schema['Resume']['type'][]>([]);

  constructor() {
    this.client.models.Resume.observeQuery().subscribe(({ items }) => this.resumes.set(items));
  }

  save(id: string, content: unknown) {
    return this.client.models.Resume.update({ id, content: JSON.stringify(content) });
  }
}
```

`observeQuery` combine une requête initiale et un abonnement temps réel : si le CV est modifié dans un autre onglet, la liste se met à jour d'elle-même. Le typage remonte jusqu'aux templates : renommer un champ dans le schéma fait échouer la compilation Angular à chaque endroit concerné.

## L'authentification

Amplify fournit des composants d'interface prêts à l'emploi pour Angular. L'écran de connexion complet (inscription, vérification de l'email par code, mot de passe oublié) tient dans une balise :

```html
<amplify-authenticator [loginMechanisms]="['email']">
  <ng-template amplifySlot="authenticated" let-user="user" let-signOut="signOut">
    <app-shell [user]="user" (logout)="signOut()" />
  </ng-template>
</amplify-authenticator>
```

Le composant est personnalisable via des variables CSS, ce qui a permis de l'aligner sur l'identité visuelle de l'application sans le réécrire.

## Un éditeur réactif

L'éditeur est découpé en un formulaire par section du CV, avec les **formulaires réactifs typés** d'Angular. L'aperçu, à droite de l'écran, se met à jour à chaque frappe. Pour ne pas saturer l'API, la sauvegarde est différée : on attend une pause dans la saisie avant d'enregistrer.

```ts
this.form.valueChanges
  .pipe(debounceTime(800), distinctUntilChanged(isEqual), takeUntilDestroyed())
  .subscribe(value => this.store.save(this.id, value));
```

Un indicateur discret « Enregistré » rassure l'utilisateur, sans bouton de sauvegarde à penser.

Les **modèles de mise en page** sont des composants Angular qui reçoivent le même document JSON Resume en entrée. Ajouter un modèle, c'est ajouter un composant : aucune donnée à migrer.

## L'export PDF

Plutôt que de générer le PDF avec une bibliothèque JavaScript (qui transforme souvent le texte en image, le rendant illisible pour les logiciels de recrutement), l'export repose sur le **moteur d'impression du navigateur** et une feuille de style dédiée :

```css
@page { size: A4; margin: 14mm; }

@media print {
  app-toolbar, app-editor { display: none; }
  .resume { box-shadow: none; width: auto; }
  .resume section { break-inside: avoid; }
}
```

Le texte reste sélectionnable et indexable, les liens restent cliquables, et le rendu est fidèle à l'aperçu. Les options de personnalisation (couleur d'accent, densité, affichage de la photo) sont de simples variables CSS appliquées au modèle.

## Déploiement continu

Amplify Hosting se connecte directement au dépôt Git : chaque push sur la branche principale construit le frontend, déploie le backend et publie le tout. Chaque branche peut même disposer de son propre environnement complet, pratique pour tester une fonctionnalité isolément.

## Bilan

- **Amplify Gen 2** rend le backend AWS accessible à un développeur front-end, avec une infrastructure décrite en TypeScript.
- Le **typage de bout en bout**, du schéma aux templates Angular, supprime une catégorie entière de bugs.
- Les **règles d'autorisation déclaratives** (`allow.owner()`) sécurisent les données sans code supplémentaire.
- Un **export PDF par CSS d'impression** donne un meilleur résultat qu'une génération côté client.

La contrepartie est un certain couplage avec l'écosystème AWS. Pour un projet personnel qui doit avancer vite et rester sécurisé, c'est un compromis que j'accepte volontiers.
