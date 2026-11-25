# Where the animals' sounds come from

## Voices (`voice/<family>/`)

Every species' calls come from a voice family (`animals_core::voices`, pitched per species).
Prepared 2026-11-04 by the Sound engine session with `reports/sound-files/sfx.py` from
`reports/sound-files/voices.tsv` (written by `voices_plan.py`): mono, 44.1 kHz, trimmed, cut to
length (ambient 2.5 s, hurt 0.8 s, death 1.6 s), levelled, Ogg Vorbis. `hit:N` is the N-th
strongest separate call cut from a longer recording. Every source's own Freesound page said
"Creative Commons 0" on 2026-11-04; the files are its high-quality preview, kept in
`~/Coding/chiacraft-archive/references/sound/freesound/`. CC0 needs no credit; it is given anyway.

| files | source | author | licence |
|---|---|---|---|
| `canine/ambient1` | "Wolf howl" ([freesound 398430](https://freesound.org/people/NaturesTemper/sounds/398430/)) | NaturesTemper | CC0 |
| `canine/ambient2` (hit:0) | "coyote barks and howls" ([freesound 256533](https://freesound.org/people/dkaufman/sounds/256533/)) | dkaufman | CC0 |
| `canine/ambient3` (hit:0) | "Dog Bark1.wav" ([freesound 9031](https://freesound.org/people/MisterTood/sounds/9031/)) | MisterTood | CC0 |
| `canine/hurt1` | "Chihuahua Puppy Whine" ([freesound 350593](https://freesound.org/people/AustinXYZ/sounds/350593/)) | AustinXYZ | CC0 |
| `canine/hurt2` (hit:0), `canine/death2` (hit:1) | "Chihuahua Whines" ([freesound 208031](https://freesound.org/people/Mewsel/sounds/208031/)) | Mewsel | CC0 |
| `canine/hurt3` (hit:0) | "Dog bark2.wav" ([freesound 9032](https://freesound.org/people/MisterTood/sounds/9032/)) | MisterTood | CC0 |
| `canine/death1` (hit:0) | "Whining Dog" ([freesound 462660](https://freesound.org/people/Breviceps/sounds/462660/)) | Breviceps | CC0 |
| `fox/ambient1` (hit:0), `fox/ambient2` (hit:1) | "Fox Screams" ([freesound 634005](https://freesound.org/people/Soundburst/sounds/634005/)) | Soundburst | CC0 |
| `fox/hurt1` (hit:0), `fox/hurt2` (hit:1) | "Fox scream in the forest" ([freesound 554308](https://freesound.org/people/nielstii/sounds/554308/)) | nielstii | CC0 |
| `fox/death1` (hit:0) | "Fox scream.wav" ([freesound 518135](https://freesound.org/people/lonskwad2020/sounds/518135/)) | lonskwad2020 | CC0 |
| `cat/ambient1` | "cat meow" ([freesound 110011](https://freesound.org/people/tuberatanka/sounds/110011/)) | tuberatanka | CC0 |
| `cat/ambient2` | "Cat Meowing" ([freesound 333916](https://freesound.org/people/lextrack/sounds/333916/)) | lextrack | CC0 |
| `cat/ambient3` | "cat meow short" ([freesound 412017](https://freesound.org/people/skymary/sounds/412017/)) | skymary | CC0 |
| `cat/hurt1` (hit:0), `cat/hurt2` (hit:1) | "Cats Fighting Amplified" ([freesound 316915](https://freesound.org/people/Rudmer_Rotteveel/sounds/316915/)) | Rudmer_Rotteveel | CC0 |
| `cat/death1` (hit:0) | "catfight.WAV" ([freesound 45199](https://freesound.org/people/stomachache/sounds/45199/)) | stomachache | CC0 |
| `deer/ambient1` (hit:0), `deer/hurt1` (hit:1), `deer/hurt2` (hit:2), `deer/death1` (hit:3) | "Deer Meewing 01.wav" ([freesound 351113](https://freesound.org/people/msantoro11/sounds/351113/)) | msantoro11 | CC0 |
| `deer/ambient2` (hit:0) | "Bellowing deer - Animal - Nature - Hirsch - Röhren - Tiere - Wald" ([freesound 407631](https://freesound.org/people/IchBinChrist/sounds/407631/)) | IchBinChrist | CC0 |
| `rodent/ambient1` (hit:0), `rodent/ambient2` (hit:1) | "Mouse Squeaks.wav" ([freesound 463789](https://freesound.org/people/shyguy014/sounds/463789/)) | shyguy014 | CC0 |
| `rodent/hurt1` | "Mouse" ([freesound 188043](https://freesound.org/people/AntumDeluge/sounds/188043/)) | AntumDeluge | CC0 |
| `rodent/hurt2` | "Rat.ogg" ([freesound 536753](https://freesound.org/people/egomassive/sounds/536753/)) | egomassive | CC0 |
| `rodent/death1` | "Cartoon - Bat / Mouse Squeak" ([freesound 445958](https://freesound.org/people/Breviceps/sounds/445958/)) | Breviceps | CC0 |
| `songbird/ambient1` (hit:0), `songbird/ambient2` (hit:1), `songbird/ambient3` (hit:2) | "House sparrow.wav" ([freesound 404663](https://freesound.org/people/straget/sounds/404663/)) | straget | CC0 |
| `songbird/hurt1` (hit:0), `songbird/death1` (hit:1) | "Sparrows02.wav" ([freesound 628951](https://freesound.org/people/Kimskell/sounds/628951/)) | Kimskell | CC0 |
| `hawk/ambient1` (hit:0), `hawk/ambient2` (hit:1), `hawk/hurt1` (hit:2), `hawk/death1` (hit:3) | "hawk.wav" ([freesound 582183](https://freesound.org/people/denalwa/sounds/582183/)) | denalwa | CC0 |
| `owl/ambient1` | "Owl Hoot" ([freesound 465697](https://freesound.org/people/Breviceps/sounds/465697/)) | Breviceps | CC0 |
| `owl/ambient2` | "owl.wav" ([freesound 398734](https://freesound.org/people/Anthousai/sounds/398734/)) | Anthousai | CC0 |
| `owl/hurt1` (hit:0), `owl/death1` (hit:1) | "screech_owl.wav" ([freesound 353173](https://freesound.org/people/ivolipa/sounds/353173/)) | ivolipa | CC0 |
| `gull/ambient1` | "Seagull single call" ([freesound 263786](https://freesound.org/people/steaq/sounds/263786/)) | steaq | CC0 |
| `gull/ambient2` (hit:0) | "seagull 2.wav" ([freesound 166707](https://freesound.org/people/Snapper4298/sounds/166707/)) | Snapper4298 | CC0 |
| `gull/ambient3` (hit:0) | "Seagulls_short.wav" ([freesound 510917](https://freesound.org/people/Lydmakeren/sounds/510917/)) | Lydmakeren | CC0 |
| `gull/hurt1` (hit:0) | "sea gulls.wav" ([freesound 166703](https://freesound.org/people/Snapper4298/sounds/166703/)) | Snapper4298 | CC0 |
| `gull/death1` (hit:0) | "Seagull on beach" ([freesound 353416](https://freesound.org/people/squashy555/sounds/353416/)) | squashy555 | CC0 |
| `corvid/ambient1` | "Crow Caw" ([freesound 361470](https://freesound.org/people/Jofae/sounds/361470/)) | Jofae | CC0 |
| `corvid/ambient2` (hit:0) | "crow.wav" ([freesound 75162](https://freesound.org/people/nigelcoop/sounds/75162/)) | nigelcoop | CC0 |
| `corvid/ambient3` | "Raven.mp3" ([freesound 66763](https://freesound.org/people/Bidone/sounds/66763/)) | Bidone | CC0 |
| `corvid/hurt1` (hit:0) | "Raven 2.mp3" ([freesound 67356](https://freesound.org/people/Bidone/sounds/67356/)) | Bidone | CC0 |
| `corvid/death1` (hit:0) | "raven attack (No.2)" ([freesound 191258](https://freesound.org/people/Yuval/sounds/191258/)) | Yuval | CC0 |
| `hiss/ambient1` (hit:0), `hiss/death1` (hit:1) | "S27-21 Alligator hiss twice.wav" ([freesound 675513](https://freesound.org/people/craigsmith/sounds/675513/)) | craigsmith | CC0 |
| `hiss/hurt1` | "Reptiles - Snake; Mexican Black Kingsnake, Short Hiss and Lunge" ([freesound 846337](https://freesound.org/people/TheKingOfGeeks360/sounds/846337/)) | TheKingOfGeeks360 | CC0 |
| `hiss/hurt2` (hit:0) | "Angry snake hissing. OWI.wav" ([freesound 706977](https://freesound.org/people/JesterWhoo/sounds/706977/)) | JesterWhoo | CC0 |
| `bellow/ambient1` (hit:0), `bellow/ambient2` (hit:1), `bellow/hurt1` (hit:2), `bellow/hurt2` (hit:3), `bellow/death1` (hit:4) | "T-Rex Calls" ([freesound 529462](https://freesound.org/people/CaveboyTup/sounds/529462/)) | CaveboyTup | CC0 |
| `dragon/ambient1` | "DRAGON_ROAR.wav" ([freesound 85568](https://freesound.org/people/JoelAudio/sounds/85568/)) | JoelAudio | CC0 |
| `dragon/ambient2` | "Dragon Roar" ([freesound 442964](https://freesound.org/people/qubodup/sounds/442964/)) | qubodup | CC0 |
| `dragon/hurt1` (hit:0) | "dragonroar.wav" ([freesound 185069](https://freesound.org/people/juryduty/sounds/185069/)) | juryduty | CC0 |
| `dragon/death1` (hit:0) | "Dragon Death" ([freesound 366096](https://freesound.org/people/bevibeldesign/sounds/366096/)) | bevibeldesign | CC0 |

## Ambient life (`ambient/`)

The sprites' sounds (`mods/animals/src/ambient.rs`), cut by `reports/sound-files/life.tsv` with
`reports/sound-files/sfx.py` from the Freesound HQ previews (fetched by
`reports/sound-files/fetch_freesound.py`, which refuses any sound not CC0). Not yet listened to.

| files | source | author | licence |
|---|---|---|---|
| `eagle1` (hit:0), `eagle2` (hit:1) | "210227 Red Tailed Hawk, eq isolated calls, roof 77mel 10am.wav" ([freesound 616926](https://freesound.org/people/TRP/sounds/616926/)) | TRP | CC0 |
| `eagle3` (hit:0) | "red-tailed-hawk.wav" ([freesound 575524](https://freesound.org/people/1888software/sounds/575524/)) | 1888software | CC0 |
| `splash1` | "Fish Jumping Splash 1.wav" ([freesound 507092](https://freesound.org/people/paulprit/sounds/507092/)) | paulprit | CC0 |
| `splash2` (hit:0) | "Fish Jumping Splash 2.wav" ([freesound 507091](https://freesound.org/people/paulprit/sounds/507091/)) | paulprit | CC0 |
| `splash3` (hit:0), `splash4` (hit:3) | "water_splash_10shots.wav" ([freesound 50623](https://freesound.org/people/vibe_crc/sounds/50623/)) | vibe_crc | CC0 |

The chihuahua's `chihuahua/bark1-3.wav` were here before and are not covered by this file.

## AI voices (`voice/<family>/`, families marked AI in `animals_core::voices`)

Made 2026-11-25 with **Stable Audio 3 Small SFX** (`stabilityai/stable-audio-3-small-sfx`, Stability AI),
run on our own PC with Stability's `stable-audio-3` library (8 steps, 44.1 kHz stereo). Licence: the
[Stability AI Community License](https://stability.ai/license). We own the outputs and may use them
commercially while annual revenue stays under USD 1M; above that, an Enterprise licence is needed.
Its text encoder (T5Gemma) is under the [Gemma Terms of Use](https://ai.google.dev/gemma/terms).
The model was trained on audio licensed from AudioSparx and on Freesound CC0, CC-BY and CC Sampling+
recordings (its model card). Ronald approved using it on 11-25.

Each event got its variants plus two spare candidates, twice: the model gives nearly the same clip for one prompt whatever the seed, so the second round (`v2_`) words each candidate differently, and a pick is skipped when its spectrum is too like one already kept. Every candidate was cut like the CC0 voices
(`reports/sound-files/sfx.py`, copy mode: ambient 2.5 s, hurt 0.8 s, death 1.6 s; the bosses 3 / 1.2 / 3 s,
summon 3.5, attack 1.5, stunned 2). The ones that look most like a call (clear shape, tonal,
not a noise bed) were kept. The raw outputs, every candidate, the scores and the picks are in
`~/Coding/chiacraft-archive/sounds/ai-sound/2026-11-25-sets/` (`b1_` = the first test batch,
`../2026-11-25/`). The prompt each file came from:

| file | prompt | seed |
|---|---|---|
| `goat_kid/ambient1` | "young ibex kid calling, short nasal bleat, close, dry, no background noise, no music" | 1002 |
| `goat_kid/ambient2` | "young ibex kid calling, short nasal bleat, short and sharp, close, dry, no background noise, no music" | 5002 |
| `goat_kid/ambient3` | "young ibex kid calling, short nasal bleat, close, dry, no background noise, no music" | 1004 |
| `goat_kid/hurt1` | "baby goat crying out in pain, one sharp short bleat, close, dry, no background noise, no music" | 1006 |
| `goat_kid/hurt2` | "baby goat crying out in pain, one sharp short bleat, close, dry, no background noise, no music" | 1009 |
| `goat_kid/death1` | "baby goat weak dying bleat, fading out, close, dry, no background noise, no music" | 1012 |
| `calf/ambient1` | "calf calling for its mother, short moo, close, dry, no background noise, no music" | 1016 |
| `calf/ambient2` | "young calf mooing, one short low bawl, softer and calmer, close, dry, no background noise, no music" | 5017 |
| `calf/ambient3` | "young calf mooing, one short low bawl, close, dry, no background noise, no music" | 1015 |
| `calf/hurt1` | "calf bawling in pain, one short loud cry, close, dry, no background noise, no music" | 1020 |
| `calf/hurt2` | "calf bawling in pain, one short loud cry, longer and lower, close, dry, no background noise, no music" | 5020 |
| `calf/death1` | "calf weak moan, fading out, close, dry, no background noise, no music" | 5022 |
| `horned_dino/ambient1` | "elephant low rumbling call, close, dry, no background noise, no music" | 1029 |
| `horned_dino/ambient2` | "bull bison bellowing, one deep roar, close, dry, no background noise, no music" | 1025 |
| `horned_dino/ambient3` | "hippo honking grunt call, longer and lower, close, dry, no background noise, no music" | 5027 |
| `horned_dino/hurt1` | "large animal pained honk, short, louder and more intense, close, dry, no background noise, no music" | 5033 |
| `horned_dino/hurt2` | "bull bison grunting in pain, one short loud bellow, close, dry, no background noise, no music" | 1030 |
| `horned_dino/hurt3` | "bull bison grunting in pain, one short loud bellow, longer and lower, close, dry, no background noise, no music" | 5032 |
| `horned_dino/death1` | "large animal dying groan, long deep moan fading out, close, dry, no background noise, no music" | 5035 |
| `horned_dino/death2` | "large animal dying groan, long deep moan fading out, close, dry, no background noise, no music" | 1035 |
| `raptor/ambient1` | "small feathered predatory dinosaur, sharp birdlike chirp then a hiss, close, dry, no background noise, no music" | 1041 |
| `raptor/ambient2` | "small feathered predatory dinosaur, sharp birdlike chirp then a hiss, close, dry, no music" | 42 |
| `raptor/ambient3` | "small feathered predatory dinosaur, sharp birdlike chirp then a hiss, close, dry, no background noise, no music" | 5039 |
| `raptor/hurt1` | "large bird screeching in pain, one short shriek, close, dry, no background noise, no music" | 1046 |
| `raptor/hurt2` | "goose hissing angrily, short, close, dry, no background noise, no music" | 1047 |
| `raptor/hurt3` | "goose hissing angrily, short, louder and more intense, close, dry, no background noise, no music" | 5047 |
| `raptor/death1` | "large bird dying squawk, fading out, close, dry, no background noise, no music" | 1049 |
| `raptor/death2` | "large bird dying squawk, fading out, short and sharp, close, dry, no background noise, no music" | 5050 |
| `capybara/ambient1` | "capybara alarm bark, short gruff cough, close" | 53 |
| `capybara/ambient2` | "guinea pig squeaking and purring, close, dry, no background noise, no music" | 1055 |
| `capybara/ambient3` | "guinea pig squeaking and purring, close, dry, no background noise, no music" | 5053 |
| `capybara/hurt1` | "guinea pig loud squeal in pain, short, close, dry, no background noise, no music" | 1058 |
| `capybara/hurt2` | "guinea pig loud squeal in pain, short, short and sharp, close, dry, no background noise, no music" | 5059 |
| `capybara/death1` | "small animal weak squeal, fading out, short and sharp, close, dry, no background noise, no music" | 5063 |
| `reindeer/ambient1` | "reindeer grunting, short low grunts and a snort, close, dry, no background noise, no music" | 1066 |
| `reindeer/ambient2` | "reindeer grunting, short low grunts and a snort, short and sharp, close, dry, no background noise, no music" | 5066 |
| `reindeer/ambient3` | "reindeer grunting, short low grunts and a snort, close, dry, no background noise, no music" | 1068 |
| `reindeer/hurt1` | "deer bleating in pain, one short loud cry, close, dry, no background noise, no music" | 5070 |
| `reindeer/hurt2` | "deer bleating in pain, one short loud cry, close, dry, no background noise, no music" | 1073 |
| `reindeer/death1` | "deer dying groan, fading out, close, dry, no background noise, no music" | 1075 |
| `drenkaun/ambient1` | "bull moose bellow, deep grunting call, close, dry, no background noise, no music" | 1081 |
| `drenkaun/ambient2` | "red deer stag roaring, one very deep loud roar, close" | 74 |
| `drenkaun/ambient3` | "bull moose bellow, deep grunting call, short and sharp, close, dry, no background noise, no music" | 5078 |
| `drenkaun/hurt1` | "red deer stag bellowing in pain, short, close, dry, no background noise, no music" | 1083 |
| `drenkaun/hurt2` | "large bear roaring in pain, short, softer and calmer, close, dry, no background noise, no music" | 5086 |
| `drenkaun/hurt3` | "large bear roaring in pain, short, close, dry, no background noise, no music" | 5082 |
| `drenkaun/death1` | "huge animal long dying roar fading into a groan, close, dry, no background noise, no music" | 5087 |
| `drenkaun/death2` | "huge animal long dying roar fading into a groan, close, dry, no background noise, no music" | 1090 |
| `drenkaun/summon1` | "deep earth rumble then a huge stag roar, close, dry, no background noise, no music" | 1091 |
| `drenkaun/summon2` | "giant beast awakening, cracking wood then a deep roar, short and sharp, close, dry, no background noise, no music" | 5092 |
| `drenkaun/attack1` | "angry bull moose snort and bellow, short, softer and calmer, close, dry, no background noise, no music" | 5099 |
| `drenkaun/attack2` | "bear growl, short and aggressive, short and sharp, close, dry, no background noise, no music" | 5096 |
| `drenkaun/attack3` | "bear growl, short and aggressive, close, dry, no background noise, no music" | 1096 |
| `drenkaun/stunned1` | "large animal dazed, groaning and heavy breathing, close, dry, no background noise, no music" | 5100 |
| `drenkaun/stunned2` | "large animal dazed, groaning and heavy breathing, close, dry, no background noise, no music" | 1102 |
| `borkon/ambient1` | "yak grunting bellow, deep, softer and calmer, close, dry, no background noise, no music" | 5108 |
| `borkon/ambient2` | "bull roaring, one deep loud bellow, close, dry, no background noise, no music" | 1106 |
| `borkon/ambient3` | "male goat loud angry bleat, deep and rough, close" | 84 |
| `borkon/hurt1` | "goat screaming in pain, deep, short, softer and calmer, close, dry, no background noise, no music" | 5113 |
| `borkon/hurt2` | "goat screaming in pain, deep, short, close, dry, no background noise, no music" | 5109 |
| `borkon/hurt3` | "bull bellowing in pain, short, short and sharp, close, dry, no background noise, no music" | 5110 |
| `borkon/death1` | "large goat dying bleat fading into a groan, close, dry, no background noise, no music" | 5114 |
| `borkon/death2` | "large goat dying bleat fading into a groan, close, dry, no background noise, no music" | 1114 |
| `borkon/summon1` | "mountain rumble, stones falling, then a deep goat roar, close, dry, no background noise, no music" | 1119 |
| `borkon/summon2` | "mountain rumble, stones falling, then a deep goat roar, close, dry, no background noise, no music" | 1121 |
| `borkon/attack1` | "bull snort and short angry bellow, short and sharp, close, dry, no background noise, no music" | 5123 |
| `borkon/attack2` | "bull snort and short angry bellow, close, dry, no background noise, no music" | 1123 |
| `borkon/attack3` | "bull snort and short angry bellow, close, dry, no background noise, no music" | 1125 |
| `borkon/stunned1` | "large goat dazed, groaning and heavy breathing, close, dry, no background noise, no music" | 1130 |
| `borkon/stunned2` | "large goat dazed, groaning and heavy breathing, close, dry, no background noise, no music" | 5127 |
| `hare/hurt1` | "rabbit screaming in distress, one high shrill scream, short and sharp, close, dry, no background noise, no music" | 5132 |
| `hare/hurt2` | "rabbit screaming in distress, one high shrill scream, longer and lower, close, dry, no background noise, no music" | 5133 |
| `hare/death1` | "rabbit weak dying squeal, fading out, close, dry, no background noise, no music" | 1137 |
| `jaguar/ambient1` | "jaguar sawing call, repeated deep rasping grunts, close, dry, no background noise, no music" | 5138 |
| `jaguar/ambient2` | "jaguar sawing call, repeated deep rasping grunts, softer and calmer, close, dry, no background noise, no music" | 5142 |
| `jaguar/ambient3` | "jaguar sawing call, repeated deep rasping grunts, close, dry, no background noise, no music" | 1138 |
| `jaguar/hurt1` | "big cat snarling in pain, short, close, dry, no background noise, no music" | 1143 |
| `jaguar/hurt2` | "big cat snarling in pain, short, close, dry, no background noise, no music" | 1144 |
| `jaguar/death1` | "big cat dying growl, fading out, short and sharp, close, dry, no background noise, no music" | 5148 |
| `eagle/ambient1` | "golden eagle calling, thin high yelping whistle, close, dry, no background noise, no music" | 1152 |
| `eagle/ambient2` | "golden eagle calling, thin high yelping whistle, close, dry, no background noise, no music" | 1150 |
| `eagle/ambient3` | "golden eagle calling, thin high yelping whistle, softer and calmer, close, dry, no background noise, no music" | 5154 |
| `eagle/hurt1` | "eagle shriek in pain, short, longer and lower, close, dry, no background noise, no music" | 5157 |
| `eagle/death1` | "bird of prey weak cry, fading out, close, dry, no background noise, no music" | 1159 |
| `snowy_owl/ambient1` | "snowy owl barking call, hoarse croak, close, dry, no background noise, no music" | 1163 |
| `snowy_owl/ambient2` | "snowy owl barking call, hoarse croak, close, dry, no background noise, no music" | 1165 |
| `snowy_owl/ambient3` | "snowy owl barking call, hoarse croak, close, dry, no background noise, no music" | 5161 |
| `snowy_owl/hurt1` | "owl screech, short, close, dry, no background noise, no music" | 5166 |
| `snowy_owl/death1` | "owl weak hiss, fading out, close, dry, no background noise, no music" | 5169 |
| `bunting/ambient1` | "small bird twittering call, short, close, dry, no background noise, no music" | 1173 |
| `bunting/ambient2` | "small bird twittering call, short, short and sharp, close, dry, no background noise, no music" | 5173 |
| `bunting/ambient3` | "small bird twittering call, short, louder and more intense, close, dry, no background noise, no music" | 5175 |
| `bunting/hurt1` | "small bird alarm chirp, sharp, longer and lower, close, dry, no background noise, no music" | 5179 |
| `bunting/death1` | "small bird weak chirp, fading, close, dry, no background noise, no music" | 1181 |

## AI act sounds (`voice/<family>/alarm|attack<n>`, `voice/eat/`, `voice/steps/`)

Made 2026-11-25 with the same model, licence and method as the AI voices above: alarm calls (played
as an animal bolts), attack calls (pounce, bite, rear, its species' attack clips), eating by how it
eats (graze, tear meat, peck) and heavy footfalls (heavy, huge) for the big animals and the bosses
(`animals_core::voices::ACTS`, `eat_way`, `step_size`). Cut: alarm and attack 1.2 s, eating 1.2 s
(peck 1.0 s), steps 0.5 s (huge 0.8 s); eating and steps picked by shape alone (they are noises,
not calls). Every candidate, the scores and the picks are in
`~/Coding/chiacraft-archive/sounds/ai-sound/2026-11-25-acts/` (`v2_` = the reworded second round).

| File | Prompt | Seed |
|---|---|---|
| `canine/alarm1` | "dog sharp warning bark, single, longer and lower, close, dry, no background noise, no music" | 7003 |
| `canine/alarm2` | "dog sharp warning bark, single, close, dry, no background noise, no music" | 7001 |
| `canine/attack1` | "dog aggressive growl and snapping bark, short and sharp, close, dry, no background noise, no music" | 7006 |
| `canine/attack2` | "dog aggressive growl and snapping bark, close, dry, no background noise, no music" | 3006 |
| `fox/alarm1` | "red fox alarm bark, short hoarse yap, longer and lower, close, dry, no background noise, no music" | 7011 |
| `fox/alarm2` | "red fox alarm bark, short hoarse yap, short and sharp, close, dry, no background noise, no music" | 7010 |
| `fox/attack1` | "fox snarl and snap, louder and more intense, close, dry, no background noise, no music" | 7016 |
| `fox/attack2` | "fox snarl and snap, close, dry, no background noise, no music" | 3014 |
| `cat/alarm1` | "cat hissing and spitting, short, longer and lower, close, dry, no background noise, no music" | 7019 |
| `cat/alarm2` | "cat hissing and spitting, short, close, dry, no background noise, no music" | 3018 |
| `cat/attack1` | "cat angry yowl and hiss, close, dry, no background noise, no music" | 7021 |
| `cat/attack2` | "cat angry yowl and hiss, close, dry, no background noise, no music" | 3022 |
| `jaguar/alarm1` | "big cat warning snarl, short, close, dry, no background noise, no music" | 3026 |
| `jaguar/alarm2` | "big cat warning snarl, short, close, dry, no background noise, no music" | 3025 |
| `jaguar/attack1` | "jaguar attack roar, short snarl, close, dry, no background noise, no music" | 3031 |
| `jaguar/attack2` | "jaguar attack roar, short snarl, close, dry, no background noise, no music" | 7029 |
| `deer/alarm1` | "deer alarm snort, sharp blow through the nose, close, dry, no background noise, no music" | 3034 |
| `deer/alarm2` | "deer alarm snort, sharp blow through the nose, close, dry, no background noise, no music" | 3035 |
| `deer/attack1` | "stag angry grunt, charging, short and sharp, close, dry, no background noise, no music" | 7038 |
| `deer/attack2` | "stag angry grunt, charging, close, dry, no background noise, no music" | 3040 |
| `reindeer/alarm1` | "reindeer alarm snort and grunt, short and sharp, close, dry, no background noise, no music" | 7042 |
| `reindeer/alarm2` | "reindeer alarm snort and grunt, longer and lower, close, dry, no background noise, no music" | 7043 |
| `reindeer/attack1` | "reindeer angry grunt, charging, close, dry, no background noise, no music" | 3047 |
| `reindeer/attack2` | "reindeer angry grunt, charging, longer and lower, close, dry, no background noise, no music" | 7047 |
| `rodent/alarm1` | "mouse alarm squeak, short and sharp, close, dry, no background noise, no music" | 3049 |
| `rodent/alarm2` | "mouse alarm squeak, short and sharp, close, dry, no background noise, no music" | 3050 |
| `capybara/alarm1` | "capybara alarm bark, short gruff cough, close, dry, no background noise, no music" | 3054 |
| `capybara/alarm2` | "capybara alarm bark, short gruff cough, louder and more intense, close, dry, no background noise, no music" | 7056 |
| `songbird/alarm1` | "sparrow alarm chirps, rapid, close, dry, no background noise, no music" | 3057 |
| `songbird/alarm2` | "sparrow alarm chirps, rapid, louder and more intense, close, dry, no background noise, no music" | 7060 |
| `bunting/alarm1` | "small bird alarm chip calls, close, dry, no background noise, no music" | 7061 |
| `bunting/alarm2` | "small bird alarm chip calls, close, dry, no background noise, no music" | 3062 |
| `hawk/alarm1` | "hawk alarm scream, short, close, dry, no background noise, no music" | 3068 |
| `hawk/alarm2` | "hawk alarm scream, short, short and sharp, close, dry, no background noise, no music" | 7066 |
| `hawk/attack1` | "hawk attack scream, harsh, short and sharp, close, dry, no background noise, no music" | 7070 |
| `hawk/attack2` | "hawk attack scream, harsh, close, dry, no background noise, no music" | 3071 |
| `eagle/alarm1` | "eagle alarm yelps, close, dry, no background noise, no music" | 7073 |
| `eagle/alarm2` | "eagle alarm yelps, close, dry, no background noise, no music" | 3076 |
| `eagle/attack1` | "eagle attack scream, harsh, short and sharp, close, dry, no background noise, no music" | 7078 |
| `eagle/attack2` | "eagle attack scream, harsh, close, dry, no background noise, no music" | 3079 |
| `owl/alarm1` | "owl alarm hiss and bill clacking, close, dry, no background noise, no music" | 3083 |
| `owl/alarm2` | "owl alarm hiss and bill clacking, close, dry, no background noise, no music" | 3084 |
| `owl/attack1` | "owl angry hiss and screech, short and sharp, close, dry, no background noise, no music" | 7086 |
| `owl/attack2` | "owl angry hiss and screech, close, dry, no background noise, no music" | 3085 |
| `snowy_owl/alarm1` | "snowy owl alarm bark, harsh, longer and lower, close, dry, no background noise, no music" | 7091 |
| `snowy_owl/alarm2` | "snowy owl alarm bark, harsh, close, dry, no background noise, no music" | 3091 |
| `snowy_owl/attack1` | "owl angry hiss and screech, close, dry, no background noise, no music" | 7093 |
| `snowy_owl/attack2` | "owl angry hiss and screech, close, dry, no background noise, no music" | 3095 |
| `gull/alarm1` | "gull alarm call, sharp repeated kek, longer and lower, close, dry, no background noise, no music" | 7099 |
| `gull/alarm2` | "gull alarm call, sharp repeated kek, close, dry, no background noise, no music" | 3097 |
| `gull/attack1` | "angry gull screech, longer and lower, close, dry, no background noise, no music" | 7103 |
| `gull/attack2` | "angry gull screech, close, dry, no background noise, no music" | 3101 |
| `corvid/alarm1` | "crow alarm caws, harsh and rapid, close, dry, no background noise, no music" | 3107 |
| `corvid/alarm2` | "crow alarm caws, harsh and rapid, close, dry, no background noise, no music" | 3106 |
| `corvid/attack1` | "angry crow attack caw, close, dry, no background noise, no music" | 3112 |
| `corvid/attack2` | "angry crow attack caw, close, dry, no background noise, no music" | 7109 |
| `hiss/alarm1` | "crocodile warning hiss, loud, close, dry, no background noise, no music" | 3116 |
| `hiss/alarm2` | "crocodile warning hiss, loud, close, dry, no background noise, no music" | 3113 |
| `hiss/attack1` | "crocodile lunge, hiss and jaw snap, close, dry, no background noise, no music" | 3118 |
| `hiss/attack2` | "crocodile lunge, hiss and jaw snap, longer and lower, close, dry, no background noise, no music" | 7119 |
| `bellow/alarm1` | "crocodile warning growl, deep, short and sharp, close, dry, no background noise, no music" | 7122 |
| `bellow/alarm2` | "crocodile warning growl, deep, longer and lower, close, dry, no background noise, no music" | 7123 |
| `bellow/attack1` | "crocodile attack growl with jaw snap, close, dry, no background noise, no music" | 3126 |
| `bellow/attack2` | "crocodile attack growl with jaw snap, close, dry, no background noise, no music" | 3127 |
| `horned_dino/alarm1` | "large animal alarm honk, loud and short, longer and lower, close, dry, no background noise, no music" | 7131 |
| `horned_dino/alarm2` | "large animal alarm honk, loud and short, close, dry, no background noise, no music" | 3131 |
| `horned_dino/attack1` | "large animal angry bellow, charging, short and sharp, close, dry, no background noise, no music" | 7134 |
| `horned_dino/attack2` | "large animal angry bellow, charging, close, dry, no background noise, no music" | 3134 |
| `raptor/alarm1` | "large bird alarm shriek, harsh, close, dry, no background noise, no music" | 7137 |
| `raptor/alarm2` | "large bird alarm shriek, harsh, close, dry, no background noise, no music" | 3140 |
| `raptor/attack1` | "angry large bird screech and hiss, attacking, close, dry, no background noise, no music" | 7141 |
| `raptor/attack2` | "angry large bird screech and hiss, attacking, close, dry, no background noise, no music" | 3144 |
| `dragon/alarm1` | "dragon warning growl, deep, short and sharp, close, dry, no background noise, no music" | 7146 |
| `dragon/alarm2` | "dragon warning growl, deep, close, dry, no background noise, no music" | 3145 |
| `dragon/attack1` | "dragon attack roar, short, short and sharp, close, dry, no background noise, no music" | 7150 |
| `dragon/attack2` | "dragon attack roar, short, close, dry, no background noise, no music" | 3151 |
| `eat/graze1` | "animal grazing, tearing and chewing grass, louder and more intense, close, dry, no background noise, no music" | 7164 |
| `eat/graze2` | "animal grazing, tearing and chewing grass, longer and lower, close, dry, no background noise, no music" | 7163 |
| `eat/graze3` | "animal grazing, tearing and chewing grass, close, dry, no background noise, no music" | 7161 |
| `eat/meat1` | "animal tearing and chewing raw meat, louder and more intense, close, dry, no background noise, no music" | 7169 |
| `eat/meat2` | "animal tearing and chewing raw meat, longer and lower, close, dry, no background noise, no music" | 7168 |
| `eat/meat3` | "animal tearing and chewing raw meat, short and sharp, close, dry, no background noise, no music" | 7167 |
| `eat/peck1` | "bird pecking seeds on the ground, short and sharp, close, dry, no background noise, no music" | 7172 |
| `eat/peck2` | "bird pecking seeds on the ground, close, dry, no background noise, no music" | 3175 |
| `eat/peck3` | "bird pecking seeds on the ground, softer and calmer, close, dry, no background noise, no music" | 7175 |
| `steps/heavy1` | "heavy hooved animal footstep on earth, single thud, softer and calmer, close, dry, no background noise, no music" | 7180 |
| `steps/heavy2` | "heavy hooved animal footstep on earth, single thud, louder and more intense, close, dry, no background noise, no music" | 7179 |
| `steps/heavy3` | "heavy hooved animal footstep on earth, single thud, close, dry, no background noise, no music" | 3178 |
| `steps/heavy4` | "heavy hooved animal footstep on earth, single thud, longer and lower, close, dry, no background noise, no music" | 7178 |
| `steps/huge1` | "giant creature footstep, single deep heavy thud, two in a row, close, dry, no background noise, no music" | 7187 |
| `steps/huge2` | "giant creature footstep, single deep heavy thud, softer and calmer, close, dry, no background noise, no music" | 7186 |
| `steps/huge3` | "giant creature footstep, single deep heavy thud, louder and more intense, close, dry, no background noise, no music" | 7185 |
| `steps/huge4` | "giant creature footstep, single deep heavy thud, longer and lower, close, dry, no background noise, no music" | 7184 |
