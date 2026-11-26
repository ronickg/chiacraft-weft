# Reference images: the best-regarded animal mods and Mojang's own

Downloaded 2026-11-05 (Ronald approved, official pages only: Modrinth galleries, minecraft.wiki,
Mojang's bedrock-samples) for internal reference. They are not in the repository: they live in
`~/Coding/chiacraft-archive/references/models/<mod>/`, each folder with a `CREDITS.md` (mod,
author, page and image URL). `FINDINGS.md` there has the measurements (box counts and sizes,
textures, idle loops) behind "What the best ones do" in SKILL.md. All of them are 16 px a
block: double their numbers for our 32.

Paths below are under `~/Coding/chiacraft-archive/references/models/`.

Comparing our cut-out planes (antlers, crests, fins) with these: `scripts/render.py` draws a plane's back lined up in 3D, as the game does (fixed 11-05; before, a lopsided plane drew reversed from behind, and the reindeer's antler read as a ring in shots that the game drew right). Shots of a plane from behind made before that fix are wrong.

## Mojang (minecraft.wiki renders and textures; `vanilla/geo/` the bedrock-samples geometry)
- `vanilla/vanilla_sniffer_render.png`, `vanilla_sniffer_texture.png`: the moss coat as ONE shell a little bigger than the body, its hem cut out ragged, shag painted as streaks (not tufts).
- `vanilla/vanilla_camel_render.png`, `vanilla_camel_texture.png`: the hump flush on the back, 3 px in each side; a neck column; a muzzle box; soft light painted per face on a 128 sheet.
- `vanilla/vanilla_armadillo_render.png`, `vanilla_armadillo_texture.png`: a shell the body's size, notched cut-out edges; bands running on down the sides.
- `vanilla/vanilla_frog_temperate_render.png`, `vanilla_frog_temperate_texture.png`: head and body split at the mouth; eye boxes on top; feet as planes.
- `vanilla/vanilla_axolotl_lucy_render.png`, `vanilla_axolotl_lucy_texture.png`: 7 of 11 parts are cut-out planes (gills, limbs, tail).
- `vanilla/geo/{camel,sniffer,armadillo,frog,axolotl}.geo.json`: the real boxes, to count and measure.

## Naturalist (Starfish Studios; Modrinth gallery)
- `naturalist/naturalist_bears_gallery.png`: a big head, a snout box out from the skull, dark paw band; the cub's head bigger for its size.
- `naturalist/naturalist_rhinos_gallery.png`: the front body taller and wider than the rear, stepped in evenly on both sides.
- `naturalist/naturalist_deer_gallery.png`: the neck its own column, pale chest and belly, dark hoof band, an eye highlight, cut-out antlers.
- `naturalist/naturalist_giraffes_gallery.png`: patches painted straight across box seams.
- `naturalist/naturalist_lions_gallery.png`: the mane as one box with a slightly larger shaggy copy (small in frame).
- `naturalist/naturalist_elephants_gallery.png`: a trunk in jointed segments, column legs.

## Alex's Mobs (sbom_xela; Modrinth gallery)
- `alexs-mobs/alexsmobs_gorilla_gallery.png`: chest far bigger than the hips, arms twice the legs: front-heavy.
- `alexs-mobs/alexsmobs_elephant_gallery.png`: a trunk in 3 jointed boxes, flat ear plates, wrinkles painted.
- `alexs-mobs/alexsmobs_roadrunner_gallery.jpeg`: crest and tail as cut-out planes, stick legs.
- `alexs-mobs/alexsmobs_froststalker_gallery.png`, `alexsmobs_sunbird_gallery.jpeg`: spike and feather planes (small in frame).

## Friends & Foes (Faboslav; Modrinth gallery)
- `friends-and-foes/fnf_tuff_golem_gallery.webp`: a coloured iris with one highlight pixel.
- `friends-and-foes/fnf_mauler_desert_gallery.webp`: a white of the eye beside the pupil; one simple body box.
- `friends-and-foes/fnf_rascal_gallery.webp`: big eyes with a highlight, shag painted on (dark shot).
- `friends-and-foes/fnf_moobloom_buttercup_gallery.webp`: cheek blush; flowers as crossed planes.
- `friends-and-foes/fnf_crab_waving_gallery.webp`: one oversized claw, eye stalks.

## Critters and Companions (joshieman; Modrinth gallery)
- `critters-and-companions/cnc_otter_gallery.png`: whiskers as planes, a pale face mask.
- `critters-and-companions/cnc_red_panda_gallery.png`: a chunky body, dark legs, face markings.
- `critters-and-companions/cnc_ferret_gallery.png`: a long low body (small in frame).
- `critters-and-companions/cnc_dumbo_octopus_gallery.png`: the head is the body; fin and arm boxes.

## Ecologics (SameDifferent; Modrinth gallery)
- `ecologics/ecologics_camel_desert_gallery.png`: shag painted as streaks; dark hooves.
- `ecologics/ecologics_squirrel_plains_gallery.png`: a sitting pose, teeth painted; vanilla pigs for scale.
- `ecologics/ecologics_penguin_snowy_gallery.png`: a two-tone body, flipper planes.
- `ecologics/ecologics_coconut_crab_gallery.jpg`: a shell box over segmented legs.

## Untamed Wilds (raytrace82; from the Modrinth gallery of its GPL port, same models)
- `untamed-wilds/untamedwilds_shark_modrinth_port.png`: the body tapering in steps; dark top, pale belly.
- `untamed-wilds/untamedwilds_giant_salamander_modrinth_port.png`: a flat wide head, splayed legs.
- `untamed-wilds/untamedwilds_sunfish_modrinth_port.png`: a tall flat body, speckles painted.

The sheet shown to Ronald: `~/Coding/chiacraft-archive/images/model-skill/2026-11-05/model_refs_sheet.png`.


# Eggs (2026-11-06)

Saved 2026-11-06 (Ronald approved, official pages only: minecraft.wiki, the mods' Modrinth/CurseForge
and GitHub pages) to `~/Coding/chiacraft-archive/references/eggs/<source>/`, each with a
`CREDITS.md`; `FINDINGS.md` there; the study behind them `reports/Eggs in Minecraft mods.md`.
16 px a block: double their numbers for our 32. Used by "Eggs" in SKILL.md.

- `eggs/vanilla/vanilla_turtle_egg_block_texture.png`: 4 eggs' UV patches at 1:1, cream with single-texel teal speckles.
- `eggs/vanilla/vanilla_egg_item.png`: the 10 x 11, 6-colour reference icon, outline in a darkened hue.
- `eggs/vanilla/vanilla_turtle_egg_item.png`: a speckled icon, 12 x 13, two-tone outline, 2 x 2 highlight.
- `eggs/vanilla/vanilla_sniffer_egg_item.png`: a huge egg fills the slot (16 x 16), tilted.
- `eggs/vanilla/vanilla_sniffer_egg_north_texture.png`: bold two-hue patches on a big egg; each face of the one box has its own texture.
- `eggs/vanilla/vanilla_dragon_egg_texture.png`: 3-colour noise; the stepped boxes carry the shape.
- `eggs/vanilla/vanilla_turtle_egg_4_render.png`: one box per egg (4 x 7 down to 3 x 4 MC px), about 5-10x life; the texture does the rounding.
- `eggs/vanilla/vanilla_sniffer_egg_render.png`: one 14 x 16 x 12 box, nearly a whole block, for a huge fantasy egg.
- `eggs/vanilla/vanilla_dragon_egg_render.png`: the showpiece exception: 6 stepped boxes make a round egg; 1 px steps.
- `eggs/alexs-mobs/alexsmobs_crocodile_egg_block.png`: the dark ground band on the bottom rows.
- `eggs/alexs-mobs/alexsmobs_terrapin_egg_block.png`: a 32 px atlas used at 16 texels a block; brown speckles.
- `eggs/alexs-mobs/alexsmobs_emu_egg_item.png`: a big real egg gets a 14 x 16 icon.
- `eggs/alexs-caves/alexscaves_vallumraptor_egg.png`: a 4-egg clutch atlas, two-hue single-texel speckles.
- `eggs/alexs-caves/alexscaves_atlatitan_egg_side.png`: light top to dark base on a full-block egg.
- `eggs/prehistoric-fauna/prehistoricfauna_allosaurus_egg.png`: banding instead of spots, 8 colours.
- `eggs/fossils/fossils_egg_triceratops.png`: 64-colour low-contrast noise (what not to do).
- `eggs/jurassicraft/jurassicraft_velociraptor_egg.png`: 128 x 64 at 32 texels a block, airbrushed (what not to do beside 16 px blocks).
- `eggs/unusual-prehistory/unusualprehistory_small_egg_base.png`: a 4-grey tintable base (the spawn-egg method).
- `eggs/naturalist/naturalist_alligator_egg_block.png`: the vanilla turtle-egg layout, plain cream (reuses vanilla's turtle egg templates).

# 3D plants (2026-11-06)

Saved the same way to `~/Coding/chiacraft-archive/references/plants/<source>/`; the survey
`reports/3D plants in block games.md`. Used by "3D plants" in SKILL.md.

- `plants/fresh-flowers-and-plants/ffp_cherry_grove.webp`: pink petals and sapling as 3D models next to vanilla terrain.
- `plants/fresh-flowers-and-plants/ffp_birch_forest.webp`: wildflower ground cover, a dense field look.
- `plants/fresh-flowers-and-plants/ffp_lush_cave.webp`: flowering azalea as a bush model.
- `plants/fresh-flowers-and-plants/ffp_savanna.webp`: tall grass as a 3D model, biome tinted.
- `plants/fresh-flowers-and-plants/ffp_desert.webp`: dry grass and dead bush, sparse 3D plants.
- `plants/3d-crops-revamped/3dcrops_field.webp`: box-model crops in a farm.
- `plants/vanilla-plus-crops/vpcrops_1.webp`: bushy 4-6 plane crops, tilted planes.
- `plants/vanilla-plus-crops/vpcrops_2.jpeg`: same bushy tilted-plane crops, another view.
- `plants/vanilla-plus-crops/vpcrops_3_tall_wheat.jpeg`: tall mature wheat past the block top (32 x 32 textures).
- `plants/better-foliage/betterfoliage_extra_leaves.jpg`: 45-degree leaf bits breaking the cube outline.
- `plants/better-foliage/betterfoliage_short_grass.jpg`: grass tufts added on grass blocks: a field from extra quads.
- `plants/better-foliage/betterfoliage_reeds.jpg`: reeds in shallow water.
- `plants/better-foliage/betterfoliage_lilypads.jpg`: lily pad with roots and flower: layered planes.
- `plants/better-foliage/betterfoliage_cactus.jpg`: rounded cactus core with arms: boxes.
- `plants/better-leaves/betterleaves_performance.png`: the bushy leaf look: plane layers sticking out of a leaf cube (the report calls this the v9 performance banner).
- `plants/vanilla-3d-extension/v3dext_overview.png`: subtle vanilla-style 3D on blocks (the report expected plants too; this image shows only blocks).
- `plants/vanilla/vanilla_spore_blossom_render.png`: 1 flat base plus 4 petal planes tilted 22.5 deg out from the block edges (10 quads); the base sits 0.2 px from the petals to avoid z-fighting.
- `plants/vanilla/vanilla_azalea_render.png`: template_azalea: a cut-out cube shell (walls 0.01 px thick, pulled 0.01 px in) around a cross (14 quads); the bush-shell technique.
- `plants/vanilla/vanilla_flowering_azalea_render.png`: the same template_azalea shell, only the texture differs.
- `plants/vanilla/vanilla_pink_petals_4_render.png`: flowerbed: per segment a flat 8 x 8 px card near the ground plus 2 tiny crossed stems (6 quads a segment, 24 full).
- `plants/vanilla/vanilla_big_dripleaf_render.png`: a top plate at y 15, thin 0.002 px walls and 2 crossed stem planes at +-45 deg (12 quads).
