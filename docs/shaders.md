# Shaders guide

Small shader files can change how text, panels, mobs, particles and the whole screen look, and shader packs can replace the engine's own shaders altogether.\
Everything is written in WGSL and shipped as assets, so a mod needs no engine changes and next to no code.

## Note
Examples to copy from, all in `mods/testmod/assets/shaders/`:
- `fx/glitch.wgsl`: letters that jump sideways
- `gui/circuit.wgsl`: a panel with pulses running along traces
- `entity/ghost.wgsl`: a mob that breathes and has holes sweeping across it
- `block/stripes.wgsl`: magenta bands sliding up a block
- `particle/twinkle.wgsl`: sparks that flicker and shift color as they age
- `post/dream.wgsl`: a screen effect that sways and warms the picture

The engine's own text effects are in `crates/engine/assets/shaders/fx/`.

## Quick links
[snippets](#snippets) | [text effects](#text-effects) | [panels](#panels) | [entities](#entities) | [blocks](#blocks) | [particles](#particles) | [screen effects](#screen-effects) | [shader packs](#shader-packs)

## Snippets
A snippet is a file `shaders/<kind>/<name>.wgsl` in your mod, holding one function whose name depends on the kind. The engine puts all snippets of a kind into a single shader and picks one by number for each text, panel, mob or particle, so a hundred snippets don't cost a hundred shaders.

| kind | folder | function | used by |
|---|---|---|---|
| text effects | `shaders/fx/` | `fn effect(g: Glyph) -> Glyph` | `{fx:mymod:name}` in a translation |
| panels | `shaders/gui/` | `fn fill(p: GuiPixel) -> vec4<f32>` | a class's `.gui`: `shader = mymod:name` |
| entities | `shaders/entity/` | `fn shade(p: EntityPixel) -> vec4<f32>` | `r.entity(..).shader("name")` |
| blocks | `shaders/block/` | `fn shade(p: BlockPixel) -> vec4<f32>` | `r.block(..).shader("name")` |
| particles | `shaders/particle/` | `fn shade(p: ParticleFrag) -> vec4<f32>` | `r.particle(..).shader("name")` |
| sky | `shaders/sky/` | `fn shade(p: SkyPixel) -> vec4<f32>` | `r.sky_body(..).shader("name")`, see the [sky guide](sky.md#shaders) |
| screen effects | `shaders/post/` | `fn apply(p: PostPixel) -> vec4<f32>` | `client.post_effect("mymod:name", strength)` |

The file name is the snippet's name, so `shaders/fx/glitch.wgsl` in `testmod` is `testmod:glitch`. Whatever a function can see is in the struct it takes, and all of them carry `time`, the seconds since the game started. A resource pack can replace a snippet the way it replaces a texture, and a mistake in one shows up in the log with the file and the error.

## Text effects
An effect moves and tints each letter of the text between `{fx:mymod:name}` and `{/fx}`:
```wgsl
fn effect(g: Glyph) -> Glyph {
    var o = g;
    let t = floor(g.time * 8.0);
    let h = fract(sin(g.index * 91.7 + t * 13.1) * 43758.5453);
    if h > 0.8 {
        o.offset.x = (h - 0.9) * 20.0 * g.scale;
    }
    return o;
}
```
`Glyph` has `offset` (pixels added to the letter's corner), `color` (linear, with alpha), `time`, `index` (the letter's number in its text, handy for waves), `corner` (which corner of the letter, 0 to 1 on each axis), `scale` (physical pixels per GUI pixel) and `shadow`, which is true for the dark copy drawn under the text. Leave the shadow's color alone, or it turns bright.

The function runs for every corner of every letter, so it pays to keep it small. How to use the effects inside translations is in the [tooltips guide](localization.md#effects).

## Panels
A class's `.gui` (next to its texture) can name a shader, and the panel is filled by it instead of the picture:
```
shader = testmod:circuit
```
```wgsl
fn fill(p: GuiPixel) -> vec4<f32> {
    let edge = min(min(p.uv.x, 1.0 - p.uv.x) * p.size.x, min(p.uv.y, 1.0 - p.uv.y) * p.size.y);
    let border = step(edge, 3.0);
    return vec4(vec3(0.02) + vec3(0.25) * border, 0.97);
}
```
`GuiPixel` has `uv` (0 to 1 across the panel), `size` (in pixels), `color` (linear) and `time`. More on how classes look is in the [UI guide](ui.md#looks).

## Entities
An entity shader colors the pixels of a mob. It takes over from the normal lit skin, so it also decides how light affects the mob:
```wgsl
fn shade(p: EntityPixel) -> vec4<f32> {
    let pulse = 0.75 + 0.25 * sin(p.time * 3.0);
    let lit = max(p.light, vec3(0.35)) * pulse;
    return vec4(p.albedo.rgb * lit, p.albedo.a);
}
```
```rust
r.entity(&WISP).texture("wisp", 32, 16).model(..).shader("ghost");
```
`EntityPixel` has `uv`, `albedo` (the skin's texel, linear, `a` is its alpha), `light` (a vec3 with shading, the hurt flash and glow already included) and `time`. A returned alpha under 0.5 cuts the pixel out, which is how the wisp gets its holes. The `_e` glow masks of the [textures guide](textures.md#emission) are a separate and simpler way to make parts of a skin shine.

## Blocks
A block shader colors the pixels of a block's faces and takes over from the normal lighting the same way:
```wgsl
fn shade(p: BlockPixel) -> vec4<f32> {
    let band = step(0.5, fract(p.pos.y * 2.0 - p.time));
    return vec4(mix(p.albedo.rgb * p.light, vec3(0.8, 0.05, 0.8), band * 0.7), p.albedo.a);
}
```
```rust
r.block(&REINFORCED_STONE).shader("stripes");
```
`BlockPixel` has `uv` (across the face), `albedo` (the texel with its overlay, linear), `light` (shading, AO, the biome tint and glow already in), `pos` (block coordinates of the pixel, handy for patterns that run across blocks) and `time`. Fog goes on top of what it returns. The shader goes with the block's textures, so two blocks sharing a texture share it.

## Particles
A particle shader colors a particle before lighting and fog are applied:
```wgsl
fn shade(p: ParticleFrag) -> vec4<f32> {
    let flicker = 0.7 + 0.3 * sin(p.time * 30.0 + p.life * 20.0);
    return vec4(p.albedo.rgb * flicker, p.albedo.a * p.color.a);
}
```
```rust
static SPARK: Particle = Particle::named("spark");
r.particle(&SPARK).textures(&["glint_0", "glint_1"]).life(0.3, 0.6).fullbright().shader("twinkle");
```
`ParticleFrag` has `uv` (0 to 1 across the quad), `albedo` (the texture sample), `color` (the kind's color at this age, between the two you gave to `color(start, end)`), `life` (0 at birth, 1 at the end) and `time`.

The rest of the builder is `life(min, max)` in seconds, `size(min, max)` in blocks, `gravity(g)` (negative rises), `drag(d)`, `colliding()`, `fullbright()`, `translucent()` and `shrinking()`. From the server, `world.spawn_particles(&SPARK, pos).count(12).spread(0.4).speed(2.0)` makes them appear for everyone nearby. Registering a particle named `core:block` or `core:poof` changes how the engine's own block dust and smoke look.

## Screen effects
A screen effect works on the finished picture of the world, and fades in and out with a strength from 0 to 1:
```wgsl
fn apply(p: PostPixel) -> vec4<f32> {
    let wave = vec2(sin(p.uv.y * 18.0 + p.time * 2.0), 0.0) * 0.006 * p.strength;
    let c = scene_at(p, p.uv + wave).rgb;
    return vec4(c * mix(vec3(1.0), vec3(1.15, 1.0, 0.8), p.strength), 1.0);
}
```
```rust
static DREAM: Key = Key::named("dream");
r.key(&DREAM, "N").on_press(|c| c.post_effect("dream", 1.0));
```
`PostPixel` has `uv` (0 to 1, top left first), `color` (the picture so far at that point, linear), `strength`, `time` and `size` in pixels, and `scene_at(p, uv)` reads the picture anywhere else, for blur and waves. `post_effect(name, 0.0)` turns an effect off, and several can be on at once, applied one after another in the order they were turned on.

## Shader packs
Snippets add to the engine's shaders, a shader pack replaces them. It is a folder `shaderpacks/<id>/` next to the game, chosen by players in the settings, with any of these files:

| file | what it draws |
|---|---|
| `world.wgsl` | blocks and liquids |
| `model.wgsl` | mobs and the player's model |
| `entity.wgsl` | the first-person hand and held items |
| `particle.wgsl` | particles |
| `weather.wgsl` | rain and snow |
| `sky.wgsl` | the sky, what is in it, and the overlay from inside a liquid |
| `post.wgsl` | screen effects |
| `ui.wgsl` | all screens and text |

A file replaces its shader completely and the ones a pack doesn't have stay as they were, so the easiest start is a copy of the original from `crates/engine/assets/shaders/`. A replacement writes only functions (and its own private structs like `VOut`): the buffer structs, the bindings, the constants and the shared helpers of `common.wgsl` come from the engine's prelude (`crates/engine/src/gpu_layout.rs`), so a pack can't get them wrong and keeps working when they change. It has to keep the entry points. If it doesn't compile, the log says why and the built-in shader is used instead, so a broken pack can't leave anyone with a black screen. Mods' snippets keep working inside a pack's shaders, and an optional `pack.cfg` (`name = ...`, `description = ...`) names the pack in the list. `sun_shadows = true` there has the engine draw the terrain from the sun (or the moon) into a shadow map each frame, which the pack reads through `scene.shadow_view_proj` and `scene.shadow` (its bindless index, size, whether it holds this frame, blocks per texel); the pack's `world.wgsl` then needs an `fs_shadow` fragment entry (depth only, it may `discard` see-through texels). With an `fs_lod_shadow` entry as well (depth only; it should `discard` where `fs_lod` does, the columns real cubes show), the distant terrain casts into the far cascade too, so hills past the render distance throw shadows. The far map (`scene.shadow_far_view_proj`, `scene.shadow_far`, 400 blocks to each side) is drawn again only when the light or the terrain changed. The settings `core:shadows` (0 off, 1 near map only, 2 near and far) and `core:shadow_size` (1024, 2048 or 4096 texels) choose how much of this the engine draws. `translucent_depth = true` there makes water, glass and other see-through blocks write depth, so screen effects see their surface (water reflections); blended particles behind them are then hidden.

### What the frame knows
Every shader's prelude has `scene`: `fog_color` (linear rgb, and in `w` how hard it rains, 0 to 1), `params` (x and y where the fog starts and ends, z the daylight from 0 to 1, w seconds), `light` (x the dimension's ambient light, y the Brightness setting, z game ticks), `sun` (below), `weather` (x and y the wind where the player is along the world's x and z, in m/s the way it blows, z how foggy it is there from 0 to 1, w how hard it thunders; still and clear unless a mod sets the air with `World::set_air`) `eye` (the camera in the world, x and z wrapped every 8192 blocks) and `ground` (x the snow lying where the player is from 0 to 1, y how much of what falls is snow, z how much dust blows). `clouds` and `cloud_cells` describe the sky's cloud layer, for cloud shadows: `clouds.x` its picture's bindless index plus one (0 when there are no clouds), `y` the blocks one picture cell covers, `z` the height of its underside over the camera, `w` how much of the picture's gaps the weather fills; `cloud_cells.xy` is the camera's place in its cell (0 to 1) and `zw` that cell in the picture. `moon` points toward the moon where the sky draws it (xyz) and in `w` says how much light it gives: its phase times how high it stands, 0 when it is down or new; for night light and moon shadows (the shadow maps follow it at night). `options` holds the values of up to 16 settings the pack names in its `pack.cfg` (`options = base:look_exposure, base:look_sun, ...`), four to a row in that order (0 for one the engine can't find): a pack's look can then be tuned in the game's settings, or for one run with `--config`, without editing its shaders. `hand` and `hand_light` are the light the player holds, for dynamic light: `hand.xyz` where it is (camera relative, by the hand), `hand_light.rgb` its block light levels from 0 to 1 (a torch 14/15), all zeros when what is held gives no light; it should fall off as block light does, a level a block. `canopy.x` is how closed a forest's canopy is where the player is, 0 (open sky) to 1 (a rainforest's), from the biome's air (`air_flags::CANOPY_SHIFT`), for the light under the leaves.

### Screen effects
A screen effect (`shaders/post/<name>.wgsl`, `fn apply(p: PostPixel) -> vec4<f32>`, turned on by a mod with `Client::post_effect`) reads the picture so far through `p.scene` and also gets `scene`, the frame the world was drawn with: `scene.view_proj * vec4(scene.sun.xyz * 1000.0, 1.0)` puts the sun on the screen, as the base game's light shafts do (`mods/base/assets/shaders/post/shafts.wgsl`). Without a world it is all zeros.

### Sun shadows
A pack that wants shadows gets the two things it needs from the world shader's prelude: `scene.sun` (xyz points toward the sun, w is how high it stands, 0 at night and in dimensions without sky) and `solid_at(p: vec3<i32>) -> bool`, which says whether the block at `p` casts a shadow (anything that dims sky light, liquids aside). It knows the blocks about 112 blocks around the camera and answers false beyond them, so march from the pixel toward `scene.sun.xyz` through block cells and stop at the first solid one. The engine only keeps these blocks on the GPU while the chosen pack's `world.wgsl` calls `solid_at`, so packs without shadows pay nothing.

Textures and models stay with resource packs, see the [resource packs guide](resourcepacks.md).

### Tuning chiacraft's look

chiacraft's pack reads these settings (`mods/base/src/look.rs`; Mod Settings in the game; `--config "base:<name>=<value>"` for one run, so a PC job can sweep a few values). Each is 1 (or 0) at the look as made, but `look_haze` (1.6, Ronald's pick):

| setting | default, range | what it does |
|---|---|---|
| `base:look_exposure` | 1, 0.5-2 | how bright the day is taken |
| `base:look_sun` | 1, 0.5-2 | the sun's strength by day |
| `base:look_sun_warmth` | 0, -1..1 | the sun's colour: -1 whiter, +1 toward a 4300 K orange (0: 6300 K) |
| `base:look_fill` | 1, 0.5-2 | the sky's light in the shade by day |
| `base:look_fill_cool` | 0, -1..1 | the shade's colour: -1 neutral, +1 bluer (0: blue 1.4x red) |
| `base:look_contrast` | 1, 0.8-1.3 | the day curve's contrast (Lottes's a, 1.2 as made) |
| `base:look_saturation` | 1, 0.7-1.3 | saturation after the curve, by day (1.2 as made) |
| `base:look_shade` | 1, 0.5-2 | how fast the sky's light fades under leaves and roofs (more: darker forest floors) |
| `base:look_haze` | 1.6, 0-3 | the air's veil over distance: clear air blues the far land (more in blue than red, so it stays crisp); rain, the weather's fog and a forest's damp air add a grey haze. (Ronald's pick, 1.6; 1 is crisper). Rain's own mist does not follow it |
| `base:look_fog` | 1, 0-2 | how bright fog is |
| `base:night_light` | 1, 0.3-2 | the night's sky light (nights are dark by design) |
| `base:night_moon` | 1, 0.3-2 | the moon's light at night |
| `base:look_canopy` | 1, 0.2-2 | under a forest's canopy (`vitals::air::CANOPY`): how dim and green the shade under the leaves is, and how freely the sunflecks come through |
| `base:look_shafts` | 1, 0.2-3 | how strong the light shafts through a forest's crown gaps are (most in morning mist) |

