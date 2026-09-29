import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

@Component({
  selector: 'app-forgot-password',
  imports: [FormsModule, RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>Mot de passe oublié</h1>
          @if (message(); as msg) {
            <p class="alert alert-success" role="status">{{ msg }}</p>
          } @else {
            <p class="intro">Indiquez votre email : vous recevrez un lien valable 10 minutes pour choisir un nouveau mot de passe.</p>
            <form class="form-stack" (ngSubmit)="submit()">
              <div class="field">
                <label for="email">Email</label>
                <input id="email" class="input" type="email" name="email" [(ngModel)]="email" autocomplete="email" required />
              </div>
              @if (error(); as err) {
                <p class="alert alert-error" role="alert">{{ err }}</p>
              }
              <button class="btn btn-primary" [disabled]="pending()">Envoyer le lien</button>
            </form>
          }
          <div class="links"><a routerLink="/connexion">← Retour à la connexion</a></div>
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class ForgotPassword {
  private readonly auth = inject(AuthStore);
  protected email = '';
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly message = signal<string | null>(null);

  constructor() {
    inject(SeoService).set({ title: 'Mot de passe oublié', path: '/mot-de-passe-oublie', noindex: true });
  }

  async submit(): Promise<void> {
    this.pending.set(true);
    this.error.set(null);
    try {
      this.message.set((await this.auth.forgotPassword(this.email)).message);
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
