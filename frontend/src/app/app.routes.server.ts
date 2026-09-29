import { RenderMode, ServerRoute } from '@angular/ssr';

/**
 * Pages publiques : rendu serveur à chaque requête (contenu à jour + SEO).
 * Pages privées ou à usage unique (connexion, liens email, compte, administration) :
 * rendu uniquement dans le navigateur, où se trouve le cookie de session.
 */
export const serverRoutes: ServerRoute[] = [
  { path: '', renderMode: RenderMode.Server },
  { path: 'blog', renderMode: RenderMode.Server },
  { path: 'blog/:slug', renderMode: RenderMode.Server },
  { path: 'connexion', renderMode: RenderMode.Client },
  { path: 'inscription', renderMode: RenderMode.Client },
  { path: 'mot-de-passe-oublie', renderMode: RenderMode.Client },
  { path: 'auth/**', renderMode: RenderMode.Client },
  { path: 'compte', renderMode: RenderMode.Client },
  { path: 'admin/**', renderMode: RenderMode.Client },
  { path: '**', renderMode: RenderMode.Server },
];
