import { FormEvent, MouseEvent, UIEvent, useEffect, useRef, useState } from "react";
import { Loader2, Send, Wand2 } from "lucide-react";
import { api } from "../lib/tauri";
import { readSelectionSnapshot, SelectionSnapshot } from "../lib/selection";
import type { ChoiceOutput, StoryTurnInput, StoryTurnPreview, TranslationResult } from "../lib/types";
import { useAppStore } from "../stores/useAppStore";

const BEGIN_STORY_ACTION = "Begin the story with a vivid opening scene.";

interface PreviewCacheEntry {
  promise: Promise<StoryTurnPreview>;
  result?: StoryTurnPreview;
}

function translationTargetForStoryLanguage(language: string) {
  const normalized = language.trim().toLowerCase();
  if (
    normalized.includes("中文") ||
    normalized.includes("chinese") ||
    normalized.includes("简体") ||
    normalized.includes("繁體")
  ) {
    return "English";
  }
  return "Chinese";
}

export function ReaderPage() {
  const {
    activeWorld,
    activeSceneId,
    turns,
    choices,
    loading,
    quickMode,
    setLoading,
    setError,
    pushTurn
  } = useAppStore();
  const storyRef = useRef<HTMLDivElement | null>(null);
  const requestInFlightRef = useRef(false);
  const prefetchGenerationRef = useRef(0);
  const turnPositionTimerRef = useRef<number | null>(null);
  const selectionTimerRef = useRef<number | null>(null);
  const selectionGenerationRef = useRef(0);
  const lastSelectionKeyRef = useRef("");
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
      storyRef.current?.scrollTo({ top: storyRef.current.scrollHeight, behavior: "smooth" });
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
      queueSelectionTranslation(320);
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
      setError(String(err));
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
      setError(String(err));
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

  function queueSelectionTranslation(delayMs = 0) {
    if (selectionTimerRef.current) {
      window.clearTimeout(selectionTimerRef.current);
    }
    selectionTimerRef.current = window.setTimeout(() => {
      void translateCurrentSelection();
    }, delayMs);
  }

  async function translateCurrentSelection() {
    const generation = selectionGenerationRef.current + 1;
    selectionGenerationRef.current = generation;
    const snapshot = readSelectionSnapshot(storyRef.current);
    const selectionKey = snapshot ? `${snapshot.text}|${Math.round(snapshot.x)}|${Math.round(snapshot.y)}` : "";
    if (selectionKey && selectionKey === lastSelectionKeyRef.current) {
      return;
    }
    lastSelectionKeyRef.current = selectionKey;
    setSelection(snapshot);
    setTranslation(null);
    if (!snapshot) {
      setTranslating(false);
      return;
    }
    setTranslating(true);
    try {
      const result = await api.translateSelection({
        worldId: currentWorld.id,
        text: snapshot.text,
        context: snapshot.context,
        sourceLanguage: currentWorld.target_language,
        targetLanguage: translationTargetForStoryLanguage(currentWorld.target_language)
      });
      if (selectionGenerationRef.current === generation) {
        setTranslation(result);
      }
    } catch (err) {
      if (selectionGenerationRef.current === generation) {
        setError(String(err));
      }
    } finally {
      if (selectionGenerationRef.current === generation) {
        setTranslating(false);
      }
    }
  }

  async function handleMouseUp(_event: MouseEvent) {
    if (window.matchMedia("(pointer: coarse)").matches) {
      queueSelectionTranslation(180);
      return;
    }
    await translateCurrentSelection();
  }

  return (
    <div className="reader-page">
      <header className="story-header">
        <div>
          <span>{currentWorld.target_language} · {currentWorld.language_level}</span>
          <h1>{currentWorld.title}</h1>
        </div>
      </header>

      <div className="story-viewport" ref={storyRef} onMouseUp={handleMouseUp} onScroll={handleStoryScroll}>
        {storyStarted ? (
          <div className={showTurnPosition ? "turn-position visible" : "turn-position"}>
            Turn {currentTurn || 1} / {turns.length}
          </div>
        ) : null}
        {!storyStarted ? (
          <div className="opening-note">
            <h2>{currentWorld.description || "A new story is waiting."}</h2>
            <button
              className="primary-button"
              disabled={loading}
              onClick={() => void sendTurn({ kind: "free_text", text: BEGIN_STORY_ACTION })}
            >
              {loading ? <Loader2 className="spin" size={16} /> : <Wand2 size={16} />}
              Begin the story
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
        <section className="choice-panel" aria-label="Choices">
          {choices.map((choice) => (
            <button className="choice-card" key={choice.label} onClick={() => void selectChoice(choice)} disabled={loading}>
              <span>{choice.text}</span>
            </button>
          ))}
        </section>
      ) : null}

      {storyStarted ? (
        <form className="input-box" onSubmit={sendFreeText}>
          <input name="text" placeholder="Type a free action..." disabled={loading} />
          <button className="primary-button" disabled={loading}>
            {loading ? <Loader2 className="spin" size={16} /> : <Send size={16} />}
            Send
          </button>
        </form>
      ) : null}

      {selection ? (
        <div className="translate-popover" style={{ left: selection.x, top: selection.y }}>
          <strong>{selection.text}</strong>
          {translating ? <p>Translating...</p> : null}
          {translation ? (
            <>
              <p>{translation.translated_text || "No translation found."}</p>
              <div className="phones">
                {translation.us_phone ? <span>US /{translation.us_phone}/</span> : null}
                {translation.uk_phone ? <span>UK /{translation.uk_phone}/</span> : null}
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
