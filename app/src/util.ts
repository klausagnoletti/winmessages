const DAY = 86_400_000;

function startOfDay(t: number) {
  const d = new Date(t);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

export function clock(t: number) {
  return new Date(t).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}

export function listTime(t: number) {
  if (!t) return "";
  const today = startOfDay(Date.now());
  if (t >= today) return clock(t);
  if (t >= today - DAY) return "Yesterday";
  if (t >= today - 6 * DAY) return new Date(t).toLocaleDateString([], { weekday: "long" });
  return new Date(t).toLocaleDateString([], { month: "numeric", day: "numeric", year: "2-digit" });
}

export function separatorParts(t: number): [string, string] {
  const today = startOfDay(Date.now());
  if (t >= today) return ["Today", clock(t)];
  if (t >= today - DAY) return ["Yesterday", clock(t)];
  if (t >= today - 6 * DAY) return [new Date(t).toLocaleDateString([], { weekday: "long" }), clock(t)];
  return [new Date(t).toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" }), clock(t)];
}

export function initials(name: string) {
  if (/^[+\d(]/.test(name.trim())) return "";
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (!parts.length) return "";
  if (parts.length === 1) return parts[0][0].toUpperCase();
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

export function onlyEmoji(text: string) {
  const t = text.trim();
  if (!t || t.length > 24) return false;
  return /^(?:\p{Extended_Pictographic}|\p{Emoji_Component}|‍|️|\s)+$/u.test(t) && !/^[\d#*\s]+$/.test(t);
}

export function cx(...a: (string | false | null | undefined)[]) {
  return a.filter(Boolean).join(" ");
}
