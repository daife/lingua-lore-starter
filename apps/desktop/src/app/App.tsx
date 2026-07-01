import { useEffect, useRef, useState } from "react";
import { BookOpen, Library, Moon, Sun } from "lucide-react";
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
  const shellSwipeStart = useRef<{ x: number; y: number; identifier: number } | null>(null);
  const lastHandledSwipeAt = useRef(0);

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

  function applyHorizontalSwipe(deltaX: number) {
    lastHandledSwipeAt.current = Date.now();
    if (deltaX > 0) {
      if (settingsOpen) {
        setSettingsOpen(false);
      } else {
        setLibraryOpen(true);
        setSettingsOpen(false);
      }
    } else if (deltaX < 0) {
      if (libraryOpen) {
        setLibraryOpen(false);
      } else {
        setSettingsOpen(true);
        setLibraryOpen(false);
      }
    }
  }

  useEffect(() => {
    function handleTouchStart(event: TouchEvent) {
      if (event.touches.length !== 1) {
        shellSwipeStart.current = null;
        return;
      }
      const touch = event.touches[0];
      shellSwipeStart.current = { x: touch.clientX, y: touch.clientY, identifier: touch.identifier };
    }

    function findActiveTouch(event: TouchEvent) {
      const start = shellSwipeStart.current;
      if (!start) {
        return null;
      }
      return Array.from(event.changedTouches).find((touch) => touch.identifier === start.identifier) ?? null;
    }

    function handleTouchMove(event: TouchEvent) {
      const touch = findActiveTouch(event);
      if (!touch) {
        return;
      }
      handleShellSwipeProgress(touch.clientX, touch.clientY, touch.identifier);
    }

    function handleTouchEnd(event: TouchEvent) {
      const touch = findActiveTouch(event);
      if (touch) {
        handleShellSwipeProgress(touch.clientX, touch.clientY, touch.identifier);
      }
      const start = shellSwipeStart.current;
      if (start && (!touch || start.identifier === touch.identifier)) {
        shellSwipeStart.current = null;
      }
    }

    document.addEventListener("touchstart", handleTouchStart, { capture: true, passive: true });
    document.addEventListener("touchmove", handleTouchMove, { capture: true, passive: true });
    document.addEventListener("touchend", handleTouchEnd, { capture: true, passive: true });
    document.addEventListener("touchcancel", handleTouchEnd, { capture: true, passive: true });
    return () => {
      document.removeEventListener("touchstart", handleTouchStart, { capture: true });
      document.removeEventListener("touchmove", handleTouchMove, { capture: true });
      document.removeEventListener("touchend", handleTouchEnd, { capture: true });
      document.removeEventListener("touchcancel", handleTouchEnd, { capture: true });
    };
  }, [libraryOpen, settingsOpen]);

  function handleShellSwipeProgress(clientX: number, clientY: number, identifier: number) {
    const start = shellSwipeStart.current;
    if (!start || start.identifier !== identifier) {
      return;
    }
    const selection = window.getSelection()?.toString().trim();
    if (selection) {
      shellSwipeStart.current = null;
      return;
    }
    const deltaX = clientX - start.x;
    const deltaY = clientY - start.y;
    if (Math.abs(deltaX) < SWIPE_DISTANCE || Math.abs(deltaX) < Math.abs(deltaY) * SWIPE_AXIS_RATIO) {
      return;
    }
    applyHorizontalSwipe(deltaX);
    shellSwipeStart.current = null;
  }

  function closeSidePanels() {
    setLibraryOpen(false);
    setSettingsOpen(false);
  }

  function handleBackdropClick() {
    if (Date.now() - lastHandledSwipeAt.current < 350) {
      return;
    }
    closeSidePanels();
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
          onClick={handleBackdropClick}
        />
      ) : null}
      <aside className="sidebar" aria-label={t("worldLibrary")} aria-hidden={!libraryOpen}>
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
