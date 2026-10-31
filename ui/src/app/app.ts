import { ChangeDetectionStrategy, Component } from '@angular/core';

import { SessionList } from './features/sessions/session-list';

@Component({
  selector: 'app-root',
  imports: [SessionList],
  templateUrl: './app.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {}
