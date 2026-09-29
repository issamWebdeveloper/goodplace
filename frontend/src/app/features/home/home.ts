import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, computed, effect, inject, input } from '@angular/core';
import { RouterLink } from '@angular/router';

import { SITE } from '../../core/config';
import { WorkItem } from '../../core/models';
import { SeoService } from '../../core/seo/seo.service';
import { ArticleCard } from '../../shared/article-card';
import { NewsletterForm } from '../../shared/newsletter-form';
import { HomeData } from './home.resolver';

const MONTHS = ['janv.', 'févr.', 'mars', 'avr.', 'mai', 'juin', 'juil.', 'août', 'sept.', 'oct.', 'nov.', 'déc.'];

function formatMonth(date: string): string {
  const [y, m] = date.split('-').map(Number);
  return `${MONTHS[(m || 1) - 1]} ${y}`;
}

@Component({
  selector: 'app-home',
  imports: [RouterLink, ArticleCard, NewsletterForm, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './home.html',
  styleUrl: './home.scss',
})
export class Home {
  private readonly seo = inject(SeoService);

  /** Données du resolver, injectées par `withComponentInputBinding`. */
  readonly data = input.required<HomeData>();

  protected readonly profile = computed(() => this.data().profile);
  protected readonly basics = computed(() => this.profile().basics);
  protected readonly firstName = computed(() => this.basics().name.split(' ')[0]);

  protected readonly experienceYears = computed(() => {
    const starts = this.profile().work.map((w) => Number(w.startDate.slice(0, 4)));
    return new Date().getFullYear() - Math.min(...starts);
  });

  protected readonly skills = computed(() => this.profile().skills);
  protected readonly work = computed(() => this.profile().work.slice(0, 8));
  protected readonly projects = computed(() => this.profile().projects);
  protected readonly latest = computed(() => this.data().latest.items);
  protected readonly linkedin = computed(() =>
    this.basics().profiles.find((p) => p.network === 'LinkedIn'),
  );
  protected readonly portfolio = computed(() =>
    this.basics().profiles.find((p) => p.network === 'Portfolio'),
  );
  protected readonly github = computed(() =>
    this.basics().profiles.find((p) => p.network === 'GitHub'),
  );

  constructor() {
    effect(() => {
      const b = this.basics();
      this.seo.set({
        title: `${b.name} — Développeur web & IA`,
        description: SITE.description,
        path: '/',
        type: 'profile',
        jsonLd: [
          {
            '@context': 'https://schema.org',
            '@type': 'Person',
            name: b.name,
            jobTitle: b.label,
            email: `mailto:${b.email}`,
            image: this.seo.absolute(b.image),
            url: SITE.url,
            address: { '@type': 'PostalAddress', addressLocality: 'Caen', addressCountry: 'FR' },
            sameAs: b.profiles.map((p) => p.url),
            knowsAbout: this.skills().flatMap((s) => s.keywords),
          },
          {
            '@context': 'https://schema.org',
            '@type': 'WebSite',
            name: SITE.name,
            url: SITE.url,
            inLanguage: 'fr-FR',
          },
        ],
      });
    });
  }

  protected period(item: WorkItem): string {
    const end = item.endDate ? formatMonth(item.endDate) : "aujourd'hui";
    return `${formatMonth(item.startDate)} — ${end}`;
  }

  /** Tracés SVG (24×24, trait) associés à chaque famille de compétences. */
  protected skillIcon(name: string): string {
    const n = name.toLowerCase();
    if (n.includes('ia')) return 'M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z';
    if (n.includes('front')) return 'M3 5h18v14H3zM3 9h18M7 7h.01M10 7h.01';
    if (n.includes('back')) return 'M4 4h16v6H4zM4 14h16v6H4zM8 7h.01M8 17h.01';
    if (n.includes('api')) return 'M7 7h13l-3-3M17 17H4l3 3';
    if (n.includes('devops')) return 'M8 8a4 4 0 1 0 0 8c3 0 5-8 8-8a4 4 0 1 1 0 8c-3 0-5-8-8-8z';
    if (n.includes('santé')) return 'M10 3h4v7h7v4h-7v7h-4v-7H3v-4h7z';
    return 'M9 11l3 3 8-8M20 12v7H4V5h11';
  }

}
