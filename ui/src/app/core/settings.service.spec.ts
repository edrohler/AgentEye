import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SettingsService } from './settings.service';

const STORAGE_KEY = 'agenteye.settings';

function createService(): SettingsService {
  TestBed.configureTestingModule({ providers: [provideZonelessChangeDetection()] });
  return TestBed.inject(SettingsService);
}

describe('SettingsService', () => {
  const root = document.documentElement;

  beforeEach(() => {
    localStorage.removeItem(STORAGE_KEY);
    root.removeAttribute('data-theme');
  });

  afterEach(() => {
    localStorage.removeItem(STORAGE_KEY);
    root.removeAttribute('data-theme');
  });

  it('defaults to following the system and sets no attribute', () => {
    const service = createService();

    expect(service.theme()).toBe('system');
    expect(root.hasAttribute('data-theme')).toBeFalse();
  });

  it('applies and remembers an explicit theme', () => {
    const service = createService();

    service.setTheme('dark');

    expect(service.theme()).toBe('dark');
    expect(root.getAttribute('data-theme')).toBe('dark');
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}')).toEqual({ theme: 'dark' });
  });

  it('removes the attribute when switching back to system', () => {
    const service = createService();
    service.setTheme('light');

    service.setTheme('system');

    expect(root.hasAttribute('data-theme')).toBeFalse();
  });

  it('restores the saved theme on startup', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: 'light' }));

    const service = createService();

    expect(service.theme()).toBe('light');
    expect(root.getAttribute('data-theme')).toBe('light');
  });

  it('ignores invalid stored values', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: 'purple' }));
    expect(createService().theme()).toBe('system');

    TestBed.resetTestingModule();
    localStorage.setItem(STORAGE_KEY, 'not json');
    expect(createService().theme()).toBe('system');
  });

  it('keeps other stored settings when saving the theme', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ somethingElse: 1 }));
    const service = createService();

    service.setTheme('dark');

    expect(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}')).toEqual({
      somethingElse: 1,
      theme: 'dark',
    });
  });
});
