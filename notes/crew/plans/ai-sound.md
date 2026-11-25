# AI-generated sound effects: what's missing, which tool

Ronald, 11-25, asked whether to generate the missing sounds with AI or stay with CC0: "Try AI sounds".
This file lists the gaps, compares the tools, and recommends one. Nothing is downloaded or generated
until Ronald says yes (through Planner). Written 2026-11-25 on `claude/ai-sound-36697c`.

## 1. What's missing

Based on dev at 6043dfdb7: `animals_core::voices`, `mods/animals/src/voices.rs`, `mods/base/src/sounds.rs`,
`assets/sounds/groups.txt`, both `SOURCES.md` files and `mods/animals/SPECIES.md`.

**Borrowed voices (no sound of their own):**

| who | plays now | what it should sound like |
|---|---|---|
| `borkon_kid` (Borkon's ibex kid) | the reindeer's (`deer` family) | a young ibex or goat kid's bleat |
| `drenkaun_calf` (Drenkaun's calf) | the reindeer's | a calf's low bawl |
| Drenkaun and Borkon themselves | no voice wiring found in the code (only Drenkaun's stone sound group) | a giant beast's bellow and a horned beast's roar: to check with `/sounds check` |
| capybara | `rodent` (mouse squeaks at pitch 0.55) | clicks, purrs, whistles and a gruff bark |
| reindeer | `deer` (a doe's mew and a red stag's roar) | grunts and snorts |
| jaguar | `big_cat` (a tiger's hiss and growl) | a jaguar's "sawing" cough-roar |
| 6 big plant-eating dinosaurs (protoceratops, psittacosaurus, gastonia, pinacosaurus, iguanodont, cedarosaurus) | `bellow`: all five files cut from one "T-Rex Calls" recording | closed-mouth booms, honks and rumbles, different for each size |
| 4 small feathered dinosaurs (raptor, osmolskae, utahraptor, citipati) | `small_dino`: goose hisses and two "dinosaur" clips | birdlike hisses, chirps and squawks |
| confuciusornis, gobipteryx | `songbird` (a house sparrow) | simple, harsher bird calls |
| kryptobaatar, priscagama | rodent squeaks and lizard hisses (fair stand-ins) | low priority |
| fish (4 species) | silent by design | the splash on leaving water is already in `ambient/splash` |

**Every species, one by one.** Every voiced species plays a family's files (`animals_core::voices`),
pitched for its size. "Own" means the family was recorded from that animal or a very close one.
"Borrowed" means another animal stands in. "Invented" means an extinct or made-up animal with
recordings picked to suggest it. The Animals session's behaviour table will be cross-checked here
when Planner forwards it.

| species | family | status | note |
|---|---|---|---|
| dog (chihuahua) | canine | own | dog barks, chihuahua whines |
| grey_wolf | canine | own | wolf howl |
| coyote | canine | own | coyote barks |
| red_fox | fox | own | fox screams |
| arctic_fox | fox | own (close) | red fox's screams |
| cat | cat | own | |
| jaguar | big_cat | borrowed | a tiger's growl; jaguars give a sawing cough |
| white_tailed_deer | deer | own (close) | a doe's mew |
| red_deer | deer | own | a stag's roar |
| reindeer | deer | **borrowed** | reindeer grunt and click |
| capybara | rodent | **borrowed** | mouse squeaks at 0.55; real ones click, purr, whistle, bark |
| wood_mouse, field_vole | rodent | own (close) | |
| kryptobaatar | rodent | invented | mouse squeaks fit |
| hare, mountain_hare | hare | **borrowed** | rat and mouse squeaks; a hare's scream is louder and harsher. Silent by design otherwise |
| house_sparrow | songbird | own | |
| snow_bunting | songbird | borrowed | sparrow; a bunting trills |
| confuciusornis, gobipteryx | songbird | invented | sparrow chirps |
| golden_eagle | hawk | borrowed | a hawk's scream; eagles yelp and whistle thinly |
| savanna_hawk | hawk | own (close) | |
| great_horned_owl | owl | own (close) | hoots |
| snowy_owl | owl | borrowed | hoots; snowy owls bark and croak |
| herring_gull | gull | own | |
| raven | corvid | own | |
| turkey_vulture | hiss | borrowed, fair | vultures do hiss; the file is an alligator's |
| yacare_caiman, spectacled_caiman | hiss | own (close) | alligator hiss |
| crocodyliform | bellow | **invented** | cut from "T-Rex Calls" |
| priscagama | hiss | borrowed, fair | snake hiss |
| raptor, osmolskae, utahraptor, citipati | small_dino | **invented** | goose hisses and two "dinosaur" clips |
| psittacosaurus, protoceratops, gastonia, pinacosaurus, iguanodont, cedarosaurus | bellow | **invented** | all five files cut from one "T-Rex Calls" recording |
| wyvern | dragon | invented | four dragon-roar clips; fine |
| borkon_kid, drenkaun_calf | reindeer's | **borrowed** | (above) |
| Drenkaun, Borkon | none found | **none?** | to check with `/sounds check` |
| brown_trout, pacu, lungfish, lycoptera | none | silent by design | |

In bold: the ones AI could help most with, because no recording of the real animal exists (dinosaurs,
the bosses and their young) or the stand-in is clearly another animal (capybara, reindeer, hares).

**Blocks:** dropped from the list. This was an old 11-04 playtest line. Every dirt-like block already
has a fitting group of its own:
- `dirt`, `clay`, `farmland`, `podzol` and the charcoal clamp are in the `dirt` group (`groups.txt`), with their own CC0 dirt steps and digs since 11-04.
- `coarse_dirt` and `dried_mud` use `gravel`, as vanilla does.
- `dung_pat` uses `mud`.
- Leaf litter and moss use `grass`, as they should.

There are no rooted-dirt or path blocks. The gaps that remain:
- Only glass, ice and metal have their own break sounds. Every other group breaks with its dig files.
- The engine has no sneak, sprint, fall, landing or swim steps (`reports/sound.md` E8).
- No sounds for eating, drinking, splashing into water, or an item breaking.

**Order for the test batch** (4 to 6 sounds, about 3 tries each): borkon_kid bleat, drenkaun_calf bawl,
a big dinosaur boom for the plant-eaters, a small-raptor chirp and hiss, a capybara call, and a reindeer
grunt. These cover a young animal, an invented creature and two real animals whose recordings we lack,
so one batch shows where AI is good and where CC0 recordings still win.

## 2. The tools

The things that matter here:
- short one-shot clips (0.5 to 3 s)
- clean onsets
- animal calls that sound like the animal, not a smeared texture
- commercial use of the outputs
- weights that run on the PC's RTX 5090

| tool | how it's run | short-SFX quality (from model cards, papers and reports; not heard yet) | licence for commercial use of outputs | weight restrictions | cost | GPU memory |
|---|---|---|---|---|---|---|
| **Stable Audio 3 Small SFX** (`stabilityai/stable-audio-3-small-sfx`, May 2026) | open weights, local | Built for sound effects: 0.6B params and a few seconds per clip on an M4 MacBook. Trained on 1.28 M recordings: 806 k licensed from AudioSparx, plus Freesound CC0, CC-BY and Sampling+. The licensed library should help with animal and foley sounds. | Stability AI Community License: **you own the outputs** and may sell them, free for anyone under $1M in annual revenue, Enterprise licence above that | Gated on Hugging Face: you accept the licence and share your contact details. The T5Gemma text encoder bundled with it adds the Gemma Terms of Use (use restrictions, §3.2). | free | small. No figure is stated; 0.6B at F32 is about 2.4 GB plus the encoder. It would fit next to anything on the 5090 and could even run on the Mac. |
| Stable Audio Open 1.0 (`stabilityai/stable-audio-open-1.0`, 2024) | open weights, local | Its card says it is "better at sound effects and field recordings than music". 44.1 kHz stereo, up to 47 s. Trained only on Freesound and FMA CC audio, so animal calls are weaker than with a licensed library, and it is older and slower than SA3. | Stability AI Community License (same as above) | gated, same as above | free | about 4 to 7 GB at FP16 (third-party figure) |
| AudioLDM 2 | open weights | 16 kHz output, which sounds dull for game SFX | **CC BY-NC-SA 4.0: no commercial use** | — | free | about 8 GB |
| MMAudio | open weights | Strong, mostly video-to-audio, 16 kHz and 44 kHz versions | **weights CC BY-NC 4.0: no commercial use** | — | free | about 6 to 10 GB |
| Tango 2 / TangoFlux | open weights | Good benchmarks, 44.1 kHz stereo, up to 30 s | **non-commercial research only** (WavCaps data, UK exemption) | — | free | about 6 GB |
| Meta AudioGen (AudioCraft) | open weights | 16 kHz, older | **weights CC BY-NC 4.0** | — | free | — |
| HunyuanVideo-Foley (Tencent) | open weights | Strong foley, but made for video-to-audio | Tencent Hunyuan Community License: grants nothing in the EU, UK or Korea, with conditions elsewhere | gated | free | large (an 8B-class video model) |
| ThinkSound | open weights | video-to-audio | third parties class it as non-commercial (not confirmed) | — | free | — |
| **ElevenLabs Sound Effects (v2)** | paid API / web | Widely rated the best short-SFX generator: 48 kHz, up to 30 s, loops | Commercial use only on paid plans (Starter, about $5-6 a month and up). The free tier is non-commercial. | none (closed) | about 11 credits a second via the API, roughly $0.002 a second; a 3-try, 6-sound batch costs pennies | none (cloud) |
| Stable Audio 3 Large / Stable Audio 2.5 API | paid API | Larger, music-first | outputs owned, commercial | closed | per-generation price not found | none (cloud) |

The licence fence: most research models (AudioLDM 2, MMAudio, Tango, AudioGen, ThinkSound) are
**non-commercial**, so their outputs cannot go into a game we may sell. Of the open-weight tools, only
Stability's models let us own and sell what they make.

## 3. Recommendation

**Stable Audio 3 Small SFX, run on the PC.**
- It is the only current open-weight model made for sound effects that allows commercial use of its outputs.
- It is small (a short, cheap GPU job) and trained largely on licensed audio.
- It needs no account beyond accepting the licence on Hugging Face, and no money.

Its terms need saying plainly to Ronald:
- The Stability AI Community License (free while revenue is under $1M a year; we own the outputs).
- The Gemma Terms of Use for the text encoder.
- Accepting both on Hugging Face takes his account or his yes.

**Fallback and comparison: ElevenLabs Sound Effects** on a paid Starter plan (about $5-6 a month),
if SA3's animal calls don't convince. It needs his account and money, so it waits for his separate yes.
A one-month plan costs little to compare side by side.

**Every output's licence is recorded** in `SOURCES.md` next to the CC0 rows: tool, model, prompt, seed
and date. The raw tries go to `~/Coding/chiacraft-archive/sounds/ai-sound/<date>/`, and none are ever
deleted.

## 4. Steps after the yes

1. Accept the licence on Hugging Face. This takes Ronald's own account, or a token he puts on the PC himself; I never handle it. Then download about 3 GB to the PC.
2. Tell Tester 2 before the GPU job: a few minutes, small.
3. Generate about 3 tries each for the 6 sounds above.
4. Cut and level them with `reports/sound-files/sfx.py`, the same way as the CC0 files:
   - mono analysis, written as 44.1 kHz Ogg Vorbis at 112k
   - ambient calls 2.5 s at -3 dB RMS
   - hurt 0.8 s and death 1.6 s at -2 dB RMS
   - steps short, like `step/*`
5. Build a listening page that plays each try next to the closest CC0 file now in the game.
6. After Ronald picks: put the picks in the voice families (new families for the kid, the calf, the plant-eating dinosaurs and the capybara), wire them in `animals_core::voices`, add the licence rows, and say "ready to merge".

## Sources
- [Stable Audio 3 announcement](https://stability.ai/news-updates/meet-stable-audio-3-the-model-family-built-for-artistic-experimentation-with-open-weight-models)
- [stable-audio-3-small-sfx model card](https://huggingface.co/stabilityai/stable-audio-3-small-sfx)
- [stable-audio-open-1.0 model card](https://huggingface.co/stabilityai/stable-audio-open-1.0)
- [Stability AI licence](https://stability.ai/license)
- [TangoFlux licence](https://github.com/declare-lab/TangoFlux)
- [HunyuanVideo-Foley licence notes](https://github.com/reeleehq/foley/pull/174)
- [ElevenLabs sound effects cost](https://elevenlabs.io/docs/help-center/product/content-production/sound-effects/how-much-does-it-cost-to-generate-sound-effects)
- [ElevenLabs API pricing](https://elevenlabs.io/pricing/api)
- [ElevenLabs plans](https://www.elevenlabs.io/pricing)
- [Open TTS and audio model licences (AudioGen, MMAudio)](https://invideo.io/blog/open-ai-voice-audio-models/)

## Test batch 1 (2026-11-25)

Run on the PC in `~/ai-sound`: its own venv (Python 3.12, torch 2.7.1 cu128, Stability's `stable-audio-3`
library) and the weights in `~/ai-sound/model`. 8 sampling steps, about 0.15 s a clip, 2.3 GB peak VRAM.

Every file is kept in `~/Coding/chiacraft-archive/sounds/ai-sound/2026-11-25/`:
- `raw/`: the raw tries, 44.1 kHz stereo WAV
- `cut/`: the same tries cut and levelled with `sfx.py` (copy mode, 2.5 s, -3 dB RMS, Ogg)
- `now/`: the game's current sounds, rendered at each species' pitch
- `prompts.tsv`: name, length, seed, prompt
- spectrogram sheets
- `index.html`: the listening page (also https://claude.ai/artifact/1B8w5DX5keKs4V7oyLHm9G)
- `build.sh` and `page.py`, which rebuild the rest

Round 1 had 3 tries each for kid, calf, big dinosaur, raptor, capybara, reindeer, Drenkaun and Borkon.
Round 2 had 3 more for the dinosaurs, both bosses and the capybara. In round 1 those came out as noise
beds, so the round 2 prompts name a real animal instead.

Judged from the spectrograms only, these show clear calls:
- the kid
- the calf
- raptor 1-2
- reindeer 3
- capybara 3-6
- Drenkaun 4-6
- Borkon 4

The rest look like noise beds. Ronald picks by ear.
