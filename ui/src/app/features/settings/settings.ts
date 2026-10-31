import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import {
  SESSION_PAGE_SIZES,
  SettingsService,
  THEME_PREFERENCES,
  ThemePreference,
} from '../../core/settings.service';

const THEME_LABELS: Record<ThemePreference, string> = {
  system: 'System (follow my operating system)',
  light: 'Light',
  dark: 'Dark',
};

@Component({
  selector: 'app-settings',
  templateUrl: './settings.html',
  host: { class: 'block max-w-2xl px-6 py-4' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Settings {
  private readonly settings = inject(SettingsService);

  protected readonly themes = THEME_PREFERENCES;
  protected readonly theme = this.settings.theme;
  protected readonly pageSizes = SESSION_PAGE_SIZES;
  protected readonly sessionPageSize = this.settings.sessionPageSize;

  protected label(theme: ThemePreference): string {
    return THEME_LABELS[theme];
  }

  protected selectTheme(theme: ThemePreference): void {
    this.settings.setTheme(theme);
  }

  protected selectPageSize(event: Event): void {
    this.settings.setSessionPageSize(Number((event.target as HTMLSelectElement).value));
  }
}
