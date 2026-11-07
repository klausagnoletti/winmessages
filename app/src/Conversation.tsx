import { Fragment, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, Chat, Message, spotifyLink, spotifyRe } from "./api";
import { Menu, MenuItem, TapbackGlyph, TapbackPicker, useOutside } from "./Bits";
import { Clock, Info, Photo, Plus, Reply, SendArrow, Smile, SpotifyLogo, Video, Wave } from "./Icons";
import { SpotifyCard, SpotifyPanel } from "./Spotify";
import { SmoothInput } from "./SmoothInput";
import { clock, cx, onlyEmoji, separatorParts } from "./util";

const HOUR = 3_600_000;
const GROUP = 60_000;

export function Conversation(props: {
  chat: Chat | null;
  composing: boolean;
  onStartChat: (to: string) => void;
  onToast: (s: string) => void;
  messages: Message[];
  injectRef: React.MutableRefObject<((s: string) => void) | null>;
}) {
  const { chat, messages } = props;
  const [to, setTo] = useState("");
  const toRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (props.composing) {
      setTo("");
      setTimeout(() => toRef.current?.focus(), 0);
    }
  }, [props.composing]);

  return (
    <section className="pane">
      <div className="toolbar drag" data-tauri-drag-region onDoubleClick={() => getCurrentWindow().toggleMaximize()}>
        <div className="to" data-tauri-drag-region>
          {props.composing ? (
            <>
              <span className="label">To:</span>
              <input
                ref={toRef}
                value={to}
                placeholder=""
                onChange={(e) => setTo(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && to.trim()) props.onStartChat(to.trim());
                }}
              />
            </>
          ) : chat ? (
            <>
              <span className="label">To:</span>
              <span className="who" title={chat.handles.join(", ")}>
                {chat.name}
              </span>
            </>
          ) : null}
        </div>
        {chat && !props.composing && (
          <>
            <button className="icon-btn" title="FaceTime" onClick={() => props.onToast("FaceTime comes after messaging works")}>
              <Video size={21} />
            </button>
            <button className="icon-btn" title="Details" onClick={() => props.onToast(`${chat.name} · ${chat.handles.join(", ")} · ${chat.service}`)}>
              <Info size={18} />
            </button>
          </>
        )}
      </div>
      {chat && !props.composing ? (
        <>
          <Thread chat={chat} messages={messages} onToast={props.onToast} injectRef={props.injectRef} />
        </>
      ) : (
        <div className="empty-pane">{props.composing ? "" : "No Conversation Selected"}</div>
      )}
    </section>
  );
}

function Thread({
  chat,
  messages,
  onToast,
  injectRef,
}: {
  chat: Chat;
  messages: Message[];
  onToast: (s: string) => void;
  injectRef: React.MutableRefObject<((s: string) => void) | null>;
}) {
  const scroller = useRef<HTMLDivElement>(null);
  const [picker, setPicker] = useState<{ x: number; y: number; m: Message } | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; m: Message } | null>(null);
  const [replyTo, setReplyTo] = useState<Message | null>(null);
  const [editing, setEditing] = useState<Message | null>(null);
  const sms = chat.service === "SMS";
  const byId = useMemo(() => new Map(messages.map((m) => [m.id, m])), [messages]);
  const lastMine = useMemo(() => [...messages].reverse().find((m) => m.from_me && m.status !== "scheduled"), [messages]);
  const known = useRef<Set<string>>(new Set());
  const settled = useRef(false);

  useLayoutEffect(() => {
    const el = scroller.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages.length, chat.id]);

  useEffect(() => {
    known.current = new Set();
    settled.current = false;
  }, [chat.id]);

  useEffect(() => {
    messages.forEach((m) => known.current.add(m.id));
    settled.current = true;
  }, [messages]);

  const isNew = (id: string) => settled.current && !known.current.has(id);

  useEffect(() => {
    setReplyTo(null);
    setEditing(null);
  }, [chat.id]);

  const pick = (m: Message, k: string) => {
    api.react(m.id, k).catch((e) => onToast(String(e)));
    setPicker(null);
  };

  const menuItems = (m: Message): MenuItem[] => {
    const link = spotifyLink(m.text);
    const age = Date.now() - m.ts;
    const out: MenuItem[] = [
      { label: "Tapback…", icon: <TapbackGlyph kind="love" size={12} />, onClick: () => setPicker({ x: menu!.x, y: menu!.y, m }) },
      { label: "Reply", icon: <Reply size={13} />, onClick: () => setReplyTo(m) },
      { label: "Copy", onClick: () => navigator.clipboard.writeText(m.text) },
    ];
    if (link) {
      out.push("sep");
      out.push({ label: "Play on Spotify", icon: <SpotifyLogo size={12} />, onClick: () => api.spotifyLookup(link).then((t) => t && api.spotifyPlay(t.uri)) });
      out.push({ label: "Add to Spotify Queue", onClick: () => api.spotifyLookup(link).then((t) => t && api.spotifyQueue(t.uri)).then(() => onToast("Added to queue")) });
    }
    if (m.from_me) {
      out.push("sep");
      if (m.status === "scheduled") {
        out.push({ label: "Delete Scheduled Message", danger: true, onClick: () => api.unsend(m.id) });
      } else {
        if (age < 15 * 60_000 && !link) out.push({ label: "Edit", onClick: () => setEditing(m) });
        if (age < 2 * 60_000) out.push({ label: "Undo Send", danger: true, onClick: () => api.unsend(m.id) });
      }
    }
    return out;
  };

  const rows: React.ReactNode[] = [];
  rows.push(
    <div className="thread-top" key="top">
      {sms ? "Text Message · SMS" : "iMessage"}
    </div>,
  );
  messages.forEach((m, i) => {
    const prev = messages[i - 1];
    const next = messages[i + 1];
    const newSep = !prev || m.ts - prev.ts > HOUR;
    const startGroup = newSep || !prev || prev.from_me !== m.from_me || prev.sender !== m.sender || m.ts - prev.ts > GROUP;
    const endGroup = !next || next.from_me !== m.from_me || next.sender !== m.sender || next.ts - m.ts > GROUP || next.ts - m.ts > HOUR;
    if (newSep) {
      const [d, t] = separatorParts(m.ts);
      rows.push(
        <div className="sep" key={"s" + m.id}>
          <b>{d}</b> {t}
        </div>,
      );
    }
    const link = spotifyLink(m.text);
    const rest = link ? m.text.replace(spotifyRe, "").trim() : m.text;
    const big = !link && onlyEmoji(m.text);
    const reply = m.reply_to ? byId.get(m.reply_to) : null;
    const isGroup = chat.handles.length > 1;
    rows.push(
      <div
        key={m.id}
        className={cx("mrow", m.from_me ? "me" : "them", sms && "sms", startGroup && i > 0 && "gap", m.reactions.length > 0 && "has-react")}
        onContextMenu={(e) => {
          e.preventDefault();
          setMenu({ x: e.clientX, y: e.clientY, m });
        }}
        onDoubleClick={(e) => setPicker({ x: e.clientX, y: (e.currentTarget as HTMLElement).getBoundingClientRect().top, m })}
      >
        {isGroup && !m.from_me && startGroup && <div className="sender">{m.sender}</div>}
        {reply && <div className="quote">{reply.text}</div>}
        <div className={cx("bubble-wrap", isNew(m.id) && "enter")} style={{ alignItems: m.from_me ? "flex-end" : "flex-start" }}>
          {m.reactions.length > 0 && (
            <div className="tapbacks">
              {m.reactions.map((r, j) => (
                <span key={j} className={cx("tb", r.from === "me" && "mine")}>
                  <TapbackGlyph kind={r.kind} size={12} />
                </span>
              ))}
            </div>
          )}
          {big ? (
            <div className="emoji-only">{m.text}</div>
          ) : (
            <>
              {rest && (
                <div className={cx("bubble", endGroup && !link && "tail", (picker?.m.id === m.id || menu?.m.id === m.id) && "selected")}>
                  <Linkify text={rest} />
                </div>
              )}
              {link && (
                <div style={{ marginTop: rest ? 3 : 0 }}>
                  <SpotifyCard link={link} onToast={onToast} />
                </div>
              )}
            </>
          )}
        </div>
        {m.edited && <div className="edited">Edited</div>}
        {m.from_me && m.status === "failed" && (
          <div className="receipt failed">
            <b>Not Delivered</b>
          </div>
        )}
        {m.from_me && m.status === "scheduled" && (
          <div className="receipt" style={{ color: "var(--blue)" }}>
            <Clock size={11} /> Send Later · {clock(m.ts)}
          </div>
        )}
        {lastMine?.id === m.id && m.status === "sending" && (
          <div className="send-bar" aria-label="Delivering">
            <i />
          </div>
        )}
        {lastMine?.id === m.id && (m.status === "read" || m.status === "delivered") && (
          <div className="receipt">
            {m.status === "read" ? (
              <>
                <b>Read</b> {m.read_ts ? clock(m.read_ts) : ""}
              </>
            ) : (
              <b>Delivered</b>
            )}
          </div>
        )}
      </div>,
    );
  });

  return (
    <>
      <div className="thread" ref={scroller}>
        {rows}
      </div>
      <Composer
        chat={chat}
        replyTo={replyTo}
        clearReply={() => setReplyTo(null)}
        editing={editing}
        clearEdit={() => setEditing(null)}
        onToast={onToast}
        injectRef={injectRef}
      />
      {picker && (
        <TapbackPicker
          x={picker.x}
          y={picker.y}
          current={picker.m.reactions.find((r) => r.from === "me")?.kind}
          onPick={(k) => pick(picker.m, k)}
          onClose={() => setPicker(null)}
        />
      )}
      {menu && <Menu x={menu.x} y={menu.y} items={menuItems(menu.m)} onClose={() => setMenu(null)} />}
    </>
  );
}

function Linkify({ text }: { text: string }) {
  const parts = text.split(/(https?:\/\/[^\s]+)/g);
  return (
    <>
      {parts.map((p, i) =>
        /^https?:\/\//.test(p) ? (
          <a
            key={i}
            href={p}
            onClick={(e) => {
              e.preventDefault();
              import("@tauri-apps/plugin-opener").then((o) => o.openUrl(p));
            }}
          >
            {p}
          </a>
        ) : (
          <Fragment key={i}>{p}</Fragment>
        ),
      )}
    </>
  );
}

const EMOJI = "😀😂🤣😭😍🥰😘😎🤔🙄😴😮‍💨😤😡🥶🤯🥳🤩😇🙃😏😬🫠🤡💀👻👽🤖💩👍👎👏🙌🙏💪🫶👀🔥✨💯❤️🧡💛💚💙💜🖤💔🎉🎶🎧🎮🍕🍔🍟☕️🧋🌙⭐️☀️🌈⚡️🍀🌹🐐🦍🐸🐶🐱";

function Composer({
  chat,
  replyTo,
  clearReply,
  editing,
  clearEdit,
  onToast,
  injectRef,
}: {
  chat: Chat;
  replyTo: Message | null;
  clearReply: () => void;
  editing: Message | null;
  clearEdit: () => void;
  onToast: (s: string) => void;
  injectRef: React.MutableRefObject<((s: string) => void) | null>;
}) {
  const [text, setText] = useState("");
  const [pop, setPop] = useState<"apps" | "spotify" | "emoji" | "later" | null>(null);
  const [sendAt, setSendAt] = useState<number | null>(null);
  const ta = useRef<HTMLTextAreaElement>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const sms = chat.service === "SMS";
  useOutside(popRef, () => pop && pop !== "spotify" && setPop(null));

  useEffect(() => {
    setText("");
    setSendAt(null);
    setPop(null);
    ta.current?.focus();
  }, [chat.id]);

  useEffect(() => {
    if (editing) {
      setText(editing.text);
      ta.current?.focus();
    }
  }, [editing]);

  useEffect(() => {
    if (replyTo) ta.current?.focus();
  }, [replyTo]);

  useLayoutEffect(() => {
    const el = ta.current;
    if (!el) return;
    el.style.height = "0px";
    el.style.height = Math.min(160, el.scrollHeight) + "px";
  }, [text]);

  const sendText = (body: string) => {
    if (!body.trim()) return;
    api.send(chat.id, body, replyTo?.id, sendAt).catch((e) => onToast(String(e)));
    clearReply();
    setSendAt(null);
  };

  injectRef.current = (s: string) => sendText(s);

  const submit = () => {
    const body = text;
    if (!body.trim()) return;
    if (editing) {
      api.edit(editing.id, body).catch((e) => onToast(String(e)));
      clearEdit();
      setText("");
      return;
    }
    setText("");
    sendText(body);
  };

  const later = (ms: number) => {
    setSendAt(ms);
    setPop(null);
    ta.current?.focus();
  };

  const at = (h: number, m = 0, addDay = 0) => {
    const d = new Date();
    d.setDate(d.getDate() + addDay);
    d.setHours(h, m, 0, 0);
    return d.getTime();
  };

  return (
    <>
      {replyTo && (
        <div className="replying">
          <Reply size={12} />
          <span>Replying to “{replyTo.text}”</span>
          <button onClick={clearReply}>✕</button>
        </div>
      )}
      {editing && (
        <div className="replying">
          <span>Editing message</span>
          <button
            onClick={() => {
              clearEdit();
              setText("");
            }}
          >
            ✕
          </button>
        </div>
      )}
      {sendAt && (
        <div className="sched-chip">
          <Clock size={11} /> Send Later · {new Date(sendAt).toLocaleString([], { weekday: "short", hour: "numeric", minute: "2-digit" })}
          <button onClick={() => setSendAt(null)}>✕</button>
        </div>
      )}
      <div className={cx("composer", sms && "sms")}>
        <button className={cx("plus", pop && pop !== "emoji" && "on")} onMouseDown={(e) => e.stopPropagation()} onClick={() => setPop(pop ? null : "apps")}>
          <Plus />
        </button>
        <div className="field">
          <SmoothInput
            ref={ta}
            value={text}
            placeholder={editing ? "Edit message" : sms ? "Text Message • SMS" : "iMessage"}
            onChange={setText}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                submit();
              }
              if (e.key === "Escape") {
                if (editing) {
                  clearEdit();
                  setText("");
                }
                clearReply();
              }
              if (e.key === "ArrowUp" && !text) {
                e.preventDefault();
              }
            }}
          />
          <button className="fbtn" onMouseDown={(e) => e.stopPropagation()} onClick={() => setPop(pop === "emoji" ? null : "emoji")} title="Emoji">
            <Smile />
          </button>
          {text.trim() ? (
            <button className="sendbtn" onClick={submit} title="Send">
              <SendArrow size={12} />
            </button>
          ) : (
            <button className="fbtn" title="Audio message" onClick={() => onToast("Audio messages turn on once iMessage is connected")}>
              <Wave />
            </button>
          )}
        </div>
        {pop === "apps" && (
          <div className="popover apps" ref={popRef}>
            <div className="item" onClick={() => setPop("spotify")}>
              <span className="appic" style={{ background: "#000" }}>
                <SpotifyLogo size={16} />
              </span>
              Spotify
            </div>
            <div className="item" onClick={() => setPop("later")}>
              <span className="appic" style={{ background: "#0a84ff" }}>
                <Clock size={13} />
              </span>
              Send Later
            </div>
            <div className="item" onClick={() => setPop("emoji")}>
              <span className="appic" style={{ background: "#ffcc00" }}>
                <Smile size={14} />
              </span>
              Emoji
            </div>
            <div className="item" onClick={() => onToast("Photos turn on once iMessage is connected")}>
              <span className="appic" style={{ background: "linear-gradient(135deg,#ff9500,#ff2d55,#af52de,#007aff)" }}>
                <Photo size={13} />
              </span>
              Photos
            </div>
          </div>
        )}
        {pop === "spotify" && (
          <div ref={popRef}>
            <SpotifyPanel onSend={(u) => sendText(u)} onClose={() => setPop(null)} />
          </div>
        )}
        {pop === "emoji" && (
          <div className="popover emoji-pop" ref={popRef}>
            <div className="emoji-grid">
              {Array.from(new Intl.Segmenter().segment(EMOJI)).map(({ segment }, i) => (
                <button
                  key={i}
                  onClick={() => {
                    setText((t) => t + segment);
                    ta.current?.focus();
                  }}
                >
                  {segment}
                </button>
              ))}
            </div>
          </div>
        )}
        {pop === "later" && (
          <div className="popover sched-pop" ref={popRef}>
            <h4>Send Later</h4>
            <div className="opts">
              <button onClick={() => later(Date.now() + 10_000)}>In 10 seconds</button>
              <button onClick={() => later(Date.now() + 3_600_000)}>In 1 hour</button>
              <button onClick={() => later(at(21))}>Tonight, 9:00 PM</button>
              <button onClick={() => later(at(8, 0, 1))}>Tomorrow, 8:00 AM</button>
            </div>
            <input
              type="datetime-local"
              onChange={(e) => {
                const t = new Date(e.target.value).getTime();
                if (t > Date.now()) later(t);
              }}
            />
          </div>
        )}
      </div>
    </>
  );
}
