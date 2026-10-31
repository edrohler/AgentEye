import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { SearchService } from '../../core/search.service';
import { SearchBox } from '../search/search-box';
import { SearchResults } from '../search/search-results';
import { SessionList } from '../sessions/session-list';

/** Main page: a search box above either the search results or, with no query, all sessions. */
@Component({
  selector: 'app-home',
  imports: [SearchBox, SearchResults, SessionList],
  templateUrl: './home.html',
  host: { class: 'block' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Home {
  protected readonly hasQuery = inject(SearchService).hasQuery;
}
