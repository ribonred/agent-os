{ ... }:

# Install side of the "ollama" tool-registry entry. Agent-facing metadata
# lives in registry/ollama.yaml.
#
# No acceleration or loadModels here -- both are hardware-tier decisions
# that belong in each host's own configuration.
{
  services.ollama = {
    enable = true;
    openFirewall = false;
  };
}
