# Clothing and armor: real insulation, real outfits, real armor

Research for the clothing and armor content: what keeps a person warm or cool, what people in each
biome's real place wore, how the materials were made, what armor did, and a proposed item set for
the game's model in `mods/base/vitals/src/temperature.rs` (a naked person is comfortable in air of
15-28 °C, worn things add °C of cold and heat insulation, wet keeps 35% of it). 2026-11-03.

**How sure is it.** Numbers with a source carry its link [n]. Numbers marked *est.* are my own
reasoning or common knowledge. The required-clothing table (A2) is computed from the ISO 11079
IREQ algorithm (Holmér and Nilsson's code, as published in the R package `comf` [3]); I ran it,
it is not copied from the standard. Clo values for traditional skin clothing are the weakest
part: few were ever measured, and the measured ones report skin or surface temperatures, not clo.
Several fetches failed (Inter-Research, DRDC), so some papers are known by their abstracts only.

---

## A. Insulation numbers

### A1. What a clo is, and garment values

1 clo = 0.155 m²·K/W: the insulation that keeps a resting person in balance at about 21 °C
indoors [1]. As a rule of thumb, clothing plus its trapped air gives about **4 clo per inch
(1.6 clo per cm)** of thickness; still air alone about 7 per inch [5]. One clo is roughly 0.64 cm
of cloth plus 0.5 cm of air [6]. ASHRAE 55 takes 0.5 clo as summer dress and 1.0 clo as winter
indoor dress [2] (comfort at about 23-26 °C and 20-23.5 °C for a seated person, *est.* from the
standard's chart).

| Garment (ASHRAE / ISO 9920 table) | clo | | Garment | clo |
|---|---|---|---|---|
| T-shirt | 0.08 [1] | | Long-sleeve flannel shirt | 0.34 [1] |
| Long underwear top / bottom | 0.20 / 0.15 [1] | | Sweater, thin / thick | 0.25 / 0.36 [1] |
| Trousers, thin / thick | 0.15 / 0.24 [1] | | Suit jacket, thick | 0.44 [1] |
| Sweatpants | 0.28 [1] | | Long wrap robe, thick | 0.69 [1] |
| Boots | 0.10 [1] | | Knee socks, thick | 0.06 [1] |
| Coveralls | 0.49 [1] | | Insulated coveralls + thermal underwear | 1.37 [1] |
| Trousers, shirt, sweater, T-shirt | 1.01 [1] | | Light / heavy polar kit | ~3 / ~4 *est.* (commonly cited) |

Garment values add up (roughly) because each is weighted by the share of the body it covers [1].

Materials by thickness, using 1.6 clo per cm [5] (all *est.* from that rule):

| Material, as worn | Loft or thickness | clo of the layer where it covers | Notes |
|---|---|---|---|
| Linen or cotton shirt | 0.5 mm | ~0.1-0.25 | thin; insulation is mostly the air under it |
| Knitted wool sweater | 3-6 mm | ~0.3-0.5 | matches 0.25-0.36 [1] |
| Felt (valenki wall, felt cloak) | 5-10 mm | ~0.8-1.5 | dense, windproof, slow to wet through |
| Fulled wool coat (loden) | 3-5 mm + lining | ~0.6-0.9 | sheds wind and rain [34] |
| Buckskin (brain-tanned) | 1-2 mm | ~0.2-0.3 | windproof, little loft |
| Sheepskin, fleece on (tulup) | 15-25 mm | ~2.5-4 | heavy (3-5 kg coat) |
| Caribou winter skin, hair on | 20-30 mm | ~3-5 | hollow hair, light [10] |
| Bison winter robe | 25-50 mm | ~4-6 | 7.7-10 kg tanned [26] |
| Down, lofted (eider, goose) | 20-50 mm | ~3-8 | best per gram; collapses wet |
| Hare skin woven blanket | ~25 mm | ~3-4 | Great Basin and Athabaskan robes [52] |

Furs: Scholander's measurements of arctic mammal pelts show insulation rising with fur thickness
up to about fox size and then flattening; wolf, caribou and polar bear winter fur all give about
0.8 m²·K/W, roughly **5 clo** [15]. A pelt worn as clothing gives less than on the animal
(compressed at the shoulders and seat, gaps at the openings) *est.*

Skin clothing measured: in a chamber made to feel like riding a snowmobile sled in arctic winter,
skin temperature and comfort fell significantly less in a caribou-skin outfit than in military or
expedition clothing [7]. At -21 to -23 °C, the outer surface of caribou-skin clothing was
-22.1 to -22.7 °C, modern retail cold gear -21.6 and the Canadian army winter uniform -18.9 to
-20.0 (a colder surface means less heat getting through) [8]. A full Inuit outfit (parka, pants,
mittens, inner and outer footwear) weighed only **3-4.5 kg** [9].

Hands, feet, head: mittens beat gloves because the fingers share one pocket of air [10]. Inuit
footwear could be five layers (socks, boots, overboots) [9]. The head and hands are where people
feel cold first; a hood with a fur ruff (wolf or wolverine, which sheds breath frost) keeps the
face out of the wind [10]. In the game's piece shares (head 0.15, feet 0.15) this is about right.

### A2. Clothing needed for comfort (ISO 11079 IREQ)

Basic clothing insulation (ICL, clo) needed to stay in heat balance, from IREQ minimal (slightly
cool, accepted for hours) to neutral (no cooling). Still air (0.4 m/s), 80% humidity, a
windproof outer layer. Computed with the ISO 11079 code [3][4]. Metabolic rates: rest 58 W/m²
(1 met), sitting or light handwork 70, walking 116 (2 met), work 165, heavy work 230.

| Air °C | Rest | Light handwork | Walking | Working | Heavy work |
|---|---|---|---|---|---|
| 10 | 2.5-2.9 | 1.9-2.3 | 0.9-1.2 | 0.4-0.7 | 0.0-0.3 |
| 0 | 3.9-4.3 | 3.1-3.5 | 1.6-1.9 | 0.9-1.2 | 0.4-0.7 |
| -10 | 5.2-5.6 | 4.3-4.6 | 2.3-2.7 | 1.4-1.8 | 0.8-1.2 |
| -20 | 6.6-7.0 | 5.4-5.8 | 3.1-3.4 | 2.0-2.3 | 1.2-1.6 |
| -30 | 8.0-8.4 | 6.6-7.0 | 3.8-4.2 | 2.5-2.9 | 1.7-2.0 |
| -40 | 9.4-9.8 | 7.7-8.1 | 4.5-4.9 | 3.1-3.4 | 2.1-2.4 |
| -50 | 10.8-11.2 | 8.9-9.3 | 5.3-5.6 | 3.6-4.0 | 2.5-2.8 |

Rules of thumb from the table:

- **Walking: about 12-14 °C per clo. Resting: about 7 °C per clo. Working: 18-20 °C per clo.**
- A walking person in a 4 clo outfit is fine at -30 °C; the same person sitting still needs
  8 clo. This is why the Inuit sat on skins, slept under robes in a shelter, and kept moving
  outside. Clothing alone cannot keep a still person warm at -40 °C.
- 4-5 clo is about the most a person can move in. Beyond that comes from shelter, fire,
  bedding and work.

### A3. Wind and wet

Wind (IREQ, walking, basic clo needed) [3]:

| Air °C | Wind | Windproof outer (8 l/m²s) | Open weave (100 l/m²s) |
|---|---|---|---|
| -10 | still (0.4 m/s) | 2.3-2.7 | 2.4-2.8 |
| -10 | 6 m/s | 3.7-4.3 (+60%) | 4.6-5.2 (+90%) |
| -30 | still | 3.8-4.2 | 4.0-4.3 |
| -30 | 6 m/s | 5.9-6.5 (+55%) | 7.2-7.8 (+85%) |

Wind thins the still air in and around the clothing. Air permeability barely matters under
2 m/s [17], but at 6 m/s an open weave (knitted wool, loose fur) needs a third more insulation
than a windproof one. A tight outer shell (skin, fulled wool, gut) is what Arctic layering is
built on [19]. Movement pumps air too: ISO 9920 corrects insulation down for walking and wind,
and tests find even bigger losses than its equations predict [17].

Wet, by material:

| Material | When wet | Source |
|---|---|---|
| Any clothing, damp mid layer away from skin | total insulation down 9-16% | [13] |
| Wool | holds ~30-35% of its weight in water before it feels wet, and gives off heat while taking it up | [16] |
| Wool, cotton, fleece soaked | all poor; field tests find little difference once soaked | [19] |
| Down | clumps and loses its loft: most insulation gone | [19] |
| Cotton, linen | wicks water to the skin, dries slowly; little insulation to lose | [19] |
| Reindeer calf fur, wetted | heat conductance up almost **5 times** (≈80% of insulation lost) | [14] |
| Svalbard reindeer fur, lightly wet | no measurable loss | [14] |
| Polar bear fur, submerged | heat flow up **25-50 times** | [15] |
| Caribou skin clothing | sheds hair badly when wet; not for wet weather | [9] |
| Sealskin, gut | water-repellent and waterproof; sinew seams swell shut | [9][10] |

So: light wetting (drizzle, sweat in an outer layer) costs little; soaking costs most of it, worst
for down and fur, least for wool and felt. Arctic people kept fur dry and wore sealskin and gut
over it in the wet [9].

### A4. Heat: what helps and what costs

| Factor | Effect | Source |
|---|---|---|
| Loose robe in desert sun | Bedouins gained the same heat in a black or a white robe; the extra heat the black one absorbs is carried off by air moving inside a loose robe (bellows and chimney effect) | [18] |
| Coverage in sun | clothing blocks sun on the skin; fit and looseness matter more than color | [18] |
| Insulation in heat | any clo cuts sweat evaporation; ASHRAE moves the comfort zone ~3 °C warmer from 1.0 to 0.5 clo | [2] *est.* |
| Body armour in heat | blocks sweat evaporation on the torso; counts as roughly **+2.8 °C WBGT** over a plain uniform | [63] |
| Plate armour walking | metabolic cost of walking **2.1-2.3 times** unloaded; running 1.9 times (30-50 kg suits) | [60] |
| Bronze panoply (Dendra, 23.3 kg replica) | 11 h of simulated combat at 24-29 °C, 70-85% humidity: core 36.4-37.7 °C ("minor hyperthermia"); only a hot, windless, very intense case forced a stop at 7.5 h | [61] |
| Shade | the game already adds +3 °C in open sun; a wide hat or sunshade removes it for the head *est.* |

Above about 35 °C (skin temperature) air heats the body, so in the hottest deserts a loose cover
is better than bare skin *est.*. Desert people dressed loosely and covered the head, and worked in
the cool hours *est.*

---

## B. Traditional clothing in each biome's real place

| Biome (place) | Winter | Summer | Materials, counts |
|---|---|---|---|
| Snowy plains (Utqiaġvik; Iñupiat) | Two caribou parkas: inner fur-in, outer fur-out for deep cold and storms; fur trousers (men often two pairs); fur-in socks, caribou kamik boots (bull leg and back skin; soles from a fall bull's back); mittens over gloves; wolf or wolverine ruff [9][10] | One layer; dehaired sealskin boots and pants; gut parka for rain and boats [9] | 4 caribou hides per person for two everyday sets (20 for a family of five) [9]; a Netsilik family used ~30 caribou a year for clothes and bedding [12]; 8 seal skins for two parkas and two pants for one man [9]; calf skins for parkas, cow skins for mittens and socks, August-October skins best [10][11] |
| Taiga (Fairbanks; Gwich'in, Athabaskan) | Caribou-skin tunic and moccasin trousers (feet attached), hair turned in for winter; mitts, mukluks; moose hide for soles; hare-skin robes and blankets [20] | White-tanned caribou tunic and trousers, sewn with sinew, quilled [20][21] | Smoked caribou and moose leather; hare skins cut in strips and woven *est.* |
| Birch forest (Moscow; Russian peasants) | Sheepskin tulup (often one per family, handed down), felt valenki, wool armyak or zipun coat, fur hat, mittens [22][23] | Linen rubakha shirt, linen trousers, women's sarafan; onuchi leg wraps and lapti bast shoes (linden bast, birch bark) [23] | Valenki take **4-7 kg** of wool per pair, felted wet with hot water, no seams [22] |
| Forest (Kassel; Central Europe) | Wool tunic and hose, hooded wool cloak, fur-lined coats for those who could pay; leather shoes *est.* | Linen shirt and underclothes *est.* | Flax and wool grown at home *est.* |
| Mountains (Davos; Alpine) | Loden: home-spun wool woven loose, then fulled in hot water with soap and pounding till dense, wind- and rain-shedding; fulling mills since about the 15th c. [34] | Linen, lighter loden *est.* | A family's own sheep; travelling weavers; the village fulling mill [34] |
| Plains (Konza; Plains nations) | Bison robe, hair in, from prime November-March hides; buckskin shirt and leggings; fur-in moccasins *est.* | Buckskin, breechcloth, robe as needed [25] | One hide, one robe [25]; a tanned robe 7.7-10 kg, 3.3-4.2 m² [26]; a tipi 13-16 bison hides [24] |
| Desert (Hermosillo; O'odham and Seri) | Cotton blanket wrap; deerskin shirt in cold [27] | Breechcloth, cotton skirt or wrap; rawhide sandals; red-dyed deerskin moccasins for travel [27] | Seri robes of sun-dried **pelican skins** [28]; yucca and agave cord *est.* |
| Savanna (Arauca; llanos) | Same (no winter) | Liqui-liqui (cotton or linen suit), wide hat (felt "pelo e' guama" or palm-straw), cotizas or alpargatas [29][30] | Indigenous Sikuani make bags, hats, hammocks from moriche palm leaf fibre [31][32] |
| Swamp (Pantanal) | Same; cool fronts bring chilly days *est.* | Straw hat, short boots with long spurs, leather saddle gear (the "baldrana") [33] | Cattle leather for tack; capybara and caiman hides traded by the million in the 20th c. [54] |
| Ocean, beach (Azores, Faro) | Wool coat or cape for wind and rain *est.* | Cotton and linen *est.* | Wool and linen *est.* |

What the cold places share: two layers of fur (one hair in, one hair out), a windproof outside,
mittens, a hood with a ruff, layered footwear, and a separate waterproof skin for the wet.
What the hot places share: loose cotton or linen, a wide hat, sandals, and nothing heavy.

---

## C. Making the materials

### C1. Hides and furs per animal

| Animal (game) | Hide area | Hide mass | What it's good for | Source |
|---|---|---|---|---|
| Cow / bison | cowhide 1.5-3.6 m² (16-39 ft²); bison robe 3.3-4.2 m² | green hide ≈7% of live weight: 35-41 kg for a 470-550 kg animal; bison robe tanned 7.7-10 kg | rawhide, sole leather, robes, tipi covers, shields | [45][46][26] |
| Deer (white-tail) | 0.75-1.1 m² tanned (0.5-1.5 range) | ~3-5 kg green *est.* | buckskin shirts, leggings, moccasins | [47] |
| Red deer / elk | 1.35-1.9 m² / ~2.2 m² | ~8-15 kg green *est.* | heavy buckskin, Tlingit armour hide | [46][48] |
| Caribou / reindeer | ~1.2-1.6 m² *est.* | ~2-4 kg green *est.* | the best winter fur (hollow hair); calf skins for parkas; bull leg skin for boots | [9][10] |
| Sheep | ~0.6-0.9 m² *est.* | ~3-5 kg with wool *est.* | sheepskin coats; wool | |
| Boar (pig) | ~1-1.5 m² *est.* | | tough leather, bristles | |
| Wolf | pelt ~1-1.3 m² *est.* | | ruffs, hood and mitt trim (long guard hairs) | [9] |
| Fox | ~0.4 m² *est.* | | trim, caps, sock linings | [9] |
| Hare | ~0.1 m² *est.* | | ~85 skins for a small woven blanket | [52][53] |
| Jaguar | ~1.2-1.6 m² *est.* | | prized pelt (a prime one was worth six months' wages); short fur, little warmth | [55] |
| Capybara | ~0.6-1.0 m² *est.* | | soft leather ("carpincho", glove leather); ~1 million hides from the central-western Brazilian Amazon in the 20th c. | [54] |
| Caiman | belly ~0.3-0.6 m² *est.* | | belly leather; the back has bony plates (osteoderms) | [54] |
| Seal | ~1 m² *est.* | | waterproof boots and pants; 8 skins = 2 parkas + 2 pants | [9] |
| Birds (eider, gull, raven, diving birds) | | | Inuit used 24+ bird species; eider was the main clothing for people without caribou; diving birds the toughest | [9] |

### C2. Down and feathers

| Source | Yield | Source |
|---|---|---|
| Eider nest | 15-20 g of down per nest | [49] |
| Eider, per kg | 60-66 nests per kilo | [50] |
| Goose, live plucking | 80 g first plucking, 100-120 g later ones; down is 15-20% of it; ~3 pluckings a season | [51] |
| Goose or duck, slaughtered | ~150-250 g feathers and down *est.* | |
| Chicken | coarse feathers, little true down *est.* | |
| A down coat | ~300-600 g down *est.* (≈ 20-40 eider nests) | |

### C3. Hide to leather and fur

| Process | Steps | Time | Source |
|---|---|---|---|
| Rawhide | flesh, dehair (or not), stretch and dry | 1-3 days *est.* | |
| Brain tanning (buckskin) | flesh (~1 h), dry, scrape, soak in brain and hot water (1 brain per deer hide, 2-8 h), wring, stretch while drying, smoke (~1 h) | ~2 days of work for a deer hide; days to weeks overall | [42] |
| Fur dressing (hair on) | flesh, dry, scrape the skin side soft (Inuit up to 12 stages) | ~15 h per caribou hide (300 h for 20) | [9][10] |
| Sewing caribou clothing | cut (an amauti ~1 h), sew with sinew | 225 h for a family of five's two sets each; 2-4 weeks of sewing after the hunt | [9] |
| Bark (vegetable) tanning, oak | 3 months through pits of rising strength, then 9 months layered with oak bark | **12 months**; historically 11-18 months, up to 2 years for thick hides | [43][44] |
| Smoking | over a smoky softwood fire | ~1 h a side | [42] |
| Hardened leather (cuir bouilli) | wet leather shaped and hardened by heat (wax uncertain) | hours *est.* | [64] |

Smoking makes buckskin dry soft after a wetting and resist water [42]. Sinew thread (caribou leg
or back tendon) swells when wet and seals the seams [9][10]. Needles were caribou bone, walrus
ivory or antler [10].

### C4. Plant and animal fibres

| Fibre | From field to cloth | Numbers | Source |
|---|---|---|---|
| Wool | shear (or pluck primitive breeds), wash, card or comb, spin, weave | Soay-type sheep 1-2.25 kg fleece a year; primitive ~1 kg; modern 3-5 kg | [35] |
| Spinning wool | spindle | 10-20 h per kg for a practiced spinner; 40-60 h per kg for a beginner | [36][37] |
| A sweater | | 0.7-1 kg of wool, 10-15 h of wheel spinning | [37] |
| Felt | wool + hot water + rubbing and rolling | valenki 4-7 kg wool a pair | [22] |
| Fulling (loden) | woven wool beaten in hot soapy water till it shrinks and thickens | Alpine mills since ~1400s | [34] |
| Flax to linen | pull, ret, dry, break, scutch (knock off the woody core), hackle (comb), spin | water retting 3-7 days (7 days at 20 °C); dew retting up to 6 weeks, turned 3-4 times; fibre 420-980 kg/ha | [38][39] |
| Nettle | stems retted or just stripped, then spliced or spun | Bronze Age thread spliced from nettle, flax and lime bast; a 2,800-year-old Danish cloth of nettle | [40][41] |
| Cotton | pick, gin by hand, card, spin, weave | ~200-500 kg lint/ha traditional *est.* | |
| Moriche palm | young spear leaves dried for weeks, crushed, boiled, sun-dried, rolled into thread on the thigh | Warao and Sikuani hammocks, bags, hats; leaves taken without killing the palm | [31][32] |
| Linden bast, birch bark | bark strips plaited | lapti shoes | [23] |
| Yucca, agave | leaves pounded and scraped (or roasted) for long fibre | sandals, cord, the core of rabbit-skin blankets | [53] *est.* |

---

## D. Armor in real history

### D1. Kinds, masses and what they stop

Impact energies to defeat each (Alan Williams' tests, as reported [56]); arrows from war bows
arrive with **~80-150 J** (86 J from a 140 lb bow, 111-146 J from 150 lb; ~80 J left at 180 m)
[59].

| Armor | Mass | Stops | Fails against | Source |
|---|---|---|---|---|
| Padded jack / gambeson (16-30 layers of linen, cotton or wool) | ~2-4 kg *est.* | cuts; 16 layers ≈80 J, a 26-layer jack ≈200 J; arrow-*resistant*, not arrow-proof; sharp heads go deeper | heavy arrows and bolts, thrusts | [56][57][71] |
| Hardened leather (cuir bouilli) | ~2-5 kg *est.* | blunt blows (used alone in tournaments); cuts glance; reduces arrow depth | direct blade blows, firearms | [64] |
| Rawhide / layered elk hide (Tlingit) | | layered hide up to 1.27 cm thick, plus wooden slats; turned clubs and, by some accounts, musket balls | firearms at close range (disputed) | [65] |
| Wooden slats and rods (Tlingit, Chukchi) | *est.* 5-8 kg | arrows and clubs; worn over a hide tunic | | [65][67] |
| Bone, antler, ivory plates (Chukchi, Iñupiat, St Lawrence Island) | | arrows; plates lashed in rows; Chukchi suits of reindeer and seal leather with bone | | [66][67] |
| Crocodile skin (Roman Egypt) | | a helmet and cuirass survive; probably ceremonial | | [70] |
| Bronze panoply (Dendra, ~1400 BC) | original ~18 kg, replica 23.3 kg | cuts, arrows; wearable for 11 h of combat | | [61] |
| Mail hauberk | museum examples 9.3-11.1 kg | cuts; bodkin arrow split rings at 80 J, reached the padding at 100 J, failed at 120 J; spear 140-200 J | heavy arrows close in, thrusts, blunt force (needs padding under) | [56][62] |
| Plate (1 / 2 / 3 mm mild steel) | full field harness 30-50 kg (Askew); 15-25 kg in many other estimates *est.* | arrow 55 / 175 / 300 J; real slaggy iron weaker | crossbows and guns at close range | [56][60] |
| Shield (Viking round, 90-94 cm, 8-9 mm fir or linden) | ~3-5 kg (2.1-5.2) | everything it covers | | [68][69] |

A 75 lb bodkin-arrow longbow at 10 yards went through gambeson and mail together [58].
Bites: a dog or wolf bite crushes rather than cuts; padding and thick leather on the forearm and
legs are what work against it *est.*

### D2. The heat and effort burden

| Armor | Cost | Source |
|---|---|---|
| Plate (30-50 kg) | walking costs 2.1-2.3 times as much energy; breathing is restricted | [60] |
| Bronze (23 kg) | minor hyperthermia in 24-29 °C humid air over 11 h; one of 13 men hypoglycaemic | [61] |
| Torso armour (modern) | blocks sweat on the torso: ≈ +2.8 °C WBGT | [63] |
| Gambeson | is itself ~1 clo of insulation *est.*: warm in winter, a burden in summer | |

### D3. Was clothing worn under armor?

Yes, always. Mail was worn over a padded layer that took the blow the rings stopped [56][71];
plate went over an arming doublet and often mail; Tlingit slat armor over a hide tunic [65];
Arctic plate armor over skin clothing *est.*. Armor without the layer under it chafes and passes
blunt force straight through. The gambeson was also worn alone as cheap armor [71].

---

## E. Proposal: clothing and armor for the game

*Everything in this section is my proposal, built on the numbers above.*

### E1. Converting clo to the game's °C

The game's player is active (walks, works), so use the walking column of A2:

- **1 clo ≈ 12 °C of cold insulation.** A full outfit of `c` clo gives `12·c` °C.
- Resting or sleeping needs about twice the insulation (7 °C per clo). The game could halve worn
  insulation when sleeping and let bedding, shelter and fire make up the rest, as people did.
- Heat insulation (the game's heat side) is negative for anything with loft; only loose, light
  cover with a hat earns a positive number, by blocking the sun.

### E2. What each biome needs (from the game's own climate table)

From `mods/base/vitals/src/climate.rs`: winter mean minus the night swing, snowfall (-5) and wind
chill as `air::feels` applies them; summer day high plus +3 in sun. "Needed" is how far past the
naked range (15-28 °C) that is. Fat (up to 4), a hot meal and fortitude cover the last few
degrees.

| Biome | Coldest night (felt) | Cold needed | Hottest day (sun) | Heat needed | Real check |
|---|---|---|---|---|---|
| Snowy plains (Utqiaġvik) | ~-28; snowing -33; wind 6 m/s → **~-41** | **~50-56** | ~11 | 0 (+5 cold) | Utqiaġvik winter -25 mean, -40 with wind *est.* |
| Taiga (Fairbanks) | ~-28 → ~-32 | **~45** | ~26 | 0 | NWS "extreme cold" at -37 °C [19] |
| Mountains (Davos height, 1,594 m) | ~-18 → ~-23 | ~38 | ~12 | 0 | the game's mountain at Davos height looks several degrees colder than real Davos (annual mean ~3 °C *est.*); worth a check |
| Birch forest (Moscow) | ~-10 → ~-14 | ~29 | ~26 | 0 | |
| Plains (Konza) | ~-8 → ~-13 | ~28 | ~36 | **~8** | |
| Forest (Kassel) | ~-3 → ~-8 | ~23 | ~25 | 0 | |
| Beach (Faro) | ~8 → ~6 | ~9 | ~32 | ~4 | |
| Ocean (Azores) | ~11 | ~4 | ~28 | 0 | |
| Desert (Hermosillo) | ~10 (winter) | ~5 | **~45** | **~17** | |
| Savanna (Arauca) | ~21 | 0 | ~36 | ~8 | humid heat; the model has no humidity |
| Swamp (Pantanal) | ~15 | 0 | ~38 | ~10 | |

The current sets (leather 8, fur 16) reach Kassel's winter at best. The Arctic needs 3-4 times
the fur set.

### E3. Two layers: clothing and armor

Real people wore armor over clothing (D3), and the game can do the same:

- **Clothing slots** (head, chest, legs, feet, same shares 0.15 / 0.4 / 0.3 / 0.15): warmth and
  sun cover, no protection.
- **Armor slots** (the same four): protection, a small or negative effect on warmth, a heat
  penalty and weight.
- Optional **over slot** (cloak, robe, rain parka): a robe or cape adds warmth, a gut parka or
  oiled cape keeps the layers under it dry. Vintage Story keeps clothing and armor in separate
  slots in this way; Valheim has a cape slot.

### E4. Clothing (warmth, no protection)

Full-set numbers in the game's °C (cold, heat), the clo they stand for, and what keeps when wet
(the game now keeps 0.35 for everything; per-material is proposed).

| Set | Pieces (head / chest / legs / feet) | Made from | clo | Cold | Heat | Wet keeps | For |
|---|---|---|---|---|---|---|---|
| Linen | cap / shirt / trousers / bast shoes | flax (linen), linden bast | 0.5 | 6 | 0 | 0.2 | summer anywhere |
| Desert robe | wide hat / loose robe / wrap / sandals | cotton or linen, palm straw, rawhide | 0.6 | 5 | **+6** | 0.2 | desert, savanna, swamp summer |
| Buckskin | fur cap / shirt / leggings / moccasins | deer, elk or caribou leather (brain-tanned) | 0.8 | 9 | -1 | 0.3 (0.5 smoked) | spring, autumn |
| Wool | hood / tunic / hose / wool socks + shoes | wool (spun, woven), leather soles | 1.2 | 14 | -2 | **0.6** | Kassel, Azores, Faro winter |
| Loden | felt hat / fulled coat / loden breeches / felt-lined boots | fulled wool over the wool set | 2.0 | 24 | -4 | 0.6, windproof | Kassel winter, mountain autumn |
| Plains winter | fur cap / bison robe / buckskin leggings / fur-in moccasins | bison (cow), deer | 2.5 | 30 | -8 | 0.4 | Konza winter |
| Sheepskin | fur hat / tulup / wool trousers / valenki | sheepskin with fleece, felt (4-7 kg wool) | 2.7 | 32 | -8 | 0.5 (felt) | Moscow winter |
| Bird-skin | hood / bird-skin parka / caribou pants / boots | eider, gull, raven skins with down | 2.5 | 28 | -8 | 0.15 | coastal Arctic, no caribou |
| Caribou, single | hood / inner parka fur-in / fur-in pants / fur socks + kamik | ~2 caribou (calf for the parka, cow for pants, bull leg for boots), sinew | 2.5 | 30 | -9 | 0.3 | Fairbanks autumn, Arctic spring |
| Caribou, double | ruffed hood / inner + outer parka / two pants / socks, kamik, overboots + mittens | ~4 caribou, wolf or wolverine ruff, fox lining, sinew | 4.0 | **48** | -14 | 0.25, windproof | Fairbanks and Utqiaġvik winter |
| Sealskin | hood / gut rain parka / sealskin pants / sealskin boots | seal, gut (intestine), sinew | 0.8 | 8 | -2 | 1.0, and keeps you dry | Arctic summer, rain, boats |

Over slot (adds to whatever is worn):

| Item | From | Cold | Heat | Notes |
|---|---|---|---|---|
| Wool cloak | wool | +4 | -1 | sheds rain if fulled |
| Hare-skin robe | ~85 hares | +8 | -3 | Athabaskan, Great Basin |
| Bison robe | 1 bison | +10 | -4 | also bedding |
| Gut parka / oiled cape | seal or cow gut, wool + fat | +1 | 0 | rain does not set *wet* |
| Down quilt cloak | 300-600 g down + linen | +12 | -5 | wet keeps 0.15 |
| Feather cloak (Cretaceous) | Utahraptor, Velociraptor feathers | +8 | -3 | like bird feathers: wet keeps 0.2 |

Check against E2: double caribou (48) + fat (4) + a hot meal + some fortitude covers Utqiaġvik's
-41 while active; at rest outside it does not (as in reality: shelter and robes at night).
Sheepskin (32) covers Moscow; loden (24) Kassel; bison (30) Konza; the desert robe (+6) with
shade, water and the cool hours covers Hermosillo's summer only in part, which is real too.

### E5. Armor (protection, weight, heat cost)

Protection is relative (plate = 10) against cut, pierce (arrows, stabs), blunt and bite, from D1.
Cold is the set's own small effect; heat is the penalty. Weight drives a stamina cost (Askew:
2.1-2.3 times the walking cost for the heaviest).

| Armor set | From | Cut | Pierce | Blunt | Bite | Cold | Heat | Mass | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Gambeson | linen or wool cloth (16-30 layers), or quilted with down | 4 | 3 | 4 | 4 | +10 | -6 | 3 kg | the under-layer for mail and plate |
| Rawhide / hardened leather | cow, bison, boar hide; wax optional | 4 | 3 | 5 | 6 | +2 | -3 | 4 kg | cheap; good against bites |
| Layered hide + wood slats | elk or bison hide, alder or birch slats | 5 | 5 | 5 | 6 | +2 | -4 | 7 kg | Tlingit style |
| Bone or ivory lamellar | antler, walrus ivory, bone, sinew lacing | 6 | 6 | 4 | 6 | 0 | -4 | 8 kg | Chukchi / Iñupiat; worn over the parka |
| Caiman-plate | caiman back hide with osteoderms | 5 | 5 | 4 | 7 | 0 | -4 | 6 kg | game invention on the Roman crocodile suit |
| Protoceratops-hide (Cretaceous) | thick frill and body hide | 5 | 4 | 6 | 6 | +1 | -3 | 5 kg | game invention |
| Bronze | copper + tin | 7 | 7 | 6 | 8 | -1 | -6 | 20 kg | Dendra |
| Mail (over gambeson) | iron wire rings | 8 | 6 | 2 alone, 6 with gambeson | 8 | -2 | -5 | 10 kg | needs the under-layer |
| Plate | steel | 10 | 10 | 8 | 10 | -2 | -8 | 25 kg | |
| Shield | linden or fir boards, rawhide rim, iron boss | blocks | blocks | blocks | blocks | 0 | 0 | 4 kg | |

Notes for balance:

- Metal is cold to touch but the armor layer is thin and outside the clothing, so its cold effect
  is small (-1 to -2) *est.*; the big cost is heat and weight. The current iron set's +4 cold is
  better given to the gambeson that should be under it.
- The current leather set's +3 heat is the wrong sign: thick leather traps heat. Use -3.
- Fur is not armor. Move it to clothing as above.
- In heat, a full metal kit should hurt: desert summer (needs +17) minus plate (-8) leaves a
  player in plate unable to rest in the sun, as the Dendra study's hot, still, hard case shows.

### E6. Materials by animal and plant (summary)

| Source | Gives |
|---|---|
| Sheep | wool (1-2 kg a year a primitive sheep, shear yearly), sheepskin |
| Cow (bison) | rawhide, sole and harness leather, bison robe (one per hide), sinew, horn |
| Pig (boar) | tough leather for armor and boots, bristles |
| Deer | buckskin (about one shirt front and back from 2 hides *est.*), sinew, antler, bone needles |
| Reindeer / caribou | winter fur (4 for a double suit), calf skin, leg skin for boots, sinew thread, antler |
| Wolf, coyote | ruffs and trim (frost-shedding), cap fur |
| Fox | trim, linings, caps |
| Hare | skins for woven robes (~85 a blanket), mitten linings |
| Capybara | soft leather for gloves and light clothes |
| Jaguar | a trophy pelt (cap, cape); little warmth |
| Caiman | belly leather, back plates for armor |
| Chicken, gull, raven, small birds | feathers for fletching and stuffing; poor down |
| Eagle, owl | fletching; owl down |
| Eider-type ducks (if added) | down: 15-20 g a nest, ~60 nests a kilo |
| Seal (if added) | waterproof boots and pants; gut for rain parkas |
| Fish | fish skin for bags and boots (Ainu, Nanai: fish-skin clothing) *est.*; glue |
| Flax | linen (3-7 days water retting), oil for waterproofing |
| Nettle | coarse linen-like cloth |
| Cotton (desert, llanos) | robes, blankets |
| Moriche palm, yucca, agave | hats, sandals, cord, hammocks |
| Linden, birch | bast shoes, bark containers |
| Oak bark | tannin: 12 months in the pits for sole leather |

---

## Sources

1. Clothing insulation (ASHRAE garment and ensemble tables). https://en.wikipedia.org/wiki/Clothing_insulation
2. ASHRAE 55 (0.5 and 1.0 clo comfort zones). https://en.wikipedia.org/wiki/ASHRAE_55
3. Holmér and Nilsson, ISO 11079 IREQ algorithm, in R package comf, `calcIREQ`. https://search.r-project.org/CRAN/refmans/comf/html/calcIREQ.html (source: https://github.com/cran/comf/blob/master/R/calcIREQ.R)
4. Lund University, IREQ and wind chill calculator. https://www.design.lth.se/om-institutionen/forskningslaboratorier/aerosol-klimat-laboratoriet/klimat/verktyg/berakna-ireq-och-wct/
5. Dry thermal insulation of thick clothing (4 clo per inch). US Army report ADA031488. https://apps.dtic.mil/sti/pdfs/ADA031488.pdf
6. American Meteorological Society glossary, "clo". https://glossary.ametsoc.org/wiki/clo/
7. Oakes et al. 1995, Comparison of traditional and manufactured cold weather ensembles. Climate Research 5:83-90. https://www.int-res.com/articles/cr/5/c005p083.pdf
8. Thermal imaging and physiological analysis of cold-climate caribou-skin clothing. Arctic, 2020. https://journalhosting.ucalgary.ca/index.php/arctic/article/view/69909
9. Inuit clothing. https://en.wikipedia.org/wiki/Inuit_clothing
10. Gates of the Arctic National Park, Caribou skin clothing. https://www.nps.gov/gaar/learn/historyculture/caribou-skin-clothing.htm
11. Indigenous Peoples Atlas of Canada, Traditional clothing. https://indigenouspeoplesatlasofcanada.ca/article/clothing/
12. Northern Caribou, People. https://www.northerncaribou.ca/people/
13. Bröde, Havenith et al. 2008, Non-evaporative effects of a wet mid layer on heat transfer through protective clothing. Eur J Appl Physiol 104:341-349. https://repository.lboro.ac.uk/articles/journal_contribution/Non-evaporative_effects_of_a_wet_mid_layer_on_heat_transfer_through_protective_clothing/9348953
14. Rangifer (2004), insulation of reindeer fur, wet and dry. https://septentrio.uit.no/index.php/rangifer/article/download/262/248/998
15. Polar bear behavior: morphologic and physiologic adaptations (citing Scholander et al. 1950). https://digitalcommons.odu.edu/cgi/viewcontent.cgi?article=1535&context=biology_fac_pubs
16. Woolmark, Wool is breathable (moisture factsheet). https://pre.woolmark.com/globalassets/_06-new-woolmark/_industry/research/factsheets/gd2405-wool-breathable-_134.pdf
17. Lund University, reduction of clothing insulation by wind, permeability and walking. https://lup.lub.lu.se/record/709089
18. Shkolnik, Taylor, Finch, Borut 1980, Why do Bedouins wear black robes in hot deserts? Nature 283:373-375. https://www.nature.com/articles/283373a0
19. Extreme cold weather clothing. https://en.wikipedia.org/wiki/Arctic_clothing
20. Canadian Museum of History, Gwich'in clothing artifacts. https://www.historymuseum.ca/gwichin/artifacts/
21. Kritsch and Wright-Fraser 2002, The Gwich'in Traditional Caribou Skin Clothing Project. Arctic 55(2). https://journalhosting.ucalgary.ca/index.php/arctic/article/view/63761
22. 5 facts about valenki. https://gw2ru.com/lifestyle/225633-facts-valenki-russian-wool-boots
23. What did traditional Russian clothing look like? https://www.gw2ru.com/arts/74292-russian-national-costume
24. Encyclopedia of the Great Plains, tipis. https://plainshumanities.unl.edu/encyclopedia/doc/egp.arc.048.html
25. Buffalo robe. https://en.wikipedia.org/wiki/Buffalo_robe
26. Braintan.com, buffalo robes (weights and sizes). https://braintan.com/?p=981
27. Beals, ethnology of the Pima-Papago (NPS online books). https://www.nps.gov/parkhistory/online_books/berkeley/beals1/beals1i.htm
28. San Diego History Center, Seri pelican skin robe photographs. https://sandiegohistory.org/?p=41635
29. Liqui liqui. https://en.wikipedia.org/wiki/Liqui_liqui
30. Lifeder, Vestuario de la región Orinoquía. https://www.lifeder.com/vestuario-region-orinoquia/
31. Manejo de Mauritia flexuosa para artesanías en la altillanura colombiana. https://repository.udistrital.edu.co/handle/11349/17911
32. Arte Amazonia, moriche hammocks. https://arte-amazonia.com/objects-materials/moriche-hammocks
33. Equitrekking, Cowboys of Brazil's Pantanal. https://equitrekking.com/articles/entry/cowboys_of_brazils_pantanal
34. Permanent Style, Loden: the cloth, the coat, the history. https://www.permanentstyle.com/2025/03/loden-the-cloth-the-coat-the-history.html
35. Rare Breeds Survival Trust, Soay. https://www.rbst.org.uk/soay
36. Carleton College archaeology lab, wool spinning data. https://222.arcn.sites.carleton.edu/lab-group-data/week-4-wool-lab-group-d-data/
37. How fast can hand spinners spin? https://gansey.blogspot.com/2014/02/how-fast-can-hand-spinners-spin.html
38. Journal of Agricultural Science, desiccating and retting flax (yields by retting method). https://resolve.cambridge.org/core/journals/journal-of-agricultural-science/article/study-of-the-use-of-the-trimesium-salt-of-glyphosate-to-desiccate-and-ret-flax-and-linseed-linum-usitatissimum-and-of-its-effects-on-the-yield-of-straw-seed-and-fibre/387176173CCD0244BAA01F07014CD117
39. Flax to linen yarn. https://allfiberarts.com/flax-to-linen-yarn/
40. University of Glasgow, Making thread in Bronze Age Britain. https://www.gla.ac.uk/news/archiveofnews/2018/july/headline_599105_en.html
41. Nettle as a distinct Bronze Age textile plant (Voldtofte). https://www.eurekalert.org/news-releases/661786
42. Outdoor Life, How to brain-tan a deer hide in 8 steps. https://www.outdoorlife.com/articles/hunting/2016/06/how-brain-tan-deer-hide-8-steps/
43. The New Craftsmen, Britain's last oak-bark tannery. https://www.thenewcraftsmen.com/blogs/news/britain-s-last-oak-bark-tannery
44. Encyclopaedia Britannica 4th ed. (1810), Tanning. https://jimclifford.ca/early_encyclopedia_britannica/articles/1810/eb_4th_1810_020179.html
45. Green beef hide weights and hide share of live weight (Thaksin University). https://ebook.tsu.ac.th/store/book/research/full2009/full2009/assets/basic-html/page263.html
46. Garrett Leather, hide sizes. https://garrettleather.com/resources/hide-sizes.html
47. The Leather Guy, buckskin deer hides 8-16 ft². https://theleatherguy.org/products/gold-3-5-oz-8-16-sf-buckskin-deer-hides
48. Natural Exotics, elk rawhide. https://naturalexotics.com/store/p/3571-Elk-Rawhide-1.aspx
49. Northern Prairie Wildlife Research Center, eiders and eiderdown. https://www.arlis.org/docs/vol1/35000323/eider.html
50. Vollebak, eiderdown. https://vollebak.com/pages/eiderdown
51. FAO, goose production: feather and down harvesting. https://www.fao.org/4/y4359E/y4359e0c.htm
52. South Peace Historical Society, textiles (rabbit-skin blankets). https://calverley.ca/article/01-076-textiles/
53. Pipe Spring National Monument, Rabbit blanket. https://www.nps.gov/pisp/historyculture/rabbit-blanket.htm
54. Antunes et al. 2016, Empty forest or empty rivers? A century of commercial hunting in Amazonia. Science Advances. https://pmc.ncbi.nlm.nih.gov/articles/PMC5061472
55. Smith 1976, Spotted cats and the Amazon skin trade. Oryx. https://cambridge.org/core/services/aop-cambridge-core/content/view/9863BF23F756DF6E94A418A9293B800E/S0030605300014095a.pdf/spotted-cats-and-the-amazon-skin-trade.pdf
56. ACOUP, Punching through some armor myths (reporting Williams, The Knight and the Blast Furnace, 2003). https://acoup.blog/2019/06/21/collections-punching-through-some-armor-myths/
57. Arrows against linen and leather armour. https://www.academia.edu/5520314/Arrows_Against_Linen_and_Leather_Armour
58. Bodkin point. https://en.wikipedia.org/wiki/Bodkin_point
59. Longbow vs. armor (summarising Strickland and Hardy, The Great Warbow, and replica tests). https://www.isegoria.net/?p=26061
60. Askew, Formenti, Minetti 2012, Limitations imposed by wearing armour on Medieval soldiers' locomotor performance. Proc R Soc B 279:640-644. https://pmc.ncbi.nlm.nih.gov/articles/PMC3248716
61. Flouris et al. 2024, Analysis of Greek prehistoric combat in full body armour based on physiological principles. PLOS ONE. https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0301494
62. Cleveland Museum of Art, hauberks (9.3-11.1 kg). https://www.clevelandart.org/art/1916.1543
63. Fogarty et al., Soldiers' perceived versus actual heat strain in a jungle environment (and the armour WBGT figure it cites). https://pmc.ncbi.nlm.nih.gov/articles/PMC4580807/
64. Boiled leather. https://en.wikipedia.org/wiki/Boiled_leather
65. Anchorage Daily News, Master carver Tommy Joseph explores the art of battle (Tlingit armor). https://www.adn.com/arts/article/master-carver-tommy-joseph-explores-art-battle/2014/06/02
66. Hood Museum of Art, ivory plate, possibly part of plate armor. https://hoodmuseum.dartmouth.edu/objects/181.2.26162
67. Kunstkamera (Peter the Great Museum), Chukchi armour. https://catalog.ethnomuseum.ru/entity/OBJECT/294748
68. The weight of Viking shields. https://science-and-fiction.org/swordfighting/viking_shield.html
69. Hurstwic, Viking shields. https://hurstwic.com/history/articles/manufacturing/text/viking_shields.htm
70. British Museum, Crocodile-skin suit of armour. https://artsandculture.google.com/asset/crocodile-skin-suit-of-armour/MwEr3NNie7N5QA
71. Gambeson. https://en.wikipedia.org/wiki/Gambeson
