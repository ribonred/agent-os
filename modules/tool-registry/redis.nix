{ ... }:

# Install/activation side of the "redis" tool-registry entry.
# Agent-facing metadata lives in registry/redis.yaml.
{
  services.redis.servers."" = {
    enable = true;
    port = 6379;
    bind = "127.0.0.1";
  };
}
