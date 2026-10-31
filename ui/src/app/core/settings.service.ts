import { DOCUMENT } from '@angular/common';
import { Injectable, inject, signal } from '@angular/core';

/** `system` follows the operating system's light or dark setting. */
export type ThemePreference = 'system' | 'light' | 'dark';

export const THEME_PREFERENCES: readonly ThemePreference[] = ['system', 'light', 'dark'];

const STORAGE_KEY = 'agenteye.settings';

/** Everything stored under {@link STORAGE_KEY}. New settings are added here as optional fields. */
interface StoredSettings {
  theme?: unknown;
}

/** User settings, remembered between launches and applied as soon as they change. */
@Injectable({ providedIn: 'root' })
export class SettingsService {
  private readonly root = inject(DOCUMENT).documentElement;

  private readonly themeState = signal<ThemePreference>(this.loadTheme());

  readonly theme = this.themeState.asReadonly();

  constructor() {
    // Apply right away (not in an effect) so the first paint already has the right theme.
    this.applyTheme(this.themeState());
  }

  setTheme(theme: ThemePreference): void {
    this.themeState.set(theme);
    this.applyTheme(theme);
    this.save({ theme });
  }

  /** `system` removes the attribute so the stylesheet falls back to the OS preference. */
  private applyTheme(theme: ThemePreference): void {
    if (theme === 'system') {
      this.root.removeAttribute('data-theme');
    } else {
      this.root.setAttribute('data-theme', theme);
    }
  }

  private loadTheme(): ThemePreference {
    const theme = this.read().theme;
    return THEME_PREFERENCES.find((candidate) => candidate === theme) ?? 'system';
  }

  /** Storage can be missing or blocked, and its content can be anything; fall back to defaults. */
  private read(): StoredSettings {
    try {
      const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
      return typeof parsed === 'object' && parsed !== null ? (parsed as StoredSettings) : {};
    } catch {
      return {};
    }
  }

  private save(changes: StoredSettings): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...this.read(), ...changes }));
    } catch {
      // Not persisting is acceptable; the setting still applies for this session.
    }
  }
}
