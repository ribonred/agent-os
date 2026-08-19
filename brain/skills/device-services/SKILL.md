---
name: device-services
description: "Inspect and use the device's Docker, PostgreSQL, Redis, and Python/uv runtime safely."
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [device, docker, postgres, redis, python, uv, storage]
---

# Device Services

Use this skill when onboarding starts, when the owner asks what storage or local application runtime is available, or when a task may benefit from containers, Python tools, durable relational data, caching, queues, or short-lived session state.

Treat these as internal appliance capabilities. Do not volunteer operating-system, package-manager, or service-manager details. Explain the capability and outcome in the owner's language unless they explicitly ask for technical detail.

## Live discovery

Never infer availability or a server version from configuration, client package metadata, or memory. Run the checks below and trust only successful, parseable responses from the live service.

PostgreSQL:

```bash
sudo -u postgres psql -XAtqc "SELECT current_setting('server_version')"
```

Redis:

```bash
redis-cli --raw PING
redis-cli --raw INFO server
```

Docker:

```bash
docker info --format '{{.ServerVersion}}'
```

uv / Python:

```bash
uv --version
```

For Redis, require `PING` to return exactly `PONG`, then read the `redis_version:` field from `INFO server`. For Docker, require the command to return a non-empty server version and complete without `sudo`; this confirms both that the engine is running and that the agent can use it as the device owner. For `uv`, confirm the CLI returns a valid version string without `sudo`. Do not include credentials, connection strings, socket paths, database contents, or unrelated server metadata in memory.

After a successful check, write a concise fact with the UTC check time to Hermes memory using the `memory` tool with `target: "memory"`. Update the existing fact for that service rather than adding duplicates. If a command fails, times out, or returns malformed output, say the service could not be verified and leave its remembered version unchanged.

## Docker

Docker is the device's local application runtime. Use it when the owner's task needs an isolated application, a repeatable tool environment, or a service with a published container image.

Run Docker commands as the device owner without `sudo`. The owner account is granted Docker access during device setup, and a new login or reboot may be required before that access appears in the current session. Do not work around a failed permission check by silently adding `sudo`; report the access problem and verify the service first.

Before starting, stopping, removing, or replacing containers, inspect the target container and explain consequential changes. Prefer named volumes for data that must survive container replacement, and do not put owner profiles, learned business knowledge, records, secrets, or audit history only inside an untracked container filesystem.

## Python & uv

`uv` is the preferred tool for Python scripts, temporary CLI tools, and isolated environments on this device.

When a task requires running Python code or Python-based CLI utilities:

- Prefer `uv run` for executing ad-hoc scripts or one-off commands with inline or project dependencies (e.g., `uv run --with <pkg> <script>`).
- Prefer `uv tool run` or `uv tool install` when running standalone CLI tools in isolated environments rather than modifying system packages or global Python installations.
- Run `uv` commands directly as the device owner without `sudo` and without mutating system Python libraries.
- Never use `sudo pip install` or pollute the global system interpreter.

## PostgreSQL

PostgreSQL is the device's durable relational store. Use it when the owner's task needs structured records, relationships, constraints, transactions, or durable queryable history.

Connect as the `postgres` role, over the local socket:

```bash
sudo -u postgres psql -XAtqc "SELECT current_database()"
```

Always `sudo -u postgres`, never a bare `psql`. Two reasons, and both matter.

The server authenticates local connections by peer: the role must be named after the system account making the connection, and the account this agent runs under has no role of its own. A bare `psql` therefore does not fall back to something lesser -- it fails outright with "role does not exist", which is a confusing thing to hit halfway through a task and impossible to explain to the owner.

`postgres` is also the only role that can govern the others: create and drop roles, grant and revoke, and reach every database on the cluster. Administering this device includes administering its database, and a role that can read its own tables but cannot add a second one is not administration. This grants nothing that could not already be taken -- the agent has passwordless root -- and it avoids walling the agent off behind a permission error it cannot resolve.

There is no password, no host, and no port to supply: the server listens on a unix socket only and nothing on this device needs it over the network.

Connecting this way lands in the `postgres` database. Use it for the owner's records unless a task genuinely warrants a separate one. Creating additional databases is permitted, but each one is somewhere the owner's data can hide, so prefer schemas and tables inside the default over a new database.

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
