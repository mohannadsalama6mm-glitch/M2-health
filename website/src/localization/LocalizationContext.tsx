import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  directionFor,
  formatCurrencyMinor,
  formatDate,
  formatDateTime,
  formatNumber,
  isLocale,
  localeStorageKey,
  localizeSemanticState,
  translate,
  type Direction,
  type Locale,
  type MessageKey,
} from "../../../localization";

type Values = Record<string, string | number>;
interface LocalizationValue {
  locale: Locale;
  direction: Direction;
  setLocale(locale: Locale): void;
  t(key: MessageKey, values?: Values): string;
  number(value: number, options?: Intl.NumberFormatOptions): string;
  currencyMinor(value: number, currency?: string): string;
  date(value: string | number | Date): string;
  dateTime(value: string | number | Date): string;
  state(value: string): string;
}

function initialLocale(): Locale {
  try {
    const saved = window.localStorage.getItem(localeStorageKey);
    if (isLocale(saved)) return saved;
  } catch {
    // Storage may be unavailable in privacy-restricted browser contexts.
  }
  return "en";
}

const fallback: LocalizationValue = {
  locale: "en",
  direction: "ltr",
  setLocale: () => undefined,
  t: (key, values) => translate("en", key, values),
  number: (value, options) => formatNumber("en", value, options),
  currencyMinor: (value, currency) => formatCurrencyMinor("en", value, currency),
  date: (value) => formatDate("en", value),
  dateTime: (value) => formatDateTime("en", value),
  state: (value) => localizeSemanticState("en", value),
};
const Context = createContext<LocalizationValue>(fallback);

export function LocalizationProvider({ children }: { children: ReactNode }) {
  const [locale, updateLocale] = useState<Locale>(initialLocale);
  const direction = directionFor(locale);
  useEffect(() => {
    document.documentElement.lang = locale;
    document.documentElement.dir = direction;
    try {
      window.localStorage.setItem(localeStorageKey, locale);
    } catch {
      // Keep the active in-memory preference when persistence is unavailable.
    }
  }, [direction, locale]);
  const setLocale = useCallback((next: Locale) => updateLocale(next), []);
  const value = useMemo<LocalizationValue>(
    () => ({
      locale,
      direction,
      setLocale,
      t: (key, values) => translate(locale, key, values),
      number: (numberValue, options) => formatNumber(locale, numberValue, options),
      currencyMinor: (minor, currency) => formatCurrencyMinor(locale, minor, currency),
      date: (dateValue) => formatDate(locale, dateValue),
      dateTime: (dateValue) => formatDateTime(locale, dateValue),
      state: (stateValue) => localizeSemanticState(locale, stateValue),
    }),
    [direction, locale, setLocale],
  );
  return <Context.Provider value={value}>{children}</Context.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components
export function useLocalization() {
  return useContext(Context);
}
