// Save / load helpers. Two layers:
// 1. `location.hash` carries a base64 of the packed engine bytes — sharing the
//    URL is enough to share the position.
// 2. localStorage mirrors the same bytes so a reload without a hash still
//    restores the user's last game.

const STORAGE_KEY = "bao.state";
const HASH_PREFIX = "#s=";

function b64encode(bytes: Uint8Array): string {
  let s = "";
  for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
  // URL-safe: replace +,/ and strip padding.
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function b64decode(s: string): Uint8Array {
  const padded = s.replace(/-/g, "+").replace(/_/g, "/");
  const pad = padded.length % 4 === 0 ? "" : "=".repeat(4 - (padded.length % 4));
  const bin = atob(padded + pad);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export function readPersistedState(): Uint8Array | null {
  if (typeof window === "undefined") return null;
  // Hash takes precedence — shared link should override saved.
  const h = window.location.hash;
  if (h.startsWith(HASH_PREFIX)) {
    try {
      return b64decode(h.slice(HASH_PREFIX.length));
    } catch {
      /* fall through to localStorage */
    }
  }
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored) return b64decode(stored);
  } catch {
    /* ignore */
  }
  return null;
}

export function persistState(bytes: Uint8Array): void {
  if (typeof window === "undefined") return;
  const encoded = b64encode(bytes);
  // Update hash silently (no scroll).
  try {
    window.history.replaceState(null, "", `${HASH_PREFIX}${encoded}`);
  } catch {
    window.location.hash = `${HASH_PREFIX}${encoded}`;
  }
  try {
    window.localStorage.setItem(STORAGE_KEY, encoded);
  } catch {
    /* quota — ignore */
  }
}

/** Returns the share URL for the current state. Pure function over `bytes`. */
export function shareUrl(bytes: Uint8Array): string {
  if (typeof window === "undefined") return "";
  const base = `${window.location.origin}${window.location.pathname}`;
  return `${base}${HASH_PREFIX}${b64encode(bytes)}`;
}
