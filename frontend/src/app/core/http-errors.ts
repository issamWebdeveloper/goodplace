import { HttpErrorResponse } from '@angular/common/http';

/** Extrait le message d'erreur renvoyé par l'API (`{ "error": "..." }`). */
export function errorMessage(err: unknown, fallback = 'Une erreur est survenue, veuillez réessayer.'): string {
  if (err instanceof HttpErrorResponse) {
    if (typeof err.error?.error === 'string') return err.error.error;
    if (err.status === 0) return 'Serveur injoignable. Vérifiez votre connexion.';
  }
  return fallback;
}
