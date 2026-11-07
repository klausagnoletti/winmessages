import { useEffect, useLayoutEffect, useRef, useState, ReactNode } from "react";
import { initials } from "./util";
import { Bang, Haha, Heart, Question, ThumbDown, ThumbUp } from "./Icons";

export function Avatar({ name, size = 40 }: { name: string; size?: number }) {
  const ini = initials(name);
  return (
    <div className="avatar" style={{ width: size, height: size, fontSize: size * 0.42 }}>
      {ini ? (
        <span style={{ letterSpacing: "0.01em" }}>{ini}</span>
      ) : (
        <svg className="person" viewBox="0 0 20 20" fill="currentColor">
          <circle cx="10" cy="6.5" r="4.2" />
          <path d="M1.5 20c.6-5 4.2-7.6 8.5-7.6s7.9 2.6 8.5 7.6z" />
        </svg>
      )}
    </div>
  );
}

export const TAPBACKS = ["love", "like", "dislike", "laugh", "emphasize", "question"] as const;

export function TapbackGlyph({ kind, size = 13 }: { kind: string; size?: number }) {
  switch (kind) {
    case "love":
      return <Heart size={size} />;
    case "like":
      return <ThumbUp size={size} />;
    case "dislike":
      return <ThumbDown size={size} />;
    case "laugh":
      return <Haha />;
    case "emphasize":
      return <Bang size={size} />;
    case "question":
      return <Question size={size} />;
    default:
      return <span style={{ fontSize: size + 2, lineHeight: 1 }}>{kind}</span>;
  }
}

export type MenuItem = { label: string; icon?: ReactNode; danger?: boolean; onClick: () => void } | "sep";

export function Menu({ x, y, items, onClose }: { x: number; y: number; items: MenuItem[]; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ x, y });
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setPos({
      x: Math.min(x, window.innerWidth - r.width - 6),
      y: y + r.height > window.innerHeight - 6 ? Math.max(6, y - r.height) : y,
    });
  }, [x, y]);
  useOutside(ref, onClose);
  return (
    <div className="menu" ref={ref} style={{ left: pos.x, top: pos.y }}>
      {items.map((it, i) =>
        it === "sep" ? (
          <hr key={i} />
        ) : (
          <div
            key={i}
            className={"item" + (it.danger ? " danger" : "")}
            onMouseUp={() => {
              onClose();
              it.onClick();
            }}
          >
            {it.icon !== undefined && <span className="ic">{it.icon}</span>}
            {it.label}
          </div>
        ),
      )}
    </div>
  );
}

const QUICK = ["😂", "🔥", "😭", "💀", "🙏"];

export function TapbackPicker({
  x,
  y,
  current,
  onPick,
  onClose,
}: {
  x: number;
  y: number;
  current?: string;
  onPick: (k: string) => void;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useOutside(ref, onClose);
  const w = (TAPBACKS.length + QUICK.length) * 36 + 10;
  const left = Math.max(8, Math.min(x - w / 2, window.innerWidth - w - 8));
  return (
    <div className="picker" ref={ref} style={{ left, top: Math.max(8, y - 50) }}>
      {TAPBACKS.map((k) => (
        <button key={k} className={current === k ? "on" : ""} onClick={() => onPick(k)}>
          <TapbackGlyph kind={k} size={16} />
        </button>
      ))}
      {QUICK.map((k) => (
        <button key={k} className={current === k ? "on" : ""} onClick={() => onPick(k)}>
          {k}
        </button>
      ))}
    </div>
  );
}

export function useOutside(ref: React.RefObject<HTMLElement | null>, fn: () => void) {
  useEffect(() => {
    const h = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) fn();
    };
    const k = (e: KeyboardEvent) => {
      if (e.key === "Escape") fn();
    };
    const t = setTimeout(() => {
      document.addEventListener("mousedown", h);
      document.addEventListener("contextmenu", h);
    }, 0);
    document.addEventListener("keydown", k);
    return () => {
      clearTimeout(t);
      document.removeEventListener("mousedown", h);
      document.removeEventListener("contextmenu", h);
      document.removeEventListener("keydown", k);
    };
  }, [ref, fn]);
}
