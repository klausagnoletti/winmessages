import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, Account, Chat, NowPlaying, spotifyLink } from "./api";
import { Avatar, Menu, MenuItem } from "./Bits";
import { Compose, Pause, Play, Search, SendArrow, SpotifyLogo } from "./Icons";
import { cx, listTime } from "./util";

function preview(c: Chat) {
  if (spotifyLink(c.last_text)) return "Spotify";
  return c.last_text;
}

export function TrafficLights() {
  const w = getCurrentWindow();
  return (
    <div className="lights">
      <button className="light close" onClick={() => w.close()}>
        <svg width="6" height="6" viewBox="0 0 6 6" stroke="#4d0000" strokeWidth="1.1" strokeLinecap="round">
          <path d="M.8.8l4.4 4.4M5.2.8.8 5.2" />
        </svg>
      </button>
      <button className="light min" onClick={() => w.minimize()}>
        <svg width="7" height="2" viewBox="0 0 7 2" stroke="#985700" strokeWidth="1.2" strokeLinecap="round">
          <path d="M.7 1h5.6" />
        </svg>
      </button>
      <button className="light max" onClick={() => w.toggleMaximize()}>
        <svg width="6" height="6" viewBox="0 0 6 6" fill="#006500">
          <path d="M0 0h3.8L0 3.8zM6 6H2.2L6 2.2z" />
        </svg>
      </button>
    </div>
  );
}

export function Sidebar(props: {
  chats: Chat[];
  selected: string | null;
  account: Account;
  spotify: boolean;
  onSelect: (id: string) => void;
  onCompose: () => void;
  onSignIn: () => void;
  onShare: (url: string) => void;
  onToast: (s: string) => void;
  query: string;
  setQuery: (q: string) => void;
}) {
  const { chats, selected, onSelect } = props;
  const [menu, setMenu] = useState<{ x: number; y: number; chat: Chat } | null>(null);
  const q = props.query.trim().toLowerCase();
  const visible = q
    ? chats.filter((c) => c.name.toLowerCase().includes(q) || c.last_text.toLowerCase().includes(q) || c.handles.some((h) => h.includes(q)))
    : chats;
  const pins = q ? [] : visible.filter((c) => c.pinned);
  const rest = q ? visible : visible.filter((c) => !c.pinned);

  const ctx = (e: React.MouseEvent, chat: Chat) => {
    e.preventDefault();
    setMenu({ x: e.clientX, y: e.clientY, chat });
  };

  const items = (c: Chat): MenuItem[] => [
    { label: c.pinned ? "Unpin" : "Pin", onClick: () => api.setPinned(c.id, !c.pinned) },
    { label: c.unread ? "Mark as Read" : "Mark as Unread", onClick: () => api.markRead(c.id) },
    "sep",
    { label: "Delete Conversation…", danger: true, onClick: () => api.deleteChat(c.id) },
  ];

  return (
    <aside className="sidebar">
      <div className="sb-top drag" data-tauri-drag-region>
        <TrafficLights />
        <div className="spacer" data-tauri-drag-region />
        <button className="icon-btn" title="New Message (Ctrl+N)" onClick={props.onCompose}>
          <Compose />
        </button>
      </div>
      <label className="search">
        <Search />
        <input placeholder="Search" value={props.query} onChange={(e) => props.setQuery(e.target.value)} />
      </label>
      {props.account.state !== "signed_in" && (
        <div className="signin-banner">
          <b>iMessage is off.</b> You're looking at demo chats.
          <br />
          <button onClick={props.onSignIn}>Sign In with Apple ID…</button>
        </div>
      )}
      {pins.length > 0 && (
        <div className="pins">
          {pins.map((c) => (
            <div key={c.id} className={cx("pin", selected === c.id && "selected")} onClick={() => onSelect(c.id)} onContextMenu={(e) => ctx(e, c)}>
              {c.unread > 0 && <span className="dot" />}
              <Avatar name={c.name} size={60} />
              <span className="name">{c.name}</span>
            </div>
          ))}
        </div>
      )}
      <div className="list">
        {rest.map((c) => (
          <div key={c.id} className={cx("row", selected === c.id && "selected")} onClick={() => onSelect(c.id)} onContextMenu={(e) => ctx(e, c)}>
            {c.unread > 0 && <span className="dot" />}
            <Avatar name={c.name} size={40} />
            <div className="body">
              <div className="line1">
                <span className="title">{c.name}</span>
                <span className="time">{listTime(c.last_ts)}</span>
              </div>
              <div className="preview">{preview(c) || " "}</div>
            </div>
          </div>
        ))}
      </div>
      {props.spotify && <NowBar onShare={props.onShare} canShare={!!selected} onToast={props.onToast} />}
      {menu && <Menu x={menu.x} y={menu.y} items={items(menu.chat)} onClose={() => setMenu(null)} />}
    </aside>
  );
}

function NowBar({ onShare, canShare, onToast }: { onShare: (url: string) => void; canShare: boolean; onToast: (s: string) => void }) {
  const [np, setNp] = useState<NowPlaying | null>(null);
  useEffect(() => {
    let live = true;
    const tick = () => api.spotifyNow().then((x) => live && setNp(x), () => {});
    tick();
    const h = setInterval(tick, 5000);
    return () => {
      live = false;
      clearInterval(h);
    };
  }, []);
  const t = np?.track;
  if (!t) return null;
  const toggle = () => {
    const p = np!.is_playing ? api.spotifyPause() : api.spotifyPlay(null);
    setNp({ ...np!, is_playing: !np!.is_playing });
    p.catch((e) => onToast(String(e)));
  };
  return (
    <div className="nowbar">
      {t.image ? <img src={t.image} draggable={false} /> : <div className="art-ph" />}
      <div className="meta">
        <div className="t">{t.name}</div>
        <div className="a">
          <SpotifyLogo size={10} /> {t.artist}
        </div>
      </div>
      <button className="mini" onClick={toggle} title={np!.is_playing ? "Pause" : "Play"}>
        {np!.is_playing ? <Pause size={11} /> : <Play size={11} />}
      </button>
      {canShare && (
        <button className="mini" title="Share to this chat" onClick={() => onShare(t.url)}>
          <SendArrow size={12} />
        </button>
      )}
    </div>
  );
}
