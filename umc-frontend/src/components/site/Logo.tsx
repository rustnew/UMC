import type { CSSProperties } from "react";
import umcLogo from "@/assets/umc-logo.png.asset.json";

/**
 * UMC brand mark — official wordmark "UVC" with the green dot,
 * brushed metal finish on dark background.
 */
export function Logo({
  size = 28,
  className = "",
  animated = false,
  style,
}: {
  size?: number;
  className?: string;
  animated?: boolean;
  style?: CSSProperties;
}) {
  // Original image is ~2:1 ratio. We render a square crop with object-contain
  // so the wordmark scales cleanly inside the requested footprint.
  return (
    <span
      className={`inline-flex items-center justify-center overflow-hidden ${className} ${animated ? "pulse-glow" : ""}`}
      style={{ width: size * 2, height: size, ...style }}
      aria-label="UMC"
    >
      <img
        src={umcLogo.url}
        alt="UMC"
        width={size * 2}
        height={size}
        className="object-contain"
        draggable={false}
      />
    </span>
  );
}