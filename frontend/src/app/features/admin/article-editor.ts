import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, OnInit, inject, input, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';

import { errorMessage } from '../../core/http-errors';
import { Article, ArticleInput } from '../../core/models';
import { AdminService, STATUS_LABELS } from './admin.service';

/** Convertit une date ISO en valeur pour `<input type="datetime-local">` (heure locale). */
function toLocalInput(iso: string | null): string {
  if (!iso) return '';
  const d = new Date(iso);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

@Component({
  selector: 'app-article-editor',
  imports: [FormsModule, RouterLink, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './article-editor.html',
  styleUrl: './article-editor.scss',
})
export class ArticleEditor implements OnInit {
  private readonly admin = inject(AdminService);
  private readonly router = inject(Router);

  /** Paramètre de route `:id` (absent pour un nouvel article). */
  readonly id = input<string>();

  protected readonly labels = STATUS_LABELS;
  protected readonly article = signal<Article | null>(null);
  protected readonly tab = signal<'write' | 'preview'>('write');
  protected readonly previewHtml = signal('');
  protected readonly previewMinutes = signal(0);
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly notice = signal<string | null>(null);
  protected readonly confirmDelete = signal(false);

  protected form: ArticleInput = {
    title: '',
    slug: '',
    excerpt: '',
    content_md: '',
    cover_image: '',
    tags: [],
    seo_title: '',
    seo_description: '',
  };
  protected tagsText = '';
  protected scheduledAt = '';

  async ngOnInit(): Promise<void> {
    // Message transmis lors de la redirection qui suit la création.
    const notice = (history.state as { notice?: string } | null)?.notice;
    if (notice) this.notice.set(notice);
    const id = this.id();
    if (!id) return;
    try {
      this.load(await this.admin.article(id));
    } catch (err) {
      this.error.set(errorMessage(err, 'Article introuvable.'));
    }
  }

  private load(a: Article): void {
    this.article.set(a);
    this.form = {
      title: a.title,
      slug: a.slug,
      excerpt: a.excerpt,
      content_md: a.content_md,
      cover_image: a.cover_image ?? '',
      tags: a.tags,
      seo_title: a.seo_title ?? '',
      seo_description: a.seo_description ?? '',
    };
    this.tagsText = a.tags.join(', ');
    this.scheduledAt = toLocalInput(a.scheduled_at);
  }

  private payload(): ArticleInput {
    return {
      ...this.form,
      slug: this.form.slug?.trim() || null,
      tags: this.tagsText.split(',').map((t) => t.trim()).filter(Boolean),
    };
  }

  async showPreview(): Promise<void> {
    this.tab.set('preview');
    try {
      const res = await this.admin.preview(this.form.content_md);
      this.previewHtml.set(res.html);
      this.previewMinutes.set(res.reading_minutes);
    } catch (err) {
      this.error.set(errorMessage(err));
    }
  }

  /** Enregistre le contenu ; retourne l'article à jour. */
  async save(showNotice = true): Promise<Article | null> {
    return this.run(async () => {
      const current = this.article();
      const saved = current
        ? await this.admin.update(current.id, this.payload())
        : await this.admin.create(this.payload());
      this.load(saved);
      if (!current) {
        await this.router.navigate(['/admin/articles', saved.id], {
          replaceUrl: true,
          state: { notice: 'Brouillon créé.' },
        });
      } else if (showNotice) {
        this.notice.set('Article enregistré.');
      }
      return saved;
    });
  }

  async validate(): Promise<void> {
    // Lue avant l'enregistrement, qui recharge le formulaire depuis l'API.
    const scheduled = this.scheduledAt ? new Date(this.scheduledAt).toISOString() : null;
    const saved = await this.save(false);
    if (!saved) return;
    await this.run(async () => {
      this.load(await this.admin.validate(saved.id, scheduled));
      this.notice.set(
        scheduled
          ? 'Article validé : il sera publié automatiquement à la date choisie.'
          : 'Article validé. Il reste hors ligne jusqu’à sa publication.',
      );
    });
  }

  async publish(): Promise<void> {
    const saved = await this.save(false);
    if (!saved) return;
    await this.run(async () => {
      this.load(await this.admin.publish(saved.id));
      this.notice.set('Article publié et en ligne.');
    });
  }

  async unpublish(): Promise<void> {
    const current = this.article();
    if (!current) return;
    await this.run(async () => {
      this.load(await this.admin.unpublish(current.id));
      this.notice.set('Article repassé en brouillon.');
    });
  }

  async remove(): Promise<void> {
    const current = this.article();
    if (!current) return;
    await this.run(async () => {
      await this.admin.delete(current.id);
      await this.router.navigateByUrl('/admin/articles');
    });
  }

  private async run<T>(action: () => Promise<T>): Promise<T | null> {
    this.pending.set(true);
    this.error.set(null);
    this.notice.set(null);
    try {
      return await action();
    } catch (err) {
      this.error.set(errorMessage(err));
      return null;
    } finally {
      this.pending.set(false);
    }
  }
}
