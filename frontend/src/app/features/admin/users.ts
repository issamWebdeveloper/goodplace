import { DatePipe } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, input, resource, signal } from '@angular/core';
import { RouterLink } from '@angular/router';

import { errorMessage } from '../../core/http-errors';
import { Role, User } from '../../core/models';
import { AdminService } from './admin.service';

@Component({
  selector: 'app-users',
  imports: [RouterLink, DatePipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <h1>Utilisateurs</h1>
    <nav class="filters" aria-label="Filtrer par profil">
      <a routerLink="/admin/utilisateurs" [class.active]="!role()">Tous</a>
      <a routerLink="/admin/utilisateurs" [queryParams]="{ role: 'user' }" [class.active]="role() === 'user'">Utilisateurs</a>
      <a routerLink="/admin/utilisateurs" [queryParams]="{ role: 'subscriber' }" [class.active]="role() === 'subscriber'">Abonnés</a>
    </nav>
    @if (error(); as err) {
      <p class="alert alert-error">{{ err }}</p>
    }
    <div class="card table-wrap">
      <table>
        <thead>
          <tr><th>Email</th><th>Profil</th><th>État</th><th>Inscription</th><th></th></tr>
        </thead>
        <tbody>
          @for (u of users.value(); track u.id) {
            <tr>
              <td>{{ u.email }} @if (u.name) {<small>{{ u.name }}</small>}</td>
              <td>{{ labels[u.role] }}</td>
              <td>
                @if (!u.email_verified) {
                  <span class="badge badge-validated">Email non vérifié</span>
                } @else if (u.role === 'user' && !u.has_password) {
                  <span class="badge badge-validated">Sans mot de passe</span>
                } @else {
                  <span class="badge badge-published">Actif</span>
                }
              </td>
              <td>{{ u.created_at | date: 'dd/MM/y' }}</td>
              <td>
                @if (u.role !== 'admin') {
                  @if (confirming() === u.id) {
                    <button class="btn btn-danger btn-small" (click)="remove(u)">Confirmer</button>
                  } @else {
                    <button class="btn btn-default btn-small" (click)="confirming.set(u.id)">Supprimer</button>
                  }
                }
              </td>
            </tr>
          } @empty {
            <tr><td colspan="5" class="muted">Aucun utilisateur.</td></tr>
          }
        </tbody>
      </table>
    </div>
  `,
  styles: `
    .filters {
      display: flex;
      gap: 8px;
      margin: 16px 0 24px;
      a {
        padding: 5px 14px;
        border-radius: 500px;
        border: 1px solid var(--gp-border);
        background: #fff;
        font-size: 0.82rem;
        color: var(--gp-text);
        &.active {
          color: #fff;
          background: var(--gp-gradient-button);
          border-color: transparent;
        }
      }
    }
    .table-wrap {
      overflow-x: auto;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.92rem;
    }
    th,
    td {
      padding: 12px 16px;
      text-align: left;
      border-bottom: 1px solid var(--gp-border);
    }
    th {
      font-size: 0.75rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--gp-muted);
    }
    small {
      display: block;
      color: var(--gp-muted);
    }
  `,
})
export class Users {
  private readonly admin = inject(AdminService);

  readonly role = input<Role | undefined>();
  protected readonly labels = { admin: 'Administrateur', user: 'Utilisateur', subscriber: 'Abonné' };
  protected readonly confirming = signal<string | null>(null);
  protected readonly error = signal<string | null>(null);
  protected readonly users = resource({
    params: () => ({ role: this.role() }),
    loader: ({ params }) => this.admin.users(params.role),
  });

  async remove(user: User): Promise<void> {
    try {
      await this.admin.deleteUser(user.id);
      this.users.reload();
    } catch (err) {
      this.error.set(errorMessage(err));
    } finally {
      this.confirming.set(null);
    }
  }
}
