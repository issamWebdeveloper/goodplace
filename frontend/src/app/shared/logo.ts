import { ChangeDetectionStrategy, Component, input } from '@angular/core';

/** Monogramme « GP » de GoodPlace, dessiné en SVG (aucune dépendance à une police). */
@Component({
  selector: 'app-logo',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <svg
      viewBox="0 0 64 64"
      [attr.width]="size()"
      [attr.height]="size()"
      role="img"
      aria-label="GoodPlace"
    >
      <defs>
        <linearGradient [attr.id]="gradientId" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stop-color="#32beff" />
          <stop offset="1" stop-color="#0171cd" />
        </linearGradient>
      </defs>
      <rect width="64" height="64" rx="16" [attr.fill]="'url(#' + gradientId + ')'" />
      <g
        fill="none"
        stroke="#fff"
        stroke-width="5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M32.2 24.3A12 12 0 1 0 35 32H27" />
        <path d="M41 44V20h6a6 6 0 0 1 0 12h-6" />
      </g>
    </svg>
  `,
  styles: `
    :host {
      display: inline-flex;
    }
  `,
})
export class Logo {
  private static counter = 0;
  readonly size = input(40);
  protected readonly gradientId = `gp-logo-${Logo.counter++}`;
}
