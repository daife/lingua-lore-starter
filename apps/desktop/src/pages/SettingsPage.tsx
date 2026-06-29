import { FormEvent, useEffect, useMemo, useState } from "react";
import { BadgeCheck, Download, RefreshCw, ShieldCheck, Smartphone, Ticket, Zap } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { APP_LANGUAGE_OPTIONS, translate, type AppLanguage } from "../lib/i18n";
import {
  isTranslationLanguage,
  supportedTranslationLanguageForSource,
  supportedTranslationLanguagesForSource
} from "../lib/languages";
import { api } from "../lib/tauri";
import { useAppStore } from "../stores/useAppStore";
import type { DetectedPhone, QuotaInfo } from "../lib/types";
import { Dropdown } from "./WorldLibraryPage";

function formatTokens(value?: number | null) {
  if (typeof value !== "number") {
    return "--";
  }
  return new Intl.NumberFormat("zh-CN").format(value);
}

export function SettingsPanel() {
  const {
    officialAccount,
    activeWorld,
    appLanguage,
    translationLanguage,
    quickMode,
    setOfficialAccount,
    setAppLanguage,
    setTranslationLanguage,
    setSettingsError,
    setQuickMode,
    setWorlds
  } = useAppStore();
  const t = (key: Parameters<typeof translate>[1], value?: string) => translate(appLanguage, key, value);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState("");
  const [quota, setQuota] = useState<QuotaInfo | null>(null);
  const [quotaLoading, setQuotaLoading] = useState(false);
  const [detectedPhone, setDetectedPhone] = useState<DetectedPhone | null>(null);
  const [detectingPhone, setDetectingPhone] = useState(false);
  const [middleFour, setMiddleFour] = useState("");
  const [inviteCode, setInviteCode] = useState("");
  const [registering, setRegistering] = useState(false);

  useEffect(() => {
    const supportedLanguage = supportedTranslationLanguageForSource(activeWorld?.target_language, translationLanguage);
    if (supportedLanguage !== translationLanguage) {
      setTranslationLanguage(supportedLanguage);
    }
  }, [activeWorld?.target_language, setTranslationLanguage, translationLanguage]);

  async function refreshQuota() {
    setQuotaLoading(true);
    setStatus("");
    try {
      setQuota(await api.refreshQuota());
    } catch (err) {
      setSettingsError(String(err));
    } finally {
      setQuotaLoading(false);
    }
  }

  async function detectPhone() {
    setDetectingPhone(true);
    setStatus("");
    try {
      setDetectedPhone(await api.detectRegistrationPhone());
    } catch (err) {
      setSettingsError(String(err));
    } finally {
      setDetectingPhone(false);
    }
  }

  async function register(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setRegistering(true);
    setStatus("");
    try {
      const account = await api.registerOfficialAccount({
        middle_four: middleFour,
        invite_code: inviteCode.trim() || null
      });
      setOfficialAccount(account);
      setDetectedPhone(null);
      setMiddleFour("");
      setInviteCode("");
      setStatus(t("registrationDone"));
    } catch (err) {
      setSettingsError(String(err));
    } finally {
      setRegistering(false);
    }
  }

  async function importWorld() {
    setImporting(true);
    setStatus("");
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: t("importWorldZip"),
        filters: [{ name: t("worldZip"), extensions: ["zip"] }]
      });
      if (!selected || Array.isArray(selected)) {
        return;
      }
      const bytes = await readFile(selected);
      await api.importWorld(bytes);
      setWorlds(await api.listWorlds());
      setStatus(t("worldImported"));
    } catch (err) {
      setSettingsError(String(err));
    } finally {
      setImporting(false);
    }
  }

  const selectedLanguageLabel =
    APP_LANGUAGE_OPTIONS.find((option) => option.value === appLanguage)?.label ?? APP_LANGUAGE_OPTIONS[0].label;
  const translationLanguageOptions = supportedTranslationLanguagesForSource(activeWorld?.target_language);
  const usageRatio = Math.max(0, Math.min(1, quota?.usage_ratio ?? 0));
  const quotaPercent = Math.round(usageRatio * 100);
  const identityLabel = useMemo(() => {
    if (officialAccount?.registered) {
      return officialAccount.masked_phone ?? t("registeredAccount");
    }
    return officialAccount?.android_id ? t("trialDeviceReady") : t("trialDeviceUnknown");
  }, [officialAccount, t]);

  return (
    <div className="settings-form official-panel">
      <section className="official-card account-card">
        <div className="official-card-header">
          <div>
            <span>{t("officialTrial")}</span>
            <strong>{identityLabel}</strong>
          </div>
          {officialAccount?.registered ? <BadgeCheck size={22} /> : <Smartphone size={22} />}
        </div>
        <p>{officialAccount?.registered ? t("registeredAccountCopy") : t("trialAccountCopy")}</p>
        {officialAccount?.invite_code ? (
          <div className="invite-chip">
            <Ticket size={14} />
            <span>{officialAccount.invite_code}</span>
          </div>
        ) : null}
      </section>

      <section className="official-card quota-card">
        <div className="official-card-header">
          <div>
            <span>{t("quota")}</span>
            <strong>{quota ? `${quotaPercent}%` : t("quotaNotLoaded")}</strong>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={() => void refreshQuota()}
            disabled={quotaLoading}
            aria-label={t("refreshQuota")}
            title={t("refreshQuota")}
          >
            <RefreshCw size={17} />
          </button>
        </div>
        <div className="quota-meter" aria-hidden="true">
          <span style={{ width: `${quotaPercent}%` }} />
        </div>
        <div className="quota-grid">
          <span>{t("usedTokens")}</span>
          <strong>{formatTokens(quota?.used_tokens)}</strong>
          <span>{t("dailyLimit")}</span>
          <strong>{formatTokens(quota?.daily_limit)}</strong>
          {quota?.pool_balance != null ? (
            <>
              <span>{t("poolBalance")}</span>
              <strong>{formatTokens(quota.pool_balance)}</strong>
            </>
          ) : null}
        </div>
      </section>

      {!officialAccount?.registered ? (
        <section className="official-card register-card">
          <div className="official-card-header">
            <div>
              <span>{t("phoneRegistration")}</span>
              <strong>{detectedPhone?.masked_phone ?? t("verifyPhoneOwner")}</strong>
            </div>
            <ShieldCheck size={22} />
          </div>
          <p>{t("phoneRegistrationCopy")}</p>
          <button className="command-button" type="button" onClick={() => void detectPhone()} disabled={detectingPhone}>
            <Smartphone size={16} />
            {detectingPhone ? t("detectingPhone") : t("detectPhone")}
          </button>
          {detectedPhone ? (
            <form className="register-form" onSubmit={register}>
              <label>
                {t("middleFour")}
                <input
                  inputMode="numeric"
                  maxLength={4}
                  name="middle_four"
                  value={middleFour}
                  onChange={(event) => setMiddleFour(event.target.value.replace(/\D/g, "").slice(0, 4))}
                  placeholder="1234"
                />
              </label>
              <label>
                {t("inviteCode")}
                <input
                  name="invite_code"
                  value={inviteCode}
                  onChange={(event) => setInviteCode(event.target.value)}
                  placeholder={t("inviteCodeOptional")}
                />
              </label>
              <button className="primary-button" disabled={registering || middleFour.length !== 4}>
                <ShieldCheck size={16} />
                {registering ? t("registering") : t("register")}
              </button>
            </form>
          ) : null}
        </section>
      ) : null}

      <label>
        {t("appLanguage")}
        <Dropdown
          value={selectedLanguageLabel}
          options={APP_LANGUAGE_OPTIONS.map((option) => option.label)}
          onChange={(label) => {
            const option = APP_LANGUAGE_OPTIONS.find((item) => item.label === label);
            if (option) {
              setAppLanguage(option.value as AppLanguage);
            }
          }}
          placeholder={t("appLanguage")}
          allowFreeText={false}
        />
      </label>
      <label>
        {t("translationLanguage")}
        <Dropdown
          value={translationLanguage}
          options={translationLanguageOptions}
          onChange={(language) => {
            if (isTranslationLanguage(language)) {
              setTranslationLanguage(language);
            }
          }}
          placeholder={t("translationLanguage")}
          allowFreeText={false}
        />
      </label>
      <button
        className={quickMode ? "quick-mode-toggle active" : "quick-mode-toggle"}
        type="button"
        onClick={() => setQuickMode(!quickMode)}
        aria-pressed={quickMode}
      >
        <Zap size={16} />
        {t("quickMode")}
      </button>
      <button className="command-button" type="button" onClick={() => void importWorld()} disabled={importing}>
        <Download size={16} />
        {importing ? t("importing") : t("importWorld")}
      </button>
      {status ? <p className="settings-status">{status}</p> : null}
    </div>
  );
}
