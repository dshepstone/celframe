import type { Animated } from "./backend";

/** The value at a frame, as the core works it out. See docs/specs/keyframes.md. */
export function valueAt(animated: Animated, frame: number): number {
  const keys = animated.keys;
  const first = keys[0];
  const last = keys[keys.length - 1];
  if (!first || !last) return animated.value;
  if (frame <= first.frame) return first.value;
  if (frame >= last.frame) return last.value;
  const next = keys.findIndex((k) => k.frame > frame);
  const a = keys[next - 1];
  const b = keys[next];
  if (!a || !b) return last.value;
  let t = (frame - a.frame) / (b.frame - a.frame);
  if (a.interpolation === "ease") t = t * t * (3 - 2 * t);
  if (a.interpolation === "step") t = 0;
  return a.value + (b.value - a.value) * t;
}
