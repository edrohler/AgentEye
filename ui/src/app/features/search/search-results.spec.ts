import { provideZonelessChangeDetection } from '@angular/core';
import { TestBed } from '@angular/core/testing';

import { SearchHit } from '../../core/models';
import { SearchService } from '../../core/search.service';
import { HIGHLIGHT_END, HIGHLIGHT_START } from '../../core/snippet';
import { FakeBackend } from '../../core/testing/fake-backend';
import { SearchResults } from './search-results';

function hit(path: string, snippet: string, byteOffset: number): SearchHit {
  return { agent: 'codex', path, role: 'user', timestamp: null, byteOffset, snippet };
}

describe('SearchResults', () => {
  async function render(hits: SearchHit[]) {
    const backend = new FakeBackend({ search: hits });
    TestBed.configureTestingModule({
      imports: [SearchResults],
      providers: [provideZonelessChangeDetection(), backend.provide()],
    });
    await TestBed.inject(SearchService).search('word');
    const fixture = TestBed.createComponent(SearchResults);
    fixture.detectChanges();
    return fixture.nativeElement as HTMLElement;
  }

  it('groups hits by session and highlights matches without treating text as HTML', async () => {
    const element = await render([
      hit('C:\\logs\\one.jsonl', `see ${HIGHLIGHT_START}word${HIGHLIGHT_END} <b>here</b>`, 1),
      hit('C:\\logs\\one.jsonl', 'again', 2),
      hit('C:\\logs\\two.jsonl', 'other', 3),
    ]);

    expect(element.querySelectorAll('[data-testid=group]').length).toBe(2);
    expect(element.querySelector('[data-testid=file]')?.textContent).toContain('one.jsonl');
    expect(element.querySelectorAll('[data-testid=group]:first-child [data-testid=hit]').length).toBe(2);
    expect(element.querySelector('mark')?.textContent).toBe('word');
    expect(element.querySelector('[data-testid=snippet] b')).toBeNull();
    expect(element.textContent).toContain('3 matches in 2 sessions');
  });

  it('says so when nothing matches', async () => {
    const element = await render([]);

    expect(element.textContent).toContain('No matches.');
  });
});
