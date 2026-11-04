"""The game's house style as numbers: what every model shares so they all sit together, a creature
next to a prop next to a nest. Build scripts import these instead of choosing their own.

- DENSITY: texels per block (a block is a meter). Every model: creatures, props, placeable blocks
  with their own model, eggs. 64, Hytale's density for characters. Terrain blocks keep the base
  game's 16-pixel textures; inventory icons are 16x16 too (ICON).
- A model is laid out in units of one texel (1/64 m): a 20 cm chihuahua is 13 units at the
  shoulder, a 1.5 m Utahraptor 96 at the hip. SCALE turns units into the engine's model pixels
  (to_bedrock's `scale`); BLOCK_UNITS units make a block (to_java's elements).
- Light and shading come from `paint` (one light from above and a bit front-left, hue-shifted
  ramps in quarter steps, ambient occlusion): never paint light by hand.
- Colors come from MATERIALS: natural, a little muted, the shadow hue by what the thing is made of
  (warm for anything alive or earthen, cool for stone, metal and shell).
"""

from paint import tones

DENSITY = 64
BLOCK_UNITS = DENSITY
SCALE = 16 / DENSITY
ICON = 16

# Small creatures (a mouse, a vole, a small lizard: a head only 2-3 units across) keep their
# layout in house units but are painted FINE texels a unit (256 a block): at 64 a block their eye
# is a whole black side of the head and every face one flat color. Their faces get FINE times the
# texels; the boxes, the export scale and the engine's size stay the same.
FINE = 4

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
