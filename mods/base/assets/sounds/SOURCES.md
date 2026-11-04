# Where base's sounds come from

All the files listed here are **CC0** (public domain dedication): no attribution needed. They are
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

## A falling tree (`tree/`)

Made 2026-11-05 by the Tree felling session with `reports/sound-files/fall_sounds.py` (numpy and
ffmpeg), from nothing but synthesis and CC0 files already here:

| files | made from | licence |
|---|---|---|
| `tree/creak1-2` | synthesized stick-slip creak through wood resonances; the crack at its end is Kenney's Impact Sounds `impactWood_light_001-002` | ours + CC0 |
| `tree/fall1-2` | swept noise (synthesized) with base's own `dig/leaves1-3` (Kenney, Impact Sounds) scattered through | ours + CC0 |
| `tree/thud1-2` | Kenney's Impact Sounds `impactWood_heavy_001-002` an octave down, a synthesized low thump, `dig/leaves1-3` | CC0 + ours |
