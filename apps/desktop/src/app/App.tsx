import { type MouseEvent, type MutableRefObject, type PointerEvent, useEffect, useRef, useState } from "react";
import { BookOpen, Library, Settings } from "lucide-react";
import { WorldLibraryPage } from "../pages/WorldLibraryPage";
import { ReaderPage } from "../pages/ReaderPage";
import { SettingsPanel } from "../pages/SettingsPage";
import { translate } from "../lib/i18n";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/useAppStore";

const SWIPE_DISTANCE = 86;
const SWIPE_AXIS_RATIO = 1.35;

export function App() {
  const {
    activeWorld,
    appLanguage,
    settingsError,
    setWorlds,
    setApiProfile,
    setLibraryError,
    setSettingsError
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const shouldShowPanels = () =>
    typeof window === "undefined" ? true : !window.matchMedia("(max-width: 1180px)").matches;
  const [libraryOpen, setLibraryOpen] = useState(shouldShowPanels);
  const [settingsOpen, setSettingsOpen] = useState(shouldShowPanels);
  const [availableVersion, setAvailableVersion] = useState("");
  const readerSwipeStart = useRef<{ x: number; y: number } | null>(null);
  const librarySwipeStart = useRef<{ x: number; y: number } | null>(null);
  const settingsSwipeStart = useRef<{ x: number; y: number } | null>(null);
  const ignoreNextShellClickRef = useRef(false);

  useEffect(() => {
    void (async () => {
      const result = await api.checkVersion();
      if (result.has_update) {
        setAvailableVersion(result.latest_version);
      }
    })();
  }, [appLanguage]);

  useEffect(() => {
    void api
      .listWorlds()
      .then(setWorlds)
      .catch((err) => setLibraryError(String(err)));
    void api
      .getApiProfile()
      .then(setApiProfile)
      .catch((err) => setSettingsError(String(err)));
  }, [setApiProfile, setLibraryError, setSettingsError, setWorlds]);

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
      ignoreNextShellClickRef.current = true;
      setLibraryOpen(true);
      setSettingsOpen(false);
    } else if (deltaX < 0) {
      ignoreNextShellClickRef.current = true;
      setSettingsOpen(true);
      setLibraryOpen(false);
    }
  }

  function handleLibrarySwipeEnd(event: PointerEvent<HTMLElement>) {
    const deltaX = readHorizontalSwipe(librarySwipeStart, event);
    if (deltaX < 0) {
      ignoreNextShellClickRef.current = true;
      setLibraryOpen(false);
    }
  }

  function handleSettingsSwipeEnd(event: PointerEvent<HTMLElement>) {
    const deltaX = readHorizontalSwipe(settingsSwipeStart, event);
    if (deltaX > 0) {
      ignoreNextShellClickRef.current = true;
      setSettingsOpen(false);
    }
  }

  function handleShellClick(event: MouseEvent<HTMLElement>) {
    if (ignoreNextShellClickRef.current) {
      ignoreNextShellClickRef.current = false;
      return;
    }
    const target = event.target as HTMLElement;
    if (libraryOpen && !target.closest(".sidebar")) {
      setLibraryOpen(false);
    }
    if (settingsOpen && !target.closest(".inspector")) {
      setSettingsOpen(false);
    }
  }

  return (
    <main
      className={[
        "shell",
        libraryOpen ? "" : "library-collapsed",
        settingsOpen ? "" : "settings-collapsed"
      ].filter(Boolean).join(" ")}
      onClick={handleShellClick}
    >
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
    </main>
  );
}
