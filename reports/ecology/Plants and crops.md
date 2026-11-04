# Plants and crops: real agronomy for the plant table

Research for `notes/ECOLOGY.md`, part 3 (plants grow at `warmth × water × light × fertility`) and
part 9 (fertile soil). The same numbers, one row per plant, are in `plants.json` next to this file,
with a `forage_by_biome` table for grass and browse. 2026-11-03.

**How sure is it.** Numbers with a source are marked by the link in the table. Numbers marked
*est.* are agronomy common knowledge or my reasoning, and `plants.json` lists them per plant under
`estimates`. FAO's crop pages (fao.org/land-water/.../crop-information) have been retired since
the search engines indexed them; their figures here come from the indexed text. The Cretaceous
numbers are the weakest: there are no "yields" in the fossil record, only what grew where and what
was eaten.

---

## A. Crops

### A1. Temperature, frost and season

`t_base` is the temperature below which a crop doesn't develop (the base for degree days).
`best` is the mean daily temperature range of fastest growth. `max` is where growth stops or the
crop is badly hurt (often at flowering).

| Crop | Base °C | Best °C | Max °C | Frost | Days sowing to harvest | Degree days (base) |
|---|---|---|---|---|---|---|
| Winter wheat | 0 | 15-23 [1] | ~34 *est.* | hardy: about -20 when hardened [2] | 180-250 [2] | ~1,900-2,400 (0 °C) *est.* |
| Spring wheat | 0 | 15-23 | ~34 *est.* | sensitive [2] | 100-130 [2] | ~1,300-1,700 (0 °C) *est.* |
| Barley | 0 | 15-20 *est.* | ~32 *est.* | light; hardiest winter barley -15 [20] | 90-120 *est.* (FAO: small grains 120-150 [3]) | *est.* |
| Winter rye | 0 | 12-20 *est.* | ~30 *est.* | the hardiest cereal: -30 [20] | 280-320 *est.* | - |
| Carrot | 3.3-4.4 [9][10] | 16-21 *est.* | ~30 *est.* | moderate (roots keep at -5) *est.* | 70-120 *est.* | ~750 (4.4 °C) [9] |
| Potato | ~7 *est.* (2-7 in the literature) | tubers 15-18 soil (FAO) [5], 15-20 [6] | 30 [5] | none: leaves die at -1 | 120-150 [5] | ~1,100-1,500 (7 °C) *est.* |
| Beetroot | 4.6 [11] | 15-20 *est.* | ~30 *est.* | light | 55-90 *est.* | ~650 (4.4 °C) [9] |
| Pumpkin / squash | 10 [9] | 20-30 *est.* | ~35 *est.* | none | 95-120 [3] | ~1,300 (10 °C) [9] |
| Melon | ~12 *est.* | 25-30 *est.* | ~38 *est.* | none | 120-160 [3] | - |
| Sugar cane | ~12 *est.*; active growth from ~20 [7] | 22-30; sprouting 32-38 [7] | ~38 *est.* | none; ripening wants cool, frost-free [7] | 9-24 months, usually 15-16 [7] | - |
| Maize | 10 [4] | 24-30 *est.*; germination 18-20 [4] | ~38 *est.* | none; grown where mean > 15 [4] | 125-180 grain [3] | 2,500-3,000 medium; 1,800 early; 3,700 late (10 °C) [4] |
| Rice | 10 [8] | 25-30 (20-35) | 35 [8] | none | 90-150 [3] | ~1,800-2,500 (10 °C) *est.* |
| Common bean | ~10 *est.* | 18-25 *est.* | ~30 (flowers drop) *est.* | none | dry 95-110, green 75-90 [3] | *est.* |
| Tepary bean | ~12 *est.* | 25-32 *est.* | ~40+ *est.* | none | 60-120 [21] | - |
| Sorghum | 8 (or 11) [12] | 27-34 (30-34) [12] | 40-42 [12] | none | 120-130 [3] | ~1,500-2,000 (10 °C) *est.* |
| Pearl millet | 10 (8-13.5) [12] | 30-34 [12] | 41-47 (germination) [12] | none | 105-140 [3]; early types 70-90 *est.* | - |
| Cassava | ~12 *est.*; growth stops below 15 [13] | 25-29 (25-35 by source) [13] | ~37 *est.* (tolerant at 34 [13]) | none | 8-10 months early, 12-18 late [13] | - |
| Blueberry | ~7 *est.* | 20-26 *est.* | ~32 *est.* | dormant bush hardy (-30 *est.*), flowers die at -2 | bloom (May) to fruit 60-90 *est.*; chill 800-1,200 h [14] | - |
| Apple | 4.4 (degree hours) [15] | 18-24 *est.* | ~35 *est.* | dormant tree hardy, blossom dies at -2 | bloom (Apr-May) to harvest 120-180 *est.*; chill 800-1,200 h [15] | - |

What this means for the game:

- **Three temperature groups do most of the work.** Cool-season crops (wheat, barley, rye,
  carrot, beet, potato) grow from about 0-7 °C and suffer above 30. Warm-season crops (maize,
  beans, pumpkin, melon, rice, sorghum) need about 10 °C and are killed by frost. Tropical crops
  (sugar cane, cassava, millet) want 12-15 °C or more and never see frost.
- **Winter wheat and rye are the only crops that live through a winter in the field.** They need
  the cold to head (vernalization), so the player sows them in autumn. Everything else is sown
  after the last frost.
- **Perennials (blueberry, apple) need winter chill to flower**: 800-1,200 hours under about 7 °C.
  A berry bush in the savanna should never fruit.

### A2. Water

| Crop | Water mm / season | Drought | Waterlogging | Source |
|---|---|---|---|---|
| Wheat | 450-650 | moderate | no | [2] |
| Barley, rye | 350-500 *est.* | rye tolerant | no | |
| Potato | 500-700 (120-150 day crop) | sensitive | no | [5] |
| Carrot, beet | 350-550 *est.* | moderate | no | |
| Pumpkin, melon | 400-600 *est.* | moderate | no | |
| Sugar cane | 1,500-2,500, evenly spread | sensitive | tolerates short floods *est.* | [7] |
| Maize | 500-800 | most sensitive at flowering (FAO group 4 of 4) | no | [3][4] |
| Rice | ~1,000-1,800 on the paddy *est.* | sensitive | **wants it** | |
| Beans | 300-500 | group 3 | no | [3] |
| Sorghum / millet | 450-650 | group 2 (stalls and recovers) | sorghum some | [3] |
| Cassava | 1,000-1,500 best; survives 500-5,000; 3+ months without rain | tolerant | no (roots rot) | [13] |
| Tepary bean | ~200-350 *est.* (monsoon) | very tolerant | no | [21] |

For the game's `water_need`: **dry** = sorghum, millet, tepary bean, rye, cactus, mesquite;
**wet** = rice, sugar cane, water plants, cloudberry; **moderate** = the rest.

### A3. Calendar, yield, nitrogen and biomes

Months are the temperate north unless marked. Yields are fresh weight of the harvested part
(grain at market moisture). "N" is nitrogen removed by a harvest, or fixed for a legume.

| Crop | Sow | Harvest | Yield kg/ha | N | Biomes in the game |
|---|---|---|---|---|---|
| Winter wheat | Sep-Oct | Jul-Aug | 1,000 unfertilized - 8,000 well fed; world ~3,500 *est.* | removes ~23 kg N per t grain [16]; 70-200 kg/ha | plains, forest, birch forest, mountains; irrigated in desert valleys (sown Nov-Dec, cut Apr-May) *est.* |
| Spring wheat | Apr-May | Aug-Sep | 1,500-5,000 *est.* | as wheat | plains, southern taiga, mountains |
| Barley | Apr-May | Jul-Aug | 1,500-6,000 *est.* | depletes | taiga, mountains, plains |
| Winter rye | Sep | Jul-Aug | 1,500-5,000 *est.* | depletes, less | taiga, birch forest, poor plains |
| Carrot | Mar-Jun | Jun-Nov | 25,000-60,000 *est.* | depletes ~80-120 *est.* | plains, forests, taiga, mountains |
| Potato | Apr-May | Aug-Oct | 15,000-45,000; world ~21,000 *est.* | depletes ~120-200 *est.* | plains, forests, taiga, mountains (from the Andes) |
| Beetroot | Apr-Jul | Jun-Oct | 25,000-50,000 *est.* | depletes | plains, forests, taiga, mountains |
| Pumpkin | May-Jun | Sep-Oct | 15,000-40,000 *est.* | depletes | plains, forest, savanna, desert (monsoon fields) |
| Melon | Apr-May | Jul-Sep | 15,000-35,000 *est.* | depletes | desert (irrigated), savanna, plains, beach |
| Sugar cane | tropics: start of rains (Apr-May) | the next dry season (Dec-Apr); regrows 3-5 times | 50,000-120,000 cane *est.* | heavy, 100-200 *est.* | savanna, swamp, beach |
| Maize | Apr-May; llanos May | Sep-Oct; llanos Aug-Oct | 1,000 (smallholder) - 11,000 (US); world ~5,800 *est.* | heavy: ~13-15 kg N per t grain [17] | plains, forest, savanna, desert (monsoon floodwater) |
| Rice | Apr-May; llanos May-Jun (and irrigated Nov-Dec) | Sep-Oct; llanos Sep-Oct (Mar-Apr) | 2,000-8,000 paddy *est.* | moderate | swamp, savanna (Calabozo is a rice district) |
| Common bean | May-Jun | Aug-Sep | 500-2,500; world ~900 *est.* | **fixes 20-60 kg N/ha** [18] | plains, forest, savanna, mountains |
| Tepary bean | Jul (monsoon) | Sep-Oct | 400-800 dry-farmed, 900-1,700 irrigated [21] | fixes | desert |
| Sorghum | May-Jun (also llanos) | Sep-Oct | 1,000-5,000 *est.* | depletes | savanna, plains (Kansas grows the most US sorghum), desert edge |
| Pearl millet | Jun-Jul (tropics) | Sep-Oct | 500-2,000 *est.* | depletes, light | dry savanna, desert edge (African and Indian, not American) |
| Cassava | start of rains, Apr-May | any time after 8-12 months, mostly the dry season | 8,000-30,000; improved 25-30 t [13] | moderate N, heavy K | savanna (the llanos' staple), raised ground in swamp |
| Blueberry | plant Apr or Oct | Jul-Sep | highbush ~10,100 [14]; wild bilberry 4-22 per ha of forest [22] | needs little | forest, birch forest, taiga, mountains, tundra edge |
| Apple | plant Mar-Apr | Aug-Oct | orchards 33,000-45,000 (France 42.8 t) [15] | needs little | forest, birch forest, plains, mountains |

Notes:

- **The llanos' farm** (the savanna's real crops): cassava, maize, rice, sorghum, plantain, and
  sugar cane, all sown when the rains come in May. Cassava is special: it can be left in the ground
  and dug in the dry season, a living store.
- **The Sonoran desert's farm**: the "three sisters" (maize, beans and squash) sown with the July
  monsoon on floodwater, tepary beans, and today irrigated winter wheat. Plus the wild harvest:
  saguaro fruit in June, mesquite pods Jun-Oct, prickly pear in Aug-Sep.
- **The taiga's farm**: barley, rye, potatoes, short-season spring wheat.
- **The swamp's farm**: rice and sugar cane. Cassava only on raised ground.

---

## B. Wild plants and grazing

### B1. Forage per biome

"Forage" here is what a grazer or browser can reach: grass and herbs, plus leaves and twigs under
about 2 m. The canopy of a forest grows far more, but it's out of reach. The months follow each
biome's real place.

| Biome (real place) | kg dry matter / ha / yr | Range | Grows | Rests | Source |
|---|---|---|---|---|---|
| Plains (Konza, Kansas tallgrass) | **3,940** | 1,790-7,560 by year | Apr-Oct, peak Jun-Jul | Nov-Mar | [23] |
| Forest (eastern oak forest) | ~600 *est.* | browse 225-560 uncut, 560-1,120 in a clearing | Apr-Oct | Nov-Mar (twigs only) | [24] |
| Birch forest | ~500 *est.* | young birch stand 167 of browse | May-Sep | Oct-Apr | [25] |
| Taiga (Alaska, Quebec) | ~300 *est.* | browse 17 (poor north) - 241 (moose core); old stands < 10 | May-Sep | Oct-Apr | [25][26] |
| Snowy plains (Toolik tundra) | ~800 *est.* | 320-3,050 vascular, by tundra type | Jun-Aug | Sep-May | [27] |
| Desert (Sonoran) | ~900 | 920-1,290 (Whittaker); shrub model 723, 113-2,296 by year; Chihuahuan grama 511 | Feb-Apr (winter annuals), Jul-Sep (monsoon) | May-Jun, Nov-Jan | [28] |
| Savanna (Calabozo, llanos) | **3,750** | rainy-season standing crop up to 5,440 | May-Nov | Dec-Mar (stands dry and brown) | [29] |
| Swamp (Pantanal) | ~4,500 *est.* | standing 944-5,219 grassland, 1,500-6,000 lowland | Oct-May (rains, flood Feb-Jun) | Jul-Sep (over half dead) | [30] |
| Mountains (Rockies) | ~1,500 *est.* | 600-2,900 rangeland types; aspen understory 1,120-2,240 | Jun-Sep, moving up with snowmelt | Oct-May | [31] |
| Beach (dunes) | ~500 *est.* | - | Apr-Oct | Nov-Mar | none |
| Ocean | 0 | kelp and seagrass are the sea's base | - | - | - |

The Cretaceous biomes are in `plants.json` (red dunes ~200, scrubland ~800, floodplain ~4,000,
fern forest ~1,500, crags ~300): all estimates by modern analogy, explained under D.

The shape to take from this: **the plains, savanna and swamp are the big grass biomes (about 4
t/ha a year), the desert and tundra grow about a quarter of that, and forests feed grazers little
except in clearings and burns.** In the taiga a fresh burn or clearing holds 10-20 times the
browse of old forest: disturbance makes food.

`plants.json` gives each biome a `monthly_share` (12 numbers summing to 1, estimates) for spreading
the year's growth over the calendar.

### B2. How fast grass grows back

- **Rest between grazings** (UK dairy guide [32]): 15-21 days in spring, 25-30 in summer, 30-40
  in autumn, 90-100 in winter. In heat or drought, 40-60+ days [33]; rangeland plants need 30-60
  days to recover [34].
- **Daily growth** of ryegrass pasture in Wales: about 25 kg DM/ha/day in late March, up to 139
  in late April, 77-86 in May-June, as low as 18 in a dry August [35]. In winter a new leaf can
  take 30 days [36].
- For the game: regrowth time = about **3 weeks at the best warmth and water**, stretched by the
  growth rule. In winter or the dry season the same rule gives about 3 months, which matches.

### B3. Leaves as browse

- Deer and moose eat leaves, twigs and buds below about 2 m. Browse is most plentiful in young
  regrowth: an Alaskan aspen stand 1 year after disturbance held 198 kg/ha, birch at 11 years 167,
  willow at 16 years 66, stands older than 25 years under 10 [25]. In Michigan oak forest,
  clearcuts held 560-1,120 kg/ha against 225-560 in uncut stands [24].
- Best browse: willow, aspen, birch (moose); oak, maple, hazel, brambles (deer); acacia and
  mesquite leaves and pods (dry-season browse in savanna and desert). Conifer needles are poor:
  only grouse and, in hard winters, deer eat them.
- For the game: leaves are a slow, steady food that **stays available in the dry season** (trees
  reach deeper water) but in winter only as twigs, at a lower food value.

### B4. Wild fruit and seed seasons

| Biome | Food | Category | Season (months) | Source |
|---|---|---|---|---|
| Plains | grass seed heads | seeds | Jul-Oct | *est.* |
| Plains | wild sunflower | seeds | Aug-Oct | *est.* |
| Plains, forests | dandelion leaves and roots | leaves, roots | Mar-Oct, flowers Apr-Jun | *est.* |
| Plains, forests | rose hips | fruit | Sep-Feb (hang on all winter) | *est.* |
| Forest | wild strawberry, raspberry, blackberry | fruit | Jun, Jul, Aug-Sep | *est.* |
| Forest | hazelnuts | seeds | Aug-Oct | *est.* |
| Forest | acorns: heavy every 3-4 years, bumper 2 years in 10 | seeds | Sep-Nov | [37] |
| Forest, birch forest | crab apples | fruit | Sep-Oct | *est.* |
| Forest, birch, taiga | blueberry / bilberry | fruit | Jul-Sep | [14][22] |
| Taiga, birch, tundra | lingonberry (8-18 kg per ha of forest; keeps under snow) | fruit | Aug-Oct, into spring | [22] |
| Taiga, tundra | cloudberry (bogs) | fruit | Jul-Aug | [22] |
| Tundra | crowberry (holds over winter) | fruit | Aug-Oct, and May-Jun | *est.* |
| Taiga, forests | mushrooms | (fungi) | Jul-Oct after rain | *est.* |
| Taiga, mountains | spruce and pine seed | seeds | Aug-winter | *est.* |
| Mountains | pine nuts (whitebark, limber) | seeds | Aug-Oct | *est.* |
| Desert | saguaro fruit | fruit | Jun-Jul (the O'odham new year) | [38] |
| Desert | mesquite pods (3-12 kg a tree; 3-4 t/ha planted) | seeds | Jun-Oct, second crop after the monsoon | [39] |
| Desert | prickly pear fruit; pads all year | fruit, leaves | Aug-Sep (late summer-early autumn) | [38][40] |
| Savanna | moriche palm fruit (morichales along streams) | fruit | Mar-Aug | [41] |
| Savanna, desert | acacia pods | seeds | dry season | *est.* |
| Swamp | acuri palm nuts | fruit | dry season (uncertain) | *est.* |
| Swamp | wild rice | seeds | as the flood peaks, Mar-May | *est.* |

---

## C. Soil fertility

### C1. How fast cropping tires the soil

- **A harvest takes nitrogen away in the crop.** Wheat removes about 22-26 kg N per tonne of grain
  [16], maize about 13 [17]. A 6 t wheat crop takes about 140 kg N/ha; potatoes, maize and sugar
  cane are the heavy feeders, beans and blueberries the light ones.
- **It doesn't fall to zero.** Broadbalk at Rothamsted has grown wheat on the same plot with no
  fertilizer or manure since 1843. It still yields about **1 t/ha**, the same as a century ago
  [19]: the soil settles at a low level fed by rain, weathering and free-living microbes. Old
  guides gave the unfertilized plot 1.2-1.6 t/ha, against **6-7 t/ha** where manure or fertilizer
  supply plenty, and up to 13.8 t/ha (the record, 2014) with the best modern fertilizer and
  variety [19].
- **The drop is fast at first.** On freshly ploughed prairie or forest soil, yields fall by about
  half within 10-30 years of cropping without manure (*est.*: the organic matter built under grass
  is used up, then it levels off).

### C2. What restores it

| Restorer | Real numbers | Source |
|---|---|---|
| **Farmyard manure (cattle)** | 6 kg total N per tonne fresh, 25% dry matter; typical rate 25 t/ha, i.e. about 150 kg N/ha. Only 10-20% is available in the first year (fresh 20%, stored 10%), the rest over years. Broadbalk's manured plot gets 35 t/ha a year (about 210 kg N). Samples range 1.3-32.6 kg N/t. | [42][43] |
| **Compost** | Hot composting: 3-6 weeks of heat (55-65 °C for 1-2 weeks kills pathogens), then 2-8 weeks curing. Passive heaps and dairy manure: 10-14 weeks to months; curing can run 6-12 months. | [44] |
| **Legumes** | Fixed per season: common bean 20-60 kg N/ha (a poor fixer), faba bean ~90, soybean ~100 (rain-fed) - 175 (irrigated), lupin ~165, red clover 60-370, white clover 80-540 (55-295 grazed), alfalfa 100-350 (up to 466 in year 3). Grain legumes leave little after harvest; a clover or alfalfa ley ploughed in leaves most of it. | [18][45] |
| **Fallow** | A season or year unplanted lets organic matter mineralize and, in dry land, stores water; in the tropics, shifting cultivation leaves fields 5-20 years under bush. *est.* | none fetched |

### C3. Fertility and yield: a rough curve

From Broadbalk (wheat): nil about 1 t/ha, manure alone or medium fertilizer 6-7 t, the best
treatment 8-10+ t. So the unfertilized field gives **about 15-25% of a well-fed one**, and the
curve flattens at the top (more manure past a point gives little). For the game's fertility 0 to 3:

| Fertility | Meaning | Yield (share of best) | Real equivalent |
|---|---|---|---|
| 0 | worn out | 0.2 | Broadbalk nil plot, 170 years without manure |
| 1 | tired | 0.5 | a few years of cropping without manure |
| 2 | ordinary (start) | 0.8 | fresh farmland, or manured every few years |
| 3 | rich | 1.0 | manured every year, or after a clover ley |

(The yield shares are my fit to the Broadbalk numbers, an estimate.) This also suggests fertility
should never stop growth: at 0 the growth rate multiplier is about 0.2-0.3, not 0.

How the real numbers map to ECOLOGY.md's rules:

- **A harvest of a soil-tiring crop lowers fertility by one**: fits heavy feeders (wheat, maize,
  potatoes, sugar cane, carrots). Light feeders (rye, barley on poor soil, berries, apples) could
  lower it only every other harvest.
- **Compost or dung raises it by one**: one application of about 25 t/ha of manure, or its
  compost.
- **Beans raise it by one when harvested**: generous next to the real 20-60 kg N/ha, most of it
  removed with the beans. Clover (80-540 kg N/ha) is the real soil-feeder; a clover block or a
  "clover ley" would make this honest.
- **A season left unplanted brings back one in the rains**: reasonable for a fallow, if slow
  in reality.

---

## D. Cretaceous plants

### D1. Grass: not in our Cretaceous

Grasses existed in the Cretaceous but were rare, and grasslands didn't exist.

- The oldest grasses: silicified epidermis and phytoliths stuck in the teeth of the early
  hadrosauroid *Equijubus*, from the Albian (113-101 Ma) of China (Wu et al. 2018, *National
  Science Review*) [46].
- By the Maastrichtian (~66 Ma), India had at least five grass lineages in titanosaur dung,
  among dicots, conifers and palms: the sauropods ate "a wide range of plants", grass only a little
  of it (Prasad et al. 2005, *Science*) [47]. The rice tribe was already there too (Prasad et al.
  2011, *Nature Communications*) [48].
- Grass-dominated habitats came in the Cenozoic: open-habitat grasses in the Paleogene, C3
  grasslands spreading in the mid to late Cenozoic, C4 grasslands in the late Neogene (Strömberg
  2011, *Annual Review of Earth and Planetary Sciences*) [49].
- **No grass is recorded from the Cedar Mountain or the Djadokhta.** The game's Cretaceous should
  keep grass out, as it does: the ground cover is ferns, horsetails, mosses and low flowering herbs.

### D2. What grew in each biome

| Biome | Real place | What grew | Evidence |
|---|---|---|---|
| **Red dunes** | Djadokhta Fm., Bayn Dzak, Mongolia, Campanian ~75-71 Ma | Almost no plant fossils. Paleosols with caliche hold fossil roots (*Radicites gobiensis*), read as semi-arid with seasonal rain. Interdune ponds and short-lived streams were the green spots. A general source gives conifers and cycads as the dominant plants of the time. | [50][51][52] |
| (nearby, wetter) | Nemegt Fm., Gobi, a little later | Probably araucarian conifers dominant, with ginkgos and angiosperms from plane trees to duckweeds, a pondweed-like plant, and a grass-like plant. | [53] |
| **Scrubland** | none named | By analogy: cycadophytes (cycads, bennettitaleans), conifer shrubs, ferns in gullies, early angiosperm shrubs. | inference |
| **Floodplain** | Cedar Mountain Fm., Utah | *Short Canyon Member (Aptian-Albian)*: spores 90% of the pollen (ferns), conifers 7%, angiosperms 3%; petrified *Taxodioxylon* (a swamp-cypress-like conifer) and angiosperm wood. *Mussentuchit Member (Cenomanian, ~99 Ma)*: a coastal plain with a high water table; its leaf flora is angiosperm (152 of 153 leaves, 18 dicot types) while its pollen still has ferns (*Gleicheniidites*) and conifers. Across the formation: the tree fern *Tempskya* (nine species) "in swamps on flood plains", with fossil dung of three sizes in tunnels in its stems. | [54][55][56][57][58] |
| **Fern forest** | analog: Clearwater Fm., Alberta (Albian, ~110 Ma) | A conifer forest with a fern understory; the nodosaur *Borealopelta* ate in a recently burned patch regrowing as fern (6% charcoal in its stomach). | [59] |
| **Crags** | none named | Ferns and lycopsids in crevices, scattered conifers and cycadophytes. | inference |

Across the Cretaceous: flowering plants spread step by step: water plants first (Barremian),
then wet and disturbed ground, then every environment from the Cenomanian on (Coiffard et al.
2012, *PNAS*) [60]. So a mid-Cretaceous floodplain (the Mussentuchit) is already full of small
flowering trees and shrubs, while the earlier Cedar Mountain is fern and conifer country.

### D3. What herbivorous dinosaurs ate

| Eater | What it ate | Evidence |
|---|---|---|
| Sauropods (in general) | Best energy: *Equisetum* (horsetail), *Araucaria*, *Ginkgo*, *Angiopteris* (a fern). Poor: cycads, tree ferns, podocarps. Araucaria is rich but slow to ferment, so a food for long-gutted giants. A 2023 re-test again found *Equisetum*, *Araucaria* and *Angiopteris* the most digestible; an older ash test ranked cycads best, so cycads are disputed. | Hummel et al. 2008, *Proc. R. Soc. B* [61]; 2023 re-test [62] |
| Horsetails as food | Living *Equisetum* species: "highly digestible, nutritious", likely a preferred food. | Gee et al. 2019 [63] |
| Titanosaurs (India) | Indiscriminate: dicots, conifers, palms, some grass. | Prasad et al. 2005 [47] |
| Nodosaur *Borealopelta* | Stomach: 88% leaf tissue, 85% of it leptosporangiate ferns, 3% cycads, trace conifer; also stems, wood, charcoal. Selective: it avoided Osmundaceae and marattialean ferns. | Brown et al. 2020, *R. Soc. Open Sci.* [59] |
| Hadrosaur *Maiasaura* (Two Medicine Fm.) | Dung with 13-85% fragmented, fungus-rotted conifer wood, eaten on purpose; later Kaiparowits dung also has crustaceans (from rotting logs). | Chin 2007, *Palaios* [64] |
| Hadrosaur *Brachylophosaurus* | Gut contents: mm-scale leaf fragments, chewed fine; pollen of ferns, conifers and flowering plants (secondhand). | Tweet et al. 2008, *Palaios* [65] |
| Hadrosaur *Edmontosaurus* | Tooth wear says a grazer of low plants, horsetails the guess (no grass then). | Williams et al. 2009, *PNAS* [66] |
| Early hadrosauroid *Equijubus* | Grass, the oldest known (Albian China). | Wu et al. 2018 [46] |
| Ceratopsian *Protoceratops* (Djadokhta) | Herbivore with a shearing beak and teeth for tough, fibrous plants; no gut contents known. | general sources [52] |

For the game: **horsetails are the Cretaceous's best food and fit the "grass" category; ferns are
the common "leaves"; conifers (Araucaria) and ginkgo are the high browse; cycads are a poor, tough
fall-back; rotten logs are food for hadrosaurs (and their insects).**

---

## E. Content gaps: what each biome needs

★ = matters most for the food chain (feeds many species, or fills a season when nothing else is
there).

| Biome | Missing food plants | Why |
|---|---|---|
| **Plains** | ★ grass seed heads (seeds from tall grass, for rodents and birds); ★ clover (grazing, and the honest soil-feeder); wild sunflower (autumn seeds); prickly pear at the dry edge; maize, beans and squash as crops | Seed-eaters (mice, sparrows, doves) are the prairie's middle tier; today grass gives no seeds. |
| **Forest** | ★ acorns from oaks (autumn mast: deer, boar, bears, squirrels, jays); ★ mushrooms (also the rot half of the loop); hazelnuts; brambles (strawberry, raspberry, blackberry); crab apples from the trees, not only leaves | Autumn fattening before winter needs mast and fruit. |
| **Birch forest** | ★ mushrooms; raspberries; birch seed and catkins (finches); rowan berries (winter birds) | |
| **Taiga** | ★ lingonberry (winter berries under snow); ★ willow and young birch browse (moose); ★ spruce seed in cones (squirrels, crossbills); ★ mushrooms; lichen; cloudberry in bogs; barley, rye and potatoes as crops | Winter is the bottleneck: food that lasts under snow. |
| **Snowy plains** | ★ reindeer lichen (the caribou's winter food); ★ sedges and cottongrass (the tundra's grass); ★ dwarf willow and birch (hares, ptarmigan); crowberry; cloudberry | Today there is almost nothing to eat; tundra grazers can't live there. |
| **Desert** | ★ mesquite pods (the desert's largest food, Jun-Oct); ★ prickly pear fruit and pads; ★ saguaro fruit from the existing cactus (June); desert annual seeds after rain (kangaroo rats, quail); tepary beans and monsoon maize as crops | Dry-season foods that make the desert habitable. |
| **Savanna (llanos)** | ★ moriche palm and its morichales (fruit and water in the dry season); ★ water plants and sedges in the esteros (capybara); grass that browns in the dry season (look and food value); cassava, maize and rice as crops; acacia pods | The llanos' life gathers at wet places in the dry season. |
| **Swamp (Pantanal)** | ★ water hyacinth and other floating plants (capybara, marsh deer); ★ sedges and floodplain grasses; ★ acuri palm nuts (macaws, peccaries); wild rice (ducks, rodents); cattail and reeds | The swamp's grazers live on water plants: in Pantanal dung studies, capybaras are mixed feeders on grasses and sedges [67]. Sugar cane is a crop there, not a native. |
| **Mountains** | ★ pine nuts (bears, nutcrackers); alpine meadow herbs; huckleberries (the berry bush can stand in); lichen; barley and potatoes as high crops | |
| **Ocean** | ★ kelp and seaweed (fish and urchins); seagrass | The sea's plant base, best as fish catch. |
| **Beach** | ★ seaweed wrack (its insects feed shorebirds); dune grass; beach plum or sea grape | |
| **Cretaceous, all** | ★ cycads and bennettitaleans; ★ rotten logs (hadrosaurs, beetles); ginkgo; early flowering shrubs with small fruit (for small mammals and early birds) | |
| **Floodplain** | ★ Tempskya tree ferns; ★ swamp cypress (Taxodioxylon); flowering shrubs; water plants in pools | |
| **Red dunes** | ★ interdune pond plants (horsetail, water plants) as the only green; sparse shrubs | |

The top five for the food chain overall: **seeds from grass (plains), acorns (forest), lichen and
sedge (tundra), mesquite and prickly pear (desert), and water plants (swamp and llanos)**. Each
fills a biome or a season where the game's grazers have nothing to eat now.

---

## Sources

1. FAO, Crop yield response to water (Irrigation and Drainage Paper 66), wheat: https://www.fao.org/4/i2800e/i2800e07.pdf
2. FAO Land & Water, Wheat (indexed text): https://www.fao.org/land-water/databases-and-software/crop-information/wheat/fr/
3. FAO, Irrigation Water Management: crop water needs, growing periods, drought sensitivity: https://www.fao.org/4/u3160e/u3160e04.htm
4. FAO Land & Water, Maize: https://www.fao.org/land-water/databases-and-software/crop-information/maize/fr/
5. FAO Land & Water, Potato: https://www.fao.org/land-water/databases-and-software/crop-information/potato/fr
6. Mbugua et al. 2020, potato temperature review: https://www.cropj.com/mbugua_14_9_2020_1405_1414.pdf
7. FAO Land & Water, Sugarcane: https://www.fao.org/land-water/databases-software/crop-information/sugarcane/en/ and https://www.fao.org/land-water/databases-and-software/crop-information/sugarcane/fr
8. Rice base temperature 10 °C: https://www.phytojournal.com/archives/2020/vol9issue4S/PartH/S-9-4-67-408.pdf ; https://agris.fao.org/search/fr/records/66e845a3cd53c72de2ca1999
9. Crop heat requirement table (low authority): https://growbydate-site.pages.dev/data/crop-heat-requirements/
10. Purdue MRCC, growing degree days: https://mrcc.purdue.edu/resources/growing-degree-day-description
11. Cardinal temperatures of vegetables (red beet 4.6 °C): https://agris.fao.org/search/ru/records/6765996f6784547a43e36c19
12. Cardinal temperatures, sorghum and millet: https://pmc.ncbi.nlm.nih.gov/articles/PMC7581754/table/Tab3 ; ICRISAT: https://oar.icrisat.org/4112/1/CP_154.pdf
13. Cassava: https://naads.or.ug/?p=5026 ; https://www.redalyc.org/pdf/331/33144653006.pdf ; https://robpyne.com.au/cassava-growing/
14. Blueberry chill and yield: https://growables.org/information/LowChillFruit/BlueberryArcRFC.htm ; https://redi.cedia.edu.ec/document/7223
15. Apple chill, heat and yields: https://www.horticulturejournal.usamv.ro/pdf/2023/issue_2/Art4.pdf ; https://ojs.lib.unideb.hu/actaagrar/article/download/13162/11743/29136 ; https://apples.extension.org/timing-of-apple-tree-bloom
16. Wheat N removal per tonne: https://www.agrojournal.org/23/03-14.html
17. Grain N contents (maize 13.1 g/kg): https://applbiolchem.springeropen.com/articles/10.1186/s13765-020-0493-6/tables/2
18. Common bean N fixation: https://www.frontiersin.org/journals/plant-science/articles/10.3389/fpls.2020.01172/pdf ; https://open-science.canada.ca/server/api/core/bitstreams/ee09721f-acbe-4af8-881a-ea38b0136ebb/content
19. Rothamsted Broadbalk: https://www.era.rothamsted.ac.uk/dataset/rbk1/02-OAWWYields ; https://era.rothamsted.ac.uk/eradoc/OCR/guide1991-10-16 ; https://era.rothamsted.ac.uk/eradoc/OCR/guide1984-9-18 ; https://era.rothamsted.ac.uk/Broadbalk
20. Cereal cold hardiness (oats -13, barley -15, wheat -21, rye -30): https://scilit.com/publications/0e7c5f90c31f7b119b398bb344b2df2f ; Nordic limits: https://journal.fi/afs/article/download/72654/34443/96775
21. Tepary bean: https://en.wikipedia.org/wiki/Phaseolus_acutifolius ; https://prosea.prota4u.org/view.aspx?id=3311 ; https://www.kirkhousetrust.org/_files/ugd/b134c0_770becd89c514de99542dc62176c6a6d.pdf
22. Wild berry yields in Central Finland (Silva Fennica 1984): https://doaj.org/article/9b6f10d886fc456b82704247eec0ea62
23. Konza Prairie NPP: https://daac.ornl.gov/NPP/guides/NPP_KNZ.html ; https://link.springer.com/doi/10.1007/s100210000057
24. Deer browse, clearcut and uncut oak stands (Gysel and Stearns 1968, cited): https://research.fs.usda.gov/treesearch/41034
25. Moose browse after disturbance, interior Alaska: https://www.adfg.alaska.gov/static/home/library/pdfs/wildlife/propubs/wb17_1_55_seaton-et-al.pdf
26. Moose forage in unproductive boreal Quebec: https://openpolar.no/Record/crwiley:10.1111%2Fj.1469-7998.1997.tb05825.x/Description
27. Shaver and Chapin 1991, arctic tundra production: https://openpolar.no/Record/crwiley:10.2307%2F1942997/Details ; https://daac.ornl.gov/NPP/guides/NPP_TLK.html
28. Desert production: https://archive.jornada.nmsu.edu/files/bibliography/86-UNM-Whitford.pdf ; https://fds.duke.edu/db/aas/Biology/faculty/jfreynol/publications/259979 ; https://link.springer.com/article/10.1007/s10021-023-00850-4 ; https://read.qxmd.com/read/17968592/aboveground-net-primary-production-dynamics-in-a-northern-chihuahuan-desert-ecosystem
29. Calabozo savanna NPP: https://daac.ornl.gov/NPP/guides/NPP_CLB.html ; Breymeyer review: https://rcin.org.pl/miiz//Content/115849/PDF/WA058_91427_P2840-T49_Eko-Pol-A-Nr-4.pdf
30. Embrapa Pantanal forage: https://www.infoteca.cnptia.embrapa.br/infoteca/bitstream/doc/812049/1/CT52.pdf ; https://www.alice.cnptia.embrapa.br/alice/bitstream/doc/1057897/1/crispimbiomass.pdf ; https://www.infoteca.cnptia.embrapa.br/infoteca/bitstream/doc/812653/1/FOL07.pdf
31. Montana rangeland production: https://journals.uair.arizona.edu/index.php/jrm/article/download/7210/6822 ; aspen understory: https://research.fs.usda.gov/treesearch/27788
32. FAS Scotland, rotational grazing for dairy farms: https://www.fas.scot/downloads/guide-to-rotational-grazing-for-dairy-farms
33. Rest periods in heat and drought: https://www.halterhq.com/articles/rotational-grazing-for-cattle
34. Texas Parks and Wildlife, livestock recommendations: https://tpwd.texas.gov/landwater/land/private/agricultural_land/epxt2010/Appendix%20D%20Livestock%20Management%20Recommendations.pdf
35. Welsh Pasture Project daily growth: https://businesswales.gov.wales/farmingconnect/node/8817
36. ForFarmers, grazing management: https://www.forfarmers.co.uk/news-knowledge/manage-grazing-02-2
37. Oak mast years: https://beaufort.ces.ncsu.edu/2025/11/the-secret-life-of-acorns-oaks-wildlife-and-the-science-of-mast-years/ ; https://www.douglascountymg.org/mg_articles/Acorns.pdf
38. Sonoran desert fruits: https://savorthesouthwest.blog/2016/07/07/promise-preparedness-present-fulfillment-with-fruits-of-the-desert/
39. Velvet mesquite: https://www.fs.usda.gov/database/feis/plants/tree/provel/all.html ; https://www.osti.gov/biblio/6838849-prosopis-pod-production-comparison-north-american-south-american-hawaiian-african-germplasm-young-plantations ; https://dunbarspringneighborhoodforesters.org/?p=66
40. Prickly pear harvest: https://tucsonfoodie.com/2018/09/07/prickly-pear-harvesting-preparation/
41. Moriche palm in the llanos (Gragson 1995, the Pume): https://ethnobiology.org/sites/default/files/pdfs/JoE/15-2/Gragson1995.pdf ; https://scalar.usc.edu/works/plants-and-people/muritia-flexuosa-habitat---amazon-wetlands-and-llanos
42. AHDB, using farmyard manure (RB209 values): https://ahdb.org.uk/knowledge-library/using-farmyard-manure-fym ; https://www.fas.scot/downloads/soil-nutrient-network-case-study-bogindollo-angus-2/
43. NRM, manure N variability: https://cawood.co.uk/wp-content/uploads/2023/07/NRM-The-Value-of-Manure-and-Slurry-Summary-Cattle-Information-Sheet.pdf
44. Composting times: https://eawag.ch/en/department/sandec/projects/mswm/decentralised-composting/windrow-technique ; https://core.ac.uk/download/32544769.pdf
45. Legume N fixation: https://research.slu.se/en/publications/nitrogen-fixation-in-perennial-forage-legumes-in-the-field/ ; https://www.saskforage.ca/s/Growing-Forage-Legumes-in-Rotation.pdf ; https://orgprints.dk/id/eprint/3567/1/3567.pdf
46. Wu et al. 2018, Dinosaur-associated Poaceae epidermis and phytoliths from the Early Cretaceous of China, *National Science Review*: https://www.dinodata.de/bibliothek/pdf_d/2018/nwx145.pdf
47. Prasad et al. 2005, Dinosaur coprolites and the early evolution of grasses and grazers, *Science* 310: https://ui.adsabs.harvard.edu/abs/2005Sci...310.1177P/abstract
48. Prasad et al. 2011, Late Cretaceous origin of the rice tribe, *Nature Communications*: https://ideas.repec.org/a/nat/natcom/v2y2011i1d10.1038_ncomms1482.html
49. Strömberg 2011, Evolution of grasses and grassland ecosystems: https://www.biology.washington.edu/pubs/evolution-grasses-and-grassland-ecosystems
50. Naugolnykh 2016, Upper Cretaceous paleosols of Bain-Dzak: https://journals.rcsi.science/0031-0301/article/view/166820
51. Djadokhta Formation (summary of Saneyoshi, Dingus): https://mdwiki.org/wiki/Djadochta_Formation
52. Protoceratops: https://www.ebsco.com/research-starters/earth-and-atmospheric-sciences/protoceratops ; https://en.wikipedia.org/wiki/Protoceratops
53. Nemegt Formation flora: https://en.wikipedia.org/wiki/Nemegt_Formation
54. Harris and Arens 2016, A mid-Cretaceous angiosperm-dominated macroflora from the Cedar Mountain Formation: https://www.cambridge.org/core/journals/journal-of-paleontology/article/midcretaceous-angiospermdominated-macroflora-from-the-cedar-mountain-formation-of-utah-usa/03B8EE5F8918C6206DDDBFEB9DDE3ACA
55. Tucker et al. 2020, Mussentuchit Member: https://par.nsf.gov/servlets/purl/10167697 ; Tucker et al. 2022, paralic sedimentology: https://par.nsf.gov/biblio/10339584
56. Lupia et al. 2023, Short Canyon Member palynoflora (GSA): https://gsa.confex.com/gsa/2023SC/webprogram/Paper385130.html ; Murphy 2023 thesis: https://shareok.org/handle/11244/337587
57. Tidwell and Hebbert 1992, Species of the Cretaceous tree fern Tempskya from Utah: https://www.journals.uchicago.edu/doi/10.1086/297057 ; https://scholarsarchive.byu.edu/facpub/1445/
58. Thayn 1973, angiosperm woods with Tempskya (BYU thesis): https://scholarsarchive.byu.edu/etd/7989
59. Brown et al. 2020, Dietary palaeoecology of an Early Cretaceous armoured dinosaur: https://ui.adsabs.harvard.edu/abs/2020RSOS....700305B
60. Coiffard et al. 2012, Rise to dominance of angiosperm pioneers in European Cretaceous environments, *PNAS*: https://pmc.ncbi.nlm.nih.gov/articles/PMC3529080
61. Hummel et al. 2008, In vitro digestibility of fern and gymnosperm foliage: https://pmc.ncbi.nlm.nih.gov/articles/PMC2600911
62. Digestibility of dinosaur food plants revisited and expanded (2023): https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10723699/
63. Gee et al. 2019, horsetails as Mesozoic superfood: https://sciencedaily.com/releases/2019/11/191108155447.htm
64. Chin 2007, herbivorous dinosaur coprolites from the Two Medicine Formation: https://bioone.org/journals/Palaios/volume-22/issue-5/palo.2006.p06-087r/THE-PALEOBIOLOGICAL-IMPLICATIONS-OF-HERBIVOROUS-DINOSAUR-COPROLITES-FROM-THE-UPPER/10.2110/palo.2006.p06-087r.full ; crustaceans: https://www.nature.com/articles/nature.2017.22649
65. Tweet et al. 2008, gut contents of Brachylophosaurus: https://bioone.org/journals/palaios/volume-23/issue-9/palo.2007.p07-044r/Probable-Gut-Contents-Within-A-Specimen-Of-Brachylophosaurus-Canadensis-Dinosauria/10.2110/palo.2007.p07-044r.full
66. Williams, Barrett and Purnell 2009, dental microwear in hadrosaurs: https://pmc.ncbi.nlm.nih.gov/articles/PMC2708679/
67. Pantanal herbivore diets (cattle, capybara, pampas deer): https://journals.uair.arizona.edu/index.php/jrm/article/download/20242/19872
