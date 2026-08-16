---
name: device-services
description: "Inspect and use the device's PostgreSQL and Redis services safely."
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [device, postgres, redis, storage]
---

# Device Services

Use this skill when onboarding starts, when the owner asks what storage is available, or when a task may benefit from durable relational data, caching, queues, or short-lived session state.

Treat these as internal appliance capabilities. Do not volunteer operating-system, package-manager, or service-manager details. Explain the capability and outcome in the owner's language unless they explicitly ask for technical detail.

## Live discovery

Never infer availability or a server version from configuration, client package metadata, or memory. Run checks as the current `hermes` user and trust only successful, parseable responses from the live service.

PostgreSQL:

```bash
psql -XAtqc "SELECT current_setting('server_version')"
```

Redis:

```bash
redis-cli --raw PING
redis-cli --raw INFO server
```

For Redis, require `PING` to return exactly `PONG`, then read the `redis_version:` field from `INFO server`. Do not include credentials, connection strings, socket paths, database contents, or unrelated server metadata in memory.

After a successful check, write a concise fact with the UTC check time to Hermes memory using the `memory` tool with `target: "memory"`. Update the existing fact for that service rather than adding duplicates. If a command fails, times out, or returns malformed output, say the service could not be verified and leave its remembered version unchanged.

## PostgreSQL

PostgreSQL is the device's durable relational store. Use it when the owner's task needs structured records, relationships, constraints, transactions, or durable queryable history.

Connect as the current user over the local socket. There is no password, no host, and no port to supply -- `psql` with no connection arguments is correct, and it lands in the `postgres` database:

```bash
psql -XAtqc "SELECT current_database()"
```

Use that default database for the owner's records unless a task genuinely warrants a separate one. Creating additional databases is permitted, but each one is somewhere the owner's data can hide, so prefer schemas and tables inside the default over a new database.

Before changing data or schema:

1. Inspect the target database and schema.
2. Explain consequential or destructive operations and obtain the required approval.
3. Use transactions for related writes.
4. Verify the result with a focused read.

Do not put the canonical owner profile in PostgreSQL. Hermes `USER.md` owns that profile.

## Redis

Redis is ephemeral infrastructure for caches, queues, coordination, rate limits, and short-lived session state. Use namespaced keys, set expirations for temporary data, and verify writes with the narrowest matching read.

Never use Redis as the sole store for owner profiles, learned business knowledge, records that must survive restart or eviction, secrets, or audit history. Move durable data to PostgreSQL or another purpose-built durable store.

## Failure boundary

A service failure must not become a command retry loop. Retry once only when the failure is clearly transient and a retry is harmless. Otherwise stop that operation, preserve the error, and tell the owner what could not be verified or completed.
