# Sound: how the best do it, where we are, what to build

Researcher, 2026-11-04, round 11, for Planner and Ronald.

**What was read:**
- `dev` at `62681181`: `crates/engine/src/audio.rs`, the registry and network paths for sounds, the SDK, `mods/base/src/sounds.rs`, `mobs.rs`, and the animals mod's sounds;
- how Minecraft, Sound Physics Remastered, AmbientSounds, Presence Footsteps, Dynamic Surroundings, Vintage Story, Valheim and Hytale do sound (sources at the end).

## The short answer

1. **The engine has more sound plumbing than the game uses.** Already wired:
   - sound events with random variants and pitch;
   - volume sliders per category;
   - 3D positioned sounds;
   - block and entity sound groups;
   - automatic footsteps, mining knocks, and break and place sounds;
   - mob idle, hurt and death calls;
   - a low-pass filter under water and a reverb in caves;
   - streamed music.

   Until yesterday almost all of it was silent because the files were missing. The quick win (merged as `99f901df`) filled the 7 block groups and the UI click and pickup.
2. **The big gap in code is one piece: sounds that loop and can be changed.** No play call returns a handle, so nothing can loop, fade, stop or follow a moving source. Every ambient bed, rain, wind, streams, fire and a creature's moving voice needs it. Kira 0.12 already does all of it (`loop_region`, `set_volume(Tween)`, `stop(Tween)`, `SpatialTrackHandle::set_position(Tween)`), so it's a medium-sized engine job, not a new audio engine.
3. **The best model to copy is Vintage Story's weather sound plus Hytale's data layout.**
   - Vintage Story uses a few looped beds whose volumes ease toward targets set by climate, plus one enclosure value that muffles them indoors.
   - Hytale uses sound events with round-robin and voice caps, ambience as beds plus scattered one-shots with light and time conditions, and music with long random silences.
   - Both are cheap. Ray-traced sound physics, as in Sound Physics Remastered, is not worth it for us yet.

## How the best do it

### Ambient beds
- **Minecraft:**
  - Cave "mood" samples one random block near the eye each tick. In full darkness it plays one of 23 cave sounds about every 5 minutes.
  - Nether biomes have one loop each, plus "additions" one-shots with set chances.
  - Underwater has a loop at 0.65 plus additions at 90%, 9% and 1%.
- **AmbientSounds (mod):**
  - Scans the surroundings every 2 s: height class (underground, surface, air), nearby biomes weighted by distance, and an "air pocket" flood fill plus sky visibility to tell open from enclosed.
  - Fades slowly, 0.005 per tick.
  - Conditions include rain, under water, temperature ranges and day/night weights.
- **Hytale (AmbienceFX):**
  - Looped beds with transition speeds.
  - One-shot "Sounds" with frequency and radius ranges, and a `SunlightRange` that tells inside from outside.
  - A cap of bodies per emitter, a priority, a list of ambiences it blocks, and a reverb or EQ choice per ambience.
  - From Hytale's 2020 blog: zones stack (wind + birds + cicadas) and affect each other, so a storm silences birds. Inputs include the share of nearby block types (trees bring wind in leaves).
- **AAA layering:**
  - 2D stereo beds for the steady background;
  - randomised one-shot "sweeteners" placed around the listener, 5-10 s apart, after the last one ends;
  - 3D spot loops on real sources such as water and fire.
  - Ghost of Tsushima covered a world 14× its previous one with rules, not by hand placement.

### Footsteps and block sounds
- **Minecraft's sound types** bundle break, step, place, hit and fall for each material.
  - Footsteps use the block underfoot.
  - Carpet, snow layers and moss replay the step of the block beneath at 1/3 volume and 4/5 pitch.
  - Hard-coded multipliers: steps at volume 0.15; hits at 0.25 volume and 0.5 pitch; break and place at pitch 0.8; falls at 0.5 volume and 0.75 pitch. That is why steps feel quiet.
- **Presence Footsteps (mod):**
  - Maps blocks to "acoustics", each with `walk`, `run`, `wander`, `jump`, `land`, `stand` and `swim` events.
  - Acoustics can be delayed (a landing plays two steps 30 ms apart), weighted, or played by chance.
  - Each entity has a gait: biped, quadruped, flying. This fits our animals.
- **Dynamic Surroundings** cut its footstep effects back "to reduce sound fatigue". Rich footsteps get tiring; that is the main pitfall.

### Creature voices
- Minecraft: idle, hurt and death per mob, with a random idle timer.
- Presence Footsteps: gaits per species.
- Hytale: a voice cap (`MaxInstance`, default 50) and round-robin, so the same call doesn't repeat back to back.

### Weather
- **Minecraft:**
  - `weather.rain` at volume 0.2 where rain hits nearby blocks.
  - `weather.rain.above` at 0.1 volume and 0.5 pitch when rain falls within 2 blocks above the eye but not on you. That one sound is the whole "rain on the roof" effect.
  - Thunder is heard dimension-wide, with no delay for distance.
- **Vintage Story (`WeatherSimulationSound.cs`, open source):**
  - Two rain loops, one leafy and one leafless, crossfaded by the leaves nearby.
  - A low "tremble" loop for heavy rain, a hail loop, and leafy and leafless wind loops.
  - A background task measures how far you are from where rain falls, and turns it into `roomVolumePitchLoss = ((dist − 2)/10)²`. That lowers volume and pitch, so indoors sounds muffled.
  - Volumes ease toward targets each frame: `cur += (target − cur)·dt/2`.

### Reverb and occlusion
- **Sound Physics Remastered:**
  - 32 environment rays with 4 bounces and up to 16 occlusion rays per sound;
  - reflectivity, occlusion and absorption per block.
  - Its own changelog calls the original mod's bounce work "excessive". Too costly per sound for us.
- **Cheap version (Vintage Story 1.20 and the mods' defaults):**
  - one estimate of the room around the listener every few hundred ms, which sets the reverb send and decay;
  - one voxel ray from source to listener per sound, which drives a low-pass on that sound's track.
- **Vintage Story 1.20** added cave reverb and also fixed fire "saturating the sound engine": every category needs a voice cap.

### Music
- **Minecraft:**
  - picked per situation: menu, overworld biome, creative, underwater, Nether biome, the End;
  - 10-20 minute random gaps;
  - the Pale Garden fades music out, and recent versions have a frequency setting.
- **Vintage Story:** tracks can require sunlight, hours, season, latitude, temperature, rainfall, or nearness to structures, with priorities, a global cooldown and per-track cooldowns.
- **Valheim:** short themes per biome, plus Morning, Evening, Home, Sailing and boss themes; otherwise sparse.
- **Hytale (MusicContainers):** random containers with `AvoidRepeatCount`, sequences, layers driven by game state, and random `SilenceAfter` after each track.

## Where we are

**Playback** (`crates/engine/src/audio.rs`, kira 0.12.5):
- One listener, a reverb send (damping 0.6, feedback 0.85), and a "world" track with a low-pass filter.
- One track per category, each with a volume slider; UI has no slider of its own.
- A positioned sound gets its own spatial sub-track: full volume within 1 block, fading linearly to its range (default 16).
- Stereo panning only: no HRTF and no Doppler.
- Random pitch is applied as playback speed.

**Environment** (listener-wide on/off states):
- under water, the world track's low-pass drops to 900 Hz;
- in a cave (sky light at the head below 6), a −9 dB reverb send fades in over 400 ms.

Neither state depends on where a sound comes from.

**Hooks the engine already fires:**
- block `.break`, `.place`, and `.hit` every 0.2 s while mining;
- player footsteps every 1.6 blocks;
- entity `.step` and `.ambient` (about every 12 s);
- `.hurt` and `.death`.

The server culls sounds by range against the player's body and sends each tick's sounds in one message.

**Music:**
- Takes the first registered entry whose conditions hold. Conditions are dimension plus day, night, underground, surface and creative.
- The first track starts at 20 s; later ones start 120-300 s after the last.
- Fades in over 3 s and plays to the end. No crossfade, no random pick, no fade-out when the conditions stop holding.
- Base has two short tracks, `calm1.wav` and `deep1.wav`.

**Files on dev:**
- **Block groups:** the 7 groups (stone, wood, gravel, grass, sand, glass, cloth) now have break, place, step and hit files.
- **Engine:** the UI click and pickup now have files.
- **Mob voices:** base's mobs (`mobs.rs:179-277`) point at `entity/<kind>/{ambient,hurt,death}*`, and those files don't exist yet.
- **Animals:** the animals mod has only the chihuahua's bark (`signatures.rs:91`).
- **Blocks without a group:** about 49 of base's block definitions have one. Many blocks have none: farming, cooking, crafts, beds and the 35 redstone pieces, plus snow, ice, leaves, dirt and mud, which have no fitting group anyway.

**What a mod can do:**
- **Can:** register sounds, categories and music; play a positioned or unpositioned one-shot for everyone in range or for one client; give blocks and entities sound groups. A client frame hook can read `day_time`, `raining`, `block`, `light` and `me`.
- **Can't:** loop, stop, fade, change or move a playing sound; pass a pitch to `World::play_sound`.

**Risks:**
- Files are decoded fully on first use, on the main thread, with no size cap and outside `catch_unwind` (security finding S6). Packages may ship up to 64 MiB of sound.
- Voice limits are 256 per category and 64 on the world track. There is no priority and no cap per event, so a fire field or a herd could fill them.
- Weather (`raining`, `thunder`) reaches the client, but nothing plays for it, and there is no lightning.

## What to build

### Engine pieces (Weft)
- **E1. Loop and handle API (M).** The one piece that unlocks the most.
  - `client.loop_start(sound, pos | None, volume, fade) -> id`, `set_volume(id, v, tween)`, `set_pos(id, p, tween)`, `stop(id, fade)`, and a `looped` flag on a sound event.
  - Long beds stream instead of decoding fully.
  - Kira has every part: `loop_region`, handle tweens, and `SpatialTrackHandle::set_position`.
- **E2. Safe decoding (S).** A size and length cap, streaming above a few seconds, decoding off the main thread, and `catch_unwind`.
- **E3. Voice manager (S).** A cap per event (Hytale's `MaxInstance`), round-robin variants, and dropping the quietest or farthest sound when a category is full.
- **E4. An "ambient" category track with its own tweenable low-pass and volume (S).** The indoor muffle for beds and weather.
- **E5. Enclosure and room estimate (M).** Every 0.5 s, cast a few rays from the head:
  - an enclosure value 0-1 (from rays and the rain distance, Vintage Story-style) feeds E4;
  - a room size tweens the reverb's feedback, damping and send.

  This replaces today's on/off cave state.
- **E6. Occlusion per sound (M).** When a sound starts, march one voxel ray from source to listener through full opaque blocks, and set a low-pass and a volume cut on its sub-track. Loops update every few frames.
- **E7. Music rules (M).**
  - A mod condition decides whether an entry fits (biome, season, weather, depth, danger).
  - Random pick among matches, with priorities and avoid-repeat.
  - Fade out when conditions stop holding, and crossfade between tracks.
  - Gap times per entry, defaulting to 5-15 minutes.
- **E8. Small footstep pieces (S):**
  - quieter when sneaking, louder when sprinting;
  - a `.fall` event scaled by fall height, and a jump sound;
  - swimming and wading sounds in place of steps;
  - thin covers (snow layers, carpet) play the block beneath at 1/3 volume, as Minecraft does;
  - a pitch argument on `World::play_sound`;
  - an idle interval per entity kind.

### Data and mod work (base, animals)
- **D1. Groups for every block.** New groups: dirt, snow, leaves, ice, mud, metal, wool and wet sand. Assign one to each block, with a check that fails on blocks with none.
- **D2. Mob and animal voices.** Base's mob files, plus calls for the animals: alarm, call, eat and egg-laying, played from their brains. The numbers live in `mods/animals/core`.
- **D3. Weather sound in base**, on Vintage Story's model:
  - loops for light rain, heavy rain, rain on leaves and wind (open and forest);
  - volumes from rain and wind strength and the leaves nearby, eased each frame and muffled by E5's enclosure value;
  - thunder as a one-shot delayed by distance ÷ 343 m/s (Minecraft lacks this).
- **D4. Ambience in base,** Hytale-style:
  - beds by biome, hour and season (forest day, dawn chorus, forest night, crickets, plains wind, taiga wind, desert, swamp, shore);
  - scattered one-shots (birds, owl, raven, gull, frogs) with sunlight and hour ranges;
  - storms silence birds;
  - cave drips by depth and darkness, as Minecraft's mood does;
  - water loops on clusters of nearby liquid blocks (creek, shore).
- **D5. Music:** a few CC0 tracks per situation with long gaps, the Morning and Evening themes as Valheim does, and none in some places.

## CC0 sources

Licences were checked on 2026-11-04. Every Freesound item named here showed CC0 1.0 on its own page. A Freesound pack can mix licences, so check each file before taking it. CC0 needs no credit, but we log each file's source in the folder's `SOURCES.md`, as the quick win did.

**Accepted:**
- **Freesound,** with the CC0 filter: `https://freesound.org/search/?q=<term>&f=license:"Creative Commons 0"`. Full downloads need an account; the HQ previews (`-hq.mp3`) don't.
- **Kenney:** all CC0. Impact Sounds (130), Interface Sounds (100), Music Jingles (85), RPG Audio (50, footsteps), UI Audio (50).
- **OpenGameArt,** with the CC0 filter in advanced search.

**Not CC0, do not use:**
- **Sonniss GDC bundle:** its own licence. It forbids distributing the sounds raw or modified in packs, and committing them to a public repository very likely counts.
- **BBC Sound Effects:** personal, educational and research use only.
- **Pixabay:** its own licence, with no standalone redistribution.
- **Mixkit:** its own licence, no resale in original form.
- **Musopen:** per recording; check each one.

**Starter list:**

| category | picks |
|---|---|
| forest day | Freesound [613995](https://freesound.org/people/Villehardt/sounds/613995/), [843607](https://freesound.org/people/kevinklang/sounds/843607/); OGA [forest ambience loop](https://opengameart.org/content/forest-ambience) |
| dawn chorus | [856634](https://freesound.org/people/Noisyjones/sounds/856634/), [852696](https://freesound.org/people/Garuda1982/sounds/852696/) |
| forest night, crickets | [662095](https://freesound.org/people/Dokuta_Gerovv/sounds/662095/), [479041](https://freesound.org/people/felix.blume/sounds/479041/), [522299](https://freesound.org/people/Defelozedd94/sounds/522299/) |
| wind (plains, forest, taiga) | [459981](https://freesound.org/people/florianreichelt/sounds/459981/), [477187](https://freesound.org/people/doxent/sounds/477187/), [505999](https://freesound.org/people/DBlover/sounds/505999/) |
| desert, swamp | [156414](https://freesound.org/people/felix.blume/sounds/156414/), [633817](https://freesound.org/people/DBlover/sounds/633817/) |
| shore, creek | [155641](https://freesound.org/people/felix.blume/sounds/155641/), [240645](https://freesound.org/people/GLM_KG/sounds/240645/), [637960](https://freesound.org/people/kyles/sounds/637960/) |
| rain, rain on a roof | [200273](https://freesound.org/people/jmbphilmes/sounds/200273/) (light), [200272](https://freesound.org/people/jmbphilmes/sounds/200272/) (heavy), [669487](https://freesound.org/people/xkeril/sounds/669487/), [527499](https://freesound.org/people/timothyd4y/sounds/527499/) |
| thunder | [324588](https://freesound.org/people/humanoide9000/sounds/324588/), [169534](https://freesound.org/people/klankbeeld/sounds/169534/) |
| cave drips, fire | [234554](https://freesound.org/people/Electroviolence/sounds/234554/), [660298](https://freesound.org/people/Ambient-X/sounds/660298/) |
| footsteps: snow, leaves, wet sand | [384423](https://freesound.org/people/cabled_mess/sounds/384423/), [218305](https://freesound.org/people/SpliceSound/sounds/218305/), [564893](https://freesound.org/people/Nox_Sound/sounds/564893/); OGA [steps on wood, stone, leaves, gravel, mud](https://opengameart.org/content/different-steps-on-wood-stone-leaves-gravel-and-mud) |
| break and place: dirt, mud, leaves | [353907](https://freesound.org/people/dr19/sounds/353907/); OGA [25 mud SFX](https://opengameart.org/content/25-cc0-mud-sfx), [20 dry-leaf rustles](https://opengameart.org/content/20-rustles-dry-leaves), [100 metal and wood SFX](https://opengameart.org/content/100-cc0-metal-and-wood-sfx) |
| animals | cow [513565](https://freesound.org/people/spurioustransients/sounds/513565/), pig [442907](https://freesound.org/people/qubodup/sounds/442907/), sheep [710300](https://freesound.org/people/michaelperfect/sounds/710300/), deer [696776](https://freesound.org/people/ferventtorpor/sounds/696776/), wolf [500646](https://freesound.org/people/betchkal/sounds/500646/), raven [723570](https://freesound.org/people/SoundsLikeYukon/sounds/723570/), owl [871240](https://freesound.org/people/SyntaxSeed/sounds/871240/), gull [263786](https://freesound.org/people/steaq/sounds/263786/) |
| water | splash [524524](https://freesound.org/people/vero.marengere/sounds/524524/); OGA [40 water and splash SFX](https://opengameart.org/content/40-cc0-water-splash-slime-sfx); swim [530158](https://freesound.org/people/tbsounddesigns/sounds/530158/) (has pool reverb) |
| UI | Kenney Interface Sounds and UI Audio; hover [405160](https://freesound.org/people/rayolf/sounds/405160/) |
| music | OGA with the CC0 filter; [HoliznaCC0](https://freemusicarchive.org/music/holiznacc0/winter-lofi) (albums marked CC0 one by one; not the "holiznapatreon" account); Komiku; Kenney Music Jingles |

Many field recordings, felix.blume's and kyles' among them, carry road traffic. Listen before picking one, and cut loops at zero crossings.

## Steps, ranked

| # | step | gain | effort | owner |
|---|---|---|---|---|
| 1 | **E1 loop and handle API, with E2 safe decoding.** Unlocks beds, weather, water, fire and moving voices, and closes S6. | high | M | an engine session (a new "Sound" session, or Distant terrain after the performance work) |
| 2 | **D3 weather sound** (rain loops, wind, delayed thunder) with **E4** the ambient muffle track and a first enclosure value (sky light plus Vintage Story's rain distance). The world's biggest silence today. | high | M | base, after step 1 |
| 3 | **D1 groups for every block, and D2 mob and animal voices.** Assets plus the check; no engine change. | high | S | Researcher (files), Voxel dog and animals (assigning groups, calls) |
| 4 | **D4 ambience beds and one-shots** by biome, hour and season, plus cave drips. | high | M | base |
| 5 | **E3 voice manager** (caps per event, round-robin, steal the quietest). Needed before fire fields and herds. | medium | S | engine |
| 6 | **E8 footstep pieces** (sneak and sprint, fall and jump, swim, thin covers) and a per-kind idle interval. | medium | S | engine |
| 7 | **E7 music rules and D5 tracks.** | medium | M | engine + base |
| 8 | **E5 room reverb and E6 occlusion per sound.** | medium | M | engine |

**Top 3 for Planner:** steps 1, 2 and 3. Step 3 can start today, in parallel, since it needs no code. Steps 1 and 2 give the world its weather and its air.

## Sources

- **Minecraft:** [sounds.json](https://minecraft.wiki/w/Sounds.json), [MC-177717](https://bugs-legacy.mojang.com/browse/MC-177717), [sound type](https://minecraft.wiki/w/Sound_type), [ambience](https://minecraft.wiki/w/Ambience), [rain](https://minecraft.wiki/w/Rain), [lightning](https://minecraft.wiki/w/Lightning), [music](https://minecraft.wiki/w/Music).
- **Mods:**
  - [Sound Physics Remastered config](https://github.com/henkelmax/sound-physics-remastered/blob/1.21.4/common/src/main/java/com/sonicether/soundphysics/config/SoundPhysicsConfig.java) and [repo](https://github.com/henkelmax/sound-physics-remastered);
  - [AmbientSounds engine](https://github.com/CreativeMD/AmbientSounds/wiki/Engine) and [format](https://raw.githubusercontent.com/CreativeMD/AmbientSounds/1.21/engine.md);
  - [Presence Footsteps](https://github.com/Sollace/Presence-Footsteps), [acoustics.json](https://raw.githubusercontent.com/Sollace/Presence-Footsteps/1.21/src/main/resources/assets/presencefootsteps/config/acoustics.json), [locomotion map](https://github.com/Sollace/Presence-Footsteps/wiki/Resourcepacks:-Locomotionmap.json);
  - [Dynamic Surroundings](https://github.com/OreCruncher/DynamicSurroundingsFabric).
- **Vintage Story:** [WeatherSimulationSound.cs](https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/Weather/WeatherSimulationSound.cs), [stormy weathers](https://www.vintagestory.at/blog.html/news/the-stormy-weathers-update-v112-pre1-r230/), [SurfaceMusicTrack](https://apidocs.vintagestory.at/api/Vintagestory.API.Client.SurfaceMusicTrack.html), [1.20 notes](https://www.vintagestory.at/blog.html/news/v120-rc1-lore-update-the-journey-r389).
- **Valheim:** [CaptainAudio](https://thunderstore.io/c/valheim/p/korCaptain/CaptainAudio/), [custom sounds](https://github.com/Valheim-Modding/Wiki/wiki/Custom-Sounds).
- **Hytale:** [SoundEvents](https://docs.hytale.com/assets/audio/soundevents/), [AmbienceFX](https://docs.hytale.com/assets/audio/ambiencefx/), [MusicContainers](https://docs.hytale.com/assets/audio/musiccontainers/), [2020 blog](https://hytale.com/news/2020/1/progress-update-january-29-2020).
- **AAA:** [Eon Altar ambience](https://www.gamedeveloper.com/audio/video-game-audio-under-the-hood-eon-altar-ambience-system), [Ghost of Tsushima (GDC)](https://gdcvault.com/play/1026976/Big-World-Small-Team-Designing).
- **kira 0.12:** [crate](https://docs.rs/kira/0.12.0/kira/), [SpatialTrackBuilder](https://docs.rs/kira/0.12.0/kira/track/struct.SpatialTrackBuilder.html), [effects](https://docs.rs/kira/0.12.0/kira/effect/index.html), [PlaySoundError](https://docs.rs/kira/0.12.0/kira/enum.PlaySoundError.html).
- **Licences:** [Freesound FAQ](https://freesound.org/help/faq/), [Kenney](https://kenney.nl/support), [OpenGameArt CC0](https://opengameart.org/content/cc0-sound-effects), [Sonniss licence](https://sonniss.com/gdc-bundle-license/), [Pixabay](https://pixabay.com/service/license-summary/), [Mixkit](https://mixkit.co/license/).
