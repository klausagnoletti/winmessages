import { useEffect, useRef, useState } from "react";
import { api, lookupTrack, NowPlaying, Track } from "./api";
import { Play, QueueIcon, Search, SpotifyLogo } from "./Icons";

export function SpotifyCard({ link, onToast }: { link: string; onToast: (s: string) => void }) {
  const [t, setT] = useState<Track | null | undefined>(undefined);
  useEffect(() => {
    let live = true;
    lookupTrack(link).then((x) => live && setT(x));
    return () => {
      live = false;
    };
  }, [link]);
  if (t === undefined) {
    return (
      <div className="sp-card">
        <div className="art" />
        <div className="info">
          <div className="txt">
            <div className="ttl">&nbsp;</div>
            <div className="src">
              <SpotifyLogo size={11} /> open.spotify.com
            </div>
          </div>
        </div>
      </div>
    );
  }
  if (t === null) return null;
  const play = (e: React.MouseEvent) => {
    e.stopPropagation();
    api.spotifyPlay(t.uri).then(
      () => onToast(`Playing ${t.name} on Spotify`),
      (err) => onToast(String(err)),
    );
  };
  const queue = (e: React.MouseEvent) => {
    e.stopPropagation();
    api.spotifyQueue(t.uri).then(
      () => onToast(`Added ${t.name} to your queue`),
      (err) => onToast(String(err)),
    );
  };
  return (
    <div className="sp-card" onClick={play} title="Play on Spotify">
      {t.image ? <img className="art" src={t.image} draggable={false} /> : <div className="art" />}
      {t.kind === "track" && (
        <button className="qbtn" onClick={queue} title="Add to queue">
          <QueueIcon size={14} />
        </button>
      )}
      <div className="info">
        <div className="txt">
          <div className="ttl">{t.name}</div>
          <div className="sub">{[t.artist, t.album].filter(Boolean).join(" · ") || t.kind}</div>
          <div className="src">
            <SpotifyLogo size={11} /> Spotify
          </div>
        </div>
        <button className="playbtn" onClick={play}>
          <Play size={12} />
        </button>
      </div>
    </div>
  );
}

export function SpotifyPanel({ onSend, onClose }: { onSend: (url: string) => void; onClose: () => void }) {
  const [now, setNow] = useState<NowPlaying | null>(null);
  const [q, setQ] = useState("");
  const [res, setRes] = useState<Track[]>([]);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    api.spotifyNow().then(setNow, (e) => setErr(String(e)));
    input.current?.focus();
  }, []);
  useEffect(() => {
    if (!q.trim()) {
      setRes([]);
      return;
    }
    setBusy(true);
    const h = setTimeout(() => {
      api
        .spotifySearch(q)
        .then(setRes, (e) => setErr(String(e)))
        .finally(() => setBusy(false));
    }, 280);
    return () => clearTimeout(h);
  }, [q]);
  const send = (t: Track) => {
    onSend(t.url);
    onClose();
  };
  return (
    <div className="popover sp-panel" onMouseDown={(e) => e.stopPropagation()}>
      <div className="sp-head">
        <SpotifyLogo size={16} /> Spotify
      </div>
      {now?.track && (
        <div className="sp-now">
          <img src={now.track.image} draggable={false} />
          <div className="meta">
            <div className="lbl">{now.is_playing ? "Now Playing" : "Paused"}</div>
            <div className="t">{now.track.name}</div>
            <div className="a">{now.track.artist}</div>
          </div>
          <button className="pill-btn" onClick={() => send(now.track!)}>
            Share
          </button>
        </div>
      )}
      <div className="search">
        <Search />
        <input
          ref={input}
          placeholder="Search songs, albums, playlists"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && res[0]) send(res[0]);
            if (e.key === "Escape") onClose();
          }}
        />
      </div>
      <div className="sp-results">
        {err && <div className="sp-empty">{err}</div>}
        {!err && q && !busy && !res.length && <div className="sp-empty">No results</div>}
        {!q && !err && <div className="sp-empty">Search Spotify or share what you're playing. Songs show up as playable cards.</div>}
        {res.map((t) => (
          <div className="sp-res" key={t.uri} onClick={() => send(t)}>
            <img src={t.image} draggable={false} />
            <div className="meta">
              <div className="t">{t.name}</div>
              <div className="a">
                <span className="kind">{t.kind}</span> · {t.artist}
              </div>
            </div>
            <button
              className="mini"
              title="Play"
              onClick={(e) => {
                e.stopPropagation();
                api.spotifyPlay(t.uri);
              }}
            >
              <Play size={10} />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
