import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

/** Connexion d'un abonné via le lien magique (valable 10 minutes, usage unique). */
@Component({
  selector: 'app-magic-link',
  imports: [RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>Connexion</h1>
          @if (error(); as err) {
            <p class="alert alert-error" role="alert">{{ err }}</p>
            <p><a class="btn btn-default" routerLink="/connexion">Demander un nouveau lien</a></p>
          } @else {
            <p class="intro" role="status">Connexion en cours…</p>
          }
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class MagicLink implements OnInit {
  private readonly auth = inject(AuthStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  protected readonly error = signal<string | null>(null);

  constructor() {
    inject(SeoService).set({ title: 'Connexion', path: '/auth/lien-magique', noindex: true });
  }

  async ngOnInit(): Promise<void> {
    try {
      await this.auth.consumeMagicLink(this.route.snapshot.queryParamMap.get('token') ?? '');
      await this.router.navigateByUrl('/compte', { replaceUrl: true });
    } catch (err) {
      this.error.set(errorMessage(err));
    }
  }
}
