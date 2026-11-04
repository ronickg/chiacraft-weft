"""The game's house style as numbers: what every model shares so they all sit together, a creature
next to a prop next to a nest. Build scripts import these instead of choosing their own.

- DENSITY: texels per block (a block is a meter). 32, the same as the blocks and the items
  (Ronald, 2026-11-03: "Yes, rebuild at 32"): every model, creatures, props, eggs, nests. A small
  creature whose face would be lost at 32 paints finer, at a power-of-two multiple (`fine`).
- UNITS: layout units per block. Models are laid out in units of 1/64 m (a 20 cm chihuahua is 13
  units at the shoulder, a 1.5 m Utahraptor 96 at the hip), finer than the texels, so boxes can sit
  half a texel apart and poses keep their numbers. SCALE turns units into the engine's model
  pixels (to_bedrock's `scale`); TX is texels per unit (0.5).
- A model on its own scale (its units not 1/64 m) paints `texels(scale)` texels a unit.
- Light and shading come from `paint` (one light from above and a bit front-left, hue-shifted
  ramps in quarter steps, ambient occlusion): never paint light by hand.
- Colors come from MATERIALS: natural, a little muted, the shadow hue by what the thing is made of
  (warm for anything alive or earthen, cool for stone, metal and shell).
"""

from paint import tones

DENSITY = 32
UNITS = 64
BLOCK_UNITS = UNITS
SCALE = 16 / UNITS
TX = DENSITY / UNITS
ICON = 16


def texels(scale=SCALE, k=1):
    """Texels per layout unit for a model exported at `scale` model pixels a unit, painted at k
    times the house density (`fine`)."""
    return DENSITY * k * scale / 16


# Small creatures paint finer, so their face still carries an eye: the face the eyes sit on must be
# at least MIN_FACE texels across (both ways; a block-game parrot's is 2-3), painted at the least
# power-of-two multiple of DENSITY that gives it that. A wolf's face is 6 texels at 32, a cat's 5,
# a fox's 3.5: all 32. A raven's head is 2.5 (64), a mouse's 1.1 (128: 4.4). `fine(face_units)`
# gives the multiple for a face `face_units` layout units across (its smaller side).
MIN_FACE = 3
FINE = 4  # the old fixed factor (4 texels a unit, 256 a block); use `fine` and `texels` instead


def fine(face_units, scale=SCALE):
    k = 1
    while texels(scale, k) * face_units < MIN_FACE - 1e-6:
        k *= 2
    return k


# name: (base color, shadow hue). Living and earthen things have warm shadows, stone and metal
# purple ones, eggshell cool ones.
MATERIALS = {
    "fur": ((168, 120, 78), 0.02),
    "feather": ((150, 104, 70), 0.02),
    "skin": ((196, 150, 128), 0.02),
    "scale": ((112, 120, 84), 0.04),
    "horn": ((208, 196, 168), 0.05),
    "claw": ((58, 46, 44), 0.02),
    "soil": ((156, 118, 80), 0.04),
    "soil_dark": ((116, 86, 58), 0.04),
    "straw": ((206, 178, 112), 0.06),
    "twig": ((118, 86, 54), 0.04),
    "wood": ((150, 112, 70), 0.04),
    "stone": ((132, 128, 122), 0.74),
    "metal": ((150, 152, 160), 0.74),
    "shell": ((168, 198, 188), 0.62),
}


def material(name, rgb=None):
    """A material's tones (`paint.tones`), its own base color or `rgb` with its shadow hue."""
    base, hue = MATERIALS[name]
    return tones(rgb or base, hue)
