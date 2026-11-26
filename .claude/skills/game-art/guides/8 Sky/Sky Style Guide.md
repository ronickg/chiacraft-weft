# Sky Style Guide

This guide covers the sun, moon phases, weather (rain and snow), clouds, stars and special skies.

`Sky Prompt References/01 Sky.png` shows the reference's sun, all 8 moon phases, the rain and snow strips, and a crop of its end-dimension sky.

---

## 1. What the reference does

- **Sun:** a small bright disc with a soft glow and short rays on a transparent background, 64×64. The colours run from a red-orange outer glow (`#ff2b00 → #ff6600`) to a near-white core (`#ffffde`). It's drawn with a glow rather than as hard pixel art, because the sky is huge on screen.
- **Moon:** the full moon is pale blue-white (`#9abdd5 → #e1f3ff`) with soft grey craters. **8 phases**: full, waning gibbous, third quarter, waning crescent, new (a dark disc), waxing crescent, first quarter and waxing gibbous. The shadow side is near-black, so the moon cuts a clear shape out of the night sky.
- **Rain and snow:** tall vertical strips that **tile seamlessly top to bottom** and scroll down. Rain is thin 1-px grey streaks of different lengths. Snow is single white specks spread out irregularly.
- **End sky:** deep navy (`#00003f`) with scattered blue four-pointed stars of different sizes, tiled across the whole sky.
- The reference also uses custom sky layers (sunrise and sunset gradients, star fields), drawn as large soft images rather than pixel art.

---

## 2. Rules

- **Sun and moon are the one place soft glow is allowed.** Keep the disc itself crisp, at the same pixel size as everything else, and add a soft glow or ray ring around it.
- **Moon phases must line up:** the same disc, the same position and the same size in all 8, with only the shadow moving.
- **Weather tiles:** rain and snow must tile vertically without a visible seam. Vary streak length and spacing so no pattern repeats.
- **Clouds:** big, flat, blocky shapes in 2–3 tones (white top, light grey underside), with no outline, so they match the blocky world.
- **Colour by time of day:**
  - day sky: about `#7fb0f0` → `#c4dcf7` at the horizon
  - sunset: orange `#f79701` → pink `#e86a6a` → purple `#5a3a7a`
  - night: `#0c1024` → `#1a2240`
  - Keep these warm and gentle, so the bronze UI and warm items stay the hero.

---

## 3. Prompts

### A) Master prompt

> **Style:** Pixel-art sky element for a warm fantasy survival game with a blocky world, crisp pixels at the same size as 32×32 item icons, transparent background. Celestial objects are crisp pixel discs with a soft glow around them. Weather strips tile seamlessly from top to bottom. Calm, warm, slightly painterly mood.

Add **one** of these lines:

| Type | Add to the prompt |
|---|---|
| **Sun** | "A 32-px pixel sun disc with a near-white core, a yellow-orange middle and a soft orange glow with short rays, centred on a 64×64 transparent canvas." |
| **Moon phases** | "8 moon phases in a row, each 64×64: full, waning gibbous, third quarter, waning crescent, new, waxing crescent, first quarter, waxing gibbous. A pale blue-white disc with soft grey craters, the same disc and position in every frame, near-black shadow side." |
| **Rain** | "A 64×256 strip that tiles vertically: thin 1-px grey-blue rain streaks of different lengths, slightly slanted, irregular spacing, transparent background." |
| **Snow** | "A 64×256 strip that tiles vertically: small 1–2 px white snowflakes, irregularly spaced, some slightly larger, transparent background." |
| **Clouds** | "A tileable cloud layer seen from below, big flat blocky cloud shapes in white with a light grey underside, no outline, transparent gaps." |
| **Star field** | "A tileable night sky of tiny 1-px stars and a few 4-pointed bright stars, on deep navy, a faint band of a galaxy." |
| **Sky gradient** | "A tall sky gradient for [day / sunset / night], from [colour] at the top to [colour] at the horizon, in 8–12 flat pixel bands." |

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Sun, moon, weather, star field | `01 Sky.png` |

### C) Checklist

- Sun, 8 moon phases, star field, clouds
- Sky gradients: dawn, day, sunset, night, storm
- Weather: rain, snow, thunderstorm flash, fog
- Special places: underground or cave fog colour, other dimensions (each with its own sky colour and stars)
