import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { IndexStatusService } from '../../core/index-status.service';

@Component({
  selector: 'app-index-status',
  templateUrl: './index-status.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class IndexStatusIndicator {
  private readonly service = inject(IndexStatusService);

  protected readonly status = this.service.status;
  protected readonly percent = this.service.percent;
  protected readonly isRunning = this.service.isRunning;
  protected readonly failureMessage = this.service.failureMessage;

  protected reindex(): void {
    void this.service.reindex();
  }
}
