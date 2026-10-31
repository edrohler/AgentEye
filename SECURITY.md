# Security policy

## Reporting a vulnerability

Please report security problems **privately**, not in a public issue or pull request.

Use GitHub's private reporting: open the repository's **Security** tab and choose **Report a vulnerability**. Include what you found, how to reproduce it, and the AgentEye version (shown in the installer or on the release page). Please do not attach real session logs, since they can contain secrets; a minimal made-up example is enough.

AgentEye is maintained by one person, so responses are best effort. You can expect an acknowledgement within about a week, and a fix or an explanation after that, depending on severity. There is no bug bounty. Reporters are credited in the release notes unless they prefer not to be.

## Supported versions

Only the latest release receives fixes. AgentEye is pre-1.0, so older `0.x` releases are not patched.

## What AgentEye handles

AgentEye reads the session logs that AI coding agents (currently Claude Code and Codex CLI) write on your machine. Those logs can contain source code, file paths, prompts and secrets. To limit exposure:

- It is **read-only** with respect to agent files: it never writes to, locks or deletes them.
- It is **local only**: no network calls and no telemetry.
- Its search index (`index.db` in the app's data folder) stores the **text of messages** so search can be fast. It is **not encrypted**, so protect it like the logs themselves. It is a cache, so deleting it loses nothing, and entries for sessions you delete are removed the next time the index syncs.

## In scope

- Reading or writing files outside the agent session folders, for example through a crafted session file or a path passed from the UI to the backend.
- Code execution, or script injection into the app window, through the content of a session log.
- Leaking session content off the machine.
- Weaknesses in the release pipeline or published installers, such as an installer that does not match its checksum.

## Out of scope

- Anyone who already has access to your user account or disk can read your logs and the index. AgentEye does not defend against that.
- Windows SmartScreen warnings: the installers are not code-signed yet.
- Findings in third-party dependencies with no demonstrated impact on AgentEye. Report those upstream.
