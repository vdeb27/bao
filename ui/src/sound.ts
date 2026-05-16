// Procedural Web Audio: tiny percussive beeps for sow / capture / win / error.
// Picking synthesis over sample-loading avoids a separate asset bundle for a
// handful of sub-second tones. AudioContext is lazily created on first user
// gesture (browsers block autoplay otherwise).

import { create } from "zustand";

type EventKind = "sow" | "capture" | "win" | "error" | "click";

let ctx: AudioContext | null = null;

function ensure(): AudioContext | null {
  if (ctx) return ctx;
  if (typeof window === "undefined") return null;
  try {
    ctx = new (window.AudioContext ||
      (window as unknown as { webkitAudioContext: typeof AudioContext })
        .webkitAudioContext)();
  } catch {
    return null;
  }
  return ctx;
}

function beep(freq: number, durMs: number, type: OscillatorType, gain = 0.12) {
  const ac = ensure();
  if (!ac) return;
  const osc = ac.createOscillator();
  const g = ac.createGain();
  osc.type = type;
  osc.frequency.value = freq;
  const now = ac.currentTime;
  g.gain.setValueAtTime(0, now);
  g.gain.linearRampToValueAtTime(gain, now + 0.005);
  g.gain.exponentialRampToValueAtTime(0.0001, now + durMs / 1000);
  osc.connect(g).connect(ac.destination);
  osc.start(now);
  osc.stop(now + durMs / 1000 + 0.02);
}

const SOUNDS: Record<EventKind, () => void> = {
  sow: () => beep(620, 65, "triangle", 0.08),
  capture: () => {
    beep(220, 140, "sawtooth", 0.14);
    setTimeout(() => beep(165, 180, "sawtooth", 0.12), 70);
  },
  win: () => {
    beep(523, 120, "sine", 0.16);
    setTimeout(() => beep(659, 120, "sine", 0.16), 110);
    setTimeout(() => beep(784, 280, "sine", 0.18), 220);
  },
  error: () => beep(180, 140, "square", 0.1),
  click: () => beep(880, 35, "sine", 0.05),
};

type SoundStore = {
  enabled: boolean;
  toggle: () => void;
  setEnabled: (v: boolean) => void;
  play: (kind: EventKind) => void;
};

const STORAGE_KEY = "bao.sound";

function initialEnabled(): boolean {
  if (typeof window === "undefined") return true;
  return window.localStorage.getItem(STORAGE_KEY) !== "0";
}

export const useSound = create<SoundStore>((set, get) => ({
  enabled: initialEnabled(),
  toggle: () => {
    const next = !get().enabled;
    set({ enabled: next });
    try {
      window.localStorage.setItem(STORAGE_KEY, next ? "1" : "0");
    } catch {
      /* ignore */
    }
  },
  setEnabled: (v) => {
    set({ enabled: v });
    try {
      window.localStorage.setItem(STORAGE_KEY, v ? "1" : "0");
    } catch {
      /* ignore */
    }
  },
  play: (kind) => {
    if (!get().enabled) return;
    SOUNDS[kind]();
  },
}));
