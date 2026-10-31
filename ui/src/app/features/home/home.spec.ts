import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SearchService } from '../../core/search.service';
import { FakeBackend } from '../../core/testing/fake-backend';
import { Home } from './home';

describe('Home', () => {
  beforeEach(() => {
    const backend = new FakeBackend({ list_sessions: [], search: [] });
    TestBed.configureTestingModule({
      imports: [Home],
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
  });

  it('shows the session list until the user searches', async () => {
    const fixture = TestBed.createComponent(Home);
    const element = fixture.nativeElement as HTMLElement;
    fixture.detectChanges();

    expect(element.querySelector('app-search-box')).toBeTruthy();
    expect(element.querySelector('app-session-list')).toBeTruthy();
    expect(element.querySelector('app-search-results')).toBeNull();

    await TestBed.inject(SearchService).search('needle');
    fixture.detectChanges();

    expect(element.querySelector('app-search-results')).toBeTruthy();
    expect(element.querySelector('app-session-list')).toBeNull();
  });
});
