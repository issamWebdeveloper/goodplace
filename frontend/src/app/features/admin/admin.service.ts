import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { firstValueFrom } from 'rxjs';

import { AdminStats, Article, ArticleInput, ArticleStatus, Role, User } from '../../core/models';

@Injectable({ providedIn: 'root' })
export class AdminService {
  private readonly http = inject(HttpClient);
  private readonly base = '/api/admin';

  stats(): Promise<AdminStats> {
    return firstValueFrom(this.http.get<AdminStats>(`${this.base}/articles/stats`));
  }

  articles(status?: ArticleStatus): Promise<Article[]> {
    const params: Record<string, string> = status ? { status } : {};
    return firstValueFrom(this.http.get<Article[]>(`${this.base}/articles`, { params }));
  }

  article(id: string): Promise<Article> {
    return firstValueFrom(this.http.get<Article>(`${this.base}/articles/${id}`));
  }

  create(input: ArticleInput): Promise<Article> {
    return firstValueFrom(this.http.post<Article>(`${this.base}/articles`, input));
  }

  update(id: string, input: ArticleInput): Promise<Article> {
    return firstValueFrom(this.http.put<Article>(`${this.base}/articles/${id}`, input));
  }

  delete(id: string): Promise<unknown> {
    return firstValueFrom(this.http.delete(`${this.base}/articles/${id}`));
  }

  /** Valide l'article ; avec `scheduledAt`, il sera publié automatiquement par le cron. */
  validate(id: string, scheduledAt: string | null): Promise<Article> {
    return firstValueFrom(
      this.http.post<Article>(`${this.base}/articles/${id}/validate`, { scheduled_at: scheduledAt }),
    );
  }

  publish(id: string): Promise<Article> {
    return firstValueFrom(this.http.post<Article>(`${this.base}/articles/${id}/publish`, {}));
  }

  unpublish(id: string): Promise<Article> {
    return firstValueFrom(this.http.post<Article>(`${this.base}/articles/${id}/unpublish`, {}));
  }

  preview(contentMd: string): Promise<{ html: string; reading_minutes: number }> {
    return firstValueFrom(
      this.http.post<{ html: string; reading_minutes: number }>(`${this.base}/articles/preview`, {
        content_md: contentMd,
      }),
    );
  }

  users(role?: Role): Promise<User[]> {
    const params: Record<string, string> = role ? { role } : {};
    return firstValueFrom(this.http.get<User[]>(`${this.base}/users`, { params }));
  }

  deleteUser(id: string): Promise<unknown> {
    return firstValueFrom(this.http.delete(`${this.base}/users/${id}`));
  }
}

export const STATUS_LABELS: Record<ArticleStatus, string> = {
  draft: 'Brouillon',
  validated: 'Validé',
  published: 'Publié',
};
