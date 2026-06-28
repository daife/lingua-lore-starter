export type TranslationLanguage = "简体中文" | "English" | "日本語" | "한국어";

export const SUPPORTED_STORY_LANGUAGES: TranslationLanguage[] = [
  "简体中文",
  "English",
  "日本語",
  "한국어"
];

export const TRANSLATION_LANGUAGE_OPTIONS = SUPPORTED_STORY_LANGUAGES;

export function supportedTranslationLanguagesForSource(sourceLanguage?: string): TranslationLanguage[] {
  const source = languageKind(sourceLanguage ?? "");
  if (source === "zh") {
    return ["English", "日本語", "한국어", "简体中文"];
  }
  if (source === "en") {
    return ["简体中文", "English"];
  }
  if (source === "ja" || source === "ko") {
    return ["简体中文"];
  }
  return TRANSLATION_LANGUAGE_OPTIONS;
}

export function supportedTranslationLanguageForSource(
  sourceLanguage: string | undefined,
  translationLanguage: TranslationLanguage
): TranslationLanguage {
  const options = supportedTranslationLanguagesForSource(sourceLanguage);
  return options.includes(translationLanguage) ? translationLanguage : options[0];
}

export function defaultTranslationLanguage(): TranslationLanguage {
  if (typeof window === "undefined") {
    return "简体中文";
  }
  const stored = window.localStorage.getItem("lingua-lore-translation-language");
  return isTranslationLanguage(stored) ? stored : "简体中文";
}

export function isTranslationLanguage(value: unknown): value is TranslationLanguage {
  return typeof value === "string" && SUPPORTED_STORY_LANGUAGES.includes(value as TranslationLanguage);
}

function languageKind(language: string) {
  const normalized = language.trim().toLowerCase();
  if (
    normalized.includes("chinese") ||
    normalized.includes("中文") ||
    normalized.includes("简体") ||
    normalized.includes("繁體") ||
    normalized === "zh-chs"
  ) {
    return "zh";
  }
  if (normalized === "english" || normalized === "en") {
    return "en";
  }
  if (normalized.includes("日本") || normalized.includes("japanese") || normalized === "ja") {
    return "ja";
  }
  if (normalized.includes("한국") || normalized.includes("korean") || normalized === "ko") {
    return "ko";
  }
  return "other";
}
