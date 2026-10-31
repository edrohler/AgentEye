import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SettingsService } from '../../core/settings.service';
import { Settings } from './settings';

describe('Settings', () => {
  beforeEach(() => {
    localStorage.removeItem('agenteye.settings');
    TestBed.configureTestingModule({
      imports: [Settings],
      providers: [provideZonelessChangeDetection()],
    });
  });

  afterEach(() => {
    localStorage.removeItem('agenteye.settings');
    document.documentElement.removeAttribute('data-theme');
  });

  function radios(element: HTMLElement): HTMLInputElement[] {
    return [...element.querySelectorAll<HTMLInputElement>('input[type=radio]')];
  }

  it('offers system, light and dark with the current choice selected', () => {
    const fixture = TestBed.createComponent(Settings);
    fixture.detectChanges();

    const options = radios(fixture.nativeElement);
    expect(options.map((o) => o.value)).toEqual(['system', 'light', 'dark']);
    expect(options.map((o) => o.checked)).toEqual([true, false, false]);
  });

  it('changes the theme when an option is chosen', async () => {
    const fixture = TestBed.createComponent(Settings);
    fixture.detectChanges();

    radios(fixture.nativeElement)[2].click();
    await fixture.whenStable();
    fixture.detectChanges();

    expect(TestBed.inject(SettingsService).theme()).toBe('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    expect(radios(fixture.nativeElement).map((o) => o.checked)).toEqual([false, false, true]);
  });
});
