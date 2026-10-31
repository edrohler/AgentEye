import { DOCUMENT } from '@angular/common';
import { Injectable, inject, signal } from '@angular/core';

/** `system` follows the operating system's light or dark setting. */
export type ThemePreference = 'system' | 'light' | 'dark';

export const THEME_PREFERENCES: readonly ThemePreference[] = ['system', 'light', 'dark'];

/** How many sessions the list shows at first, and adds each time "View more" is pressed. */
export const SESSION_PAGE_SIZES: readonly number[] = [50, 100, 250, 500];

const DEFAULT_SESSION_PAGE_SIZE = 100;

const STORAGE_KEY = 'agenteye.settings';

/** Everything stored under {@link STORAGE_KEY}. New settings are added here as optional fields. */
interface StoredSettings {
  theme?: unknown;
  sessionPageSize?: unknown;
}

/** User settings, remembered between launches and applied as soon as they change. */
@Injectable({ providedIn: 'root' })
export class SettingsService {
  private readonly root = inject(DOCUMENT).documentElement;

  private readonly stored = this.read();
  private readonly themeState = signal<ThemePreference>(
    pickOption(THEME_PREFERENCES, this.stored.theme, 'system'),
  );
  private readonly sessionPageSizeState = signal<number>(
    pickOption(SESSION_PAGE_SIZES, this.stored.sessionPageSize, DEFAULT_SESSION_PAGE_SIZE),
  );

  readonly theme = this.themeState.asReadonly();
  readonly sessionPageSize = this.sessionPageSizeState.asReadonly();

  constructor() {
    // Apply right away (not in an effect) so the first paint already has the right theme.
    this.applyTheme(this.themeState());
  }

  setTheme(theme: ThemePreference): void {
    this.themeState.set(theme);
    this.applyTheme(theme);
    this.save({ theme });
  }

  /** Ignores a value that is not one of {@link SESSION_PAGE_SIZES}. */
  setSessionPageSize(size: number): void {
    const sessionPageSize = pickOption(SESSION_PAGE_SIZES, size, this.sessionPageSizeState());
    this.sessionPageSizeState.set(sessionPageSize);
    this.save({ sessionPageSize });
  }

  /** `system` removes the attribute so the stylesheet falls back to the OS preference. */
  private applyTheme(theme: ThemePreference): void {
    if (theme === 'system') {
      this.root.removeAttribute('data-theme');
    } else {
      this.root.setAttribute('data-theme', theme);
    }
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

/** The allowed option equal to `value`, or `fallback` when `value` is not one of them. */
function pickOption<T>(options: readonly T[], value: unknown, fallback: T): T {
  return options.find((option) => option === value) ?? fallback;
}
