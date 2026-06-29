import { invoke } from "@tauri-apps/api/core";
import type {
  AnnouncementResult,
  CheckVersionResult,
  CreateWorldRequest,
  DetectedPhone,
  GenerateWorldDraftRequest,
  OfficialAccount,
  QuotaInfo,
  RegisterOfficialAccountRequest,
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
  getOfficialAccount: () => invoke<OfficialAccount>("get_official_account"),
  detectRegistrationPhone: () => invoke<DetectedPhone>("detect_registration_phone"),
  registerOfficialAccount: (request: RegisterOfficialAccountRequest) =>
    invoke<OfficialAccount>("register_official_account", { request }),
  refreshQuota: () => invoke<QuotaInfo>("refresh_quota"),
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
    }),
  checkVersion: () => invoke<CheckVersionResult>("check_version"),
  checkAnnouncement: () => invoke<AnnouncementResult>("check_announcement"),
  quitApp: () => invoke<void>("quit_app")
};
