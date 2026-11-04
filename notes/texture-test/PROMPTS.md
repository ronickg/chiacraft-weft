# Block texture test: ChatGPT prompts

Attach with the master prompt (new chat):
- `notes/texture-test/palette.png`: the colours to use (first draft of the shared palette)
- `notes/ui-reference/fantasy-pixel-ui-reference-poster.jpg`: the game's warm mood

Don't attach the old block textures (`current-blocks-tiled.png`): they are grey and grainy, the
look we are moving away from.

## 1. Master prompt (paste once, with the two images)

```text
For this whole chat you make square block textures for my game, one material per image, all in one consistent style. The attached palette gives the colours; the attached UI sheet gives the warm mood.

Every texture:
- A 64 x 64 pixel game texture, enlarged with square pixels: soft painterly clusters in big simple shapes of three sizes, smooth light in fine steps, like a small hand-painted illustration. Not a photo, not a 3D render.
- Colours from the attached palette only: warm and earthy, shadows warmer and slightly purple, highlights toward yellow, never pure black or white.
- Each small shape (stone, board, leaf clump) lit on its own from the top left, with a soft shadow at its bottom right; the whole tile evenly bright, no light-to-dark gradient across it.
- Seen straight on, flat, filling the whole image edge to edge, repeating seamlessly on all four sides, no border or dark edges, no standout feature.
- Square, 1024 x 1024, PNG.

Reply "ready" and wait for my first material.
```

## 2. One message per material

```text
Stone: rounded stones of three sizes packed together, soft grey-brown from the "stone" row, darker gaps between them, a few faint cracks.
```

When you like the stone, it is the style key. For each next material:

```text
Next: [material]. Match the stone you made exactly in shape size, softness and lighting.
```

- `Dirt: soft clumps of earth in three sizes, small pebbles here and there, warm red-brown from the "soil" row.`
- `Sand: fine rippled sand with small soft grains, pale gold from the "sand" row.`
- `Oak planks: four horizontal boards, chunky grain lines, small dark knots, honey-brown from the "wood" row.`
- `Grass top: dense short grass blades in soft clumps, seen from above, fresh green from the "grass" row.`
- `Bricks: staggered rectangular bricks, three rows, soft mortar lines, warm red-brown from the "soil" and "copper" rows.`

If it drifts toward a photo or a 3D render: start a new chat, attach the palette, the UI sheet
and your approved stone, and add to the first line: "small game texture enlarged with hard
square pixels".

## 3. Gemini test

The same master prompt and the same two images (plus your approved stone) in Gemini, then the
same materials. Compare which one keeps closer to the palette and drifts less into realism.

## What a pass looks like

- The materials look like one family, and like the UI's warm mood.
- Put a texture next to itself four times (2 x 2): no visible seam, no obvious repeating blotch.
- Shrunk to a thumbnail it still reads as stone, dirt, planks.

## Generating many at once (Codex, no copying)

`codex/gen_grid.sh NAME "top left" "top right" "bottom left" "bottom right"` makes four block
textures in one image with the palette and `codex/approved_blocks.png` attached (the prompt is
`codex/grid_common.txt`); `codex/gen_sprites.sh` the same for plants and other see-through pictures
on magenta. `codex/batches*.tsv` list what was made; `run_batches.sh` runs a list two at a time.
Then:

    python3 notes/texture-test/grid_cut.py codex/grid_x.png name1 name2 name3 name4   # into tiles/
    python3 notes/texture-test/install_blocks.py                                       # into the game

`process.py` knows which textures the game tints (grass, leaves: made grey) and which are
see-through (leaves, plants, the ladder); `grid_cut.py` which are one picture rather than a
surface that runs on (their edges are not evened out). Item icons: `notes/ui-codex/gen_items.sh`.
