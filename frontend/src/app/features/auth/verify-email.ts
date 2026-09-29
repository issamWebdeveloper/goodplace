import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

/** Page ouverte depuis le lien de confirmation reçu par email. */
@Component({
  selector: 'app-verify-email',
  imports: [RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>Confirmation de l'email</h1>
          @switch (state()) {
            @case ('pending') {
              <p class="intro" role="status">Vérification en cours…</p>
            }
            @case ('subscriber') {
              <p class="alert alert-success" role="status">
                Merci, votre inscription à la newsletter est confirmée ! Vous êtes connecté à votre espace abonné.
              </p>
              <p><a class="btn btn-primary" routerLink="/compte">Mon espace</a></p>
            }
            @case ('done') {
              <p class="alert alert-success" role="status">Votre adresse est confirmée. Vous pouvez vous connecter.</p>
              <p><a class="btn btn-primary" routerLink="/connexion">Se connecter</a></p>
            }
            @case ('error') {
              <p class="alert alert-error" role="alert">{{ error() }}</p>
              <p class="intro">Le lien a peut-être expiré ou déjà été utilisé. Vous pouvez recommencer l'inscription pour en recevoir un nouveau.</p>
              <p><a class="btn btn-default" routerLink="/inscription">Recommencer</a></p>
            }
          }
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class VerifyEmail implements OnInit {
  private readonly auth = inject(AuthStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);

  protected readonly state = signal<'pending' | 'subscriber' | 'done' | 'error'>('pending');
  protected readonly error = signal('');

  constructor() {
    inject(SeoService).set({ title: 'Confirmation', path: '/auth/verification', noindex: true });
  }

  async ngOnInit(): Promise<void> {
    const token = this.route.snapshot.queryParamMap.get('token') ?? '';
    try {
      const result = await this.auth.verifyEmail(token);
      if (result.set_password_token) {
        // Compte utilisateur activé : étape suivante, définir le mot de passe.
        await this.router.navigate(['/auth/mot-de-passe'], {
          queryParams: { token: result.set_password_token, creation: 1 },
          replaceUrl: true,
        });
        return;
      }
      this.state.set(result.role === 'subscriber' ? 'subscriber' : 'done');
    } catch (err) {
      this.error.set(errorMessage(err));
      this.state.set('error');
    }
  }
}
