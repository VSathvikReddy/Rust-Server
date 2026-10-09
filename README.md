# RCX: Reverse Coding Exchange

A small Rust/Axum server that hosts a set of "reverse coding" questions. Each question shows an input/output example, and the player works out the hidden rule by sending their own inputs to the server and reading the outputs. A browser console (`index.html`) is included for playing against it.

Access is protected by short-lived JWTs, and every token is rate limited.

## How it works

```
 browser (index.html) ──POST /auth──────────────▶ issues a JWT (1 hour)
          │
          ├─GET  /load        (Bearer token)───▶ list of question prompts
          └─POST /ask/{n}     (Bearer token)───▶ output of hidden function n
```

Layers, outermost first:

1. **Trace** logs every request (`tower_http`).
2. **CORS** allows any origin, method and header, so the console can be opened from a file or any host.
3. **JWT check** rejects requests without a valid `Authorization: Bearer <token>` header and stores the claims on the request.
4. **Rate limiter** (`tower_governor`) buckets requests by the user id inside the token.

`/auth` is merged in after the protected layers, so it is public.

## Requirements

- Rust 1.85 or newer (the crate uses edition 2024)
- Python 3.8+ for the stress tester (standard library only)

Main dependencies (from `Cargo.toml`):

| Crate                | Version  | Used for                                   |
|----------------------|----------|--------------------------------------------|
| `axum`               | 0.8.9    | HTTP routing (`/{param}` path syntax)      |
| `tokio`              | 1.53.1   | Async runtime, signal handling             |
| `tower-http`         | 0.7.1    | CORS and request tracing                   |
| `tower_governor`     | 0.8.0    | Rate limiting                              |
| `jsonwebtoken`       | 11.1.0   | JWT signing and verification (`rust_crypto` backend) |
| `rand`               | 0.10.3   | Random user ids for tokens                 |
| `serde`, `serde_json`| 1.0.x    | JSON handling                              |
| `dotenvy`            | 0.15.7   | `.env` loading                             |
| `tracing`, `tracing-subscriber` | 0.1.44, 0.3.23 | Logging with `RUST_LOG` filtering |

## Configuration

All settings come from environment variables. A `.env` file in the working directory is loaded automatically.

| Variable      | Required | Description                                      |
|---------------|----------|--------------------------------------------------|
| `SERVER_HOST` | yes      | Interface to bind, e.g. `0.0.0.0` or `127.0.0.1` |
| `SERVER_PORT` | yes      | Port to listen on, e.g. `3000`                   |
| `JWT_SECRET`  | yes      | HMAC secret used to sign and verify tokens       |
| `RUST_LOG`    | no       | Log filter. Default: `rcx=info,tower_http=info`  |

Example `.env`:

```env
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
JWT_SECRET=change-me-to-a-long-random-string
```

The server panics at startup with a clear message if a required variable is missing or invalid.

## Running

```bash
cargo run --release
```

Then open `index.html` in a browser, enter the host and port (the port defaults to `3000` in the console), and press **Auth**. The console fetches a token, loads the questions, and lets you run inputs against each one.

## API

All routes except `/auth` need the header `Authorization: Bearer <token>`.

| Method | Path           | Body                   | Response                                    |
|--------|----------------|------------------------|---------------------------------------------|
| POST   | `/auth`        | none                   | The JWT as a plain-text body (not JSON)     |
| GET    | `/load`        | none                   | JSON array of question prompts              |
| POST   | `/ask/{index}` | a JSON string, e.g. `"10 20"` | `{"question": n, "output": "..."}`   |

Errors:

- `401` for a missing, malformed, invalid or expired token.
- `404` with `{"error":"question_not_found", ...}` for an index outside the question list.
- `429` when the rate limit is hit.

The body of `/ask` is a JSON **string**, so the quotes are part of the payload. From a browser use `JSON.stringify(input)`.

### curl example

```bash
TOKEN=$(curl -s -X POST http://localhost:3000/auth)

curl -s http://localhost:3000/load -H "Authorization: Bearer $TOKEN"

curl -s -X POST http://localhost:3000/ask/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '"10 20 30 40"'
```

## Questions

Indices are 0-based on the API, and the console displays them 1-based.

| Index | Rule                                      |
|-------|-------------------------------------------|
| 0     | Reverse the string                        |
| 1     | Sum of all integers in the input          |
| 2     | Largest integer in the input              |
| 3     | Number of set bits in an integer          |
| 4     | Fibonacci number at index `n`             |

Non-numeric tokens are skipped for questions 1 and 2. Questions 3 and 4 return `invalid integer` if the input does not parse as a `u64`.

## Authentication and rate limiting

- `/auth` mints an HS256 JWT with claims `{ id, exp }`. `id` is a random `u64`, and `exp` is one hour from issue.
- The limiter keys on that `id` with a burst of 5 requests and a replenish interval set by `per_second(2)`.
- A background thread logs the limiter's storage size and drops stale entries every 60 seconds.
- `add_rate_limit(jwt_secret, distribute_keys)` also controls whether the public `/auth` route is mounted. Pass `false` to run without token issuing.

## Project layout

```
src/
  main.rs     entry point: loads .env, tracing, config, builds and runs the server
  lib.rs      module declarations (config, server, auth)
  config.rs   ServerConfig, reads environment variables
  server.rs   routes, question logic, rate limiting, CORS, graceful shutdown
  auth.rs     JWT keys, token verification middleware, /auth handler, rate-limit key extractor
index.html    standalone browser console
stress_test.py  load and abuse tester (see below)
```

The server shuts down gracefully on Ctrl+C, and on SIGTERM on Unix.

## Stress tester

`stress_test.py` uses only the Python standard library. It reports requests per second, status code counts, error counts and latency percentiles (p50, p95, p99).

```bash
python3 stress_test.py --host 127.0.0.1 --port 3000 --mode sustained --workers 20 --duration 15
```

| Mode         | What it does                                                                 |
|--------------|------------------------------------------------------------------------------|
| `sustained`  | Each worker gets its own token and hammers `/load` and `/ask`                |
| `shared`     | All workers share one token, so they compete for a single rate-limit bucket  |
| `reauth`     | Each worker calls `/auth` before every request (checks the limiter can be dodged) |
| `auth-flood` | Only hits `/auth`                                                            |
| `unauth`     | Sends requests with no token and verifies none succeed                       |

Useful flags: `--workers`, `--duration` (seconds), `--delay` (seconds between requests per worker), `--bad-rate` (fraction of requests aimed at an out-of-range question index), `--seed`.

What to expect:

- `sustained` and `shared` should show mostly `429` once the burst is spent, which means the limiter works.
- `reauth` showing mostly `200` means the per-token limiter is being bypassed by minting new tokens.
- `unauth` should report only `401`. Anything else is flagged as a failure.

## Known limitations

These are known and planned for a later release:

- The rate limit is keyed on a token id that anyone can re-roll through `/auth`.
- `per_second(2)` is a replenish interval of 2 seconds per request, not 2 requests per second.
- Question 4 (Fibonacci) has no cap on `n`, so very large values are slow and large results overflow.
- Question 1 sums into an `i64`, which can overflow on huge inputs.
- No request timeout or concurrency limit is configured.
- `JWT_SECRET` strength is not validated.