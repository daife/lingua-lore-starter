import { invoke } from "@tauri-apps/api/core";
import type {
  ApiProfile,
  CreateWorldRequest,
  GenerateWorldDraftRequest,
  StoryTurnPreview,
  StoryTurnRequest,
  StoryTurnResult,
  TranslationResult,
  WorldBootstrap,
  WorldRecord
} from "./types";

export const api = {
  listWorlds: () => invoke<WorldRecord[]>("list_worlds"),
  createWorld: (request: CreateWorldRequest) => invoke<WorldRecord>("create_world", { request }),
  deleteWorld: (worldId: string) => invoke<void>("delete_world", { worldId }),
  exportWorld: (worldId: string) => invoke<number[]>("export_world", { worldId }),
  importWorld: (bytes: Uint8Array) => invoke<WorldRecord>("import_world", { bytes: Array.from(bytes) }),
  generateWorldDraft: (request: GenerateWorldDraftRequest) =>
    invoke<CreateWorldRequest>("generate_world_draft", { request }),
  getWorldBootstrap: (worldId: string) =>
    invoke<WorldBootstrap>("get_world_bootstrap", { worldId }),
  getApiProfile: () => invoke<ApiProfile | null>("get_api_profile"),
  saveApiProfile: (profile: ApiProfile) => invoke<ApiProfile>("save_api_profile", { profile }),
  previewStoryTurn: (input: StoryTurnRequest) =>
    invoke<StoryTurnPreview>("preview_story_turn", { input }),
  commitStoryTurnPreview: (preview: StoryTurnPreview) =>
    invoke<StoryTurnResult>("commit_story_turn_preview", { preview }),
  translateSelection: (payload: {
    worldId: string;
    text: string;
    context?: string;
    sourceLanguage: string;
    targetLanguage: string;
  }) =>
    invoke<TranslationResult>("translate_selection", {
      worldId: payload.worldId,
      text: payload.text,
      context: payload.context ?? null,
      sourceLanguage: payload.sourceLanguage,
      targetLanguage: payload.targetLanguage
    })
};
