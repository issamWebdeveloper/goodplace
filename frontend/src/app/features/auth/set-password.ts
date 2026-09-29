import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

/** Définition (après vérification) ou modification (lien 10 min) du mot de passe. */
@Component({
  selector: 'app-set-password',
  imports: [FormsModule, RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>{{ creation ? 'Choisissez votre mot de passe' : 'Nouveau mot de passe' }}</h1>
          @if (creation) {
            <p class="alert alert-success">Votre email est confirmé et votre compte est actif.</p>
          }
          <p class="intro">Au moins 10 caractères, avec des lettres et des chiffres. Ce lien est valable 10 minutes.</p>
          <form class="form-stack" (ngSubmit)="submit()">
            <div class="field">
              <label for="password">Mot de passe</label>
              <input id="password" class="input" type="password" name="password" [(ngModel)]="password" autocomplete="new-password" minlength="10" required />
            </div>
            <div class="field">
              <label for="confirm">Confirmation</label>
              <input id="confirm" class="input" type="password" name="confirm" [(ngModel)]="confirm" autocomplete="new-password" required />
            </div>
            @if (error(); as err) {
              <p class="alert alert-error" role="alert">{{ err }}</p>
            }
            <button class="btn btn-primary" [disabled]="pending() || !token()">Enregistrer</button>
          </form>
          <div class="links"><a routerLink="/mot-de-passe-oublie">Lien expiré ? En demander un nouveau</a></div>
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class SetPassword {
  private readonly auth = inject(AuthStore);
  private readonly router = inject(Router);
  private readonly params = inject(ActivatedRoute).snapshot.queryParamMap;

  protected readonly token = signal(this.params.get('token') ?? '');
  protected readonly creation = this.params.has('creation');
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected password = '';
  protected confirm = '';

  constructor() {
    inject(SeoService).set({ title: 'Mot de passe', path: '/auth/mot-de-passe', noindex: true });
    if (!this.token()) this.error.set('Lien invalide.');
  }

  async submit(): Promise<void> {
    if (this.password !== this.confirm) {
      this.error.set('Les deux mots de passe ne correspondent pas.');
      return;
    }
    this.pending.set(true);
    this.error.set(null);
    try {
      await this.auth.setPassword(this.token(), this.password);
      await this.router.navigate(['/compte'], { queryParams: { motdepasse: 'ok' } });
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
