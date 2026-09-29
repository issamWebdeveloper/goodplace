import { Routes } from '@angular/router';

import { AdminLayout } from './admin-layout';

export const ADMIN_ROUTES: Routes = [
  {
    path: '',
    component: AdminLayout,
    children: [
      { path: '', loadComponent: () => import('./dashboard').then((m) => m.Dashboard) },
      { path: 'articles', loadComponent: () => import('./article-list').then((m) => m.ArticleList) },
      {
        path: 'articles/nouveau',
        loadComponent: () => import('./article-editor').then((m) => m.ArticleEditor),
      },
      {
        path: 'articles/:id',
        loadComponent: () => import('./article-editor').then((m) => m.ArticleEditor),
      },
      { path: 'utilisateurs', loadComponent: () => import('./users').then((m) => m.Users) },
    ],
  },
];
