# API endpoints

Type definitions are here: [pacman-core/src/contract.rs](https://github.com/jDomantas/pacman/blob/master/pacman-core/src/contract.rs).

All names are converted to `camelCase`.

- `POST /api/submit` - accepts `Submit`, returns `SubmitResponse`
- `POST /api/authenticate` - accepts `Authenticate`, returns 200 on success and 401 on failure
- `GET /api/submissions` - returns `Submissions`
- `GET /api/submission/{id}` - returns `SubmissionDetails`
- `GET /api/scoreboard` - returns `Scoreboards`
- `POST /api/admin/level` - accepts `SetLevel`
- `POST /api/admin/levelstate` - accepts `SetLevelState`
- `POST /api/admin/ratelimit` - accepts `RateLimit` (sets a custom rate limit for a single user)
- `POST /api/admin/reset` - accepts `Reset` (resets the whole game to a fresh state).

Currently user sumbissions are rate limited to at most 2 submissions in the last 10 seconds.
