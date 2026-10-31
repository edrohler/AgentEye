import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { SearchService } from './core/search.service';
import { IndexStatusIndicator } from './features/index-status/index-status';
import { SearchBox } from './features/search/search-box';
import { SearchResults } from './features/search/search-results';
import { SessionList } from './features/sessions/session-list';

@Component({
  selector: 'app-root',
  imports: [IndexStatusIndicator, SearchBox, SearchResults, SessionList],
  templateUrl: './app.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
  protected readonly hasQuery = inject(SearchService).hasQuery;
}
