import { Routes } from '@angular/router';

import { adminGuard, authGuard, guestGuard } from './core/auth/guards';
import { homeResolver } from './features/home/home.resolver';
import { articleResolver, blogListResolver } from './features/blog/blog.resolvers';

export const routes: Routes = [
  {
    path: '',
    loadComponent: () => import('./features/home/home').then((m) => m.Home),
    resolve: { data: homeResolver },
  },
  {
    path: 'blog',
    loadComponent: () => import('./features/blog/blog-list').then((m) => m.BlogList),
    resolve: { data: blogListResolver },
    runGuardsAndResolvers: 'paramsOrQueryParamsChange',
  },
  {
    path: 'blog/:slug',
    loadComponent: () => import('./features/blog/article-page').then((m) => m.ArticlePage),
    resolve: { article: articleResolver },
  },
  {
    path: 'connexion',
    canActivate: [guestGuard],
    loadComponent: () => import('./features/auth/login').then((m) => m.Login),
  },
  {
    path: 'inscription',
    canActivate: [guestGuard],
    loadComponent: () => import('./features/auth/register').then((m) => m.Register),
  },
  {
    path: 'mot-de-passe-oublie',
    loadComponent: () => import('./features/auth/forgot-password').then((m) => m.ForgotPassword),
  },
  {
    path: 'auth/verification',
    loadComponent: () => import('./features/auth/verify-email').then((m) => m.VerifyEmail),
  },
  {
    path: 'auth/mot-de-passe',
    loadComponent: () => import('./features/auth/set-password').then((m) => m.SetPassword),
  },
  {
    path: 'auth/lien-magique',
    loadComponent: () => import('./features/auth/magic-link').then((m) => m.MagicLink),
  },
  {
    path: 'compte',
    canActivate: [authGuard],
    loadComponent: () => import('./features/account/account').then((m) => m.Account),
  },
  {
    path: 'admin',
    canActivate: [adminGuard],
    loadChildren: () => import('./features/admin/admin.routes').then((m) => m.ADMIN_ROUTES),
  },
  {
    path: '**',
    loadComponent: () => import('./features/not-found/not-found').then((m) => m.NotFound),
  },
];
