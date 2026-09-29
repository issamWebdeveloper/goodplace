+++
title = "Performance web : les bases essentielles"
slug = "performance-web-les-bases-essentielles"
excerpt = "Optimisation des ressources et bonnes pratiques pour des sites rapides."
tags = ["Performance", "Angular", "Core Web Vitals", "Frontend"]
cover_image = "/images/blog/performance.png"
seo_description = "Core Web Vitals, poids des bundles, lazy loading, images, cache et rendu côté serveur : les fondamentaux de la performance web appliqués à une application Angular."
status = "published"
published_at = "2023-05-03T08:00:00Z"
+++

Un site lent coûte cher. Chaque seconde de chargement supplémentaire fait chuter le taux de conversion, augmente le taux de rebond et pèse désormais sur le référencement naturel, puisque Google intègre les Core Web Vitals à ses signaux de classement. Pourtant, la performance reste souvent traitée en fin de projet, quand les mauvaises décisions sont déjà coûteuses à corriger.

Cet article reprend les fondamentaux : ce qu'il faut mesurer, où partent réellement les millisecondes, et comment appliquer ces principes à une application Angular — le framework sur lequel je travaille au quotidien depuis de nombreuses années.

## Mesurer avant d'optimiser

La première règle est simple : **on n'optimise pas ce qu'on ne mesure pas**. Beaucoup d'équipes passent des jours à micro-optimiser une boucle JavaScript alors que 80 % du temps de chargement se joue sur une image hero de 2 Mo.

### Les Core Web Vitals

Google a formalisé trois indicateurs centrés sur l'expérience réelle de l'utilisateur :

- **LCP (Largest Contentful Paint)** : le moment où le plus grand élément visible (souvent une image ou un titre) est affiché. Objectif : moins de 2,5 secondes.
- **FID (First Input Delay)** : le délai entre la première interaction de l'utilisateur et la réaction du navigateur. Objectif : moins de 100 ms. Google a annoncé son remplacement prochain par l'**INP (Interaction to Next Paint)**, plus exigeant car il mesure toutes les interactions de la session, pas seulement la première. Autant s'y préparer dès maintenant.
- **CLS (Cumulative Layout Shift)** : la somme des décalages de mise en page inattendus. Objectif : moins de 0,1.

À ces trois indicateurs s'ajoutent des métriques de diagnostic utiles : le **TTFB** (temps de réponse du serveur), le **FCP** (premier contenu affiché) et le **TBT** (temps total pendant lequel le thread principal est bloqué).

### Données de laboratoire et données de terrain

Il existe deux familles de mesures, complémentaires :

- **Les données de laboratoire** (Lighthouse, WebPageTest, l'onglet Performance des DevTools) sont reproductibles et idéales pour diagnostiquer un problème. Mais elles simulent un seul appareil sur un seul réseau.
- **Les données de terrain** (le rapport CrUX de Chrome, ou votre propre collecte) reflètent l'expérience réelle de vos utilisateurs, avec leurs smartphones d'entrée de gamme et leurs connexions 4G instables.

Pour collecter vos propres données de terrain, la bibliothèque `web-vitals` de Google tient en quelques lignes :

```ts
import { onCLS, onFID, onLCP } from 'web-vitals';

function sendToAnalytics(metric: { name: string; value: number }) {
  navigator.sendBeacon('/analytics', JSON.stringify(metric));
}

onCLS(sendToAnalytics);
onFID(sendToAnalytics);
onLCP(sendToAnalytics);
```

Un conseil : testez toujours avec un profil mobile bridé (CPU ralenti 4×, réseau « Fast 3G »). Votre MacBook sur la fibre n'est pas représentatif.

## Le chemin critique de rendu

Pour afficher une page, le navigateur doit télécharger le HTML, découvrir les CSS et scripts, construire le DOM et le CSSOM, calculer la mise en page puis peindre les pixels. Tout ce qui s'intercale sur ce chemin retarde l'affichage.

Les leviers principaux :

1. **Réduire le nombre de ressources bloquantes.** Une feuille de style dans le `<head>` bloque le rendu ; un script sans `defer` ni `async` bloque l'analyse du HTML.
2. **Réduire leur poids.** Minification, compression, suppression du code mort.
3. **Raccourcir la distance.** Un CDN proche de l'utilisateur, des connexions anticipées vers les domaines tiers avec `preconnect`.

```html
<link rel="preconnect" href="https://api.mon-domaine.fr">
<link rel="preload" href="/assets/hero.webp" as="image">
```

Le CSS critique — celui nécessaire à l'affichage du haut de page — peut être inliné directement dans le HTML, et le reste chargé de façon asynchrone. Bonne nouvelle : Angular le fait pour vous en production grâce à l'option `inlineCritical`, activée par défaut depuis la version 12.

## Maîtriser le poids du JavaScript

Le JavaScript est la ressource la plus coûteuse du web. Contrairement à une image de même poids, il doit être téléchargé, **puis analysé, compilé et exécuté** sur le thread principal. Sur un smartphone milieu de gamme, 1 Mo de JavaScript peut représenter plusieurs secondes de travail processeur.

### Fixer des budgets

Angular permet de définir des budgets dans `angular.json`. Le build échoue si un seuil est dépassé : c'est le meilleur garde-fou contre la dérive progressive.

```json
"budgets": [
  { "type": "initial", "maximumWarning": "500kb", "maximumError": "1mb" },
  { "type": "anyComponentStyle", "maximumWarning": "4kb", "maximumError": "8kb" }
]
```

### Analyser le contenu des bundles

Quand un budget saute, il faut comprendre pourquoi. Générez les source maps et inspectez le bundle :

```bash
ng build --source-map
npx source-map-explorer dist/mon-app/*.js
```

Les coupables habituels : une bibliothèque de dates complète importée pour formater une seule date, une librairie d'icônes chargée en entier, `lodash` importé globalement au lieu de fonctions ciblées, ou un module partagé qui embarque la moitié de l'application.

### Découper avec le lazy loading

La technique la plus rentable dans une application Angular reste le **chargement différé des routes**. Chaque fonctionnalité devient un chunk distinct, téléchargé uniquement quand l'utilisateur y navigue. Avec les composants standalone, stabilisés dans Angular 15, la syntaxe est devenue très légère :

```ts
export const routes: Routes = [
  { path: '', component: HomeComponent },
  {
    path: 'admin',
    loadChildren: () => import('./admin/admin.routes').then(m => m.ADMIN_ROUTES),
  },
  {
    path: 'blog/:slug',
    loadComponent: () => import('./blog/article.component').then(m => m.ArticleComponent),
  },
];
```

Pour éviter une latence à la navigation, on peut précharger les chunks en arrière-plan une fois la page initiale chargée, avec `PreloadAllModules` ou une stratégie personnalisée qui ne précharge que les routes les plus probables.

### Un build plus rapide avec esbuild

Angular 16, sorti ce mois-ci, propose en developer preview un nouveau builder basé sur **esbuild**. Les gains annoncés sur le temps de build sont spectaculaires (souvent plus de 70 %). Cela n'accélère pas directement l'application en production, mais un cycle de build court encourage à mesurer plus souvent. Il suffit de remplacer `browser` par `browser-esbuild` dans `angular.json` pour l'essayer.

## Les images : le gisement le plus important

Sur la majorité des sites, les images représentent l'essentiel du poids de la page et sont très souvent l'élément LCP. C'est là que les gains sont les plus rapides.

- **Choisir le bon format** : WebP est désormais supporté partout, AVIF offre une compression encore meilleure sur les navigateurs récents. Le JPEG ne devrait plus être servi que comme solution de repli.
- **Servir la bonne taille** : inutile d'envoyer une image de 2 400 px de large à un mobile de 390 px. Les attributs `srcset` et `sizes` laissent le navigateur choisir.
- **Charger en différé ce qui est hors écran** avec `loading="lazy"`, mais **jamais l'image LCP**, qui doit au contraire être chargée en priorité.
- **Toujours indiquer les dimensions** (`width` et `height`) pour réserver l'espace et éviter les décalages de mise en page, donc le CLS.

Angular fournit depuis la version 15 la directive `NgOptimizedImage`, qui applique la plupart de ces règles automatiquement et affiche des avertissements en développement quand une image est mal configurée :

```html
<img ngSrc="/assets/hero.webp" width="1200" height="630" priority alt="Illustration de l'article">
<img ngSrc="/assets/avatar.webp" width="64" height="64" alt="Photo de l'auteur">
```

L'attribut `priority` marque l'image LCP : elle est chargée avec une priorité haute et Angular vérifie en développement que c'est bien l'élément LCP. Les autres images sont automatiquement en `loading="lazy"`. Couplée à un loader d'images (Cloudinary, Imgix, ou votre propre service), la directive génère également le `srcset`.

## La détection de changements dans Angular

Une application qui charge vite mais réagit lentement donne une mauvaise impression, et un mauvais score FID. Dans Angular, la réactivité dépend largement du mécanisme de détection de changements.

Par défaut, Zone.js déclenche une vérification de **tout l'arbre de composants** après chaque événement asynchrone : clic, requête HTTP, timer. Sur une application riche, cela devient coûteux. Trois réflexes :

### La stratégie OnPush

Avec `ChangeDetectionStrategy.OnPush`, un composant n'est revérifié que si l'une de ses entrées change de référence, si un événement se produit dans son template, ou si un observable consommé via le pipe `async` émet.

```ts
@Component({
  selector: 'app-article-card',
  standalone: true,
  templateUrl: './article-card.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ArticleCardComponent {
  @Input({ required: true }) article!: ArticleSummary;
}
```

Cela impose une discipline d'immutabilité : on remplace les objets au lieu de les modifier. C'est une bonne pratique de toute façon.

### trackBy dans les listes

Sans `trackBy`, un `*ngFor` recrée tous les éléments DOM dès que la référence du tableau change, par exemple après chaque rechargement depuis l'API. Avec une fonction d'identité, seuls les éléments réellement modifiés sont touchés :

```html
<app-article-card *ngFor="let a of articles; trackBy: trackBySlug" [article]="a" />
```

```ts
trackBySlug = (_: number, a: ArticleSummary) => a.slug;
```

### Les signals : l'avenir de la réactivité

Angular 16 introduit en developer preview les **signals**, une primitive de réactivité fine qui indique précisément quelles valeurs ont changé et qui en dépend :

```ts
const articles = signal<ArticleSummary[]>([]);
const count = computed(() => articles().length);
```

À terme, ils permettront de se passer de Zone.js et de ne mettre à jour que les portions de vue concernées. Il est encore tôt pour les généraliser en production, mais c'est le moment idéal pour les expérimenter sur des composants isolés.

Enfin, pour les traitements lourds (parsing d'un gros fichier CSV, calculs statistiques), sortez-les du thread principal avec un **Web Worker** : `ng generate web-worker` crée toute la configuration nécessaire.

## Le rendu côté serveur

Une application monopage classique envoie un HTML presque vide : l'utilisateur ne voit rien tant que le JavaScript n'est pas téléchargé et exécuté. Le **rendu côté serveur** (SSR) inverse la logique : le serveur envoie une page déjà construite, affichée immédiatement, puis l'application prend le relais dans le navigateur.

Les bénéfices sont doubles : un FCP et un LCP nettement meilleurs, et un contenu directement indexable par les moteurs de recherche. Avec Angular Universal, l'ajout se fait en une commande :

```bash
ng add @nguniversal/express-engine
```

Jusqu'ici, le passage du HTML serveur à l'application client était destructif : Angular effaçait le DOM rendu par le serveur pour le reconstruire, provoquant parfois un clignotement. Angular 16 introduit en preview l'**hydratation non destructive**, qui réutilise le DOM existant. Une simple ligne dans la configuration suffit :

```ts
bootstrapApplication(AppComponent, {
  providers: [provideClientHydration()],
});
```

Pour les pages dont le contenu change rarement, le **prérendu** (génération statique au moment du build) offre les mêmes avantages sans aucun serveur Node en production.

## Le réseau : compression et cache

Même le meilleur code front-end ne compense pas une configuration serveur négligée.

**Compression.** Tous les fichiers texte (HTML, CSS, JS, JSON, SVG) doivent être servis compressés. Brotli compresse 15 à 20 % mieux que Gzip et est supporté par tous les navigateurs modernes. Le plus efficace est de précompresser les fichiers au moment du build plutôt qu'à chaque requête.

**Cache HTTP.** Angular génère des noms de fichiers hashés (`main.3f2a9c.js`) : leur contenu ne change jamais, ils peuvent donc être mis en cache un an.

```nginx
location ~* \.(js|css|woff2|webp|avif)$ {
  add_header Cache-Control "public, max-age=31536000, immutable";
}
location = /index.html {
  add_header Cache-Control "no-cache";
}
```

Le fichier `index.html`, lui, ne doit jamais être mis en cache durablement, puisqu'il référence les nouveaux fichiers à chaque déploiement.

**HTTP/2 et HTTP/3.** Le multiplexage rend obsolètes certaines vieilles pratiques comme la concaténation extrême ou le domain sharding. Vérifiez simplement que votre hébergeur les active.

**Service worker.** Pour les visites récurrentes, `ng add @angular/pwa` ajoute un service worker qui met en cache l'application et permet un affichage quasi instantané, voire un fonctionnement hors ligne.

## Les polices et les scripts tiers

Deux sources de lenteur passent souvent sous le radar :

- **Les polices web** : préférez WOFF2, utilisez `font-display: swap`, limitez le nombre de graisses et préchargez la police principale. Angular inline automatiquement la feuille Google Fonts en production.
- **Les scripts tiers** : outils d'analyse, chat en ligne, tests A/B, widgets sociaux. Chacun ajoute des requêtes, du JavaScript et parfois des décalages de mise en page. Faites-en l'inventaire régulièrement, chargez-les en `defer` ou après l'interaction, et supprimez ceux que personne ne consulte.

## Intégrer la performance dans le processus

La performance n'est pas un chantier ponctuel mais une propriété qui se dégrade naturellement si personne ne la surveille. Quelques habitudes suffisent à la maintenir :

1. **Budgets dans le build** : le pipeline échoue si le bundle dépasse le seuil.
2. **Lighthouse CI** sur chaque merge request, avec des seuils minimaux sur les scores clés.
3. **Surveillance des données de terrain** : un tableau de bord des Core Web Vitals réels, consulté à chaque sprint.
4. **Revue des dépendances** : avant d'ajouter une librairie, vérifier son poids (bundlephobia.com) et se demander si quelques lignes de code ne suffiraient pas.

```yaml
# .gitlab-ci.yml
lighthouse:
  stage: test
  script:
    - npm ci && npm run build
    - npx @lhci/cli autorun --assert.preset=lighthouse:recommended
```

## En résumé

- Mesurez d'abord, avec des données de laboratoire pour diagnostiquer et des données de terrain pour décider.
- Surveillez LCP, FID (bientôt INP) et CLS.
- Réduisez le JavaScript initial : budgets, analyse des bundles, lazy loading des routes.
- Traitez les images en priorité : formats modernes, bonnes dimensions, `NgOptimizedImage`.
- Allégez la détection de changements : OnPush, `trackBy`, et gardez un œil sur les signals.
- Servez du HTML dès la première requête grâce au SSR et à l'hydratation.
- Configurez correctement compression et cache côté serveur.
- Automatisez les contrôles pour que la performance ne régresse pas en silence.

Un site rapide n'est pas le fruit d'une astuce miracle, mais d'une somme de décisions raisonnables prises tout au long du projet. Et vos utilisateurs sur mobile, dans le train, avec une batterie à 15 %, vous en seront reconnaissants.
