# Where base's sounds come from

All the files listed here are **CC0** (public domain dedication): no attribution needed; all but the AI-made sounds in the last two sections, which have their own licence. They are
credited anyway, so anyone can find the originals. The downloads are kept in
`~/Coding/chiacraft-archive/references/sound/` (`kenney/`, `freesound/`). Prepared 2026-11-04 by
the Researcher with `reports/sound-files/sfx.py`, which does the following:
- mono, 44.1 kHz;
- leading silence trimmed;
- cut to game length with a 30 ms fade;
- levelled to a peak by kind, saved as Ogg Vorbis (dual-mono stereo, which the built-in encoder needs).

The music (`music/`) and `random/advancement.wav` were already here and aren't covered by this file.

| files | source | author | licence |
|---|---|---|---|
| `dig/stone1-3` | Impact Sounds 1.0, `impactMining_000-002` | Kenney ([kenney.nl/assets/impact-sounds](https://kenney.nl/assets/impact-sounds)) | CC0 |
| `step/stone1-3` | Impact Sounds, `footstep_concrete_000-002` | Kenney | CC0 |
| `dig/wood1-3` | Impact Sounds, `impactWood_medium_000-002` | Kenney | CC0 |
| `step/wood1-3` | Impact Sounds, `footstep_wood_000-002` | Kenney | CC0 |
| `dig/grass1-3` | Impact Sounds, `impactSoft_medium_000-002` | Kenney | CC0 |
| `step/grass1-3` | Impact Sounds, `footstep_grass_000-002` | Kenney | CC0 |
| `dig/cloth1-3` | Impact Sounds, `impactSoft_heavy_000-002` | Kenney | CC0 |
| `step/cloth1-3` | Impact Sounds, `footstep_carpet_000-002` | Kenney | CC0 |
| `dig/glass1-3` | Impact Sounds, `impactGlass_light_000-002` | Kenney | CC0 |
| `step/glass1-3` | Impact Sounds, `footstep_concrete_002-004` | Kenney | CC0 |
| `random/glass1-3` (glass breaking) | Impact Sounds, `impactGlass_heavy_000-002` | Kenney | CC0 |
| `dig/gravel1-3` | three hits cut from "Shoveling Gravel" ([freesound 764821](https://freesound.org/people/Charisma_Vigoureux_241085/sounds/764821/)) | Charisma_Vigoureux_241085 | CC0 |
| `step/gravel1-3` | three steps cut from "Footsteps outdoors gravel.wav" ([freesound 505831](https://freesound.org/people/jedg/sounds/505831/)) | jedg | CC0 |
| `dig/sand1`, `dig/sand2` | "Digging in wet course sand (1)" and "(2)" ([651292](https://freesound.org/people/f3bbbo/sounds/651292/), [651293](https://freesound.org/people/f3bbbo/sounds/651293/)) | f3bbbo | CC0 |
| `dig/sand3` | a hit cut from "Digging in wet course sand (raw file)" ([651294](https://freesound.org/people/f3bbbo/sounds/651294/)) | f3bbbo | CC0 |
| `step/sand1-3` | three steps cut from "Steps sand.wav" ([freesound 74925](https://freesound.org/people/epsilon24/sounds/74925/)) | epsilon24 | CC0 |

The Freesound sources are their public high-quality previews (128 kbps), each checked as "Creative
Commons 0" on its own page on 2026-11-04.

## Weather (`weather/`)

Prepared 2026-11-04 by the Sound engine session with `reports/sound-files/sfx.py` and
`reports/sound-files/weather.tsv`. The loops (`loop:` mode) are 24 s with their tail crossfaded into
their head (1.5 s, equal power) and levelled by loudness. Each source's own Freesound page said
"Creative Commons 0" on 2026-11-04; the files are its high-quality preview (128 kbps mp3), kept in
`~/Coding/chiacraft-archive/references/sound/freesound/`.

| files | source | author | licence |
|---|---|---|---|
| `weather/rain1` (loop) | "Slowly Raining Loop" ([freesound 177479](https://freesound.org/people/unfa/sounds/177479/)) | unfa | CC0 |
| `weather/rain_heavy1` (loop) | "Rain heavy 2 (rural)" ([freesound 200272](https://freesound.org/people/jmbphilmes/sounds/200272/)) | jmbphilmes | CC0 |
| `weather/rain_roof1` (loop) | "Rain from Indoors - Perfect loop" ([freesound 242889](https://freesound.org/people/samesamesame/sounds/242889/)) | samesamesame | CC0 |
| `weather/wind1` (loop) | "Looping Gentle Wind Ambience on an Open Desert Plain" ([freesound 697217](https://freesound.org/people/dhallcomposer/sounds/697217/)) | dhallcomposer | CC0 |
| `weather/wind_trees1` (loop) | "wind_forest_08_strong_l_01.wav" ([freesound 403051](https://freesound.org/people/teadrinker/sounds/403051/)) | teadrinker | CC0 |
| `weather/thunder_near1` | "sfx_thunder blast.wav" ([freesound 399656](https://freesound.org/people/bajko/sounds/399656/)) | bajko | CC0 |
| `weather/thunder_near2` | "Heavy Thunder Strike - no Rain - QUADRO.wav" ([freesound 446753](https://freesound.org/people/BlueDelta/sounds/446753/)) | BlueDelta | CC0 |
| `weather/thunder_far1` | "Distant Thunder Clap_Gentle.wav" ([freesound 347562](https://freesound.org/people/mikala_oidua/sounds/347562/)) | mikala_oidua | CC0 |
| `weather/thunder_far2` | "thunder" ([freesound 193170](https://freesound.org/people/netaj/sounds/193170/)) | netaj | CC0 |
| `weather/thunder_far3`, `thunder_far4` | two rumbles cut from "Distant Thunder" ([freesound 361401](https://freesound.org/people/richwise/sounds/361401/)), at 91.5 s and 230 s | richwise | CC0 |

## Mob voices (`entity/<kind>/`)

Prepared 2026-11-04 by the Sound engine session the same way as the animals' voices
(`reports/sound-files/voices.tsv`, see `mods/animals/assets/sounds/SOURCES.md`): ambient 2.5 s,
hurt 0.8 s, death 1.6 s. Each source checked CC0 on its own Freesound page on 2026-11-04.

| files | source | author | licence |
|---|---|---|---|
| `zombie/ambient1` (hit:0), `zombie/ambient2` (hit:1), `zombie/hurt2` (hit:2) | "A Moaning Zombie.wav" ([freesound 58626](https://freesound.org/people/kathol/sounds/58626/)) | kathol | CC0 |
| `zombie/ambient3` | "Burp Monster Zombie groan moan" ([freesound 463721](https://freesound.org/people/EricsSoundschmiede/sounds/463721/)) | EricsSoundschmiede | CC0 |
| `zombie/hurt1` | "Grunt.wav" ([freesound 370036](https://freesound.org/people/punisherman/sounds/370036/)) | punisherman | CC0 |
| `zombie/death1` | "groaning.mp3" ([freesound 244320](https://freesound.org/people/Philip_Daniels/sounds/244320/)) | Philip_Daniels | CC0 |
| `skeleton/ambient1` (hit:0), `skeleton/ambient2` (hit:1), `skeleton/death1` | "Rattling Bones.wav" ([freesound 202102](https://freesound.org/people/spookymodem/sounds/202102/)) | spookymodem | CC0 |
| `skeleton/hurt1` (hit:0), `skeleton/hurt2` (hit:1) | "bones.mp3" ([freesound 473526](https://freesound.org/people/Kneeling/sounds/473526/)) | Kneeling | CC0 |
| `spider/ambient1`, `spider/hurt2` | "hiss2.wav" ([freesound 343927](https://freesound.org/people/Reitanna/sounds/343927/)) | Reitanna | CC0 |
| `spider/ambient2`, `creeper/hurt1` | "hiss3.wav" ([freesound 343928](https://freesound.org/people/Reitanna/sounds/343928/)) | Reitanna | CC0 |
| `spider/hurt1`, `creeper/hurt2` | "catHisses1.wav" ([freesound 146963](https://freesound.org/people/Zabuhailo/sounds/146963/)) | Zabuhailo | CC0 |
| `spider/death1` (hit:0), `creeper/death1` (hit:1) | "Hiss.flac" ([freesound 160350](https://freesound.org/people/qubodup/sounds/160350/)) | qubodup | CC0 |
| `pig/ambient1` | "Pig Oink" ([freesound 442906](https://freesound.org/people/qubodup/sounds/442906/)) | qubodup | CC0 |
| `pig/ambient2` | "Pig Grunt" ([freesound 442907](https://freesound.org/people/qubodup/sounds/442907/)) | qubodup | CC0 |
| `pig/ambient3` | "Pig Sound" ([freesound 442905](https://freesound.org/people/qubodup/sounds/442905/)) | qubodup | CC0 |
| `pig/hurt1` (hit:0), `pig/hurt2` (hit:1), `pig/death1` | "Pig - Scream 5 - 96kHz.wav" ([freesound 233151](https://freesound.org/people/JarredGibb/sounds/233151/)) | JarredGibb | CC0 |
| `cow/ambient1` | "z-moo01.wav" ([freesound 59245](https://freesound.org/people/Zozzy/sounds/59245/)) | Zozzy | CC0 |
| `cow/ambient2` | "Moo.wav" ([freesound 275154](https://freesound.org/people/Bird_man/sounds/275154/)) | Bird_man | CC0 |
| `cow/ambient3` (hit:0) | "Cow moo #8" ([freesound 513565](https://freesound.org/people/spurioustransients/sounds/513565/)) | spurioustransients | CC0 |
| `cow/hurt1` (hit:0), `cow/hurt2` (hit:1) | "Cow moos" ([freesound 177253](https://freesound.org/people/JosephSardin/sounds/177253/)) | JosephSardin | CC0 |
| `cow/death1` (hit:0) | "Cow Scream" ([freesound 365134](https://freesound.org/people/gibarroule/sounds/365134/)) | gibarroule | CC0 |
| `sheep/ambient1` | "sheep 3.mp3" ([freesound 171149](https://freesound.org/people/esperar/sounds/171149/)) | esperar | CC0 |
| `sheep/ambient2` | "sheep 2.mp3" ([freesound 171151](https://freesound.org/people/esperar/sounds/171151/)) | esperar | CC0 |
| `sheep/ambient3` (hit:0), `sheep/hurt1` (hit:1), `sheep/hurt2` (hit:2) | "Sheep bleating" ([freesound 362283](https://freesound.org/people/zachrau/sounds/362283/)) | zachrau | CC0 |
| `sheep/death1` (hit:0) | "Lamb and Mother" ([freesound 182509](https://freesound.org/people/swiftoid/sounds/182509/)) | swiftoid | CC0 |
| `chicken/ambient1` (hit:0), `chicken/ambient2` (hit:1), `chicken/ambient3` (hit:2) | "Chicken clucking" ([freesound 456803](https://freesound.org/people/Breviceps/sounds/456803/)) | Breviceps | CC0 |
| `chicken/hurt1` | "Chicken Single Alarm Call" ([freesound 316920](https://freesound.org/people/Rudmer_Rotteveel/sounds/316920/)) | Rudmer_Rotteveel | CC0 |
| `chicken/hurt2` (hit:0), `chicken/death1` (hit:1) | "Chicken Alarm Call full with Occasional bird sound" ([freesound 316921](https://freesound.org/people/Rudmer_Rotteveel/sounds/316921/)) | Rudmer_Rotteveel | CC0 |

## Block groups: dirt, snow, leaves, ice, mud, metal

Prepared 2026-11-04 by the Sound engine session from `reports/sound-files/blocks.tsv` (same
steps as the first groups). Kenney files are from Impact Sounds (CC0): `step/snow1-3` =
`footstep_snow_000-002`, `dig/ice1-3` = `impactGlass_medium_000-002`, `dig/metal1-3` =
`impactMetal_medium_000-002`, `step/metal1-3` = `impactMetal_light_000-002`, `break/metal1-3` =
`impactPlate_heavy_000-002`. The rest, each checked CC0 on its Freesound page on 2026-11-04:

| files | source | author | licence |
|---|---|---|---|
| `dig/dirt1` (hit:0), `dig/dirt2` (hit:1), `dig/dirt3` (hit:2) | "Shovel_dirt.wav" ([freesound 353907](https://freesound.org/people/dr19/sounds/353907/)) | dr19 | CC0 |
| `step/dirt1` (hit:0), `step/dirt2` (hit:1), `step/dirt3` (hit:2) | "Footsteps Dirt 01" ([freesound 264469](https://freesound.org/people/aglinder/sounds/264469/)) | aglinder | CC0 |
| `dig/snow1` (hit:0), `dig/snow2` (hit:1), `dig/snow3` (hit:2) | "footsteps in fresh snow" ([freesound 453168](https://freesound.org/people/florianreichelt/sounds/453168/)) | florianreichelt | CC0 |
| `break/ice1` (hit:0), `break/ice2` (hit:1), `break/ice3` (hit:2) | "Ice cracking" ([freesound 342546](https://freesound.org/people/timbreknight/sounds/342546/)) | timbreknight | CC0 |
| `step/ice1` (hit:0), `step/ice2` (hit:1), `step/ice3` (hit:2) | "Walking on Crunchy Ice (01/30/2014)" ([freesound 447822](https://freesound.org/people/courter/sounds/447822/)) | courter | CC0 |
| `dig/mud1` (hit:0), `dig/mud2` (hit:1), `dig/mud3` (hit:2) | "MudSquelch.wav" ([freesound 207142](https://freesound.org/people/ahill86/sounds/207142/)) | ahill86 | CC0 |
| `step/mud1` (hit:0), `step/mud2` (hit:1), `step/mud3` (hit:2) | "MUDDY AND WET FOOTSTEPS.wav" ([freesound 376809](https://freesound.org/people/Alex_hears_things/sounds/376809/)) | Alex_hears_things | CC0 |
| `dig/leaves1` (at:2.6), `dig/leaves2` (at:3.3), `dig/leaves3` (at:5.4) | "Rustling leaves" ([freesound 437356](https://freesound.org/people/giddster/sounds/437356/)) | giddster | CC0 |
| `step/leaves1` (at:1.2), `step/leaves2` (at:2.5), `step/leaves3` (at:4.0) | "Leaves_Rustling_Edited.wav" ([freesound 443065](https://freesound.org/people/AmberdeMeillon/sounds/443065/)) | AmberdeMeillon | CC0 |

## The land, water and lava (`ambient/`)

Added 2026-11-24 by the Sound worker for `ambience.rs`: Freesound HQ previews (the first 4 MB of each, fetched with `reports/sound-files/fetch_freesound.py`, which refuses any page whose licence isn't CC0 1.0), cut and levelled with `reports/sound-files/sfx.py` from `reports/sound-files/ambience.tsv`. Raw downloads: `~/Coding/chiacraft-archive/references/sound/freesound/`. Not yet listened to by a person: picked by loudness and steadiness.

| files | source | author | licence |
|---|---|---|---|
| `ambient/birds1` (loop:73) | "Forest Day - Bird Chirps, Insects" ([freesound 788118](https://freesound.org/people/RealSquink/sounds/788118/)) | RealSquink | CC0 |
| `ambient/dawn1` (loop:24) | "Henllys Woods 22-06-2026 Dawn Chorus" ([freesound 856634](https://freesound.org/people/Noisyjones/sounds/856634/)) | Noisyjones | CC0 |
| `ambient/night1` (loop:80) | "Crickets At Night - Raw sound" ([freesound 522299](https://freesound.org/people/Defelozedd94/sounds/522299/)) | Defelozedd94 | CC0 |
| `ambient/dry1` (loop:67) | "Wind blowing into some cactus spine ... Atacama (Chile)" ([freesound 156414](https://freesound.org/people/felix.blume/sounds/156414/)) | felix.blume | CC0 |
| `ambient/swamp1` (loop:110) | "Frogs and geese in a swamp" ([freesound 633817](https://freesound.org/people/DBlover/sounds/633817/)) | DBlover | CC0 |
| `ambient/surf1` (loop:33) | "Ocean waves crushing on the chilean shore." ([freesound 155641](https://freesound.org/people/felix.blume/sounds/155641/)) | felix.blume | CC0 |
| `ambient/water_flow1` (loop:2) | "Creek 06 (loop)" ([freesound 231536](https://freesound.org/people/VKProduktion/sounds/231536/)) | VKProduktion | CC0 |
| `ambient/water_still1` (loop:116) | "Gentle waves on a lake" ([freesound 614299](https://freesound.org/people/TheFlyFishingFilmmaker/sounds/614299/)) | TheFlyFishingFilmmaker | CC0 |
| `ambient/lava1` (loop:13) | "Lava Loop 3" ([freesound 474851](https://freesound.org/people/Fission9/sounds/474851/)) | Fission9 | CC0 |
| `ambient/call/lava_pop1-2` (hit:0-1) | "lava.wav" ([freesound 145036](https://freesound.org/people/ATG142/sounds/145036/)) | ATG142 | CC0 |
| `ambient/call/drip1-3` (hit:0-2) | "Dripping water" ([freesound 234554](https://freesound.org/people/Electroviolence/sounds/234554/)) | Electroviolence | CC0 |
| `ambient/call/raven1-2` (hit:0-1) | "Solitary raven cawing" ([freesound 723570](https://freesound.org/people/SoundsLikeYukon/sounds/723570/)) | SoundsLikeYukon | CC0 |
| `ambient/call/owl1` (copy) | "Owl-4" ([freesound 871240](https://freesound.org/people/SyntaxSeed/sounds/871240/)) | SyntaxSeed | CC0 |
| `ambient/call/owl2` (copy) | "Owl Hoot" ([freesound 465697](https://freesound.org/people/Breviceps/sounds/465697/)) | Breviceps | CC0 |
| `ambient/call/gull1` (copy) | "Seagull single call" ([freesound 263786](https://freesound.org/people/steaq/sounds/263786/)) | steaq | CC0 |
| `ambient/call/frog1-2` (hit:0-1) | "frog Croaks.wav" ([freesound 527846](https://freesound.org/people/D.jones/sounds/527846/)) | D.jones | CC0 |
| `ambient/call/frog3` (hit:0) | "Croaking" ([freesound 397028](https://freesound.org/people/ceelum/sounds/397028/)) | ceelum | CC0 |
| `ambient/call/bird1-2` (hit:0-1) | "Quiet Forest, Single Bird" ([freesound 730301](https://freesound.org/people/HECKFRICKER/sounds/730301/)) | HECKFRICKER | CC0 |
| `ambient/call/bird3` (hit:0) | "Single Blackbird at Dawn" ([freesound 346853](https://freesound.org/people/Kinoton/sounds/346853/)) | Kinoton | CC0 |
| `ambient/call/bird4` (hit:0) | "robin song" ([freesound 671058](https://freesound.org/people/Andrew.soundscape/sounds/671058/)) | Andrew.soundscape | CC0 |

## A falling tree (`tree/`)

Made 2026-11-05 by the Tree felling session with `reports/sound-files/fall_sounds.py` (numpy and
ffmpeg), from nothing but synthesis and CC0 files already here:

| files | made from | licence |
|---|---|---|
| `tree/creak1-2` | synthesized stick-slip creak through wood resonances; the crack at its end is Kenney's Impact Sounds `impactWood_light_001-002` | ours + CC0 |
| `tree/fall1-2` | swept noise (synthesized) with base's own `dig/leaves1-3` (Kenney, Impact Sounds) scattered through | ours + CC0 |
| `tree/thud1-2` | Kenney's Impact Sounds `impactWood_heavy_001-002` an octave down, a synthesized low thump, `dig/leaves1-3` | CC0 + ours |

## Made here

| files | how | licence |
|---|---|---|
| `random/ley_twitch.wav` (the dowsing rod's tick, `ley.rs`) | synthesised: bell-like partials over a short decay, `mods/base/source/sounds/ley_twitch.py` | CC0 |
| `random/ley_wake.wav` (a waystone waking, `waystones.rs`) | synthesised: a low swell rising into a bell chime, `mods/base/source/sounds/ley_wake.py` | CC0 |

## The player and using things (`player/`, `use/`): AI-made

Not CC0, unlike the rest of this file. Made 2026-11-25 with **Stable Audio 3 Small SFX**
(`stabilityai/stable-audio-3-small-sfx`, Stability AI) on our own PC, as the animals' AI voices
(`mods/animals/assets/sounds/SOURCES.md` has the whole note). Licence: the
[Stability AI Community License](https://stability.ai/license): we own the outputs and may use them
commercially while annual revenue stays under USD 1M (above that an Enterprise licence). Its text
encoder (T5Gemma) is under the [Gemma Terms of Use](https://ai.google.dev/gemma/terms). Ronald
approved the model on 11-25 and asked for these sounds the same day ("Yes, top 4", Planner's sound
audit).

Each event got its variants plus two spare candidates, each worded a little differently (the
model gives nearly the same clip for one prompt whatever the seed). Every candidate was trimmed
and cut (`reports/sound-files/sfx.py`, copy mode), scored by numbers (voiced sounds by how
call-like they are, knocks and rustles by a clean hit over a quiet floor, the fire loops by how
steady they are) and kept only when its spectrum is unlike the ones already kept. The picks were
levelled to **-24 LUFS integrated, under -1 dBTP** (a limiter holds the peaks; a few peaky loops
stay up to 2.5 dB under), and the fire, torch and furnace loops have their tail crossfaded into
their head (0.4 s) so they repeat without a seam. Every candidate, the scores, the picks and a
spectrogram sheet are in `~/Coding/chiacraft-archive/sounds/ai-sound/2026-11-25-player/`. Where
they play: `src/player_sounds.rs`.

| File | Prompt | Seed | LUFS / dBTP |
|---|---|---|---|
| `player/hurt1` | "man short pained grunt, hit, louder and more intense, close, dry, no background noise, no music" | 9004 | -24.0 / -12.3 |
| `player/hurt2` | "man short pained grunt, hit, short and sharp, close, dry, no background noise, no music" | 9002 | -23.9 / -9.3 |
| `player/hurt3` | "man short pained grunt, hit, two in a row, close, dry, no background noise, no music" | 9006 | -24.0 / -12.9 |
| `player/hurt4` | "man short pained grunt, hit, softer and calmer, close, dry, no background noise, no music" | 9005 | -23.9 / -12.2 |
| `player/death1` | "man dying groan, short, falling, short and sharp, close, dry, no background noise, no music" | 9008 | -24.0 / -13.0 |
| `player/death2` | "man dying groan, short, falling, longer and lower, close, dry, no background noise, no music" | 9009 | -24.0 / -11.6 |
| `player/eat1` | "person biting and chewing an apple, crunchy, short and sharp, close, dry, no background noise, no music" | 9012 | -24.1 / -4.8 |
| `player/eat2` | "person biting and chewing an apple, crunchy, louder and more intense, close, dry, no background noise, no music" | 9014 | -24.0 / -5.6 |
| `player/eat3` | "person biting and chewing an apple, crunchy, close, dry, no background noise, no music" | 9011 | -24.0 / -4.9 |
| `player/burp1` | "short soft satisfied burp, longer and lower, close, dry, no background noise, no music" | 9018 | -24.0 / -7.0 |
| `player/drink1` | "person gulping water from a flask, two swallows, close, dry, no background noise, no music" | 9019 | -23.9 / -4.4 |
| `player/drink2` | "person gulping water from a flask, two swallows, longer and lower, close, dry, no background noise, no music" | 9021 | -24.0 / -8.2 |
| `player/drink3` | "person gulping water from a flask, two swallows, softer and calmer, close, dry, no background noise, no music" | 9023 | -26.8 / -7.9 |
| `player/swim1` | "swimmer arm stroke splashing water, single stroke, louder and more intense, close, dry, no background noise, no music" | 9027 | -24.0 / -1.8 |
| `player/swim2` | "swimmer arm stroke splashing water, single stroke, close, dry, no background noise, no music" | 9024 | -24.3 / -2.0 |
| `player/swim3` | "swimmer arm stroke splashing water, single stroke, two in a row, close, dry, no background noise, no music" | 9029 | -24.0 / -4.4 |
| `player/swim4` | "swimmer arm stroke splashing water, single stroke, longer and lower, close, dry, no background noise, no music" | 9026 | -24.0 / -10.7 |
| `player/splash1` | "person jumping into a lake, big splash, short and sharp, close, dry, no background noise, no music" | 9031 | -24.0 / -1.9 |
| `player/splash2` | "person jumping into a lake, big splash, louder and more intense, close, dry, no background noise, no music" | 9033 | -24.3 / -2.0 |
| `player/splash3` | "person jumping into a lake, big splash, softer and calmer, close, dry, no background noise, no music" | 9034 | -24.0 / -2.4 |
| `player/land1` | "person landing on their feet on ground after a jump, soft thud, louder and more intense, close, dry, no background noise, no music" | 9038 | -24.1 / -2.7 |
| `player/land2` | "person landing on their feet on ground after a jump, soft thud, close, dry, no background noise, no music" | 9035 | -24.1 / -4.7 |
| `player/land3` | "person landing on their feet on ground after a jump, soft thud, longer and lower, close, dry, no background noise, no music" | 9037 | -23.9 / -4.4 |
| `player/fall1` | "person falling hard onto ground, heavy body thud with grunt, longer and lower, close, dry, no background noise, no music" | 9042 | -24.0 / -8.0 |
| `player/fall2` | "person falling hard onto ground, heavy body thud with grunt, louder and more intense, close, dry, no background noise, no music" | 9043 | -23.9 / -8.8 |
| `player/fall3` | "person falling hard onto ground, heavy body thud with grunt, softer and calmer, close, dry, no background noise, no music" | 9044 | -23.9 / -6.9 |
| `player/drown1` | "air bubbles escaping underwater, gurgle, softer and calmer, close, dry, no background noise, no music" | 9049 | -23.9 / -7.7 |
| `player/drown2` | "air bubbles escaping underwater, gurgle, short and sharp, close, dry, no background noise, no music" | 9046 | -24.0 / -8.0 |
| `player/drown3` | "air bubbles escaping underwater, gurgle, louder and more intense, close, dry, no background noise, no music" | 9048 | -24.0 / -7.0 |
| `player/breath1` | "person gasping for air after surfacing from water, louder and more intense, close, dry, no background noise, no music" | 9053 | -24.0 / -13.7 |
| `player/breath2` | "person gasping for air after surfacing from water, longer and lower, close, dry, no background noise, no music" | 9052 | -23.9 / -13.8 |
| `use/door_open1` | "wooden door opening, creak and latch, close, dry, no background noise, no music" | 9054 | -24.0 / -5.0 |
| `use/door_open2` | "wooden door opening, creak and latch, louder and more intense, close, dry, no background noise, no music" | 9057 | -24.1 / -2.1 |
| `use/door_open3` | "wooden door opening, creak and latch, longer and lower, close, dry, no background noise, no music" | 9056 | -24.0 / -7.3 |
| `use/door_close1` | "wooden door closing, knock and latch, louder and more intense, close, dry, no background noise, no music" | 9062 | -24.0 / -4.8 |
| `use/door_close2` | "wooden door closing, knock and latch, short and sharp, close, dry, no background noise, no music" | 9060 | -23.9 / -6.5 |
| `use/door_close3` | "wooden door closing, knock and latch, longer and lower, close, dry, no background noise, no music" | 9061 | -24.1 / -6.6 |
| `use/chest_open1` | "wooden chest lid opening, creak, short and sharp, close, dry, no background noise, no music" | 9065 | -24.6 / -2.0 |
| `use/chest_open2` | "wooden chest lid opening, creak, longer and lower, close, dry, no background noise, no music" | 9066 | -24.0 / -3.5 |
| `use/chest_open3` | "wooden chest lid opening, creak, louder and more intense, close, dry, no background noise, no music" | 9067 | -25.2 / -1.8 |
| `use/chest_close1` | "wooden chest lid closing, thump, short and sharp, close, dry, no background noise, no music" | 9070 | -23.9 / -7.4 |
| `use/chest_close2` | "wooden chest lid closing, thump, close, dry, no background noise, no music" | 9069 | -23.9 / -5.4 |
| `use/chest_close3` | "wooden chest lid closing, thump, louder and more intense, close, dry, no background noise, no music" | 9072 | -23.9 / -3.2 |
| `use/craft1` | "crafting with wood and tools on a workbench, short hammering and shuffling, short and sharp, close, dry, no background noise, no music" | 9075 | -24.0 / -5.1 |
| `use/craft2` | "crafting with wood and tools on a workbench, short hammering and shuffling, louder and more intense, close, dry, no background noise, no music" | 9077 | -24.3 / -2.0 |
| `use/craft3` | "crafting with wood and tools on a workbench, short hammering and shuffling, longer and lower, close, dry, no background noise, no music" | 9076 | -24.2 / -1.9 |
| `use/furnace_open1` | "stone furnace door opening, iron latch scrape, short and sharp, close, dry, no background noise, no music" | 9080 | -23.9 / -3.7 |
| `use/furnace_open2` | "stone furnace door opening, iron latch scrape, softer and calmer, close, dry, no background noise, no music" | 9083 | -24.1 / -4.3 |
| `use/furnace_open3` | "stone furnace door opening, iron latch scrape, close, dry, no background noise, no music" | 9079 | -24.0 / -5.7 |
| `use/furnace_fire1` | "fire crackling in a stone furnace, steady, softer and calmer, close, dry, no background noise, no music" | 9088 | -26.5 / -1.8 |
| `use/furnace_fire2` | "fire crackling in a stone furnace, steady, close, dry, no background noise, no music" | 9084 | -23.9 / -1.9 |
| `use/furnace_fire3` | "fire crackling in a stone furnace, steady, short and sharp, close, dry, no background noise, no music" | 9085 | -26.1 / -1.9 |
| `use/fire1` | "campfire crackling and popping, steady, longer and lower, close, dry, no background noise, no music" | 9091 | -25.4 / -1.2 |
| `use/fire2` | "campfire crackling and popping, steady, close, dry, no background noise, no music" | 9089 | -25.7 / -1.6 |
| `use/fire3` | "campfire crackling and popping, steady, softer and calmer, close, dry, no background noise, no music" | 9093 | -26.4 / -1.3 |
| `use/fire4` | "campfire crackling and popping, steady, louder and more intense, close, dry, no background noise, no music" | 9092 | -26.1 / -1.7 |
| `use/torch1` | "torch flame flickering and crackling softly, short and sharp, close, dry, no background noise, no music" | 9096 | -25.2 / -1.8 |
| `use/torch2` | "torch flame flickering and crackling softly, softer and calmer, close, dry, no background noise, no music" | 9099 | -25.0 / -1.8 |
| `use/torch3` | "torch flame flickering and crackling softly, close, dry, no background noise, no music" | 9095 | -24.0 / -4.1 |
| `player/swing1` | "swinging a stick through the air, swoosh, louder and more intense, close, dry, no background noise, no music" | 9103 | -23.9 / -6.0 |
| `player/swing2` | "swinging a stick through the air, swoosh, short and sharp, close, dry, no background noise, no music" | 9101 | -23.9 / -3.0 |
| `player/swing3` | "swinging a stick through the air, swoosh, softer and calmer, close, dry, no background noise, no music" | 9104 | -23.9 / -6.1 |
| `player/swing4` | "swinging a stick through the air, swoosh, longer and lower, close, dry, no background noise, no music" | 9102 | -23.9 / -6.0 |
| `player/tool_break1` | "wooden tool snapping and breaking, crack, louder and more intense, close, dry, no background noise, no music" | 9109 | -24.1 / -3.1 |
| `player/tool_break2` | "wooden tool snapping and breaking, crack, short and sharp, close, dry, no background noise, no music" | 9107 | -24.6 / -1.7 |
| `player/tool_break3` | "wooden tool snapping and breaking, crack, softer and calmer, close, dry, no background noise, no music" | 9110 | -26.5 / -2.0 |
| `player/bow_draw1` | "drawing a wooden bow string, creak and stretch, softer and calmer, close, dry, no background noise, no music" | 9115 | -24.0 / -11.2 |
| `player/bow_draw2` | "drawing a wooden bow string, creak and stretch, close, dry, no background noise, no music" | 9111 | -24.0 / -10.2 |
| `player/bow_draw3` | "drawing a wooden bow string, creak and stretch, louder and more intense, close, dry, no background noise, no music" | 9114 | -24.0 / -9.2 |
| `player/bow_shoot1` | "bow string release and arrow whoosh, short and sharp, close, dry, no background noise, no music" | 9117 | -23.9 / -10.9 |
| `player/bow_shoot2` | "bow string release and arrow whoosh, longer and lower, close, dry, no background noise, no music" | 9118 | -23.9 / -2.1 |
| `player/bow_shoot3` | "bow string release and arrow whoosh, softer and calmer, close, dry, no background noise, no music" | 9120 | -23.9 / -3.6 |
| `player/armour1` | "putting on leather and metal armour, buckle and clink, close, dry, no background noise, no music" | 9121 | -24.1 / -1.8 |
| `player/armour2` | "putting on leather and metal armour, buckle and clink, softer and calmer, close, dry, no background noise, no music" | 9125 | -24.0 / -4.8 |
| `player/armour3` | "putting on leather and metal armour, buckle and clink, longer and lower, close, dry, no background noise, no music" | 9123 | -24.0 / -6.6 |
| `use/inventory_open1` | "leather bag opening, rustle, louder and more intense, close, dry, no background noise, no music" | 9129 | -24.0 / -6.4 |
| `use/inventory_open2` | "leather bag opening, rustle, close, dry, no background noise, no music" | 9126 | -24.0 / -7.4 |
| `use/inventory_open3` | "leather bag opening, rustle, longer and lower, close, dry, no background noise, no music" | 9128 | -24.0 / -5.4 |
| `use/inventory_close1` | "leather bag closing, soft rustle, longer and lower, close, dry, no background noise, no music" | 9133 | -24.0 / -6.5 |
| `use/inventory_close2` | "leather bag closing, soft rustle, close, dry, no background noise, no music" | 9131 | -24.0 / -7.8 |
| `use/inventory_close3` | "leather bag closing, soft rustle, louder and more intense, close, dry, no background noise, no music" | 9134 | -24.0 / -3.9 |

## Block steps, digs, place and hit, wading (`step/`, `dig/`, `place/`, `hit/`): AI-made

Not CC0. Made 2026-11-25 with the same model, licence and approval as the section above (the
sound audit's third item: water steps and wading, more variants per block group, each group's own
place and hit). Per group: steps and digs 4-6 (1-3 are the free packs' above), place 1-3 and hit
1-3 (both used to be the dig and step files pitched down); `step/water1-5` are wading steps.

Each event got its variants plus three spare candidates, worded a little differently, cut
(`reports/sound-files/sfx.py`, copy mode), scored by a clean hit over a quiet floor (new steps
and digs also by how like the group's old three they are, so a set stays one material) and kept
only when unlike the ones already kept. These clips are shorter than 0.4 s, where integrated
loudness is not defined (it gates to -inf, as the old files do), so each was levelled by its
loudest 50 ms to the old files of its group: new steps and digs to that group's, place to its
dig, hit to its step (played at 0.6, as before), wading to the steps' mean; true peak at most
-1 dBTP. A few loud digs and places stay 2-4 dB under their target, held by the peak limit (the
old files are peakier). Prompts end in ", close, dry, no background noise, no music". Work files:
`~/Coding/chiacraft-archive/sounds/ai-sound/2026-11-25-blocks/`.

| File | Prompt | Seed | Loudest 50 ms dB (target) / dBTP |
|---|---|---|---|
| `step/stone4.ogg` | single footstep on stone | 12001 | -26.7 (-26.7) / -15.9 |
| `step/stone5.ogg` | single footstep on stone, heavier | 12003 | -26.7 (-26.7) / -16.4 |
| `step/stone6.ogg` | boot step on stone, short and sharp | 12002 | -26.7 (-26.7) / -15.3 |
| `dig/stone4.ogg` | breaking a block of stone, single hit, muffled | 12012 | -14.1 (-11.9) / -2.1 |
| `dig/stone5.ogg` | breaking a block of stone, single hit, short and sharp | 12008 | -14.6 (-11.9) / -1.8 |
| `dig/stone6.ogg` | digging and scraping stone with a tool, single | 12007 | -15.1 (-11.9) / -1.6 |
| `place/stone1.ogg` | placing a block of stone down, single solid thud | 12013 | -12.2 (-11.9) / -2.0 |
| `place/stone2.ogg` | setting a stone block in place, short, muffled | 12018 | -12.7 (-11.9) / -2.1 |
| `place/stone3.ogg` | setting a stone block in place, short, short and sharp | 12014 | -15.1 (-11.9) / -2.0 |
| `hit/stone1.ogg` | tapping stone with a pickaxe, single light knock, crisp | 12023 | -26.7 (-26.7) / -13.3 |
| `hit/stone2.ogg` | striking stone with a tool, short hit, short and sharp | 12020 | -26.7 (-26.7) / -12.2 |
| `hit/stone3.ogg` | tapping stone with a pickaxe, single light knock, heavier | 12021 | -26.7 (-26.7) / -13.3 |
| `step/wood4.ogg` | boot step on wooden planks, lighter and softer | 12028 | -21.2 (-21.1) / -10.2 |
| `step/wood5.ogg` | boot step on wooden planks, muffled | 12030 | -21.2 (-21.1) / -11.1 |
| `step/wood6.ogg` | single footstep on wooden planks | 12025 | -21.1 (-21.1) / -10.4 |
| `dig/wood4.ogg` | breaking a block of wooden planks, single hit, muffled | 12036 | -16.5 (-16.5) / -6.3 |
| `dig/wood5.ogg` | breaking a block of wooden planks, single hit, lighter and softer | 12034 | -16.6 (-16.5) / -2.0 |
| `dig/wood6.ogg` | digging and scraping wooden planks with a tool, single, crisp | 12035 | -16.5 (-16.5) / -4.8 |
| `place/wood1.ogg` | setting a wooden planks block in place, short, muffled | 12042 | -16.5 (-16.5) / -3.7 |
| `place/wood2.ogg` | setting a wooden planks block in place, short, short and sharp | 12038 | -16.5 (-16.5) / -2.7 |
| `place/wood3.ogg` | placing a block of wooden planks down, single solid thud | 12037 | -16.5 (-16.5) / -6.2 |
| `hit/wood1.ogg` | tapping wooden planks with a pickaxe, single light knock | 12043 | -21.2 (-21.1) / -6.1 |
| `hit/wood2.ogg` | tapping wooden planks with a pickaxe, single light knock, crisp | 12047 | -21.2 (-21.1) / -6.5 |
| `hit/wood3.ogg` | striking wooden planks with a tool, short hit, lighter and softer | 12046 | -21.2 (-21.1) / -4.9 |
| `step/gravel4.ogg` | single footstep on gravel, heavier | 12051 | -25.9 (-25.9) / -17.0 |
| `step/gravel5.ogg` | boot step on gravel, lighter and softer | 12052 | -25.9 (-25.9) / -15.2 |
| `step/gravel6.ogg` | boot step on gravel, muffled | 12054 | -25.9 (-25.9) / -16.1 |
| `dig/gravel4.ogg` | breaking a block of gravel, single hit, short and sharp | 12056 | -16.6 (-12.2) / -2.0 |
| `dig/gravel5.ogg` | breaking a block of gravel, single hit, muffled | 12060 | -14.5 (-12.2) / -1.8 |
| `dig/gravel6.ogg` | digging and scraping gravel with a tool, single, crisp | 12059 | -14.0 (-12.2) / -1.5 |
| `place/gravel1.ogg` | placing a block of gravel down, single solid thud | 12061 | -12.7 (-12.2) / -2.1 |
| `place/gravel2.ogg` | placing a block of gravel down, single solid thud, crisp | 12065 | -13.2 (-12.2) / -1.9 |
| `place/gravel3.ogg` | placing a block of gravel down, single solid thud, heavier | 12063 | -13.9 (-12.2) / -2.1 |
| `hit/gravel1.ogg` | striking gravel with a tool, short hit, muffled | 12072 | -25.9 (-25.9) / -11.2 |
| `hit/gravel2.ogg` | striking gravel with a tool, short hit, lighter and softer | 12070 | -25.9 (-25.9) / -11.1 |
| `hit/gravel3.ogg` | tapping gravel with a pickaxe, single light knock, crisp | 12071 | -25.9 (-25.9) / -8.8 |
| `step/grass4.ogg` | boot step on grass, muffled | 12078 | -26.8 (-26.8) / -16.5 |
| `step/grass5.ogg` | boot step on grass, short and sharp | 12074 | -26.8 (-26.8) / -12.9 |
| `step/grass6.ogg` | boot step on grass, lighter and softer | 12076 | -26.7 (-26.8) / -16.8 |
| `dig/grass4.ogg` | breaking a block of grass, single hit, muffled | 12084 | -13.9 (-13.0) / -2.0 |
| `dig/grass5.ogg` | breaking a block of grass, single hit, short and sharp | 12080 | -15.5 (-13.0) / -1.6 |
| `dig/grass6.ogg` | breaking a block of grass, single hit, lighter and softer | 12082 | -15.0 (-13.0) / -1.8 |
| `place/grass1.ogg` | setting a grass block in place, short, short and sharp | 12086 | -15.1 (-13.0) / -2.0 |
| `place/grass2.ogg` | setting a grass block in place, short, muffled | 12090 | -13.1 (-13.0) / -2.0 |
| `place/grass3.ogg` | placing a block of grass down, single solid thud, crisp | 12089 | -13.0 (-13.0) / -4.0 |
| `hit/grass1.ogg` | tapping grass with a pickaxe, single light knock, heavier | 12093 | -26.8 (-26.8) / -14.9 |
| `hit/grass2.ogg` | tapping grass with a pickaxe, single light knock | 12091 | -26.8 (-26.8) / -14.6 |
| `hit/grass3.ogg` | tapping grass with a pickaxe, single light knock, crisp | 12095 | -26.8 (-26.8) / -14.4 |
| `step/sand4.ogg` | boot step on sand, muffled | 12102 | -23.0 (-23.0) / -14.3 |
| `step/sand5.ogg` | single footstep on sand | 12097 | -23.0 (-23.0) / -7.8 |
| `step/sand6.ogg` | boot step on sand, short and sharp | 12098 | -23.0 (-23.0) / -10.2 |
| `dig/sand4.ogg` | breaking a block of sand, single hit, muffled | 12108 | -18.5 (-18.5) / -7.4 |
| `dig/sand5.ogg` | breaking a block of sand, single hit, short and sharp | 12104 | -18.5 (-18.5) / -5.5 |
| `dig/sand6.ogg` | breaking a block of sand, single hit, lighter and softer | 12106 | -18.5 (-18.5) / -3.4 |
| `place/sand1.ogg` | setting a sand block in place, short, short and sharp | 12110 | -18.5 (-18.5) / -4.7 |
| `place/sand2.ogg` | placing a block of sand down, single solid thud | 12109 | -18.5 (-18.5) / -6.9 |
| `place/sand3.ogg` | placing a block of sand down, single solid thud, crisp | 12113 | -18.5 (-18.5) / -7.5 |
| `hit/sand1.ogg` | tapping sand with a pickaxe, single light knock, crisp | 12119 | -23.0 (-23.0) / -9.2 |
| `hit/sand2.ogg` | tapping sand with a pickaxe, single light knock | 12115 | -23.0 (-23.0) / -9.7 |
| `hit/sand3.ogg` | striking sand with a tool, short hit, short and sharp | 12116 | -23.0 (-23.0) / -8.4 |
| `step/glass4.ogg` | boot step on glass, lighter and softer | 12124 | -26.5 (-26.4) / -11.9 |
| `step/glass5.ogg` | boot step on glass, muffled | 12126 | -26.5 (-26.4) / -17.7 |
| `step/glass6.ogg` | single footstep on glass, heavier | 12123 | -26.4 (-26.4) / -16.5 |
| `dig/glass4.ogg` | breaking a block of glass, single hit, short and sharp | 12128 | -17.5 (-17.5) / -3.4 |
| `dig/glass5.ogg` | breaking a block of glass, single hit, muffled | 12132 | -17.5 (-17.5) / -6.7 |
| `dig/glass6.ogg` | breaking a block of glass, single hit, lighter and softer | 12130 | -17.5 (-17.5) / -3.3 |
| `place/glass1.ogg` | placing a block of glass down, single solid thud | 12133 | -17.4 (-17.5) / -4.8 |
| `place/glass2.ogg` | setting a glass block in place, short, muffled | 12138 | -17.5 (-17.5) / -4.7 |
| `place/glass3.ogg` | placing a block of glass down, single solid thud, crisp | 12137 | -17.5 (-17.5) / -4.5 |
| `hit/glass1.ogg` | striking glass with a tool, short hit, short and sharp | 12140 | -26.4 (-26.4) / -16.2 |
| `hit/glass2.ogg` | striking glass with a tool, short hit, muffled | 12144 | -26.4 (-26.4) / -17.1 |
| `hit/glass3.ogg` | striking glass with a tool, short hit, lighter and softer | 12142 | -26.4 (-26.4) / -12.1 |
| `step/cloth4.ogg` | boot step on wool cloth, muffled | 12150 | -26.1 (-26.1) / -16.1 |
| `step/cloth5.ogg` | single footstep on wool cloth, heavier | 12147 | -26.1 (-26.1) / -18.6 |
| `step/cloth6.ogg` | boot step on wool cloth, lighter and softer | 12148 | -26.1 (-26.1) / -16.8 |
| `dig/cloth4.ogg` | breaking a block of wool cloth, single hit, muffled | 12156 | -12.9 (-11.9) / -2.0 |
| `dig/cloth5.ogg` | breaking a block of wool cloth, single hit, short and sharp | 12152 | -13.7 (-11.9) / -1.9 |
| `dig/cloth6.ogg` | breaking a block of wool cloth, single hit, lighter and softer | 12154 | -17.6 (-11.9) / -1.8 |
| `place/cloth1.ogg` | placing a block of wool cloth down, single solid thud | 12157 | -12.2 (-11.9) / -2.0 |
| `place/cloth2.ogg` | placing a block of wool cloth down, single solid thud, heavier | 12159 | -11.9 (-11.9) / -3.3 |
| `place/cloth3.ogg` | placing a block of wool cloth down, single solid thud, crisp | 12161 | -12.4 (-11.9) / -2.1 |
| `hit/cloth1.ogg` | striking wool cloth with a tool, short hit, muffled | 12168 | -26.1 (-26.1) / -16.1 |
| `hit/cloth2.ogg` | striking wool cloth with a tool, short hit, short and sharp | 12164 | -26.2 (-26.1) / -16.5 |
| `hit/cloth3.ogg` | striking wool cloth with a tool, short hit, lighter and softer | 12166 | -26.2 (-26.1) / -14.2 |
| `step/dirt4.ogg` | boot step on dirt soil, lighter and softer | 12172 | -27.4 (-27.5) / -17.7 |
| `step/dirt5.ogg` | boot step on dirt soil, muffled | 12174 | -27.5 (-27.5) / -20.1 |
| `step/dirt6.ogg` | boot step on dirt soil, short and sharp | 12170 | -27.5 (-27.5) / -17.1 |
| `dig/dirt4.ogg` | breaking a block of dirt soil, single hit, lighter and softer | 12178 | -12.9 (-12.9) / -2.9 |
| `dig/dirt5.ogg` | breaking a block of dirt soil, single hit, muffled | 12180 | -13.8 (-12.9) / -2.0 |
| `dig/dirt6.ogg` | breaking a block of dirt soil, single hit, short and sharp | 12176 | -14.8 (-12.9) / -2.0 |
| `place/dirt1.ogg` | placing a block of dirt soil down, single solid thud, heavier | 12183 | -12.9 (-12.9) / -5.3 |
| `place/dirt2.ogg` | setting a dirt soil block in place, short, lighter and softer | 12184 | -16.2 (-12.9) / -1.9 |
| `place/dirt3.ogg` | placing a block of dirt soil down, single solid thud | 12181 | -12.9 (-12.9) / -4.0 |
| `hit/dirt1.ogg` | tapping dirt soil with a pickaxe, single light knock, crisp | 12191 | -27.5 (-27.5) / -16.3 |
| `hit/dirt2.ogg` | tapping dirt soil with a pickaxe, single light knock | 12187 | -27.5 (-27.5) / -16.3 |
| `hit/dirt3.ogg` | striking dirt soil with a tool, short hit, muffled | 12192 | -27.5 (-27.5) / -18.4 |
| `step/snow4.ogg` | boot step on fresh snow, muffled | 12198 | -21.9 (-22.0) / -10.9 |
| `step/snow5.ogg` | boot step on fresh snow, lighter and softer | 12196 | -22.0 (-22.0) / -11.5 |
| `step/snow6.ogg` | boot step on fresh snow, short and sharp | 12194 | -22.0 (-22.0) / -9.5 |
| `dig/snow4.ogg` | breaking a block of fresh snow, single hit, lighter and softer | 12202 | -20.7 (-20.7) / -5.9 |
| `dig/snow5.ogg` | breaking a block of fresh snow, single hit, muffled | 12204 | -20.7 (-20.7) / -7.9 |
| `dig/snow6.ogg` | breaking a block of fresh snow, single hit, short and sharp | 12200 | -20.7 (-20.7) / -4.3 |
| `place/snow1.ogg` | placing a block of fresh snow down, single solid thud | 12205 | -20.7 (-20.7) / -11.3 |
| `place/snow2.ogg` | placing a block of fresh snow down, single solid thud, heavier | 12207 | -20.7 (-20.7) / -9.9 |
| `place/snow3.ogg` | placing a block of fresh snow down, single solid thud, crisp | 12209 | -20.7 (-20.7) / -12.0 |
| `hit/snow1.ogg` | striking fresh snow with a tool, short hit, lighter and softer | 12214 | -22.0 (-22.0) / -13.0 |
| `hit/snow2.ogg` | tapping fresh snow with a pickaxe, single light knock, crisp | 12215 | -22.0 (-22.0) / -8.3 |
| `hit/snow3.ogg` | tapping fresh snow with a pickaxe, single light knock, heavier | 12213 | -22.0 (-22.0) / -8.8 |
| `step/leaves4.ogg` | boot step on dry leaves, lighter and softer | 12220 | -28.9 (-28.8) / -16.4 |
| `step/leaves5.ogg` | single footstep on dry leaves, heavier | 12219 | -28.9 (-28.8) / -14.7 |
| `step/leaves6.ogg` | boot step on dry leaves, muffled | 12222 | -28.8 (-28.8) / -15.7 |
| `dig/leaves4.ogg` | digging and scraping dry leaves with a tool, single | 12223 | -20.1 (-20.1) / -9.3 |
| `dig/leaves5.ogg` | digging and scraping dry leaves with a tool, single, heavier | 12225 | -20.1 (-20.1) / -9.8 |
| `dig/leaves6.ogg` | breaking a block of dry leaves, single hit, short and sharp | 12224 | -20.1 (-20.1) / -6.1 |
| `place/leaves1.ogg` | placing a block of dry leaves down, single solid thud | 12229 | -20.1 (-20.1) / -7.4 |
| `place/leaves2.ogg` | placing a block of dry leaves down, single solid thud, heavier | 12231 | -20.0 (-20.1) / -7.3 |
| `place/leaves3.ogg` | setting a dry leaves block in place, short, lighter and softer | 12232 | -20.0 (-20.1) / -6.1 |
| `hit/leaves1.ogg` | tapping dry leaves with a pickaxe, single light knock, crisp | 12239 | -28.8 (-28.8) / -13.6 |
| `hit/leaves2.ogg` | striking dry leaves with a tool, short hit, short and sharp | 12236 | -28.8 (-28.8) / -17.3 |
| `hit/leaves3.ogg` | striking dry leaves with a tool, short hit, lighter and softer | 12238 | -28.9 (-28.8) / -16.1 |
| `step/ice4.ogg` | boot step on ice, muffled | 12246 | -25.6 (-25.6) / -18.8 |
| `step/ice5.ogg` | single footstep on ice | 12241 | -25.5 (-25.6) / -16.4 |
| `step/ice6.ogg` | single footstep on ice, heavier | 12243 | -25.6 (-25.6) / -18.2 |
| `dig/ice4.ogg` | breaking a block of ice, single hit, muffled | 12252 | -19.4 (-19.4) / -5.5 |
| `dig/ice5.ogg` | breaking a block of ice, single hit, short and sharp | 12248 | -19.4 (-19.4) / -5.5 |
| `dig/ice6.ogg` | breaking a block of ice, single hit, lighter and softer | 12250 | -19.4 (-19.4) / -4.2 |
| `place/ice1.ogg` | placing a block of ice down, single solid thud, crisp | 12257 | -19.4 (-19.4) / -8.1 |
| `place/ice2.ogg` | placing a block of ice down, single solid thud | 12253 | -19.4 (-19.4) / -8.1 |
| `place/ice3.ogg` | placing a block of ice down, single solid thud, heavier | 12255 | -19.4 (-19.4) / -7.5 |
| `hit/ice1.ogg` | tapping ice with a pickaxe, single light knock, crisp | 12263 | -25.6 (-25.6) / -11.3 |
| `hit/ice2.ogg` | striking ice with a tool, short hit, lighter and softer | 12262 | -25.5 (-25.6) / -12.5 |
| `hit/ice3.ogg` | striking ice with a tool, short hit, short and sharp | 12260 | -25.6 (-25.6) / -10.0 |
| `step/mud4.ogg` | boot step on wet mud, lighter and softer | 12268 | -21.9 (-21.9) / -10.0 |
| `step/mud5.ogg` | boot step on wet mud, short and sharp | 12266 | -22.0 (-21.9) / -12.2 |
| `step/mud6.ogg` | boot step on wet mud, muffled | 12270 | -21.9 (-21.9) / -7.7 |
| `dig/mud4.ogg` | breaking a block of wet mud, single hit, muffled | 12276 | -23.5 (-23.5) / -8.6 |
| `dig/mud5.ogg` | breaking a block of wet mud, single hit, lighter and softer | 12274 | -23.5 (-23.5) / -8.7 |
| `dig/mud6.ogg` | breaking a block of wet mud, single hit, short and sharp | 12272 | -23.4 (-23.5) / -9.0 |
| `place/mud1.ogg` | setting a wet mud block in place, short, short and sharp | 12278 | -23.5 (-23.5) / -14.4 |
| `place/mud2.ogg` | placing a block of wet mud down, single solid thud | 12277 | -23.5 (-23.5) / -14.6 |
| `place/mud3.ogg` | placing a block of wet mud down, single solid thud, heavier | 12279 | -23.4 (-23.5) / -13.8 |
| `hit/mud1.ogg` | tapping wet mud with a pickaxe, single light knock, heavier | 12285 | -21.9 (-21.9) / -7.0 |
| `hit/mud2.ogg` | tapping wet mud with a pickaxe, single light knock, crisp | 12287 | -21.9 (-21.9) / -7.3 |
| `hit/mud3.ogg` | tapping wet mud with a pickaxe, single light knock | 12283 | -21.9 (-21.9) / -7.8 |
| `step/metal4.ogg` | boot step on iron metal, lighter and softer | 12292 | -24.2 (-24.2) / -12.3 |
| `step/metal5.ogg` | boot step on iron metal, muffled | 12294 | -24.2 (-24.2) / -12.7 |
| `step/metal6.ogg` | single footstep on iron metal, crisp | 12293 | -24.2 (-24.2) / -10.6 |
| `dig/metal4.ogg` | breaking a block of iron metal, single hit, short and sharp | 12296 | -19.5 (-19.5) / -4.4 |
| `dig/metal5.ogg` | breaking a block of iron metal, single hit, muffled | 12300 | -19.5 (-19.5) / -4.6 |
| `dig/metal6.ogg` | breaking a block of iron metal, single hit, lighter and softer | 12298 | -19.5 (-19.5) / -2.8 |
| `place/metal1.ogg` | placing a block of iron metal down, single solid thud, crisp | 12305 | -19.5 (-19.5) / -6.7 |
| `place/metal2.ogg` | placing a block of iron metal down, single solid thud | 12301 | -19.5 (-19.5) / -7.8 |
| `place/metal3.ogg` | setting a iron metal block in place, short, short and sharp | 12302 | -19.5 (-19.5) / -4.9 |
| `hit/metal1.ogg` | tapping iron metal with a pickaxe, single light knock | 12307 | -24.2 (-24.2) / -8.7 |
| `hit/metal2.ogg` | tapping iron metal with a pickaxe, single light knock, heavier | 12309 | -24.2 (-24.2) / -12.6 |
| `hit/metal3.ogg` | tapping iron metal with a pickaxe, single light knock, crisp | 12311 | -24.2 (-24.2) / -8.0 |
| `step/water1.ogg` | walking step in ankle deep water, muffled | 12318 | -25.2 (-25.1) / -11.9 |
| `step/water2.ogg` | walking step in ankle deep water, lighter and softer | 12316 | -25.0 (-25.1) / -12.3 |
| `step/water3.ogg` | single footstep wading through shallow water, splash, crisp | 12317 | -25.1 (-25.1) / -11.8 |
| `step/water4.ogg` | single footstep wading through shallow water, splash | 12319 | -25.1 (-25.1) / -12.9 |
| `step/water5.ogg` | single footstep wading through shallow water, splash, heavier | 12315 | -25.1 (-25.1) / -12.1 |
