import {
  ApplicationConfig,
  inject,
  provideAppInitializer,
  provideBrowserGlobalErrorListeners,
  provideZonelessChangeDetection,
} from '@angular/core';
import { provideRouter, withHashLocation } from '@angular/router';

import { routes } from './app.routes';
import { SettingsService } from './core/settings.service';

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZonelessChangeDetection(),
    // Hash URLs (#/settings) work when the app is served from files, with no server-side fallback.
    provideRouter(routes, withHashLocation()),
    // Create the settings service at startup so the saved theme is applied before the first paint.
    provideAppInitializer(() => {
      inject(SettingsService);
    }),
  ],
};
