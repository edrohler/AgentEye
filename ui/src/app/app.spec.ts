import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { App } from './app';
import { SearchService } from './core/search.service';
import { FakeBackend } from './core/testing/fake-backend';

describe('App', () => {
  let backend: FakeBackend;

  beforeEach(async () => {
    backend = new FakeBackend({
      list_sessions: [],
      index_status: { state: 'idle' },
      search: [],
    });
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [provideZonelessChangeDetection(), backend.provide()],
    }).compileComponents();
  });

  it('shows the session list until the user searches', async () => {
    const fixture = TestBed.createComponent(App);
    const element = fixture.nativeElement as HTMLElement;
    fixture.detectChanges();

    expect(element.querySelector('app-session-list')).toBeTruthy();
    expect(element.querySelector('app-search-results')).toBeNull();

    await TestBed.inject(SearchService).search('needle');
    fixture.detectChanges();

    expect(element.querySelector('app-search-results')).toBeTruthy();
    expect(element.querySelector('app-session-list')).toBeNull();
  });
});
