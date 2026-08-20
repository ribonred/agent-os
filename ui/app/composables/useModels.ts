import { invoke } from "@tauri-apps/api/core";
import { agentErrorMessage } from "~/lib/agentErrors";
import { group, type ModelOptions } from "~/lib/modelGroups";

export type {
  ModelEntry,
  ProviderGroup,
  ModelOptions,
  FamilyGroup,
  ProviderView,
} from "~/lib/modelGroups";

// Which model the device thinks with, and the ones it could.
//
// The inventory is Hermes' own -- it has already narrowed the catalogue
// to models an agent can actually drive, so nothing here filters for
// capability. What this adds is the shape the owner reads it in:
// provider, then maker, then model.
//
// App-level state because two surfaces show it -- the settings screen and
// the chip beside the composer -- and they must never disagree about
// which model is current.

export type CurrentModel = { id: string; name: string };

export function useModels() {
  const options = useState<ModelOptions | null>("models:options", () => null);
  /// Read from the runtime's config alone -- one file, no gateway, no
  /// network. The label beside the composer renders from this on start;
  /// the inventory below is fetched only when the picker opens.
  const currentModel = useState<CurrentModel | null>("models:current", () => null);
  const loading = useState<boolean>("models:loading", () => false);
  const error = useState<string | null>("models:error", () => null);
  const query = useState<string>("models:query", () => "");

  async function loadCurrent() {
    try {
      currentModel.value = await invoke<CurrentModel>("model_current");
      error.value = null;
    } catch (e) {
      // A device that cannot say what it thinks with says so, rather
      // than showing a label it made up.
      currentModel.value = null;
      error.value = agentErrorMessage("chat", e);
    }
  }

  async function load(refresh = false) {
    loading.value = true;
    try {
      options.value = await invoke<ModelOptions>("model_options", { refresh });
      error.value = null;
    } catch (e) {
      error.value = agentErrorMessage("chat", e);
    } finally {
      loading.value = false;
    }
  }

  /// The model in use. The inventory's own entry once it is loaded --
  /// it carries the maker and the capability flags -- and the cheap
  /// config read before that, so nothing waits on a network to render.
  const current = computed<CurrentModel | ModelEntry | null>(() => {
    const id = options.value?.current ?? currentModel.value?.id ?? "";
    for (const provider of options.value?.providers ?? []) {
      const found = provider.models.find((m) => m.id === id);
      if (found) return found;
    }
    return currentModel.value;
  });

  const groups = computed(() => group(options.value?.providers ?? [], query.value));

  async function choose(id: string) {
    try {
      await invoke("model_set", { id });
      if (options.value) options.value.current = id;
      await loadCurrent();
      error.value = null;
      return true;
    } catch (e) {
      error.value = agentErrorMessage("chat", e);
      return false;
    }
  }

  return { options, current, groups, query, loading, error, load, loadCurrent, choose };
}
