import { FormEvent, useEffect, useState } from "react";
import { Download, Languages, Save, Zap } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { APP_LANGUAGE_OPTIONS, translate, type AppLanguage } from "../lib/i18n";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/useAppStore";
import type { ApiProfile } from "../lib/types";

const DEFAULT_PROFILE: ApiProfile = {
  id: "",
  name: "DeepSeek",
  base_url: "https://api.deepseek.com/beta",
  model: "deepseek-v4-flash",
  api_key: "",
  use_strict_tools: true
};

export function SettingsPanel() {
  const {
    apiProfile,
    appLanguage,
    quickMode,
    setApiProfile,
    setAppLanguage,
    setError,
    setQuickMode,
    setWorlds
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const [saved, setSaved] = useState(false);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState("");

  useEffect(() => {
    setSaved(false);
  }, [apiProfile]);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      const profile = await api.saveApiProfile({
        id: apiProfile?.id ?? "",
        name: String(form.get("name") || "DeepSeek"),
        base_url: String(form.get("base_url") || DEFAULT_PROFILE.base_url),
        model: String(form.get("model") || DEFAULT_PROFILE.model),
        api_key: String(form.get("api_key") || ""),
        use_strict_tools: true
      });
      setApiProfile(profile);
      setSaved(true);
      setStatus("");
    } catch (err) {
      setError(String(err));
    }
  }

  async function importWorld() {
    setImporting(true);
    setStatus("");
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: t("importWorldZip"),
        filters: [{ name: t("worldZip"), extensions: ["zip"] }]
      });
      if (!selected || Array.isArray(selected)) {
        return;
      }
      const bytes = await readFile(selected);
      await api.importWorld(bytes);
      setWorlds(await api.listWorlds());
      setStatus(t("worldImported"));
    } catch (err) {
      setError(String(err));
    } finally {
      setImporting(false);
    }
  }

  const profile = apiProfile ?? DEFAULT_PROFILE;

  return (
    <form className="settings-form" onSubmit={save}>
      <label>
        {t("name")}
        <input name="name" defaultValue={profile.name} />
      </label>
      <label>
        {t("baseUrl")}
        <input name="base_url" defaultValue={profile.base_url} />
      </label>
      <label>
        {t("model")}
        <input name="model" defaultValue={profile.model} />
      </label>
      <label>
        {t("apiKey")}
        <input name="api_key" type="password" defaultValue={profile.api_key} />
      </label>
      <button className="primary-button">
        <Save size={16} />
        {saved ? t("saved") : t("saveApiProfile")}
      </button>
      <label>
        {t("appLanguage")}
        <span className="select-shell">
          <Languages size={16} />
          <select
            value={appLanguage}
            onChange={(event) => setAppLanguage(event.target.value as AppLanguage)}
          >
            {APP_LANGUAGE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </span>
      </label>
      <button
        className={quickMode ? "quick-mode-toggle active" : "quick-mode-toggle"}
        type="button"
        onClick={() => setQuickMode(!quickMode)}
        aria-pressed={quickMode}
      >
        <Zap size={16} />
        {t("quickMode")}
      </button>
      <button className="command-button" type="button" onClick={() => void importWorld()} disabled={importing}>
        <Download size={16} />
        {importing ? t("importing") : t("importWorld")}
      </button>
      {status ? <p className="settings-status">{status}</p> : null}
    </form>
  );
}
