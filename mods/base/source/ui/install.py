"""Dresses the game in the UI kit (`kit/`, Ronald's generated parts cut by `sheet.py`,
`icons.py`, `bars.py`; `notes/STYLE.md`): copies each part under the name the engine looks for
(`docs/ui.md`, "Looks") into `../../assets/textures/gui/`, with its `.gui`.

The kit is drawn for 32-pixel slots, the engine's screens lay slots out at 18 GUI pixels: every
part is shrunk by the same factor (18/32), so slots, frames, buttons and bars keep the sizes they
have to each other in the kit. Parts with more image pixels than GUI pixels are sampled smoothly
(`ui.wgsl`, `sharp_uv`). A few the kit has no picture for are made here from its own parts (half
hearts, the red quit button, the hover glow); `MISSING` lists what still needs drawing.

    python3 mods/base/source/ui/install.py
"""
import colorsys
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
KIT = os.path.join(HERE, "kit")
GUI = os.path.join(HERE, "..", "..", "assets", "textures", "gui")
F = 18 / 32  # the kit's slot is 32, the engine's 18

# engine name: kit part (nine-slice parts keep their border; pictures are drawn at the size the
# engine lays them out at)
PARTS = {
    "slot": "slot",
    "recipe_slot": "slot_disabled",
    "hotbar": "slot",
    "hotbar_selected": "slot_selected",
    "advancement_tile": "slot",
    "advancement_tile_done": "slot_selected",
    "panel": "panel",
    "sidebar": "panel",
    "news": "panel",
    "well": "well",
    "tooltip": "tooltip",
    "button": "button",
    "button_hover": "button_hover",
    "button_pressed": "button_pressed",
    "button_disabled": "button_disabled",
    "input": "input",
    "input_focused": "input_focused",
    "scrollbar": "scrollbar",
    "scrollbar_handle": "scrollbar_handle",
    "scrollbar_handle_hover": "scrollbar_handle_hover",
    "slider": "slider",
    "slider_knob": "slider_knob",
    "slider_knob_hover": "slider_knob_hover",
    "crosshair": "crosshair",
    "arrow": "arrow",
    "arrow_fill": "arrow_full",
    "flame": "flame",
    "flame_fill": "flame_full",
    "icon_close": "close",
    "icon_cycle": "actions/refresh",
    "icon_settings": "categories/settings",
    "slot_helmet": "equipment/helmet",
    "slot_chestplate": "equipment/chest",
    "slot_leggings": "equipment/legs",
    "slot_boots": "equipment/boots",
    "led_on": "radio_on",
    "led_off": "radio",
    "led_wait": "led_wait",
    "titlebar_grip": "titlebar_grip",
    "attack_bar": "attack_bar",
    "attack_bar_fill": "attack_bar_fill",
    "advancement_line": "advancement_line",
    "advancement_line_done": "advancement_line_done",
    "armor_full": "armor_full",
    "armor_half": "armor_half",
    "armor_empty": "armor_empty",
    "logo": "logo",
    "menu_singleplayer": "menu_singleplayer",
    "menu_multiplayer": "menu_multiplayer",
    "menu_settings": "menu_settings",
    "menu_mods": "menu_mods",
    # Settings > Arrange HUD (`hud.rs`)
    "hud_frame": "hud_frame",
    "hud_frame_hidden": "hud_frame_hidden",
    "hud_frame_selected": "hud_frame_selected",
    # the HUD's own: the clock's sun, the shortcuts' pictures (`hud.rs`)
    "hud_day": "temperature/warm",
    "hud_night": "hud_night",
    "hud_inventory": "categories/inventory",
    "hud_recipes": "categories/crafting",
    "hud_advancements": "categories/journal",
    "hud_stats": "categories/skills",
}

# The vitals' effects and their icons, as `vitals::effects::Effect::icon` has them.
EFFECT_ICONS = {
    "wet": "status/wet", "chilly": "temperature/cold", "cold": "temperature/cold", "freezing": "temperature/freezing",
    "warm": "temperature/warm", "hot": "temperature/hot", "heatstroke": "status/heat", "poisoned": "status/poison",
    "bleeding": "status/bleeding", "food_poisoning": "status/poison", "sedated": "status/rested",
    "encumbered": "status/encumbered", "overloaded": "status/encumbered", "near_fire": "status/heat",
    "sheltered": "status/wind", "rested": "status/rested", "well_fed": "status/food",
}

# Still drawn in the old style: the kit has nothing for them yet (to generate).
MISSING = {
    "font": "the pixel font is still the old one: Alegreya and Alegreya Sans need the engine's smooth-font change (notes/STYLE.md, Type)",
    "overlay, vignette, death": "the dimming behind menus, the vignette, the death screen",
    "input_selection, caret, suggest": "text selection, the caret, chat suggestions (engine)",
}


def load(part):
    im = Image.open(os.path.join(KIT, part + ".png")).convert("RGBA")
    spec = {}
    g = os.path.join(KIT, part + ".gui")
    if os.path.exists(g):
        spec = dict(line.split(" = ") for line in open(g).read().strip().splitlines())
    return im, spec


def fmt(v):
    return f"{round(v * 4) / 4:g}"


def write(name, im, spec):
    im.save(os.path.join(GUI, name + ".png"))
    lines = []
    if spec.get("scaling") == "nine_slice":
        w, h = (float(v) * F for v in spec["size"].split())
        border = " ".join(fmt(float(b) * F) for b in spec["border"].split())
        lines = ["scaling = nine_slice", f"border = {border}", f"size = {fmt(w)} {fmt(h)}"]
    path = os.path.join(GUI, name + ".gui")
    if lines:
        open(path, "w").write("\n".join(lines) + "\n")
    elif os.path.exists(path):
        os.remove(path)  # a picture: stretched over the box it is given


def hue(im, shift, sat=1.0):
    """The same part in another color: each pixel's hue turned by `shift`."""
    a = np.array(im).astype(float) / 255
    out = a.copy()
    for y in range(a.shape[0]):
        for x in range(a.shape[1]):
            r, g, b, al = a[y, x]
            if al == 0:
                continue
            h, s, v = colorsys.rgb_to_hsv(r, g, b)
            out[y, x, :3] = colorsys.hsv_to_rgb((h + shift) % 1, min(1, s * sat), v)
    return Image.fromarray((out * 255).round().astype(np.uint8), "RGBA")


def muted(im, sat=0.3, val=0.62):
    """The same frame in weathered bronze: its copper less saturated and darker, as the HUD
    mockup (`notes/ui-reference/hud-in-game-mockup.png`) has its slots, so only what is selected
    stays bright amber. The dark insides are left as they are."""
    a = np.array(im).astype(float) / 255
    rgb = a[..., :3]
    mx, mn = rgb.max(-1), rgb.min(-1)
    frame = (mx > 0.25) & ((mx - mn) / np.maximum(mx, 1e-6) > 0.35)
    grey = rgb.mean(-1, keepdims=True)
    toned = (grey + (rgb - grey) * sat) * val
    a[..., :3] = np.where(frame[..., None], toned, rgb)
    return Image.fromarray((a.clip(0, 1) * 255).round().astype(np.uint8), "RGBA")


def faded(im):
    """An emptied icon: its shape in dark brown, half seen."""
    a = np.array(im)
    on = a[..., 3] > 0
    lum = a[..., :3].mean(-1, keepdims=True) / 255
    dark, light = np.array([40, 28, 22.0]), np.array([96, 72, 56.0])
    a[..., :3] = np.where(on[..., None], dark + (light - dark) * lum, 0).astype(np.uint8)
    a[..., 3] = np.where(on, 200, 0)
    return Image.fromarray(a, "RGBA")


def nine(im, b, w, h):
    """A nine-slice part laid out at w x h image pixels, its corners b pixels."""
    sw, sh = im.size
    out = Image.new("RGBA", (w, h))
    for sx0, sx1, dx0, dx1 in ((0, b, 0, b), (b, sw - b, b, w - b), (sw - b, sw, w - b, w)):
        for sy0, sy1, dy0, dy1 in ((0, b, 0, b), (b, sh - b, b, h - b), (sh - b, sh, h - b, h)):
            out.paste(im.crop((sx0, sy0, sx1, sy1)).resize((dx1 - dx0, dy1 - dy0), Image.LANCZOS), (dx0, dy0))
    return out


def half(full, empty):
    """Left half full, right half empty."""
    a, b = np.array(full), np.array(empty)
    w = a.shape[1]
    a[:, w // 2:] = b[:, w // 2:]
    return Image.fromarray(a, "RGBA")


def main():
    os.makedirs(GUI, exist_ok=True)
    for name, part in PARTS.items():
        im, spec = load(part)
        write(name, im, spec)
    # the menus' background: a tile at its own size (64 GUI px), not shrunk with the slots
    im, _ = load("background")
    im.save(os.path.join(GUI, "background.png"))
    open(os.path.join(GUI, "background.gui"), "w").write("scaling = tile\nsize = 64 64\n")
    # the red quit button: the buttons turned from amber to red
    for state in ("", "_hover", "_pressed"):
        im, spec = load("button" + state)
        write("button_quit" + state, hue(im, -0.075, 0.9), spec)
    # the advancement toast is drawn as a picture in text ({img:..}), a GUI pixel an image pixel:
    # the kit's toast laid out at its box's size, 160 x 32
    im, spec = load("toast")
    write("advancement_toast", muted(nine(im, 16, 320, 64).resize((160, 32), Image.LANCZOS)), {})
    # the HUD's own frames in weathered bronze (the hotbar, the shortcuts, the clock), as the
    # mockup: bright amber only for the selected slot
    for name, part in (("hotbar", "slot"), ("tooltip", "tooltip")):
        im, spec = load(part)
        write(name, muted(im), spec)
    # the vitals' bars (`hud.rs`): the kit's frame as a picture, and each fill laid on a canvas the
    # frame's size where it sits inside it, so both stretch alike over the bar
    frame, _ = load("bar")
    for bar, part in (("hud_health", "bar_health"), ("hud_stamina", "bar_stamina"), ("hud_xp", "bar_xp")):
        write(bar, muted(frame), {})
        fill, _ = load(part)
        canvas = Image.new("RGBA", frame.size)
        canvas.alpha_composite(fill, ((frame.width - fill.width) // 2, (frame.height - fill.height) // 2))
        write(bar + "_fill", canvas, {})
    # the thermometer (`hud.rs`): the kit's temperature bar fills from cold up to the body's
    # temperature, over the same bar dimmed; an icon for each feeling
    bar, _ = load("temperature_bar")
    a = np.array(bar).astype(float)
    grey = a[..., :3].mean(-1, keepdims=True)
    a[..., :3] = grey * 0.35 + a[..., :3] * 0.1
    write("hud_temperature", Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA"), {})
    write("hud_temperature_fill", bar, {})
    for feel in ("freezing", "cold", "comfortable", "warm", "hot"):
        im, _ = load("temperature/" + feel)
        write("temp_" + feel, im, {})
    # an icon for each of the vitals' effects (`vitals::effects::Effect::icon`)
    for name, part in EFFECT_ICONS.items():
        im, _ = load(part)
        write("effect_" + name, im, {})
    # the glow over a hovered slot (drawn over the item): amber, mostly see-through
    glow = np.zeros((32, 32, 4), np.uint8)
    glow[..., :3] = (255, 200, 110)
    yy, xx = np.mgrid[0:32, 0:32]
    edge = np.minimum(np.minimum(xx, 31 - xx), np.minimum(yy, 31 - yy))
    glow[..., 3] = np.clip(70 - edge * 4, 34, 70).astype(np.uint8)
    write("slot_highlight", Image.fromarray(glow, "RGBA"), {})
    # HUD points: full from the kit's status icons, empty faded, half both
    for name, part in (("heart", "status/health"), ("food", "status/food"), ("air", "status/wet")):
        full, _ = load(part)
        empty = faded(full)
        write(name + "_full", full, {})
        if name != "air":
            write(name + "_empty", empty, {})
            write(name + "_half", half(full, empty), {})
    print(f"{len(PARTS) + 3 + 2 + 7} parts into {os.path.normpath(GUI)}")
    print("still to draw:")
    for k, v in MISSING.items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
