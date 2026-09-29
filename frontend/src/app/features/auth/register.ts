import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';
import { Logo } from '../../shared/logo';

type Kind = 'account' | 'newsletter';

@Component({
  selector: 'app-register',
  imports: [FormsModule, RouterLink, Logo],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="auth-page">
      <div class="card auth-card">
        <div class="card-body">
          <app-logo [size]="48" />
          <h1>Inscription</h1>

          <div class="tabs" role="tablist">
            <button type="button" role="tab" [attr.aria-selected]="kind() === 'account'" (click)="switch('account')">
              Compte
            </button>
            <button type="button" role="tab" [attr.aria-selected]="kind() === 'newsletter'" (click)="switch('newsletter')">
              Newsletter
            </button>
          </div>

          @if (message(); as msg) {
            <p class="alert alert-success" role="status">{{ msg }}</p>
          } @else {
            @if (kind() === 'account') {
              <p class="intro">
                Après confirmation de votre adresse email, vous pourrez définir votre mot de passe.
              </p>
            } @else {
              <p class="intro">
                Seul votre email est nécessaire. Vous le confirmez via un lien, puis vous accédez à
                votre espace par lien magique.
              </p>
            }
            <form class="form-stack" (ngSubmit)="submit()">
              @if (kind() === 'account') {
                <div class="field">
                  <label for="name">Nom</label>
                  <input id="name" class="input" name="name" [(ngModel)]="name" autocomplete="name" maxlength="80" required />
                </div>
              }
              <div class="field">
                <label for="email">Email</label>
                <input id="email" class="input" type="email" name="email" [(ngModel)]="email" autocomplete="email" required />
              </div>
              @if (error(); as err) {
                <p class="alert alert-error" role="alert">{{ err }}</p>
              }
              <button class="btn btn-primary" [disabled]="pending()">
                {{ kind() === 'account' ? 'Créer mon compte' : "M'abonner" }}
              </button>
            </form>
          }
          <div class="links">
            <a routerLink="/connexion">Déjà inscrit ? Se connecter</a>
          </div>
        </div>
      </div>
    </div>
  `,
  styleUrl: './auth-shared.scss',
})
export class Register {
  private readonly auth = inject(AuthStore);

  protected readonly kind = signal<Kind>(
    inject(ActivatedRoute).snapshot.queryParamMap.get('type') === 'newsletter' ? 'newsletter' : 'account',
  );
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly message = signal<string | null>(null);
  protected name = '';
  protected email = '';

  constructor() {
    inject(SeoService).set({ title: 'Inscription', path: '/inscription', noindex: true });
  }

  switch(kind: Kind): void {
    this.kind.set(kind);
    this.error.set(null);
  }

  async submit(): Promise<void> {
    this.pending.set(true);
    this.error.set(null);
    try {
      const res =
        this.kind() === 'account'
          ? await this.auth.register(this.name, this.email)
          : await this.auth.subscribe(this.email);
      this.message.set(res.message);
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
