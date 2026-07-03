import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zh from "./locales/zh";
import en from "./locales/en";

let initialized = false;

export function initI18n(): Promise<void> {
  if (initialized) return Promise.resolve();
  initialized = true;
  return i18n
    .use(initReactI18next)
    .init({
      resources: { zh: { translation: zh }, en: { translation: en } },
      lng: "zh",
      fallbackLng: "zh",
      interpolation: { escapeValue: false },
    })
    .then(() => undefined);
}

export default i18n;
