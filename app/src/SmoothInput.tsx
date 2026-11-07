import { forwardRef, useEffect, useImperativeHandle, useLayoutEffect, useRef, useState } from "react";

type Props = {
  value: string;
  placeholder?: string;
  onChange: (v: string) => void;
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
};

const seg = new Intl.Segmenter(undefined, { granularity: "grapheme" });

function graphemes(s: string): string[] {
  return Array.from(seg.segment(s), (x) => x.segment);
}

export const SmoothInput = forwardRef<HTMLTextAreaElement, Props>(function SmoothInput(
  { value, placeholder, onChange, onKeyDown },
  ref,
) {
  const ta = useRef<HTMLTextAreaElement>(null);
  const overlay = useRef<HTMLDivElement>(null);
  const caret = useRef<HTMLDivElement>(null);
  const born = useRef<Map<number, number>>(new Map());
  const [sel, setSel] = useState(0);
  const cur = useRef<{ x: number; y: number } | null>(null);
  const last = useRef(performance.now());
  const blinkFrom = useRef(performance.now());

  useImperativeHandle(ref, () => ta.current as HTMLTextAreaElement, []);

  const gs = graphemes(value);

  useLayoutEffect(() => {
    const el = ta.current;
    if (!el) return;
    el.style.height = "0px";
    el.style.height = Math.min(160, el.scrollHeight) + "px";
  }, [value]);

  useEffect(() => {
    let raf = 0;
    const tick = () => {
      const now = performance.now();
      const dt = Math.min(0.1, (now - last.current) / 1000);
      last.current = now;
      const anchor = overlay.current?.querySelector<HTMLElement>(`[data-i="${sel}"]`);
      const box = overlay.current?.getBoundingClientRect();
      const c = caret.current;
      if (anchor && box && c) {
        const r = anchor.getBoundingClientRect();
        const atEnd = sel >= gs.length;
        const tx = (atEnd ? r.right : r.left) - box.left;
        const ty = r.top - box.top;
        if (!cur.current) cur.current = { x: tx, y: ty };
        const k = 1 - Math.exp(-dt / 0.04);
        let nx = cur.current.x + (tx - cur.current.x) * k;
        let ny = cur.current.y + (ty - cur.current.y) * k;
        if (Math.abs(nx - tx) < 0.25) nx = tx;
        if (Math.abs(ny - ty) < 0.25) ny = ty;
        const settled = nx === tx && ny === ty;
        cur.current = { x: nx, y: ny };
        const inset = Math.max(2, (r.height - 15) / 2);
        c.style.transform = `translate(${nx}px, ${ny + inset}px)`;
        c.style.height = r.height - inset * 2 + "px";
        const focused = document.activeElement === ta.current;
        if (!focused) {
          c.style.opacity = "0";
        } else if (settled) {
          const phase = ((now - blinkFrom.current) / 1000) % 1.06;
          c.style.opacity = phase < 0.53 ? "1" : "0";
        } else {
          c.style.opacity = "1";
          blinkFrom.current = now;
        }
      } else if (c) {
        c.style.opacity = "0";
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [sel, gs.length]);

  const sync = () => {
    const el = ta.current;
    if (el) setSel(graphemes(el.value.slice(0, el.selectionStart ?? 0)).length);
  };

  useEffect(() => {
    const now = performance.now();
    const seen = born.current;
    const next = new Map<number, number>();
    for (let i = 0; i < gs.length; i++) next.set(i, seen.get(i) ?? now);
    born.current = next;
  }, [value]);

  return (
    <div className="smooth-wrap">
      <div className="smooth-overlay" ref={overlay} aria-hidden>
        {gs.length === 0 && <span className="smooth-ph">{placeholder}</span>}
        {gs.map((g, i) => {
          const b = born.current.get(i) ?? 0;
          const age = (performance.now() - b) / 1000;
          const fresh = age < 0.18;
          return (
            <span
              key={i}
              data-i={i}
              className={"smooth-ch" + (fresh ? " in" : "")}
              style={fresh ? { animationDelay: `${-Math.max(0, age)}s` } : undefined}
            >
              {g === "\n" ? "\n" : g}
            </span>
          );
        })}
        <span data-i={gs.length} className="smooth-ch end">
          &#8203;
        </span>
        <div className="smooth-caret" ref={caret} />
      </div>
      <textarea
        ref={ta}
        rows={1}
        className="smooth-ta"
        value={value}
        placeholder={placeholder}
        spellCheck
        onChange={(e) => {
          onChange(e.target.value);
          sync();
        }}
        onKeyDown={(e) => {
          onKeyDown?.(e);
          requestAnimationFrame(sync);
        }}
        onKeyUp={sync}
        onClick={sync}
        onSelect={sync}
        onFocus={() => {
          blinkFrom.current = performance.now();
          sync();
        }}
      />
    </div>
  );
});
