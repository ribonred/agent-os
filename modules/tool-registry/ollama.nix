{ ... }:

# Install/activation side of the "ollama" tool-registry entry.
# Agent-facing metadata lives in registry/ollama.yaml.
#
# Deliberately does not set acceleration or loadModels here -- which models
# are loaded and whether GPU/NPU acceleration is used are hardware-tier
# decisions that belong in each host's own configuration, not in this
# reusable module.
{
  services.ollama = {
    enable = true;
    openFirewall = false;
  };
}
