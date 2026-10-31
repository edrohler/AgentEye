import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { FakeBackend } from '../../core/testing/fake-backend';
import { SearchBox } from './search-box';

describe('SearchBox', () => {
  it('searches as the user types and clears on Escape', async () => {
    const backend = new FakeBackend({ search: [] });
    TestBed.configureTestingModule({
      imports: [SearchBox],
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
    const fixture = TestBed.createComponent(SearchBox);
    fixture.detectChanges();
    const input = (fixture.nativeElement as HTMLElement).querySelector('input') as HTMLInputElement;

    input.value = 'needle';
    input.dispatchEvent(new Event('input'));
    await fixture.whenStable();
    expect(backend.calls).toEqual([{ command: 'search', args: { query: 'needle' } }]);

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await fixture.whenStable();
    fixture.detectChanges();
    expect(input.value).toBe('');
  });
});
