import { invoke } from "@tauri-apps/api/core";

export type Reaction = { from: string; kind: string };

export type Message = {
  id: string;
  chat_id: string;
  from_me: boolean;
  sender: string;
  text: string;
  ts: number;
  status: string;
  read_ts: number | null;
  reactions: Reaction[];
  edited: boolean;
  reply_to: string | null;
};

export type Chat = {
  id: string;
  name: string;
  handles: string[];
  service: string;
  pinned: boolean;
  unread: number;
  last_text: string;
  last_ts: number;
  demo: boolean;
};

export type Account = { state: string; apple_id: string; handles: string[]; detail: string };

export type TrustedPhone = { id: number; display: string };
export type SignInResult = { kind: "NeedsTwoFactor"; phones: TrustedPhone[] } | { kind: "LoggedIn" };

export type Track = {
  uri: string;
  kind: string;
  name: string;
  artist: string;
  album: string;
  image: string;
  url: string;
};

export type NowPlaying = { track: Track | null; is_playing: boolean; context: string };

export const api = {
  boot: () => invoke<{ account: Account; chats: Chat[]; spotify: boolean }>("boot"),
  chats: () => invoke<Chat[]>("chats"),
  messages: (chatId: string) => invoke<Message[]>("messages", { chatId }),
  markRead: (chatId: string) => invoke<void>("mark_read", { chatId }),
  setPinned: (chatId: string, pinned: boolean) => invoke<void>("set_pinned", { chatId, pinned }),
  deleteChat: (chatId: string) => invoke<void>("delete_chat", { chatId }),
  newChat: (to: string) => invoke<string>("new_chat", { to }),
  send: (chatId: string, text: string, replyTo?: string | null, sendAt?: number | null) =>
    invoke<Message>("send", { chatId, text, replyTo: replyTo ?? null, sendAt: sendAt ?? null }),
  react: (messageId: string, kind: string) => invoke<Message>("react", { messageId, kind }),
  edit: (messageId: string, text: string) => invoke<Message>("edit_message", { messageId, text }),
  unsend: (messageId: string) => invoke<void>("unsend", { messageId }),
  signIn: (appleId: string, password: string) => invoke<SignInResult>("sign_in", { appleId, password }),
  useDevice: () => invoke<void>("use_device"),
  useSms: (phoneId: number) => invoke<void>("use_sms", { phoneId }),
  submitCode: (code: string) => invoke<Account>("submit_code", { code }),
  spotifyNow: () => invoke<NowPlaying>("spotify_now"),
  spotifySearch: (q: string) => invoke<Track[]>("spotify_search", { q }),
  spotifyLookup: (text: string) => invoke<Track | null>("spotify_lookup", { text }),
  spotifyPlay: (uri?: string | null) => invoke<void>("spotify_play", { uri: uri ?? null }),
  spotifyPause: () => invoke<void>("spotify_pause"),
  spotifyQueue: (uri: string) => invoke<void>("spotify_queue", { uri }),
};

export const spotifyRe =
  /(https?:\/\/open\.spotify\.com\/(?:intl-[a-z]{2}\/)?(?:track|album|playlist|artist|episode|show)\/[A-Za-z0-9]{22}[^\s]*)|(spotify:(?:track|album|playlist|artist|episode|show):[A-Za-z0-9]{22})/;

export function spotifyLink(text: string): string | null {
  const m = text.match(spotifyRe);
  return m ? m[0] : null;
}

const lookupCache = new Map<string, Promise<Track | null>>();

export function lookupTrack(link: string) {
  let p = lookupCache.get(link);
  if (!p) {
    p = api.spotifyLookup(link).catch(() => null);
    lookupCache.set(link, p);
  }
  return p;
}
