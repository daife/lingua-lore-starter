import { FormEvent, MouseEvent, UIEvent, useEffect, useRef, useState } from "react";
import { Loader2, Send, Wand2 } from "lucide-react";
import { translate } from "../lib/i18n";
import { supportedTranslationLanguageForSource } from "../lib/languages";
import { api } from "../lib/tauri";
import { readSelectionSnapshot, SelectionSnapshot } from "../lib/selection";
import type { ChoiceOutput, StoryTurnInput, StoryTurnPreview, TranslationResult } from "../lib/types";
import { useAppStore } from "../stores/useAppStore";

const BEGIN_STORY_ACTION = "Begin the story with a vivid opening scene.";
const SELECTION_SETTLE_DELAY_MS = 220;
const TOUCH_SELECTION_PROBE_MS = 900;
const TOUCH_SELECTION_PROBE_INTERVAL_MS = 120;
const TRANSLATION_TIMEOUT_MS = 5000;

interface PreviewCacheEntry {
  promise: Promise<StoryTurnPreview>;
  result?: StoryTurnPreview;
}

export function ReaderPage() {
  const {
    activeWorld,
    activeSceneId,
    appLanguage,
    translationLanguage,
    turns,
    choices,
    loading,
    quickMode,
    readerError,
    setLoading,
    setReaderError,
    pushTurn
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const storyRef = useRef<HTMLDivElement | null>(null);
  const requestInFlightRef = useRef(false);
  const prefetchGenerationRef = useRef(0);
  const turnPositionTimerRef = useRef<number | null>(null);
  const selectionTimerRef = useRef<number | null>(null);
  const selectionGenerationRef = useRef(0);
  const selectionPointerDownRef = useRef(false);
  const pendingSelectionChangeRef = useRef(false);
  const lastTranslatedSelectionKeyRef = useRef("");
  const previewCacheRef = useRef<Map<string, PreviewCacheEntry>>(new Map());
  const [selection, setSelection] = useState<SelectionSnapshot | null>(null);
  const [translation, setTranslation] = useState<TranslationResult | null>(null);
  const [translating, setTranslating] = useState(false);
  const [currentTurn, setCurrentTurn] = useState(0);
  const [showTurnPosition, setShowTurnPosition] = useState(false);
  const [, setBuffering] = useState(false);

  const world = activeWorld;
  const sceneId = activeSceneId;
  const storyStarted = Boolean(world && sceneId && turns.length > 0);
  const latestTurnId = turns[turns.length - 1]?.id ?? "opening";
  const choiceSignature = choices.map((choice) => choice.id ?? choice.label).join("|");

  useEffect(() => {
    if (!storyStarted) {
      setCurrentTurn(0);
      return;
    }
    setCurrentTurn(turns.length);
    requestAnimationFrame(() => {
      scrollToLatestAction();
    });
  }, [storyStarted, turns.length]);

  useEffect(() => {
    return () => {
      if (turnPositionTimerRef.current) {
        window.clearTimeout(turnPositionTimerRef.current);
      }
      if (selectionTimerRef.current) {
        window.clearTimeout(selectionTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!world) {
      return;
    }
    const handleSelectionChange = () => {
      pendingSelectionChangeRef.current = true;
      if (!selectionPointerDownRef.current) {
        queueSelectionTranslation(SELECTION_SETTLE_DELAY_MS);
      }
    };
    document.addEventListener("selectionchange", handleSelectionChange);
    return () => document.removeEventListener("selectionchange", handleSelectionChange);
  }, [world?.id, world?.target_language]);

  useEffect(() => {
    const generation = prefetchGenerationRef.current + 1;
    prefetchGenerationRef.current = generation;
    previewCacheRef.current.clear();

    if (!world || !sceneId || !quickMode || !storyStarted || !choices.length) {
      setBuffering(false);
      return;
    }

    const prefetchableChoices = choices.filter((choice) => choice.id);
    if (!prefetchableChoices.length) {
      setBuffering(false);
      return;
    }

    setBuffering(true);
    const prefetches = prefetchableChoices.map((choice) => {
      const input: StoryTurnInput = { kind: "choice", choice_id: choice.id as string };
      const promise = api
        .previewStoryTurn({
          world_id: world.id,
          scene_id: sceneId,
          input
        })
        .then((preview) => {
          if (prefetchGenerationRef.current !== generation) {
            return preview;
          }
          const cached = previewCacheRef.current.get(choice.id as string);
          if (cached) {
            cached.result = preview;
          }
          return preview;
        });
      previewCacheRef.current.set(choice.id as string, { promise });
      return promise;
    });

    void Promise.allSettled(prefetches).then(() => {
      if (prefetchGenerationRef.current === generation) {
        setBuffering(false);
      }
    });
  }, [quickMode, storyStarted, choiceSignature, latestTurnId, world?.id, sceneId]);

  if (!world || !sceneId) {
    return null;
  }

  const currentWorld = world;
  const currentSceneId = sceneId;

  function invalidatePreviewBuffer() {
    prefetchGenerationRef.current += 1;
    previewCacheRef.current.clear();
    setBuffering(false);
  }

  async function sendFreeText(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const text = String(form.get("text") || "").trim();
    if (!text || loading || requestInFlightRef.current) {
      return;
    }
    event.currentTarget.reset();
    await sendTurn({ kind: "free_text", text });
  }

  async function selectChoice(choice: ChoiceOutput) {
    if (loading || requestInFlightRef.current) {
      return;
    }
    if (choice.id) {
      if (quickMode) {
        const cached = previewCacheRef.current.get(choice.id);
        if (cached) {
          await commitBufferedTurn(cached);
          return;
        }
      }
      await sendTurn({ kind: "choice", choice_id: choice.id });
      return;
    }
    await sendTurn({ kind: "free_text", text: `${choice.label}. ${choice.text}` });
  }

  async function commitBufferedTurn(entry: PreviewCacheEntry) {
    if (requestInFlightRef.current) {
      return;
    }
    requestInFlightRef.current = true;
    setLoading(true);
    try {
      const preview = entry.result ?? (await entry.promise);
      const result = await api.commitStoryTurnPreview(preview);
      pushTurn(result);
    } catch (err) {
      setReaderError(String(err));
    } finally {
      invalidatePreviewBuffer();
      requestInFlightRef.current = false;
      setLoading(false);
    }
  }

  async function sendTurn(input: { kind: "free_text"; text: string } | { kind: "choice"; choice_id: string }) {
    if (requestInFlightRef.current) {
      return;
    }
    requestInFlightRef.current = true;
    setLoading(true);
    try {
      invalidatePreviewBuffer();
      const preview = await api.previewStoryTurn({
        world_id: currentWorld.id,
        scene_id: currentSceneId,
        input
      });
      const result = await api.commitStoryTurnPreview(preview);
      pushTurn(result);
    } catch (err) {
      setReaderError(String(err));
    } finally {
      requestInFlightRef.current = false;
      setLoading(false);
    }
  }

  function handleStoryScroll(event: UIEvent<HTMLDivElement>) {
    setShowTurnPosition(true);
    if (turnPositionTimerRef.current) {
      window.clearTimeout(turnPositionTimerRef.current);
    }
    turnPositionTimerRef.current = window.setTimeout(() => setShowTurnPosition(false), 900);
    const viewport = event.currentTarget;
    const viewportTop = viewport.getBoundingClientRect().top;
    const blocks = Array.from(viewport.querySelectorAll<HTMLElement>("[data-turn-index]"));
    let visibleTurn = currentTurn || 1;
    for (const block of blocks) {
      const index = Number(block.dataset.turnIndex || "1");
      if (block.getBoundingClientRect().top - viewportTop < 120) {
        visibleTurn = index;
      }
    }
    setCurrentTurn(Math.min(Math.max(visibleTurn, 1), turns.length));
  }

  function scrollToLatestAction() {
    const viewport = storyRef.current;
    if (!viewport) {
      return;
    }
    const latestTurn = viewport.querySelector<HTMLElement>(`[data-turn-index="${turns.length}"]`);
    const action = latestTurn?.querySelector<HTMLElement>(".user-action");
    if (!action) {
      viewport.scrollTo({ top: viewport.scrollHeight, behavior: "smooth" });
      return;
    }
    const viewportRect = viewport.getBoundingClientRect();
    const actionRect = action.getBoundingClientRect();
    const targetTop =
      viewport.scrollTop + actionRect.top - viewportRect.top - (viewport.clientHeight - actionRect.height) / 2;
    viewport.scrollTo({ top: targetTop, behavior: "smooth" });
  }

  function queueSelectionTranslation(delayMs = 0, probeUntilMs = 0) {
    if (selectionTimerRef.current) {
      window.clearTimeout(selectionTimerRef.current);
    }
    const generation = selectionGenerationRef.current + 1;
    selectionGenerationRef.current = generation;
    setTranslating(false);
    scheduleSelectionTranslation(generation, delayMs, probeUntilMs ? Date.now() + probeUntilMs : 0);
  }

  function scheduleSelectionTranslation(generation: number, delayMs: number, probeUntilTime: number) {
    selectionTimerRef.current = window.setTimeout(() => {
      pendingSelectionChangeRef.current = false;
      void translateCurrentSelection(generation, probeUntilTime);
    }, delayMs);
  }

  async function translateCurrentSelection(generation: number, probeUntilTime = 0) {
    if (selectionGenerationRef.current !== generation) {
      return;
    }
    const snapshot = readSelectionSnapshot(storyRef.current);
    if (!snapshot && probeUntilTime > Date.now()) {
      scheduleSelectionTranslation(generation, TOUCH_SELECTION_PROBE_INTERVAL_MS, probeUntilTime);
      return;
    }
    const selectionKey = snapshot ? `${snapshot.text}|${Math.round(snapshot.x)}|${Math.round(snapshot.y)}` : "";
    if (selectionKey && selectionKey === lastTranslatedSelectionKeyRef.current && translation) {
      return;
    }
    setSelection(snapshot);
    setTranslation(null);
    if (!snapshot) {
      setTranslating(false);
      return;
    }
    setTranslating(true);
    try {
      const result = await translateSelectionOnce(snapshot, generation);
      if (selectionGenerationRef.current === generation) {
        setTranslation(result);
        lastTranslatedSelectionKeyRef.current = selectionKey;
      }
    } catch (err) {
      if (selectionGenerationRef.current === generation) {
        setReaderError(String(err));
      }
    } finally {
      if (selectionGenerationRef.current === generation) {
        setTranslating(false);
      }
    }
  }

  async function translateSelectionOnce(snapshot: SelectionSnapshot, generation: number) {
    if (selectionGenerationRef.current !== generation) {
      throw new Error("Translation superseded");
    }
    return new Promise<TranslationResult>((resolve, reject) => {
      const timeoutId = window.setTimeout(() => {
        reject(new Error("Translation timed out"));
      }, TRANSLATION_TIMEOUT_MS);
      api
        .translateSelection({
          worldId: currentWorld.id,
          text: snapshot.text,
          context: snapshot.context,
          sourceLanguage: currentWorld.target_language,
          targetLanguage: supportedTranslationLanguageForSource(currentWorld.target_language, translationLanguage)
        })
        .then(resolve)
        .catch(reject)
        .finally(() => {
          window.clearTimeout(timeoutId);
        });
    });
  }

  function handleSelectionPointerDown() {
    selectionPointerDownRef.current = true;
    pendingSelectionChangeRef.current = false;
    selectionGenerationRef.current += 1;
    setTranslating(false);
    if (selectionTimerRef.current) {
      window.clearTimeout(selectionTimerRef.current);
    }
  }

  function handleSelectionPointerUp() {
    selectionPointerDownRef.current = false;
    if (pendingSelectionChangeRef.current || readSelectionSnapshot(storyRef.current)) {
      queueSelectionTranslation(SELECTION_SETTLE_DELAY_MS);
    }
  }

  function handleMouseUp(_event: MouseEvent) {
    handleSelectionPointerUp();
  }

  function handleTouchEnd() {
    selectionPointerDownRef.current = false;
    queueSelectionTranslation(SELECTION_SETTLE_DELAY_MS, TOUCH_SELECTION_PROBE_MS);
  }

  function handleTouchCancel() {
    selectionPointerDownRef.current = false;
    pendingSelectionChangeRef.current = false;
  }

  function handleMouseDown() {
    handleSelectionPointerDown();
  }

  function handleTouchStart() {
    handleSelectionPointerDown();
  }

  function handlePointerLeave() {
    if (selectionPointerDownRef.current) {
      handleSelectionPointerUp();
    }
  }

  return (
    <div className="reader-page">
      <header className="story-header">
        <div>
          <span>{currentWorld.target_language} · {currentWorld.language_level}</span>
          <h1>{currentWorld.title}</h1>
        </div>
      </header>

      <div
        className="story-viewport"
        ref={storyRef}
        onMouseDown={handleMouseDown}
        onMouseLeave={handlePointerLeave}
        onMouseUp={handleMouseUp}
        onTouchCancel={handleTouchCancel}
        onTouchEnd={handleTouchEnd}
        onTouchStart={handleTouchStart}
        onScroll={handleStoryScroll}
      >
        {storyStarted ? (
          <div className={showTurnPosition ? "turn-position visible" : "turn-position"}>
            {t("turn")} {currentTurn || 1} / {turns.length}
          </div>
        ) : null}
        {!storyStarted ? (
          <div className="opening-note">
            <h2>{currentWorld.description || t("newStoryWaiting")}</h2>
            <button
              className="primary-button"
              disabled={loading}
              onClick={() => void sendTurn({ kind: "free_text", text: BEGIN_STORY_ACTION })}
            >
              {loading ? <Loader2 className="spin" size={16} /> : <Wand2 size={16} />}
              {t("beginStory")}
            </button>
          </div>
        ) : (
          turns.map((turn, index) => (
            <article className="turn-block" key={turn.id} data-turn-index={index + 1}>
              {turn.userInput && turn.userInput !== BEGIN_STORY_ACTION ? (
                <p className="user-action">{turn.userInput}</p>
              ) : null}
              <p className="narration">{turn.output.narration}</p>
              {turn.output.dialogues.map((dialogue, index) => (
                <p className="dialogue" key={`${turn.id}-${index}`}>
                  <strong>{dialogue.speaker}</strong>
                  <span>{dialogue.text}</span>
                </p>
              ))}
              <div className="scene-strip">
                <span>{turn.output.scene_status.location}</span>
                <span>{turn.output.scene_status.mood}</span>
                <span>{turn.output.scene_status.current_objective}</span>
              </div>
            </article>
          ))
        )}
      </div>

      {storyStarted ? (
        <section className="choice-panel" aria-label={t("choices")}>
          {choices.map((choice) => (
            <button className="choice-card" key={choice.label} onClick={() => void selectChoice(choice)} disabled={loading}>
              <span>{choice.text}</span>
            </button>
          ))}
        </section>
      ) : null}

      {storyStarted ? (
        <form className="input-box" onSubmit={sendFreeText}>
          <input name="text" placeholder={t("freeActionPlaceholder")} disabled={loading} />
          <button className="primary-button" disabled={loading}>
            {loading ? <Loader2 className="spin" size={16} /> : <Send size={16} />}
            {t("send")}
          </button>
        </form>
      ) : null}

      {readerError ? (
        <div className="error-box reader-error" role="alert">
          <button onClick={() => setReaderError(undefined)}>{t("dismiss")}</button>
          <p>{readerError}</p>
        </div>
      ) : null}

      {selection ? (
        <div className="translate-popover" style={{ left: selection.x, top: selection.y }}>
          <strong>{selection.text}</strong>
          {translating ? <p>{t("translating")}</p> : null}
          {translation ? (
            <>
              <p>{translation.translated_text || t("noTranslation")}</p>
              <div className="phones">
                {translation.us_phone ? <span>{t("usPhone")} /{translation.us_phone}/</span> : null}
                {translation.uk_phone ? <span>{t("ukPhone")} /{translation.uk_phone}/</span> : null}
              </div>
              {translation.phrases.length ? (
                <ul>
                  {translation.phrases.slice(0, 3).map((item) => (
                    <li key={item.key}>{item.key}: {item.value}</li>
                  ))}
                </ul>
              ) : null}
            </>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
