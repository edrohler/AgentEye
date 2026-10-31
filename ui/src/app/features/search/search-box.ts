import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { SearchService } from '../../core/search.service';

@Component({
  selector: 'app-search-box',
  templateUrl: './search-box.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SearchBox {
  private readonly search = inject(SearchService);

  protected readonly query = this.search.query;

  protected onInput(event: Event): void {
    void this.search.search((event.target as HTMLInputElement).value);
  }

  protected clear(): void {
    void this.search.search('');
  }
}
