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
| `big_cat/ambient1` (hit:0), `big_cat/ambient2` (hit:1) | "G12-45-Tiger Hiss and Growl.wav" ([freesound 437978](https://freesound.org/people/craigsmith/sounds/437978/)) | craigsmith | CC0 |
| `big_cat/hurt1` | "elaine-growl.wav" ([freesound 63668](https://freesound.org/people/Miselaineous/sounds/63668/)) | Miselaineous | CC0 |
| `big_cat/hurt2` (hit:0) | "Roar.wav" ([freesound 410343](https://freesound.org/people/waxsocks/sounds/410343/)) | waxsocks | CC0 |
| `big_cat/death1` (hit:0) | "S28-07 Deep-throated growls of a big cat slowed down to create a monster.wav" ([freesound 675497](https://freesound.org/people/craigsmith/sounds/675497/)) | craigsmith | CC0 |
| `deer/ambient1` (hit:0), `deer/hurt1` (hit:1), `deer/hurt2` (hit:2), `deer/death1` (hit:3) | "Deer Meewing 01.wav" ([freesound 351113](https://freesound.org/people/msantoro11/sounds/351113/)) | msantoro11 | CC0 |
| `deer/ambient2` (hit:0) | "Bellowing deer - Animal - Nature - Hirsch - Röhren - Tiere - Wald" ([freesound 407631](https://freesound.org/people/IchBinChrist/sounds/407631/)) | IchBinChrist | CC0 |
| `rodent/ambient1` (hit:0), `rodent/ambient2` (hit:1) | "Mouse Squeaks.wav" ([freesound 463789](https://freesound.org/people/shyguy014/sounds/463789/)) | shyguy014 | CC0 |
| `rodent/hurt1`, `hare/hurt2` | "Mouse" ([freesound 188043](https://freesound.org/people/AntumDeluge/sounds/188043/)) | AntumDeluge | CC0 |
| `rodent/hurt2`, `hare/hurt1` | "Rat.ogg" ([freesound 536753](https://freesound.org/people/egomassive/sounds/536753/)) | egomassive | CC0 |
| `rodent/death1`, `hare/death1` | "Cartoon - Bat / Mouse Squeak" ([freesound 445958](https://freesound.org/people/Breviceps/sounds/445958/)) | Breviceps | CC0 |
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
| `small_dino/ambient1` | "Dino Hiss Dragon Roar" ([freesound 320345](https://freesound.org/people/999999990/sounds/320345/)) | 999999990 | CC0 |
| `small_dino/ambient2` | "Dinosaur 5.wav" ([freesound 233284](https://freesound.org/people/JarredGibb/sounds/233284/)) | JarredGibb | CC0 |
| `small_dino/hurt1` | "Goose Hiss" ([freesound 740195](https://freesound.org/people/el_boss/sounds/740195/)) | el_boss | CC0 |
| `small_dino/hurt2` | "Goose Hiss Short.aiff" ([freesound 581157](https://freesound.org/people/ChunaawChika09/sounds/581157/)) | ChunaawChika09 | CC0 |
| `small_dino/death1` | "Dinosaur Growl" ([freesound 147527](https://freesound.org/people/123agumon/sounds/147527/)) | 123agumon | CC0 |
| `dragon/ambient1` | "DRAGON_ROAR.wav" ([freesound 85568](https://freesound.org/people/JoelAudio/sounds/85568/)) | JoelAudio | CC0 |
| `dragon/ambient2` | "Dragon Roar" ([freesound 442964](https://freesound.org/people/qubodup/sounds/442964/)) | qubodup | CC0 |
| `dragon/hurt1` (hit:0) | "dragonroar.wav" ([freesound 185069](https://freesound.org/people/juryduty/sounds/185069/)) | juryduty | CC0 |
| `dragon/death1` (hit:0) | "Dragon Death" ([freesound 366096](https://freesound.org/people/bevibeldesign/sounds/366096/)) | bevibeldesign | CC0 |

The chihuahua's `chihuahua/bark1-3.wav` were here before and are not covered by this file.
