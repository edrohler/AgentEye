import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { basename } from '../../core/paths';
import { SearchService } from '../../core/search.service';
import { splitSnippet } from '../../core/snippet';

@Component({
  selector: 'app-search-results',
  templateUrl: './search-results.html',
  host: { class: 'block px-6 py-4' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SearchResults {
  private readonly search = inject(SearchService);

  protected readonly groups = this.search.groups;
  protected readonly hitCount = this.search.hitCount;
  protected readonly loading = this.search.loading;
  protected readonly error = this.search.error;

  protected readonly basename = basename;
  protected readonly splitSnippet = splitSnippet;
}
