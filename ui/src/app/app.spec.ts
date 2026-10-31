import { provideZonelessChangeDetection, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { App } from './app';
import { SessionService } from './core/session.service';

describe('App', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [App],
      providers: [
        provideZonelessChangeDetection(),
        {
          provide: SessionService,
          useValue: {
            sessions: signal([]),
            loading: signal(false),
            error: signal(null),
            refresh: () => Promise.resolve(),
          },
        },
      ],
    }).compileComponents();
  });

  it('renders the session list', () => {
    const fixture = TestBed.createComponent(App);
    fixture.detectChanges();
    expect(fixture.nativeElement.querySelector('app-session-list')).toBeTruthy();
  });
});
