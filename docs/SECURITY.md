# Security

Raster is a game engine. It runs game code, loads assets, and draws to a
screen; it does not handle credentials, network traffic or user accounts. The
realistic risk is a malformed asset — a crafted scene file, sprite or tilemap —
crashing a game or worse.

## Reporting

Report a vulnerability privately through
[GitHub's advisory form](https://github.com/Rasterie/Raster/security/advisories/new),
not as a public issue.

Include what you did, what happened, and the version or commit. A reproducer
helps more than anything else.

This is a solo project: expect a reply within a week, and a fix when one is
possible. There is no bounty programme.

## Scope

**In scope** — anything where loading an asset or a scene file lets an attacker
crash the engine, corrupt memory, read files outside the project directory, or
execute code.

**Out of scope** — a game deliberately written to do something harmful. The
engine runs the code it is given, the same way a compiler compiles what it is
handed.

## Supported versions

Only the latest commit on `main`. There are no releases yet, and no backports
until there are.
