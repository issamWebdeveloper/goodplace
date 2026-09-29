import { HttpClient } from '@angular/common/http';
import { inject } from '@angular/core';
import { ResolveFn } from '@angular/router';
import { catchError, forkJoin, of } from 'rxjs';

import { ArticlePage, PublicArticle, TagCount } from '../../core/models';

export interface BlogListData {
  page: ArticlePage;
  tags: TagCount[];
  tag: string | null;
}

export const blogListResolver: ResolveFn<BlogListData> = (route) => {
  const http = inject(HttpClient);
  const tag = route.queryParamMap.get('tag');
  const page = Math.max(1, Number(route.queryParamMap.get('page')) || 1);
  const params: Record<string, string | number> = { page, per_page: 9 };
  if (tag) params['tag'] = tag;
  return forkJoin({
    page: http.get<ArticlePage>('/api/articles', { params }),
    tags: http.get<TagCount[]>('/api/articles/tags'),
    tag: of(tag),
  });
};

/** Retourne `null` si l'article n'existe pas : la page affiche alors un 404. */
export const articleResolver: ResolveFn<PublicArticle | null> = (route) =>
  inject(HttpClient)
    .get<PublicArticle>(`/api/articles/${encodeURIComponent(route.paramMap.get('slug') ?? '')}`)
    .pipe(catchError(() => of(null)));
