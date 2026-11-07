import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, Account, Chat, Message } from "./api";
import { Sidebar } from "./Sidebar";
import { Conversation } from "./Conversation";
import { Avatar } from "./Bits";
import { cx } from "./util";
import "./styles.css";

export default function App() {
  const [chats, setChats] = useState<Chat[]>([]);
  const [account, setAccount] = useState<Account>({ state: "signed_out", apple_id: "", handles: [], detail: "" });
  const [spotify, setSpotify] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [composing, setComposing] = useState(false);
  const [query, setQuery] = useState("");
  const [toast, setToast] = useState("");
  const [signIn, setSignIn] = useState(false);
  const [palette, setPalette] = useState(false);
  const [focused, setFocused] = useState(true);
  const [maxed, setMaxed] = useState(false);
  const selRef = useRef<string | null>(null);
  const inject = useRef<((s: string) => void) | null>(null);
  selRef.current = selected;

  const onToast = useCallback((s: string) => {
    setToast(s);
    setTimeout(() => setToast((t) => (t === s ? "" : t)), 3200);
  }, []);

  const refreshChats = useCallback(() => api.chats().then(setChats), []);

  useEffect(() => {
    api.boot().then((b) => {
      setChats(b.chats);
      setAccount(b.account);
      setSpotify(b.spotify);
      if (b.chats[0]) setSelected(b.chats[0].id);
    });
    const un: Promise<() => void>[] = [
      listen("chats", () => refreshChats()),
      listen<Account>("account", (e) => setAccount(e.payload)),
      listen<Message>("message", (e) => {
        const m = e.payload;
        if (m.chat_id !== selRef.current) return;
        setMessages((ms) => {
          const i = ms.findIndex((x) => x.id === m.id);
          if (i === -1) return [...ms, m].sort((a, b) => a.ts - b.ts);
          const c = ms.slice();
          c[i] = m;
          return c;
        });
      }),
      listen<[string, string]>("removed", (e) => {
        const [chat, id] = e.payload;
        if (chat === selRef.current) setMessages((ms) => ms.filter((m) => m.id !== id));
      }),
    ];
    const w = getCurrentWindow();
    const f = w.onFocusChanged(({ payload }) => setFocused(payload));
    const r = w.onResized(() => w.isMaximized().then(setMaxed));
    return () => {
      un.forEach((p) => p.then((u) => u()));
      f.then((u) => u());
      r.then((u) => u());
    };
  }, [refreshChats]);

  useEffect(() => {
    if (!selected) {
      setMessages([]);
      return;
    }
    api.messages(selected).then(setMessages);
    api.markRead(selected);
  }, [selected]);

  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setComposing(true);
      }
      if (e.ctrlKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPalette(true);
      }
      if (e.ctrlKey && /^[1-9]$/.test(e.key)) {
        const c = chats[Number(e.key) - 1];
        if (c) {
          e.preventDefault();
          setComposing(false);
          setSelected(c.id);
        }
      }
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [chats]);

  useEffect(() => {
    const block = (e: MouseEvent) => {
      if (!(e.target as HTMLElement).closest("textarea,input")) e.preventDefault();
    };
    window.addEventListener("contextmenu", block);
    return () => window.removeEventListener("contextmenu", block);
  }, []);

  const chat = chats.find((c) => c.id === selected) ?? null;

  const startChat = async (to: string) => {
    try {
      const id = await api.newChat(to);
      await refreshChats();
      setComposing(false);
      setSelected(id);
    } catch (e) {
      onToast(String(e));
    }
  };

  return (
    <div className={cx("app", !focused && "blurred", maxed && "maximized")}>
      <Sidebar
        chats={chats}
        selected={composing ? null : selected}
        account={account}
        spotify={spotify}
        onSelect={(id) => {
          setComposing(false);
          setSelected(id);
        }}
        onCompose={() => setComposing(true)}
        onSignIn={() => setSignIn(true)}
        onShare={(u) => inject.current?.(u)}
        onToast={onToast}
        query={query}
        setQuery={setQuery}
      />
      <Conversation chat={chat} composing={composing} onStartChat={startChat} onToast={onToast} messages={messages} injectRef={inject} />
      {signIn && <SignIn onClose={() => setSignIn(false)} onDone={(a) => setAccount(a)} />}
      {palette && (
        <Palette
          chats={chats}
          onPick={(id) => {
            setPalette(false);
            setComposing(false);
            setSelected(id);
          }}
          onClose={() => setPalette(false)}
        />
      )}
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

function SignIn({ onClose, onDone }: { onClose: () => void; onDone: (a: Account) => void }) {
  const [step, setStep] = useState<"creds" | "pick" | "code">("creds");
  const [id, setId] = useState("");
  const [pw, setPw] = useState("");
  const [code, setCode] = useState("");
  const [phones, setPhones] = useState<{ id: number; display: string }[]>([]);
  const [via, setVia] = useState<"device" | "sms">("device");
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  const creds = async () => {
    setBusy(true);
    setErr("");
    try {
      const r = await api.signIn(id, pw);
      if (r.kind === "LoggedIn") {
        onDone({ state: "signed_in", apple_id: id, handles: [], detail: "" });
        onClose();
      } else {
        setPhones(r.phones);
        setStep("pick");
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const chooseDevice = async () => {
    setBusy(true);
    setErr("");
    try {
      await api.useDevice();
      setVia("device");
      setStep("code");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const pickPhone = async (phoneId: number) => {
    setBusy(true);
    setErr("");
    try {
      await api.useSms(phoneId);
      setVia("sms");
      setStep("code");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const verify = async () => {
    setBusy(true);
    setErr("");
    try {
      const a = await api.submitCode(code);
      onDone(a);
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="sheet-bg" onMouseDown={onClose}>
      <div className="sheet" onMouseDown={(e) => e.stopPropagation()}>
        <div className="bigic">
          <svg width="38" height="38" viewBox="0 0 24 24" fill="#fff">
            <path d="M12 3C6.5 3 2 6.8 2 11.4c0 2.6 1.4 4.9 3.7 6.4-.2 1.1-.8 2.4-1.9 3.3 2.2-.1 4-1 5.1-1.9 1 .2 2 .3 3.1.3 5.5 0 10-3.8 10-8.5S17.5 3 12 3z" />
          </svg>
        </div>
        {step === "creds" && (
          <>
            <h3>Sign in to iMessage</h3>
            <p>Use your Apple Account. Your messages are end-to-end encrypted between you and the people you text.</p>
            <input placeholder="Email or Phone Number" value={id} onChange={(e) => setId(e.target.value)} autoFocus />
            <input placeholder="Password" type="password" value={pw} onChange={(e) => setPw(e.target.value)} onKeyDown={(e) => e.key === "Enter" && creds()} />
          </>
        )}
        {step === "pick" && (
          <>
            <h3>Where should the code go?</h3>
            <p>Send the verification code to your Apple devices, or text it to a trusted number.</p>
            <div className="opts">
              <button className="btn primary" style={{ width: "100%", marginBottom: 6 }} disabled={busy} onClick={chooseDevice}>
                📱 My Apple devices
              </button>
              {phones.map((p) => (
                <button key={p.id} className="btn" style={{ width: "100%", marginBottom: 6 }} disabled={busy} onClick={() => pickPhone(p.id)}>
                  Text {p.display}
                </button>
              ))}
            </div>
          </>
        )}
        {step === "code" && (
          <>
            <h3>Two-Factor Authentication</h3>
            <p>{via === "sms" ? "Enter the code we texted you." : "Enter the code shown on your trusted Apple devices."}</p>
            <input placeholder="6-digit code" value={code} maxLength={6} onChange={(e) => setCode(e.target.value.replace(/\D/g, ""))} onKeyDown={(e) => e.key === "Enter" && verify()} autoFocus />
          </>
        )}
        {err && <div className="err">{err}</div>}
        <div className="btns">
          <button className="btn" onClick={onClose}>
            Cancel
          </button>
          {step === "creds" && (
            <button className="btn primary" disabled={busy || !id || !pw} onClick={creds}>
              {busy ? "Signing In…" : "Sign In"}
            </button>
          )}
          {step === "code" && (
            <button className="btn primary" disabled={busy || code.length < 6} onClick={verify}>
              {busy ? "Registering…" : "Verify"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function Palette({ chats, onPick, onClose }: { chats: Chat[]; onPick: (id: string) => void; onClose: () => void }) {
  const [q, setQ] = useState("");
  const [i, setI] = useState(0);
  const list = chats.filter((c) => c.name.toLowerCase().includes(q.toLowerCase()) || c.handles.some((h) => h.includes(q.toLowerCase()))).slice(0, 8);
  return (
    <div className="sheet-bg" style={{ background: "transparent", placeItems: "start center" }} onMouseDown={onClose}>
      <div className="palette" onMouseDown={(e) => e.stopPropagation()}>
        <input
          autoFocus
          placeholder="Jump to conversation…"
          value={q}
          onChange={(e) => {
            setQ(e.target.value);
            setI(0);
          }}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") setI((x) => Math.min(list.length - 1, x + 1));
            if (e.key === "ArrowUp") setI((x) => Math.max(0, x - 1));
            if (e.key === "Enter" && list[i]) onPick(list[i].id);
            if (e.key === "Escape") onClose();
          }}
        />
        <div className="res">
          {list.map((c, j) => (
            <div key={c.id} className={cx("pr", j === i && "on")} onMouseEnter={() => setI(j)} onClick={() => onPick(c.id)}>
              <Avatar name={c.name} size={26} />
              <div style={{ minWidth: 0 }}>
                <div>{c.name}</div>
                <div className="sn">{c.last_text}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
