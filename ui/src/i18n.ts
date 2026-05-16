// Minimal dict-based i18n. Loaded synchronously, no async fetch — the strings
// are tiny and adding `i18next` would balloon the bundle for no real win.
// Falls back to `nl` when a key isn't translated yet.

import { create } from "zustand";

export type Locale = "nl" | "en" | "sw";
export const LOCALES: Locale[] = ["nl", "en", "sw"];

type Dict = Record<string, string>;

const messages: Record<Locale, Dict> = {
  nl: {
    loading: "Engine laden…",
    toMove: "Aan zet",
    phase: "Fase",
    ply: "Ply",
    winsBanner: "{player} wint!",
    newKiswahili: "Nieuw Kiswahili",
    newKujifunza: "Nieuw Kujifunza",
    share: "Deel link",
    copied: "Gekopieerd!",
    chooseKichwa: "Kies kichwa voor capture-sow:",
    kichwaLeft: "← Links",
    kichwaRight: "Rechts →",
    safariPrompt: "Safari — eigen nyumba leegmaken?",
    safariYes: "Plunder & ga door",
    safariNo: "Stop",
    directionPrompt: "Pit {pit} ({player}) — richting?",
    directionCw: "↻ Met de klok mee",
    directionCcw: "↺ Tegen de klok in",
    cancel: "Annuleer",
    moves: "Zetten",
    noMoves: "— nog geen zetten —",
    materialBalance: "Materiaal-balans",
    mbeleLabel: "mbele {south}/{north}",
    engineVersion: "engine v{ver}",
    south: "Zuid",
    north: "Noord",
    soundOn: "Geluid aan",
    soundOff: "Geluid uit",
    locale: "Taal",
    eventNamuPlace: "Speler {player} plaatst kete uit ghala",
    eventCapture: "Capture op pit {pit}",
    eventGameOver: "Spel afgelopen — {player} wint",
  },
  en: {
    loading: "Loading engine…",
    toMove: "To move",
    phase: "Phase",
    ply: "Ply",
    winsBanner: "{player} wins!",
    newKiswahili: "New Kiswahili",
    newKujifunza: "New Kujifunza",
    share: "Share link",
    copied: "Copied!",
    chooseKichwa: "Choose kichwa for capture-sow:",
    kichwaLeft: "← Left",
    kichwaRight: "Right →",
    safariPrompt: "Safari — empty your own nyumba?",
    safariYes: "Plunder & continue",
    safariNo: "Stop",
    directionPrompt: "Pit {pit} ({player}) — direction?",
    directionCw: "↻ Clockwise",
    directionCcw: "↺ Counter-clockwise",
    cancel: "Cancel",
    moves: "Moves",
    noMoves: "— no moves yet —",
    materialBalance: "Material balance",
    mbeleLabel: "mbele {south}/{north}",
    engineVersion: "engine v{ver}",
    south: "South",
    north: "North",
    soundOn: "Sound on",
    soundOff: "Sound off",
    locale: "Language",
    eventNamuPlace: "Player {player} places kete from ghala",
    eventCapture: "Capture at pit {pit}",
    eventGameOver: "Game over — {player} wins",
  },
  sw: {
    loading: "Inapakia engine…",
    toMove: "Zamu",
    phase: "Hatua",
    ply: "Hamla",
    winsBanner: "{player} ameshinda!",
    newKiswahili: "Mchezo Mpya Kiswahili",
    newKujifunza: "Mchezo Mpya Kujifunza",
    share: "Shiriki",
    copied: "Imenakiliwa!",
    chooseKichwa: "Chagua kichwa kwa kula-sow:",
    kichwaLeft: "← Kushoto",
    kichwaRight: "Kulia →",
    safariPrompt: "Safari — toa nyumba yako?",
    safariYes: "Endelea",
    safariNo: "Simama",
    directionPrompt: "Shimo {pit} ({player}) — mwelekeo?",
    directionCw: "↻ Kulia",
    directionCcw: "↺ Kushoto",
    cancel: "Ghairi",
    moves: "Hatua",
    noMoves: "— bado hakuna hatua —",
    materialBalance: "Hesabu ya kete",
    mbeleLabel: "mbele {south}/{north}",
    engineVersion: "engine v{ver}",
    south: "Kusini",
    north: "Kaskazini",
    soundOn: "Sauti washa",
    soundOff: "Sauti zima",
    locale: "Lugha",
    eventNamuPlace: "Mchezaji {player} aweka kete kutoka ghala",
    eventCapture: "Kula kwenye shimo {pit}",
    eventGameOver: "Mchezo umeisha — {player} ameshinda",
  },
};

function interp(template: string, vars: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, k) =>
    Object.prototype.hasOwnProperty.call(vars, k) ? String(vars[k]) : `{${k}}`,
  );
}

type I18nStore = {
  locale: Locale;
  setLocale: (l: Locale) => void;
};

const STORAGE_KEY = "bao.locale";

function initialLocale(): Locale {
  if (typeof window === "undefined") return "nl";
  const saved = window.localStorage.getItem(STORAGE_KEY) as Locale | null;
  if (saved && LOCALES.includes(saved)) return saved;
  // Fall back to browser preference if it matches one of our locales.
  const nav = window.navigator.language.slice(0, 2).toLowerCase();
  if (LOCALES.includes(nav as Locale)) return nav as Locale;
  return "nl";
}

export const useI18n = create<I18nStore>((set) => ({
  locale: initialLocale(),
  setLocale: (l) => {
    set({ locale: l });
    try {
      window.localStorage.setItem(STORAGE_KEY, l);
    } catch {
      /* ignore */
    }
  },
}));

export function t(key: string, vars: Record<string, string | number> = {}): string {
  const locale = useI18n.getState().locale;
  const template = messages[locale][key] ?? messages.nl[key] ?? key;
  return interp(template, vars);
}

/** Reactive variant — re-renders the caller when the locale changes. */
export function useT(): (k: string, v?: Record<string, string | number>) => string {
  const locale = useI18n((s) => s.locale);
  return (k, v = {}) => {
    const template = messages[locale][k] ?? messages.nl[k] ?? k;
    return interp(template, v);
  };
}
