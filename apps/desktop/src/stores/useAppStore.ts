import { create } from "zustand";
import { defaultAppLanguage, type AppLanguage } from "../lib/i18n";
import { defaultTranslationLanguage, type TranslationLanguage } from "../lib/languages";
import type { ChoiceOutput, OfficialAccount, StoryTurnResult, TurnOutput, WorldRecord } from "../lib/types";

export interface ReaderTurn {
  id: string;
  userInput?: string | null;
  output: TurnOutput;
}

interface AppStore {
  worlds: WorldRecord[];
  activeWorld?: WorldRecord;
  activeSceneId?: string;
  officialAccount?: OfficialAccount | null;
  turns: ReaderTurn[];
  choices: ChoiceOutput[];
  loading: boolean;
  quickMode: boolean;
  appLanguage: AppLanguage;
  translationLanguage: TranslationLanguage;
  libraryError?: string;
  readerError?: string;
  settingsError?: string;
  setWorlds: (worlds: WorldRecord[]) => void;
  setActiveWorld: (world: WorldRecord, sceneId: string, turns?: ReaderTurn[]) => void;
  clearActiveWorld: () => void;
  setOfficialAccount: (account: OfficialAccount | null) => void;
  pushTurn: (result: StoryTurnResult) => void;
  setLoading: (loading: boolean) => void;
  setQuickMode: (quickMode: boolean) => void;
  setAppLanguage: (language: AppLanguage) => void;
  setTranslationLanguage: (language: TranslationLanguage) => void;
  setLibraryError: (error?: string) => void;
  setReaderError: (error?: string) => void;
  setSettingsError: (error?: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  worlds: [],
  turns: [],
  choices: [],
  loading: false,
  quickMode: false,
  appLanguage: defaultAppLanguage(),
  translationLanguage: defaultTranslationLanguage(),
  setWorlds: (worlds) => set({ worlds }),
  setActiveWorld: (activeWorld, activeSceneId, turns = []) => {
    const lastTurn = turns[turns.length - 1];
    set({
      activeWorld,
      activeSceneId,
      turns,
      choices: lastTurn?.output.choices ?? []
    });
  },
  clearActiveWorld: () =>
    set({ activeWorld: undefined, activeSceneId: undefined, turns: [], choices: [] }),
  setOfficialAccount: (officialAccount) => set({ officialAccount }),
  pushTurn: (result) =>
    set((state) => ({
      turns: [
        ...state.turns,
        { id: result.turn_id, userInput: result.user_input, output: result.output }
      ],
      choices: result.output.choices
    })),
  setLoading: (loading) => set({ loading }),
  setQuickMode: (quickMode) => set({ quickMode }),
  setAppLanguage: (appLanguage) => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem("lingua-lore-app-language", appLanguage);
    }
    set({ appLanguage });
  },
  setTranslationLanguage: (translationLanguage) => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem("lingua-lore-translation-language", translationLanguage);
    }
    set({ translationLanguage });
  },
  setLibraryError: (libraryError) => set({ libraryError }),
  setReaderError: (readerError) => set({ readerError }),
  setSettingsError: (settingsError) => set({ settingsError })
}));
