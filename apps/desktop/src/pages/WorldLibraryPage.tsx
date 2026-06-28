import { ChevronDown, Plus, Sparkles, Trash2, Upload } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { ChangeEvent, FormEvent, useEffect, useRef, useState } from "react";
import { translate } from "../lib/i18n";
import { SUPPORTED_STORY_LANGUAGES, isTranslationLanguage } from "../lib/languages";
import { api } from "../lib/tauri";
import type { CreateWorldRequest } from "../lib/types";
import { useAppStore } from "../stores/useAppStore";

export interface DropdownProps {
  value: string;
  options: string[];
  onChange: (value: string) => void;
  placeholder: string;
  disabled?: boolean;
  allowFreeText?: boolean;
}

export function Dropdown({ value, options, onChange, placeholder, disabled }: DropdownProps) {
  const [open, setOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    function handlePointerDown(event: MouseEvent) {
      if (wrapperRef.current && !wrapperRef.current.contains(event.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handlePointerDown);
    return () => document.removeEventListener("mousedown", handlePointerDown);
  }, [open]);

  return (
    <div className="dropdown" ref={wrapperRef}>
      <button
        type="button"
        className="dropdown-trigger"
        disabled={disabled}
        onClick={() => setOpen((state) => !state)}
      >
        <span className={value ? "dropdown-value" : "dropdown-value placeholder"}>
          {value || placeholder}
        </span>
        <ChevronDown size={16} className={open ? "dropdown-caret open" : "dropdown-caret"} />
      </button>
      {open ? (
        <ul className="dropdown-menu">
          {options.map((option) => (
            <li key={option}>
              <button
                type="button"
                className={option === value ? "dropdown-option active" : "dropdown-option"}
                onClick={() => {
                  onChange(option);
                  setOpen(false);
                }}
              >
                {option}
              </button>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

const WORLD_GENRES = [
  "玄幻",
  "奇幻",
  "武侠",
  "仙侠",
  "都市",
  "现实",
  "军事",
  "历史",
  "游戏",
  "体育",
  "科幻",
  "灵异",
  "现代言情",
  "古代言情",
  "豪门总裁",
  "青春校园",
  "穿越重生",
  "宫斗宅斗",
  "种田经商",
  "女强爽文",
  "甜宠",
  "虐恋",
  "悬疑推理",
  "年代文",
  "娱乐圈",
  "快穿",
  "星际",
  "末世",
  "自定义"
];

const TARGET_LANGUAGES = SUPPORTED_STORY_LANGUAGES;

const DEFAULT_WORLD_FORM: CreateWorldRequest = {
  title: "",
  description: "",
  genre: "",
  target_language: "",
  language_level: "",
  narrative_style: "",
  characters: []
};

export function WorldLibraryPage() {
  const {
    worlds,
    activeWorld,
    appLanguage,
    libraryError,
    setWorlds,
    setActiveWorld,
    clearActiveWorld,
    setLibraryError
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const [openForm, setOpenForm] = useState(false);
  const [creating, setCreating] = useState(false);
  const [drafting, setDrafting] = useState(false);
  const [showGenrePicker, setShowGenrePicker] = useState(false);
  const [genreInput, setGenreInput] = useState("");
  const [customGenre, setCustomGenre] = useState("");
  const [selectedLanguage, setSelectedLanguage] = useState(TARGET_LANGUAGES[0]);
  const [formValues, setFormValues] = useState<CreateWorldRequest>(DEFAULT_WORLD_FORM);

  async function openWorld(worldId: string) {
    try {
      const bootstrap = await api.getWorldBootstrap(worldId);
      setActiveWorld(
        bootstrap.world,
        bootstrap.scene_id,
        bootstrap.turns.map((turn) => ({
          id: turn.turn_id,
          userInput: turn.user_input,
          output: turn.output
        }))
      );
    } catch (err) {
      setLibraryError(String(err));
    }
  }

  async function createWorld(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setCreating(true);
    try {
      await api.createWorld({
        title: formValues.title || "Untitled World",
        description: formValues.description,
        genre: formValues.genre || "mystery",
        target_language: formValues.target_language || "English",
        language_level: formValues.language_level || "B1",
        narrative_style: formValues.narrative_style || "immersive literary prose",
        characters: formValues.characters
      });
      const next = await api.listWorlds();
      setWorlds(next);
      setOpenForm(false);
    } catch (err) {
      setLibraryError(String(err));
    } finally {
      setCreating(false);
    }
  }

  function updateField(
    field: keyof CreateWorldRequest,
    event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) {
    setFormValues((values) => ({ ...values, [field]: event.target.value }));
  }

  async function generateDraft() {
    setOpenForm(true);
    setDrafting(true);
    try {
      const genre = genreInput === "自定义" ? customGenre.trim() : genreInput.trim();
      if (!genre) {
        return;
      }
      const draft = await api.generateWorldDraft({
        genre,
        target_language: selectedLanguage
      });
      setFormValues({
        ...draft,
        language_level: normalDifficultyLabel(selectedLanguage)
      });
      setShowGenrePicker(false);
      setCustomGenre("");
    } catch (err) {
      setLibraryError(String(err));
    } finally {
      setDrafting(false);
    }
  }

  async function deleteWorld(worldId: string, title: string) {
    if (!window.confirm(t("deleteWorldConfirm", title))) {
      return;
    }
    try {
      await api.deleteWorld(worldId);
      if (activeWorld?.id === worldId) {
        clearActiveWorld();
      }
      setWorlds(await api.listWorlds());
    } catch (err) {
      setLibraryError(String(err));
    }
  }

  async function exportWorld(worldId: string, title: string) {
    try {
      const fileName = `${safeZipName(title)}.zip`;
      const selected = await save({
        title: t("exportWorldTitle"),
        defaultPath: fileName,
        filters: [{ name: t("worldZip"), extensions: ["zip"] }]
      });
      if (!selected) {
        return;
      }
      const bytes = await api.exportWorld(worldId);
      await writeFile(selected, new Uint8Array(bytes));
    } catch (err) {
      setLibraryError(String(err));
    }
  }

  return (
    <div className="world-panel">
      <button className={openForm ? "command-button active" : "command-button"} type="button" onClick={() => setOpenForm((value) => !value)}>
        <Plus size={16} />
        {t("newWorld")}
      </button>

      <button className={showGenrePicker ? "command-button active" : "command-button"} type="button" onClick={() => setShowGenrePicker((value) => !value)}>
        <Sparkles size={16} />
        {t("aiFill")}
      </button>

      {libraryError ? (
        <div className="error-box inline-error" role="alert">
          <button onClick={() => setLibraryError(undefined)}>{t("dismiss")}</button>
          <p>{libraryError}</p>
        </div>
      ) : null}

      {showGenrePicker ? (
        <div className="ai-fill-panel">
          <div className="ai-fill-controls">
            <Dropdown
              value={genreInput}
              options={WORLD_GENRES}
              onChange={(value) => {
                setGenreInput(value);
                if (value !== "自定义") {
                  setCustomGenre("");
                }
              }}
              placeholder={t("genre")}
              disabled={drafting}
            />
            <Dropdown
              value={selectedLanguage}
              options={TARGET_LANGUAGES}
              onChange={(language) => {
                if (isTranslationLanguage(language)) {
                  setSelectedLanguage(language);
                }
              }}
              placeholder={t("language")}
              disabled={drafting}
            />
          </div>
          {genreInput === "自定义" ? (
            <input
              className="custom-genre-input"
              placeholder={t("customGenrePrompt")}
              value={customGenre}
              onChange={(event) => setCustomGenre(event.target.value)}
              disabled={drafting}
            />
          ) : null}
          <button className="primary-button" onClick={() => void generateDraft()} disabled={drafting || !genreInput.trim() || (genreInput === "自定义" && !customGenre.trim())}>
            <Sparkles size={16} />
            {drafting ? t("filling") : t("generateDraft")}
          </button>
        </div>
      ) : null}

      {openForm ? (
        <form className="world-form" onSubmit={createWorld}>
          <input
            name="title"
            placeholder={t("worldTitle")}
            required
            value={formValues.title}
            onChange={(event) => updateField("title", event)}
          />
          <textarea
            name="description"
            placeholder={t("premise")}
            rows={4}
            value={formValues.description}
            onChange={(event) => updateField("description", event)}
          />
          <input
            name="genre"
            placeholder={t("genre")}
            value={formValues.genre}
            onChange={(event) => updateField("genre", event)}
          />
          <div className="split">
            <input
              name="target_language"
              placeholder={t("language")}
              value={formValues.target_language}
              onChange={(event) => updateField("target_language", event)}
            />
            <input
              name="language_level"
              placeholder={t("level")}
              value={formValues.language_level}
              onChange={(event) => updateField("language_level", event)}
            />
          </div>
          <input
            name="narrative_style"
            placeholder={t("narrativeStyle")}
            value={formValues.narrative_style}
            onChange={(event) => updateField("narrative_style", event)}
          />
          <button className="primary-button" disabled={creating}>
            <Sparkles size={16} />
            {creating ? t("creating") : t("create")}
          </button>
        </form>
      ) : null}

      <div className="world-list">
        {worlds.map((world) => (
          <div className={world.id === activeWorld?.id ? "world-item active" : "world-item"} key={world.id}>
            <button className="world-open-button" onClick={() => void openWorld(world.id)}>
              <strong>{world.title}</strong>
              <span>{world.target_language} · {world.language_level}</span>
              <p>{world.description || t("noDescription")}</p>
            </button>
            <button
              className="icon-button world-export"
              aria-label={`${t("exportWorld")} ${world.title}`}
              title={t("exportWorld")}
              onClick={() => void exportWorld(world.id, world.title)}
            >
              <Upload size={15} />
            </button>
            <button
              className="icon-button danger"
              aria-label={`${t("deleteWorld")} ${world.title}`}
              title={t("deleteWorld")}
              onClick={() => void deleteWorld(world.id, world.title)}
            >
              <Trash2 size={15} />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

function safeZipName(title: string) {
  const fallback = "world";
  const safe = title.trim().replace(/[<>:"/\\|?*\u0000-\u001f]/g, "-").replace(/\s+/g, " ");
  return safe || fallback;
}

function normalDifficultyLabel(targetLanguage: string) {
  const normalized = targetLanguage.trim().toLowerCase();
  if (normalized.includes("english")) {
    return "Normal difficulty";
  }
  if (normalized.includes("日本")) {
    return "一般的な難易度";
  }
  if (normalized.includes("한국")) {
    return "보통 난이도";
  }
  if (normalized.includes("français")) {
    return "Difficulté normale";
  }
  if (normalized.includes("deutsch")) {
    return "Normale Schwierigkeit";
  }
  if (normalized.includes("español")) {
    return "Dificultad normal";
  }
  if (normalized.includes("português")) {
    return "Dificuldade normal";
  }
  if (normalized.includes("italiano")) {
    return "Difficoltà normale";
  }
  if (normalized.includes("русский")) {
    return "Обычная сложность";
  }
  if (normalized.includes("العربية")) {
    return "صعوبة عادية";
  }
  return "一般难度";
}
