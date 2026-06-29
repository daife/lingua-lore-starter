import { type MutableRefObject, type PointerEvent, useEffect, useRef, useState } from "react";
import { BookOpen, Library, Moon, Settings, Sun } from "lucide-react";
import { WorldLibraryPage } from "../pages/WorldLibraryPage";
import { ReaderPage } from "../pages/ReaderPage";
import { SettingsPanel } from "../pages/SettingsPage";
import { translate } from "../lib/i18n";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/useAppStore";

const SWIPE_DISTANCE = 86;
const SWIPE_AXIS_RATIO = 1.35;
type ThemeMode = "day" | "night";

function defaultThemeMode(): ThemeMode {
  if (typeof window === "undefined") {
    return "day";
  }
  return window.localStorage.getItem("lingua-lore-theme") === "night" ? "night" : "day";
}

export function App() {
  const {
    activeWorld,
    appLanguage,
    settingsError,
    setWorlds,
    setOfficialAccount,
    setLibraryError,
    setSettingsError
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const shouldShowPanels = () =>
    typeof window === "undefined" ? true : !window.matchMedia("(max-width: 1180px)").matches;
  const [libraryOpen, setLibraryOpen] = useState(shouldShowPanels);
  const [settingsOpen, setSettingsOpen] = useState(shouldShowPanels);
  const [availableVersion, setAvailableVersion] = useState("");
  const [announcement, setAnnouncement] = useState("");
  const [themeMode, setThemeMode] = useState<ThemeMode>(defaultThemeMode);
  const readerSwipeStart = useRef<{ x: number; y: number } | null>(null);
  const librarySwipeStart = useRef<{ x: number; y: number } | null>(null);
  const settingsSwipeStart = useRef<{ x: number; y: number } | null>(null);

  useEffect(() => {
    void (async () => {
      const result = await api.checkVersion();
      if (result.has_update) {
        setAvailableVersion(result.latest_version);
        return;
      }
      const announcementResult = await api.checkAnnouncement();
      if (announcementResult.content) {
        setAnnouncement(announcementResult.content);
      }
    })();
  }, [appLanguage]);

  useEffect(() => {
    document.documentElement.dataset.theme = themeMode;
  }, [themeMode]);

  useEffect(() => {
    void api
      .listWorlds()
      .then(setWorlds)
      .catch((err) => setLibraryError(String(err)));
    void api
      .getOfficialAccount()
      .then(setOfficialAccount)
      .catch((err) => setSettingsError(String(err)));
  }, [setOfficialAccount, setLibraryError, setSettingsError, setWorlds]);

  function captureSwipeStart(
    ref: MutableRefObject<{ x: number; y: number } | null>,
    event: PointerEvent<HTMLElement>
  ) {
    if (event.pointerType === "mouse" && event.button !== 0) {
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    ref.current = { x: event.clientX, y: event.clientY };
  }

  function readHorizontalSwipe(
    ref: MutableRefObject<{ x: number; y: number } | null>,
    event: PointerEvent<HTMLElement>
  ) {
    const start = ref.current;
    ref.current = null;
    if (!start) {
      return 0;
    }
    const deltaX = event.clientX - start.x;
    const deltaY = event.clientY - start.y;
    if (Math.abs(deltaX) < SWIPE_DISTANCE || Math.abs(deltaX) < Math.abs(deltaY) * SWIPE_AXIS_RATIO) {
      return 0;
    }
    return deltaX;
  }

  function handleReaderSwipeEnd(event: PointerEvent<HTMLElement>) {
    const deltaX = readHorizontalSwipe(readerSwipeStart, event);
    if (deltaX > 0) {
      setLibraryOpen(true);
      setSettingsOpen(false);
    } else if (deltaX < 0) {
      setSettingsOpen(true);
      setLibraryOpen(false);
    }
  }

  function handleLibrarySwipeEnd(event: PointerEvent<HTMLElement>) {
    const deltaX = readHorizontalSwipe(librarySwipeStart, event);
    if (deltaX < 0) {
      setLibraryOpen(false);
    }
  }

  function handleSettingsSwipeEnd(event: PointerEvent<HTMLElement>) {
    const deltaX = readHorizontalSwipe(settingsSwipeStart, event);
    if (deltaX > 0) {
      setSettingsOpen(false);
    }
  }

  function closeSidePanels() {
    setLibraryOpen(false);
    setSettingsOpen(false);
  }

  function toggleThemeMode() {
    setThemeMode((current) => {
      const next = current === "night" ? "day" : "night";
      if (typeof window !== "undefined") {
        window.localStorage.setItem("lingua-lore-theme", next);
      }
      return next;
    });
  }

  return (
    <main
      className={[
        "shell",
        themeMode === "night" ? "night-mode" : "day-mode",
        libraryOpen ? "" : "library-collapsed",
        settingsOpen ? "" : "settings-collapsed"
      ].filter(Boolean).join(" ")}
    >
      {libraryOpen || settingsOpen ? (
        <button
          className="panel-backdrop"
          type="button"
          aria-label={t("closeSidePanels")}
          onPointerDown={closeSidePanels}
        />
      ) : null}
      <aside className="sidebar" aria-label={t("worldLibrary")} aria-hidden={!libraryOpen}>
        <div
          className="panel-swipe-zone"
          aria-hidden="true"
          onPointerDown={(event) => captureSwipeStart(librarySwipeStart, event)}
          onPointerUp={handleLibrarySwipeEnd}
          onPointerCancel={() => {
            librarySwipeStart.current = null;
          }}
        />
        <div className="brand">
          <BookOpen size={22} />
          <div>
            <strong>{t("brand")}</strong>
          </div>
        </div>
        <div className="section-title">
          <Library size={16} />
          <span>{t("worlds")}</span>
        </div>
        <WorldLibraryPage />
      </aside>

      <section className="reader-shell">
        <div
          className="reader-swipe-zone"
          aria-hidden="true"
          onPointerDown={(event) => captureSwipeStart(readerSwipeStart, event)}
          onPointerUp={handleReaderSwipeEnd}
          onPointerCancel={() => {
            readerSwipeStart.current = null;
          }}
        />
        <button
          className="theme-toggle"
          type="button"
          aria-label={themeMode === "night" ? t("switchToDayMode") : t("switchToNightMode")}
          title={themeMode === "night" ? t("switchToDayMode") : t("switchToNightMode")}
          onClick={toggleThemeMode}
        >
          {themeMode === "night" ? <Sun size={17} /> : <Moon size={17} />}
        </button>
        {activeWorld ? <ReaderPage /> : <div className="empty-reader">{t("emptyReader")}</div>}
      </section>

      <aside className="inspector" aria-label={t("settingsAndStatus")} aria-hidden={!settingsOpen}>
        <div
          className="panel-swipe-zone"
          aria-hidden="true"
          onPointerDown={(event) => captureSwipeStart(settingsSwipeStart, event)}
          onPointerUp={handleSettingsSwipeEnd}
          onPointerCancel={() => {
            settingsSwipeStart.current = null;
          }}
        />
        <div className="section-title">
          <Settings size={16} />
          <span>{t("settings")}</span>
        </div>
        <SettingsPanel />
        {settingsError ? (
          <div className="error-box" role="alert">
            <button onClick={() => setSettingsError(undefined)}>{t("dismiss")}</button>
            <p>{settingsError}</p>
          </div>
        ) : null}
      </aside>
      {availableVersion ? (
        <div className="update-overlay" role="dialog" aria-modal="true" aria-labelledby="update-title">
          <div className="update-dialog">
            <h2 id="update-title">{t("updateTitle")}</h2>
            <p className="update-copy">
              {t("updateAvailable")} {availableVersion}
              {"\n\n"}
              {t("updatePrompt")}
            </p>
            <button className="primary-button" type="button" onClick={() => void api.quitApp()}>
              {t("quitApp")}
            </button>
          </div>
        </div>
      ) : null}
      {announcement && !availableVersion ? (
        <div className="update-overlay" role="dialog" aria-modal="true" aria-labelledby="announcement-title">
          <div className="update-dialog announcement-dialog">
            <h2 id="announcement-title">{t("announcementTitle")}</h2>
            <p className="update-copy announcement-copy">{announcement}</p>
            <button className="primary-button" type="button" onClick={() => setAnnouncement("")}>
              {t("gotIt")}
            </button>
          </div>
        </div>
      ) : null}
    </main>
  );
}
