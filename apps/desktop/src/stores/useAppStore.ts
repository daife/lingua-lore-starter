import { create } from "zustand";
import { defaultAppLanguage, type AppLanguage } from "../lib/i18n";
import type { ApiProfile, ChoiceOutput, StoryTurnResult, TurnOutput, WorldRecord } from "../lib/types";

export interface ReaderTurn {
  id: string;
  userInput?: string | null;
  output: TurnOutput;
}

interface AppStore {
  worlds: WorldRecord[];
  activeWorld?: WorldRecord;
  activeSceneId?: string;
  apiProfile?: ApiProfile | null;
  turns: ReaderTurn[];
  choices: ChoiceOutput[];
  loading: boolean;
  quickMode: boolean;
  appLanguage: AppLanguage;
  error?: string;
  setWorlds: (worlds: WorldRecord[]) => void;
  setActiveWorld: (world: WorldRecord, sceneId: string, turns?: ReaderTurn[]) => void;
  clearActiveWorld: () => void;
  setApiProfile: (profile: ApiProfile | null) => void;
  pushTurn: (result: StoryTurnResult) => void;
  setLoading: (loading: boolean) => void;
  setQuickMode: (quickMode: boolean) => void;
  setAppLanguage: (language: AppLanguage) => void;
  setError: (error?: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  worlds: [],
  turns: [],
  choices: [],
  loading: false,
  quickMode: false,
  appLanguage: defaultAppLanguage(),
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
  setApiProfile: (apiProfile) => set({ apiProfile }),
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
  setError: (error) => set({ error })
}));
