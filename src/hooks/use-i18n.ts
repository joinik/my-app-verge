import { useTranslation } from "react-i18next";
import { useVerge } from "./use-verge";
import { useCallback, useState } from "react";
import {
  resolveLanguage,
  supportedLanguages,
  changeLanguage,
} from "@/services/i8n";

export const useI18n = () => {
  const { i18n, t } = useTranslation();
  const { patchVerge } = useVerge();
  const [isLoading, setIsLoading] = useState(false);
  const switchLanguage = useCallback(
    async (language: string) => {
      const targetLanguage = resolveLanguage(language);
      if (!supportedLanguages.includes(targetLanguage)) {
        console.warn(`Language ${targetLanguage} is not supported`);
        return;
      }
      if (i18n.language === targetLanguage) {
        return;
      }
      setIsLoading(true);
      try {
        await changeLanguage(targetLanguage);
        if (patchVerge) {
          await patchVerge({
            language: targetLanguage,
          });
        }
      } catch (err) {
        console.error("Failed to change language:", err);
      } finally {
        setIsLoading(false);
      }
    },
    [i18n.language, patchVerge],
  );

  return {
    currentLanguage: i18n.language,
    switchLanguage,
    supportedLanguages,
    isLoading,
    t,
  };
};
