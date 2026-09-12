import { useId } from "react";
export function Logo({
  variant = "horizontal",
  dark = false,
}: {
  variant?: "horizontal" | "compact" | "icon";
  dark?: boolean;
}) {
  const id = useId().replace(/:/g, "");
  return (
    <div className={`logo logo-${variant} ${dark ? "logo-dark" : ""}`}>
      <svg viewBox="0 0 82 68" role="img" aria-label="M² Health logo">
        <defs>
          <linearGradient id={`${id}a`} x1="0" y1="1" x2="1" y2="0">
            <stop stopColor="#10b981" />
            <stop offset=".55" stopColor="#047857" />
            <stop offset="1" stopColor="#064e3b" />
          </linearGradient>
          <linearGradient id={`${id}b`} x1="0" y1="0" x2="1" y2="1">
            <stop stopColor="#a7f3d0" />
            <stop offset=".5" stopColor="#10b981" />
            <stop offset="1" stopColor="#065f46" />
          </linearGradient>
        </defs>
        <path
          d="M6 59V18Q6 8 17 10L34 28 48 12Q53 8 63 9V29L35 59V37L20 23V59Z"
          fill={`url(#${id}a)`}
        />
        <path d="M9 12Q20 4 29 15L47 36 35 50 18 28Z" fill={`url(#${id}b)`} />
        <path
          className="brand-superscript"
          d="M63 7C63 1 78 0 79 7C80 12 73 16 68 19H79V24H62V19C67 15 74 11 74 8C74 5 68 5 68 8Z"
          fill={dark ? "#6ee7b7" : "#047857"}
        />
        <path d="M61 37h10v8h8v10h-8v8H61v-8h-8V45h8z" fill={`url(#${id}a)`} />
      </svg>
      {variant !== "icon" && (
        <div>
          <div className="brand-name" aria-label="M² Health">
            <span className="brand-letter">
              M
              <span className="wordmark-squared" aria-hidden="true">
                2
              </span>
            </span>{" "}
            <span>Health</span>
          </div>
          <p>
            Smart Pharmacy
            <br className="compact-break" /> Management System
          </p>
        </div>
      )}
    </div>
  );
}
export function AppBackground() {
  return (
    <div className="app-background" aria-hidden="true">
      <svg viewBox="0 0 1600 1000" preserveAspectRatio="none">
        <defs>
          <linearGradient id="wave" x1="0" y1="0" x2="1" y2="1">
            <stop stopColor="#fff" stopOpacity="0" />
            <stop offset=".5" stopColor="#10b981" stopOpacity=".2" />
            <stop offset="1" stopColor="#059669" stopOpacity=".05" />
          </linearGradient>
        </defs>
        <path
          d="M700 0C1060 230 1200-110 1600 20V250C1260 60 1180 230 700 0Z"
          fill="url(#wave)"
        />
        <path d="M0 410C240 540 30 870 610 1000H0Z" fill="url(#wave)" />
        <path d="M0 600C220 620 140 870 740 1000H0Z" fill="url(#wave)" />
        <path
          d="M0 652C300 780 40 890 610 1000M0 510C150 650 220 880 750 1000M780 0C1240 300 1240 20 1600 180"
          fill="none"
          stroke="white"
          strokeOpacity=".75"
          strokeWidth="2"
        />
      </svg>
    </div>
  );
}
