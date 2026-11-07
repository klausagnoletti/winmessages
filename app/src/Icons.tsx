type P = { size?: number; className?: string };

export const Compose = ({ size = 18 }: P) => (
  <svg width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M9.5 3.5H5.2A1.7 1.7 0 0 0 3.5 5.2v9.6a1.7 1.7 0 0 0 1.7 1.7h9.6a1.7 1.7 0 0 0 1.7-1.7v-4.3" />
    <path d="M14.6 2.9a1.5 1.5 0 0 1 2.1 2.1L10 11.7 7.5 12.5l.8-2.5z" />
  </svg>
);

export const Search = ({ size = 13 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round">
    <circle cx="6.8" cy="6.8" r="4.8" />
    <path d="M10.4 10.4 14 14" />
  </svg>
);

export const Video = ({ size = 22 }: P) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round">
    <rect x="2.5" y="6.5" width="13" height="11" rx="2.6" />
    <path d="m15.5 10.6 4.9-3v8.8l-4.9-3z" />
  </svg>
);

export const Info = ({ size = 20 }: P) => (
  <svg width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.4">
    <circle cx="10" cy="10" r="7.6" />
    <path d="M10 9v5" strokeLinecap="round" />
    <circle cx="10" cy="6.3" r=".9" fill="currentColor" stroke="none" />
  </svg>
);

export const Plus = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round">
    <path d="M7 2.2v9.6M2.2 7h9.6" />
  </svg>
);

export const Smile = ({ size = 18 }: P) => (
  <svg width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round">
    <circle cx="10" cy="10" r="7.5" />
    <path d="M6.9 11.8a3.8 3.8 0 0 0 6.2 0" />
    <circle cx="7.6" cy="8" r=".9" fill="currentColor" stroke="none" />
    <circle cx="12.4" cy="8" r=".9" fill="currentColor" stroke="none" />
  </svg>
);

export const Wave = ({ size = 18 }: P) => (
  <svg width={size} height={size} viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round">
    <path d="M3 9v2M6 6.5v7M9 4v12M12 7v6M15 5.5v9M18 8.5v3" />
  </svg>
);

export const SendArrow = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.9" strokeLinecap="round" strokeLinejoin="round">
    <path d="M7 12V2.4M3 6.2 7 2.2l4 4" />
  </svg>
);

export const Heart = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor">
    <path d="M8 14.2s-5.8-3.5-5.8-7.6A3.2 3.2 0 0 1 8 4.6a3.2 3.2 0 0 1 5.8 2c0 4.1-5.8 7.6-5.8 7.6z" />
  </svg>
);

export const ThumbUp = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor">
    <path d="M5 7.2 7.8 1.8c.9 0 1.7.8 1.6 1.8L9 6.4h3.6a1.5 1.5 0 0 1 1.5 1.8l-1 4.6a1.6 1.6 0 0 1-1.6 1.3H5zM1.8 7.2h2.1v7H1.8a.6.6 0 0 1-.6-.6V7.8a.6.6 0 0 1 .6-.6z" />
  </svg>
);

export const ThumbDown = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor" style={{ transform: "rotate(180deg)" }}>
    <path d="M5 7.2 7.8 1.8c.9 0 1.7.8 1.6 1.8L9 6.4h3.6a1.5 1.5 0 0 1 1.5 1.8l-1 4.6a1.6 1.6 0 0 1-1.6 1.3H5zM1.8 7.2h2.1v7H1.8a.6.6 0 0 1-.6-.6V7.8a.6.6 0 0 1 .6-.6z" />
  </svg>
);

export const Play = ({ size = 12 }: P) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="currentColor">
    <path d="M3 1.6v8.8a.6.6 0 0 0 .9.5l7-4.4a.6.6 0 0 0 0-1L3.9 1.1a.6.6 0 0 0-.9.5z" />
  </svg>
);

export const Pause = ({ size = 12 }: P) => (
  <svg width={size} height={size} viewBox="0 0 12 12" fill="currentColor">
    <rect x="2.3" y="1.5" width="2.6" height="9" rx=".7" />
    <rect x="7.1" y="1.5" width="2.6" height="9" rx=".7" />
  </svg>
);

export const QueueIcon = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
    <path d="M2 4h9M2 8h9M2 12h5M12 10v5M9.5 12.5h5" />
  </svg>
);

export const Clock = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round">
    <circle cx="8" cy="8" r="6.2" />
    <path d="M8 4.5V8l2.4 1.6" />
  </svg>
);

export const Photo = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinejoin="round">
    <rect x="1.8" y="2.8" width="12.4" height="10.4" rx="2" />
    <circle cx="5.6" cy="6.3" r="1.1" />
    <path d="m2.4 12 3.8-3.6 2.6 2.3 2-1.7 3 2.6" />
  </svg>
);

export const SpotifyLogo = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="12" fill="#1ED760" />
    <path d="M6.2 9.3c3.9-1.1 8.4-.8 11.8 1.2M6.8 12.5c3.2-.9 6.9-.6 9.7 1.1M7.4 15.5c2.6-.7 5.3-.4 7.5.9" stroke="#000" strokeWidth="1.6" strokeLinecap="round" fill="none" />
  </svg>
);

export const Reply = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M6.5 3.5 2.5 7.5l4 4" />
    <path d="M2.8 7.5h6.7a4 4 0 0 1 4 4v1" />
  </svg>
);

export const Bang = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="currentColor">
    <path d="M4.2 2h2.1l-.4 8H4.6zM9.7 2h2.1l-.4 8h-1.3zM4.3 12h1.9v2H4.3zM9.8 12h1.9v2H9.8z" />
  </svg>
);

export const Question = ({ size = 14 }: P) => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2.1" strokeLinecap="round">
    <path d="M5 5.4a3 3 0 1 1 4.4 2.6C8.5 8.6 8 9.1 8 10.3" />
    <circle cx="8" cy="13.6" r=".4" fill="currentColor" />
  </svg>
);

export const Haha = () => <span className="haha">HA<br />HA</span>;
