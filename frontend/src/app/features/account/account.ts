import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';

import { AuthStore } from '../../core/auth/auth.store';
import { errorMessage } from '../../core/http-errors';
import { SeoService } from '../../core/seo/seo.service';

@Component({
  selector: 'app-account',
  imports: [FormsModule, RouterLink, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (auth.user(); as user) {
      <section class="section-primary head">
        <div class="container container-narrow">
          <span class="text-meta">{{ user.role === 'subscriber' ? 'Espace abonné' : 'Mon compte' }}</span>
          <h1 class="heading-section">Bonjour {{ user.name || user.email }}</h1>
        </div>
      </section>

      <section class="section">
        <div class="container container-narrow stack">
          @if (notice(); as n) {
            <p class="alert alert-success" role="status">{{ n }}</p>
          }
          @if (error(); as err) {
            <p class="alert alert-error" role="alert">{{ err }}</p>
          }

          <div class="card">
            <div class="card-body">
              <h2>Informations</h2>
              <dl>
                <dt>Email</dt>
                <dd>{{ user.email }}</dd>
                <dt>Profil</dt>
                <dd>{{ roleLabel[user.role] }}</dd>
                <dt>Inscrit depuis</dt>
                <dd>{{ user.created_at | date: 'd MMMM y' }}</dd>
              </dl>
              @if (user.role === 'user') {
                <form class="inline-form" (ngSubmit)="saveName()">
                  <div class="field">
                    <label for="name">Nom affiché</label>
                    <input id="name" class="input" name="name" [(ngModel)]="name" maxlength="80" />
                  </div>
                  <button class="btn btn-default" [disabled]="pending()">Enregistrer</button>
                </form>
              }
            </div>
          </div>

          @if (user.role === 'user') {
            <div class="card">
              <div class="card-body">
                <h2>Mot de passe</h2>
                <p class="muted">
                  Pour votre sécurité, la modification passe par un lien envoyé à votre adresse email,
                  valable 10 minutes. Toutes vos autres sessions seront déconnectées.
                </p>
                <button class="btn btn-primary" (click)="changePassword()" [disabled]="pending()">
                  Recevoir le lien de modification
                </button>
              </div>
            </div>
          }

          @if (user.role === 'subscriber') {
            <div class="card">
              <div class="card-body">
                <h2>Newsletter</h2>
                <p class="muted">
                  Vous recevez un email à chaque nouvel article. Pour vous reconnecter, demandez
                  simplement un lien depuis la page de connexion.
                </p>
                <a class="btn btn-default" routerLink="/blog">Lire les articles</a>
              </div>
            </div>
          }

          <div class="card danger">
            <div class="card-body">
              <h2>{{ user.role === 'subscriber' ? 'Se désabonner' : 'Supprimer mon compte' }}</h2>
              @if (confirmDelete()) {
                <p>Cette action est définitive. Confirmer ?</p>
                <div class="row">
                  <button class="btn btn-danger" (click)="deleteAccount()" [disabled]="pending()">Oui, supprimer</button>
                  <button class="btn btn-default" (click)="confirmDelete.set(false)">Annuler</button>
                </div>
              } @else {
                <p class="muted">Vos données personnelles seront supprimées immédiatement.</p>
                <button class="btn btn-default" (click)="confirmDelete.set(true)">
                  {{ user.role === 'subscriber' ? 'Me désabonner' : 'Supprimer mon compte' }}
                </button>
              }
            </div>
          </div>

          <p><button class="btn btn-default" (click)="logout()">Se déconnecter</button></p>
        </div>
      </section>
    }
  `,
  styles: `
    .head {
      padding-block: 64px;
    }
    .stack {
      display: grid;
      gap: 24px;
    }
    h2 {
      font-size: 1.25rem;
      font-weight: 700;
    }
    dl {
      display: grid;
      grid-template-columns: max-content 1fr;
      gap: 6px 24px;
      margin: 0 0 20px;
    }
    dt {
      color: var(--gp-muted);
    }
    dd {
      margin: 0;
      font-weight: 600;
    }
    .inline-form {
      display: flex;
      align-items: flex-end;
      gap: 12px;
      flex-wrap: wrap;
      .field {
        flex: 1 1 220px;
      }
    }
    .danger {
      border-top: 3px solid var(--gp-danger);
    }
    .row {
      display: flex;
      gap: 12px;
      flex-wrap: wrap;
    }
  `,
})
export class Account {
  protected readonly auth = inject(AuthStore);
  private readonly router = inject(Router);

  protected readonly roleLabel = { admin: 'Administrateur', user: 'Utilisateur', subscriber: 'Abonné newsletter' };
  protected name = this.auth.user()?.name ?? '';
  protected readonly pending = signal(false);
  protected readonly notice = signal<string | null>(
    inject(ActivatedRoute).snapshot.queryParamMap.get('motdepasse') === 'ok'
      ? 'Votre mot de passe a été enregistré.'
      : null,
  );
  protected readonly error = signal<string | null>(null);
  protected readonly confirmDelete = signal(false);

  constructor() {
    inject(SeoService).set({ title: 'Mon compte', path: '/compte', noindex: true });
  }

  async saveName(): Promise<void> {
    await this.run(async () => {
      await this.auth.updateName(this.name);
      this.notice.set('Nom mis à jour.');
    });
  }

  async changePassword(): Promise<void> {
    await this.run(async () => this.notice.set((await this.auth.requestPasswordChange()).message));
  }

  async deleteAccount(): Promise<void> {
    await this.run(async () => {
      await this.auth.deleteAccount();
      await this.router.navigateByUrl('/');
    });
  }

  async logout(): Promise<void> {
    await this.auth.logout();
    await this.router.navigateByUrl('/');
  }

  private async run(action: () => Promise<void>): Promise<void> {
    this.pending.set(true);
    this.error.set(null);
    try {
      await action();
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.pending.set(false);
    }
  }
}
