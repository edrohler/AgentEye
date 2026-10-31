import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { Router, provideRouter } from '@angular/router';
import { RouterTestingHarness } from '@angular/router/testing';

import { App } from './app';
import { routes } from './app.routes';
import { FakeBackend } from './core/testing/fake-backend';

describe('App', () => {
  beforeEach(async () => {
    const backend = new FakeBackend({
      list_sessions: [],
      index_status: { state: 'idle' },
      search: [],
    });
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideZonelessChangeDetection(), provideRouter(routes), backend.provide()],
    }).compileComponents();
  });

  it('shows the header with the index status and a menu button that opens settings', () => {
    const fixture = TestBed.createComponent(App);
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;

    expect(element.querySelector('h1')?.textContent).toContain('AgentEye');
    expect(element.querySelector('app-index-status')).toBeTruthy();
    const button = element.querySelector('a[aria-label="Open settings"]');
    expect(button?.getAttribute('href')).toMatch(/\/settings$/);

    // The icon must be real SVG geometry, not unknown HTML elements the browser cannot draw.
    const path = button?.querySelector('svg path');
    expect(path?.namespaceURI).toBe('http://www.w3.org/2000/svg');
    expect(path?.getAttribute('d')).toBeTruthy();
  });

  it('turns the menu button into a close button on settings that goes back home', async () => {
    const fixture = TestBed.createComponent(App);
    const element = fixture.nativeElement as HTMLElement;
    fixture.detectChanges();

    await TestBed.inject(Router).navigateByUrl('/settings');
    await fixture.whenStable();
    fixture.detectChanges();

    expect(element.querySelector('a[aria-label="Open settings"]')).toBeNull();
    const close = element.querySelector('a[aria-label="Close settings"]');
    expect(close?.getAttribute('href')).toBe('/');
    expect(element.querySelector('app-settings')).toBeTruthy();

    (close as HTMLAnchorElement).click();
    await fixture.whenStable();
    fixture.detectChanges();

    expect(element.querySelector('app-settings')).toBeNull();
    expect(element.querySelector('a[aria-label="Open settings"]')).toBeTruthy();
  });

  it('routes between the home page and settings', async () => {
    const harness = await RouterTestingHarness.create();

    await harness.navigateByUrl('/');
    expect(harness.routeNativeElement?.tagName.toLowerCase()).toBe('app-home');

    await harness.navigateByUrl('/settings');
    expect(harness.routeNativeElement?.tagName.toLowerCase()).toBe('app-settings');

    await harness.navigateByUrl('/nowhere');
    expect(harness.routeNativeElement?.tagName.toLowerCase()).toBe('app-home');
  });
});
