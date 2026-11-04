# Biome climates: a real place for every biome

Research for `notes/ECOLOGY.md` ("From real places, like the species"). Each of the game's 17 biomes
gets one real *type locality*, a place of its kind with published climate normals, and its numbers month
by month: the mean, the daily minimum and maximum, the rain in mm, the days with rain, and the days with
snow lying. The Cretaceous biomes get what the palaeoclimate literature supports. The same numbers are in
`biome_climates.json`, one object per biome.

**How to read the tables.**
- Months are the place's own. Only the swamp (the Pantanal) is in the southern hemisphere: shift it by six
  months for the game's calendar.
- *Rain days* use each source's threshold, given under each table: NOAA counts ≥0.25 mm (0.01 in),
  Mexico's SMN ≥0.1 mm, most others ≥1 mm. A ≥0.25 mm count runs roughly a third above a ≥1 mm count.
- *Snow-cover days* are the days in the month with snow on the ground. Some are counts, and some are
  estimates from the snow season's dates; each section says which.
- *Half range* is half the difference between the warmest and coldest months: what `climate.rs` calls
  `seasons`.
- *Wet and dry months* in the JSON follow one rule. A dry month has less rain in mm than twice its mean
  in °C (the Walter–Lieth / Gaussen drought rule). A wet month has at least the year's monthly average,
  or, for the two tropical places, at least 100 mm. The prose under each table describes the seasons in
  words.

| Biome | Type locality | Köppen | Year °C | Half range °C | Rain mm/yr |
|---|---|---|---|---|---|
| `base:plains` | Manhattan, Kansas (Konza Prairie) | Dfa | 12.8 | 14.2 | 909 |
| `base:forest` | Kassel, Germany | Cfb (Dfb by the 1961-1990 normals) | 9.3 | 8.7 | 707 |
| `base:birch_forest` | Moscow, Russia | Dfb | 6.3 | 12.9 | 713 |
| `base:taiga` | Fairbanks, Alaska | Dfb bordering Dfc (subarctic) | −2.1 | 19.8 | 295 |
| `base:snowy_plains` | Utqiaġvik (Barrow), Alaska | ET | −10.2 | 14.9 | 138 |
| `base:desert` | Hermosillo, Sonora | BWh (borderline BSh) | 25.7 | 8.1 | 345 |
| `base:savanna` | Arauca, Colombia (llanos) | Aw | 27 | 1.6 | 1,868 |
| `base:swamp` | Nhecolândia, Pantanal, Brazil | Aw | 25.1 | 3.4 | 1,046 |
| `base:mountains` | Davos, Switzerland (1,594 m) | Dfc | 3.9 | 8.8 | 1,046 |
| `base:sequoia_grove` | Giant Forest, Sequoia National Park (1,721 m) | Csb | 10.4 | 8.4 | 1,047 |
| `base:ocean` | Ponta Delgada, Azores | Csa (borderline Cfa/Csb) | 18.2 | 4.1 | 988 |
| `base:beach` | Faro, Portugal | Csa | 18.2 | 6.2 | 455 |
| `base:deep_caves` | any cave (rule) | — | surface year mean | 0 | — |
| `animals:red_dunes` | Djadokhta Formation, Mongolia | BWk/BWh (estimate) | ~17 (est.) | ~12 (est.) | ~175 (est.) |
| `animals:scrubland` | Barun Goyot Formation, Mongolia | BSk/BSh (estimate) | ~17 (est.) | ~11 (est.) | ~350 (est.) |
| `animals:floodplain` | Cedar Mountain Formation (Ruby Ranch), Utah | Aw/BSh-like seasonal subtropics (estimate) | ~21 (est.) | ~8 (est.) | ~800 (est.) |
| `animals:fern_forest` | Kaiparowits Formation, Utah | Cwa/Aw-like humid subtropical with a dry season (estimate) | ~20 (est.) | ~8 (est.) | ~1400 (est.) |
| `animals:crags` | Sevier highlands (estimate) | Cfb/Cwb-like upland (estimate) | ~13 (est.) | ~8 (est.) | ~850 (est.) |

# The world's biomes

## `base:plains` — Plains: a tallgrass prairie (Manhattan, Kansas)

**Place:** Manhattan, Kansas, USA (Konza Prairie tallgrass prairie lies just south). 39.19° N, 96.60° W, 322 m. Northern hemisphere. Köppen Dfa.

**Why it fits:** Manhattan sits beside the Konza Prairie, the reference tallgrass prairie (a Long-Term Ecological Research site run by Kansas State University). Wichita, the other suggestion, is 1.5 °C warmer and Cfa by the new normals; Manhattan is still continental (Dfa) and closer to today's 12 °C.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | −1.6 | 0.9 | 6.6 | 12.4 | 18.3 | 23.9 | 26.7 | 25.4 | 20.7 | 13.7 | 6.2 | 0.4 | 12.8 |
| Daily min °C | −7.4 | −5.5 | −0.3 | 5.7 | 12.3 | 17.9 | 20.7 | 19.1 | 13.9 | 6.7 | −0.1 | −5.3 | 6.5 |
| Daily max °C | 4.2 | 7.3 | 13.5 | 19.1 | 24.4 | 29.9 | 32.7 | 31.7 | 27.4 | 20.6 | 12.6 | 6.1 | 19.1 |
| Rain mm | 16 | 29 | 55 | 86 | 133 | 139 | 117 | 112 | 87 | 64 | 41 | 30 | 909 |
| Rain days | 5.2 | 5.6 | 8.2 | 10.4 | 12.9 | 10.8 | 9.9 | 10.7 | 8.4 | 8.1 | 6.4 | 5.4 | 102 |
| Snow-cover days | 7 | 6 | 1.5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0.8 | 5 | 20.3 |

Year: 12.8 °C, half range 14.2 °C, 909 mm on 102 days (threshold: 0.25 mm (0.01 in)), mean daily range 12.7 °C.

**Wet and dry:** Rain peaks in May–June with spring thunderstorms (133–139 mm) and stays high through September; winter is the dry season (16 mm in January) but not a drought, because it is cold. Snow lies on about 20 days a year (≥1 in), mostly December–February. By the rule in the JSON: wet Apr, May, Jun, Jul, Aug, Sep; dry none.

**Caveats:** Rain days count ≥0.25 mm (0.01 in), NOAA's threshold: about a third more days than a ≥1 mm count. The monthly split of the 20.3 snow-cover days is our estimate.

**Sources:**
- https://www.ncei.noaa.gov/access/us-climate-normals/ (station USC00144972, Manhattan KS, 1991-2020)
- https://en.wikipedia.org/wiki/Manhattan,_Kansas (climate box citing NOAA 1991-2020)
- https://lter.konza.ksu.edu/

## `base:forest` — Forest: a central German deciduous forest (Kassel, Hesse)

**Place:** Kassel, Hesse, Germany (central German beech-oak upland forests: Reinhardswald, Kaufunger Wald). 51.32° N, 9.50° E, 167 m. Northern hemisphere. Köppen Cfb (Dfb by the 1961-1990 normals).

**Why it fits:** Kassel lies among the beech and oak uplands of northern Hesse (Reinhardswald, Kaufunger Wald), the core of Europe's temperate deciduous forest. WMO 1991–2020 normals.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 0.9 | 1.4 | 5 | 9.3 | 13.4 | 16.1 | 18.3 | 17.9 | 13.8 | 9.2 | 4.8 | 1.3 | 9.3 |
| Daily min °C | −1.6 | −1.4 | 1.2 | 4.4 | 8.4 | 11.1 | 13.3 | 13 | 9.5 | 5.9 | 2.3 | −0.9 | 5.4 |
| Daily max °C | 3.1 | 4.5 | 9.2 | 14.4 | 18.6 | 21.3 | 23.6 | 23.6 | 18.8 | 13.3 | 7.2 | 3.5 | 13.4 |
| Rain mm | 53.8 | 46.6 | 51.6 | 39.7 | 69.1 | 67.6 | 74.4 | 60.1 | 56.2 | 58.4 | 61.3 | 68.2 | 707 |
| Rain days | 16.6 | 16 | 16 | 13.3 | 14.7 | 14.2 | 15.7 | 14.5 | 13.5 | 16 | 18.7 | 20.1 | 189.3 |

Year: 9.3 °C, half range 8.7 °C, 707 mm on 189.3 days (threshold: labelled 1.0 mm (likely 0.1 mm)), mean daily range 8.0 °C, relative humidity 77.9%.

**Wet and dry:** Rain falls all year (40–74 mm every month), slightly more in summer thunderstorms and in December; there is no dry season. Lying snow is short and irregular. By the rule in the JSON: wet May, Jun, Jul, Aug, Nov, Dec; dry none.

**Caveats:** The rain-day row is labelled ≥1.0 mm in the source, but 188 days a year is what German lowland stations count at ≥0.1 mm: read it as days with any rain. No monthly snow-cover normal was found (the DWD Climate Data Center has daily snow depths to count one). The elevation is the source page's and may be the city's.

**Sources:**
- https://en.wikipedia.org/wiki/Kassel (climate box citing WMO 1991-2020 normals via NOAA, station 10438)
- https://www.ncei.noaa.gov/products/wmo-climate-normals
- https://opendata.dwd.de/climate_environment/CDC/

## `base:birch_forest` — Birch forest: the mixed forest round Moscow

**Place:** Moscow (VDNKh), Russia (mixed spruce-birch-pine forest of the central Russian plain). 55.83° N, 37.62° E, 147 m. Northern hemisphere. Köppen Dfb.

**Why it fits:** The central Russian plain's mixed spruce, pine and birch forest, with birch dominating wherever forest regrows: the classic birch country. Moscow VDNKh has full 1991–2020 normals and snow-cover counts. Southern Finland (Jyväskylä, FMI 1991–2020: 3.8 °C, 622 mm, 155 snow-cover days) is the colder, more boreal alternative.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | −6.2 | −5.9 | −0.7 | 6.9 | 13.6 | 17.3 | 19.7 | 17.6 | 11.9 | 5.8 | −0.5 | −4.4 | 6.3 |
| Daily min °C | −8.7 | −8.8 | −4.2 | 2.3 | 8.1 | 12.2 | 14.8 | 13 | 8 | 3 | −2.4 | −6.5 | 2.6 |
| Daily max °C | −3.9 | −3 | 3 | 11.7 | 19 | 22.4 | 24.7 | 22.7 | 16.4 | 8.9 | 1.6 | −2.3 | 10.1 |
| Rain mm | 53 | 44 | 39 | 37 | 61 | 78 | 84 | 78 | 66 | 70 | 52 | 51 | 713 |
| Rain days | 21.4 | 17.6 | 12.2 | 12.3 | 14.1 | 12.9 | 14.8 | 11.7 | 11.9 | 15.3 | 15.1 | 21.1 | 180.4 |
| Snow-cover days | 31 | 28 | 28 | 4 | 0 | 0 | 0 | 0 | 0 | 3 | 16 | 28 | 138 |

Year: 6.3 °C, half range 12.9 °C, 713 mm on 180.4 days (threshold: not stated (likely 0.1 mm)), mean daily range 7.5 °C, relative humidity 78.0%.

**Wet and dry:** Summer is the wetter half (May–October, 61–84 mm), winter the drier but snow-covered half: stable snow from late November to early April, about 140 days, 30–40 cm deep by February–March. By the rule in the JSON: wet May, Jun, Jul, Aug, Sep, Oct; dry none.

**Caveats:** VDNKh is inside the city: the countryside runs about 1 °C colder. The rain-day row is 2015–2024 (threshold not stated, probably ≥0.1 mm; the months sum to 180, the source's year column says 170); the snow-cover row is from Russian Wikipedia's table with no period given.

**Sources:**
- http://www.pogodaiklimat.ru/climate/27612.htm (Pogoda i Klimat, VDNKh 1991-2020)
- https://en.wikipedia.org/wiki/Climate_of_Moscow (climate box, 1991-2020, citing Pogoda.ru.net)
- https://ru.wikipedia.org/wiki/Климат_Москвы (snow cover days; precipitation days 2015-2024)

## `base:taiga` — Taiga: interior Alaska's boreal forest (Fairbanks)

**Place:** Fairbanks, Alaska, USA (interior boreal forest; Bonanza Creek LTER). 64.80° N, 147.88° W, 132 m. Northern hemisphere. Köppen Dfb bordering Dfc (subarctic).

**Why it fits:** Fairbanks is the home of the Bonanza Creek LTER, a reference boreal-forest site (black and white spruce, birch, aspen), with NOAA normals and snow data. Its −2.1 °C mean and ±19.8 °C seasons sit near today's taiga. Yakutsk (central Siberia, 1991–2020: −8.0 °C, January −36.9, July 19.9, 235 mm, about 157 snowy days) is the extreme continental end of the same forest, and would make the taiga far harsher than the snowy plains in winter.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | −22.4 | −17.7 | −11.8 | 0.9 | 10.2 | 16.1 | 17.2 | 13.9 | 7.7 | −3.2 | −15.5 | −20.2 | −2.1 |
| Daily min °C | −27.3 | -24 | −19.7 | −5.7 | 3.7 | 10.1 | 11.7 | 8.7 | 2.3 | −7.6 | −20.1 | -25 | −7.7 |
| Daily max °C | −17.4 | −11.3 | −3.9 | 7.6 | 16.7 | 22.1 | 22.6 | 19.1 | 12.9 | 1.2 | −10.9 | −15.4 | 3.6 |
| Rain mm | 15 | 13 | 10 | 8.6 | 14 | 38 | 57 | 53 | 34 | 19 | 19 | 14 | 294.6 |
| Rain days | 8.7 | 6.9 | 5.7 | 3.7 | 6.2 | 10.8 | 12.8 | 13.5 | 10.7 | 9.8 | 9.5 | 8.8 | 107.1 |
| Snow-cover days | 31 | 28 | 31 | 23 | 0 | 0 | 0 | 0 | 0 | 14 | 30 | 31 | 188 |

Year: -2.1 °C, half range 19.8 °C, 294.6 mm on 107.1 days (threshold: 0.25 mm (0.01 in)), mean daily range 11.4 °C.

**Wet and dry:** Most of the 296 mm falls in June–September; May is dry (the fire season begins). Snow lies from 18 October to 23 April on average, up to about 60 cm deep in March. By the rule in the JSON: wet Jun, Jul, Aug, Sep; dry May.

**Caveats:** Rain days at ≥0.25 mm. Snow-cover days come from the mean snowpack dates, not a monthly count.

**Sources:**
- https://www.ncei.noaa.gov/access/us-climate-normals/ (Fairbanks Intl Airport, USW00026411, 1991-2020)
- https://en.wikipedia.org/wiki/Fairbanks,_Alaska (climate box citing NOAA 1991-2020)
- https://www.lter.uaf.edu/

## `base:snowy_plains` — Snowy plains: arctic coastal tundra (Utqiaġvik / Barrow, Alaska)

**Place:** Utqiagvik (Barrow), Alaska, USA (arctic coastal tundra). 71.29° N, 156.79° W, 10 m. Northern hemisphere. Köppen ET.

**Why it fits:** Utqiaġvik is the reference arctic tundra station (NOAA, and NOAA's Barrow observatory), on permafrost up to 400 m deep. Its −10.2 °C mean matches today's −10 °C almost exactly.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | −24.2 | −24.4 | −23.6 | −15.6 | −5.2 | 2.2 | 5.4 | 4.3 | 0.9 | −6 | −14.6 | −21.3 | −10.2 |
| Daily min °C | −27.7 | −27.9 | −27.3 | −19.2 | −7.5 | −0.5 | 2 | 1.7 | −0.9 | −8.4 | −17.8 | −24.6 | −13.2 |
| Daily max °C | −20.7 | −20.8 | −19.9 | −11.9 | −2.8 | 4.9 | 8.7 | 6.9 | 2.8 | −3.6 | −11.4 | -18 | −7.1 |
| Rain mm | 3.6 | 5.3 | 4.6 | 4.6 | 7.1 | 11 | 25 | 28 | 20 | 14 | 9.4 | 5.6 | 138.2 |
| Rain days | 4.8 | 5.5 | 5.1 | 5.3 | 6.3 | 6.3 | 9.7 | 11.5 | 13.6 | 13.5 | 9.7 | 6.7 | 98 |
| Snow-cover days | 31 | 28 | 31 | 30 | 31 | 10 | 0 | 0 | 3 | 21 | 30 | 31 | 246 |

Year: -10.2 °C, half range 14.9 °C, 138.2 mm on 98 days (threshold: 0.25 mm (0.01 in)), mean daily range 6.0 °C, relative humidity 81.2%.

**Wet and dry:** A polar desert: 137 mm a year, most of it July–October as rain and then snow. Snow lies from about the second week of October to about the second week of June (8 months); only July and August are reliably snow-free. By the rule in the JSON: wet Jul, Aug, Sep, Oct; dry none.

**Caveats:** Rain days at ≥0.25 mm. Snow-cover days are our estimate from the melt and onset seasons (NOAA GML measures them; the snow-free season has grown about a week per decade since 1975).

**Sources:**
- https://www.ncei.noaa.gov/access/us-climate-normals/ (Utqiagvik Wiley Post-Will Rogers Airport, USW00027502, 1991-2020)
- https://en.wikipedia.org/wiki/Utqiagvik,_Alaska (climate box citing NOAA 1991-2020)
- https://gml.noaa.gov/grad/snomelt.html
- https://www.eurekalert.org/news-releases/850307

## `base:desert` — Desert: the Sonoran Desert (Hermosillo, Sonora)

**Place:** Hermosillo, Sonora, Mexico (Sonoran Desert). 29.10° N, 110.95° W, 200 m. Northern hemisphere. Köppen BWh (borderline BSh).

**Why it fits:** Hermosillo is the Sonoran Desert's main station, low (about 200 m) and truly arid (BWh). Chihuahua city, the dogs' namesake, is semi-arid steppe at 1,400 m (about 18 °C): good chihuahua country, but not a desert biome.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 17.2 | 18.8 | 21.2 | 24.5 | 28.8 | 33.1 | 33.2 | 32.8 | 31.8 | 27.6 | 21.7 | 17.1 | 25.7 |
| Daily min °C | 9.8 | 10.9 | 12.8 | 15.7 | 19.8 | 24.3 | 26.1 | 26.2 | 24.8 | 19.9 | 13.9 | 9.8 | 17.8 |
| Daily max °C | 24.6 | 26.6 | 29.5 | 33.2 | 37.8 | 41.9 | 40.2 | 39.3 | 38.8 | 35.2 | 29.5 | 24.4 | 33.4 |
| Rain mm | 15.8 | 15.4 | 6.2 | 1.9 | 1.6 | 5.1 | 91.1 | 92.2 | 63.4 | 15.5 | 13.1 | 23.4 | 344.7 |
| Rain days | 2 | 1.7 | 0.9 | 0.3 | 0.2 | 0.5 | 8.4 | 8.9 | 5.1 | 1.3 | 1.2 | 2.2 | 32.7 |

Year: 25.7 °C, half range 8.1 °C, 344.7 mm on 32.7 days (threshold: 0.1 mm), mean daily range 15.6 °C, relative humidity 47.4%.

**Wet and dry:** Two rain seasons. The North American monsoon brings 72% of the year's 345 mm in July–September as thunderstorms; light Pacific winter rains ("equipatas") come in December–February. April–June is the dry, hot heart of the year (under 2 mm a month, June days at 41.9 °C). By the rule in the JSON: wet Jul, Aug, Sep; dry Jan, Feb, Mar, Apr, May, Jun, Oct, Nov, Dec.

**Caveats:** The source page captions the table 1991–2020 but cites SMN 1981–2010 for temperature; treat it as 1981–2010. Rain days at ≥0.1 mm (SMN). By the Walter–Lieth rule every month but July–September is dry.

**Sources:**
- https://smn.conagua.gob.mx/es/informacion-climatologica-por-estado?estado=son (SMN normals, Sonora)
- https://en.wikipedia.org/wiki/Hermosillo (climate box, Hermosillo Observatorio, citing SMN)

## `base:savanna` — Savanna: the llanos (Arauca, Colombia)

**Place:** Arauca, Colombia (the llanos of the Arauca river, chiguiro / capybara country). 7.09° N, 70.76° W, 128 m. Northern hemisphere. Köppen Aw.

**Why it fits:** Arauca, on the Arauca river at the Venezuelan border, is in the same flooded llanos as San Fernando de Apure (about 330 km east): open grass with gallery forest, the capybara's (chigüiro's) heartland. It replaces San Fernando because San Fernando's sources disagree: the WMO/NCEI 1991–2020 normals give 2,022 mm with 36 mm and 13 rain days in January (implausible for the llanos dry season), while JMA's normals give 1,544 mm with 0.4 mm in January. Arauca's IDEAM normals are complete and consistent. San Fernando's temperatures (27.5 °C, months 26.0–29.3) match Arauca's.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 27.4 | 28.3 | 28.8 | 27.7 | 26.6 | 25.8 | 25.7 | 26.1 | 26.6 | 26.8 | 26.9 | 26.9 | 27 |
| Daily min °C | 20.6 | 21.2 | 22.2 | 22.9 | 23 | 22.8 | 22.6 | 22.7 | 22.8 | 22.9 | 22.9 | 21.9 | 22.4 |
| Daily max °C | 33.1 | 34.3 | 34.6 | 33.2 | 31.2 | 30 | 29.9 | 30.6 | 31.5 | 31.7 | 31.5 | 31.9 | 32 |
| Rain mm | 11.9 | 23.8 | 50.6 | 147.2 | 267.5 | 304.7 | 309.6 | 224.7 | 187.1 | 200.1 | 109.7 | 30.8 | 1867.7 |
| Rain days | 2 | 3 | 5 | 13 | 19 | 22 | 22 | 20 | 17 | 14 | 10 | 5 | 152 |

Year: 27.0 °C, half range 1.6 °C, 1867.7 mm on 152 days (threshold: 1.0 mm), mean daily range 9.6 °C, relative humidity 79.7%.

**Wet and dry:** Tropical, northern hemisphere. Wet season April–November (peak June–July, over 300 mm; the savanna floods), dry season December–March (January 12 mm on 2 days; the grass dries and burns, and capybaras crowd the last water). The year's warmest month is March, at the end of the dry season, not midsummer. By the rule in the JSON: wet Apr, May, Jun, Jul, Aug, Sep, Oct, Nov; dry Jan, Feb, Mar, Dec.

**Caveats:** IDEAM 1981–2010; rain days at ≥1.0 mm.

**Sources:**
- http://www.ideam.gov.co/ (IDEAM climatological normals 1981-2010)
- https://en.wikipedia.org/wiki/Arauca,_Arauca (climate box, Santiago Perez Quiroz Airport, citing IDEAM 1981-2010)
- https://ds.data.jma.go.jp/cpd/monitor/nrmlist/NrmMonth.php?stn=80450 (JMA normals, San Fernando de Apure, for comparison)
- https://en.wikipedia.org/wiki/San_Fernando_de_Apure (WMO/NCEI 1991-2020, for comparison)

## `base:swamp` — Swamp: the Pantanal (Nhecolândia, Mato Grosso do Sul, Brazil)

**Place:** Nhecolandia (Embrapa's Fazenda Nhumirim), Pantanal, Mato Grosso do Sul, Brazil. 18.99° S, 56.62° W, 100 m. Southern hemisphere. Köppen Aw.

**Why it fits:** The capybara's prime habitat in the game is the swamp, and the Pantanal is the world's largest tropical wetland and capybara stronghold. Nhumirim (Embrapa's research farm) is a station inside it; Corumbá, the town on its edge, is nearly the same (25.7 °C, 998 mm). The Okefenokee, the other suggestion, would sit near today's 19 °C with rain all year (not researched here).

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 27.6 | 27.5 | 27.2 | 25.5 | 22.3 | 21.2 | 20.8 | 22.6 | 24.7 | 27 | 27.3 | 27.6 | 25.1 |
| Daily min °C | 23.4 | 23.4 | 22.8 | 20.6 | 16.8 | 15.5 | 14.2 | 15.4 | 18.3 | 21.5 | 21.8 | 22.8 | 19.7 |
| Daily max °C | 33.3 | 33 | 33.3 | 32.4 | 29.7 | 29.1 | 29.8 | 32.3 | 33.1 | 34.2 | 34.1 | 33.6 | 32.3 |
| Rain mm | 172.8 | 142.1 | 128.9 | 68.2 | 55.8 | 17.2 | 10.2 | 17.5 | 33.8 | 100.7 | 135.9 | 163.1 | 1046.2 |
| Rain days | 11 | 10 | 9 | 5 | 4 | 2 | 2 | 2 | 2 | 6 | 8 | 9 | 70 |

Year: 25.1 °C, half range 3.4 °C, 1046.2 mm on 70 days (threshold: 1.0 mm), mean daily range 12.6 °C, relative humidity 78.5%.

**Wet and dry:** SOUTHERN HEMISPHERE: the table is in the place's own months; shift by six months for the game. Wet season October/November–March (the austral summer, 129–173 mm a month), dry season June–September (winter, 10–34 mm, cold fronts bringing 14 °C nights). The floods lag the rain: the local plains fill January–April, and the Paraguay river peaks around June–July, so standing water lasts into the early dry season. By the rule in the JSON: wet Jan, Feb, Mar, Oct, Nov, Dec; dry Jun, Jul, Aug, Sep.

**Caveats:** INMET 1981–2010 (no 1991–2020 table was found for this station); rain days at ≥1.0 mm. Coordinates and elevation are approximate.

**Sources:**
- https://portal.inmet.gov.br/normais (INMET Normais Climatologicas do Brasil 1981-2010)
- https://en.wikipedia.org/wiki/Corumbá (climate boxes for Corumba and Nhecolandia/Nhumirim, citing INMET 1981-2010)

## `base:mountains` — Mountains: the Swiss Alps at 1,594 m (Davos)

**Place:** Davos, Graubuenden, Switzerland (subalpine valley, 1,594 m). 46.81° N, 9.84° E, 1594 m. Northern hemisphere. Köppen Dfc.

**Why it fits:** Davos is a long-running MeteoSwiss station (since 1876) in a subalpine valley among spruce, larch and alpine meadows, in the 1,500–2,500 m band.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | −4.7 | −4.2 | −0.7 | 3 | 7.4 | 11 | 12.8 | 12.6 | 8.7 | 5.1 | 0 | −3.6 | 3.9 |
| Daily min °C | −8.8 | −9 | −5.5 | −2 | 2.3 | 5.8 | 7.6 | 7.7 | 4.1 | 0.8 | −3.8 | −7.2 | −0.7 |
| Daily max °C | 0 | 1.2 | 4.6 | 8.3 | 12.8 | 16.5 | 18.5 | 18.2 | 14.2 | 10.8 | 4.8 | 0.6 | 9.2 |
| Rain mm | 70.1 | 52 | 56.7 | 54.4 | 88.7 | 128.7 | 133.1 | 149.5 | 96.3 | 77.1 | 71.2 | 68.1 | 1045.9 |
| Rain days | 8.8 | 7.9 | 8.9 | 9 | 12 | 13.7 | 13.8 | 13.6 | 10 | 8.7 | 9.1 | 9.8 | 125.3 |
| Snow-cover days | 31 | 28 | 31 | 20 | 2 | 0 | 0 | 0 | 0 | 3 | 18 | 31 | 164 |

Year: 3.9 °C, half range 8.8 °C, 1045.9 mm on 125.3 days (threshold: 1.0 mm), mean daily range 9.9 °C, relative humidity 74.2%.

**Wet and dry:** Summer is the wet season (June–August, 129–150 mm, convective rain); winter brings 452 cm of snowfall a year. Snow lies from about mid-November to late April. By the rule in the JSON: wet May, Jun, Jul, Aug, Sep; dry none.

**Caveats:** Rain days at ≥1.0 mm. No monthly snow-cover normal was found: the snow-cover row is our estimate (about 165 days), and the season has shortened since 1970.

**Sources:**
- https://www.meteoswiss.admin.ch/services-and-publications/applications/ext/climate-tables-normals.html (MeteoSwiss normals 1991-2020, Davos DAV)
- https://en.wikipedia.org/wiki/Davos (climate box citing MeteoSwiss 1991-2020)

## `base:sequoia_grove` — Giant sequoia grove: the Sierra Nevada's west slope (Giant Forest, 1,721 m)

**Place:** Giant Forest, Sequoia National Park, California (giant sequoia grove, 1,721 m). 36.56° N, 118.77° W, 1721 m. Northern hemisphere. Köppen Csb.

**Why it fits:** Giant Forest is one of the largest groves of giant sequoia, in the band where every grove grows (1,400–2,100 m on the west slope), among white fir, sugar pine, incense cedar and black oak. Its climate is the groves': cool, snowy winters and dry summers over deep soil (Ronald asked for the biome, 11-04).

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 3.3 | 3.4 | 4.8 | 7 | 11.4 | 15.9 | 20.2 | 19.8 | 16.8 | 11.9 | 6.6 | 3.3 | 10.4 |
| Daily min °C | −1.7 | −1.8 | −0.6 | 1.3 | 5.2 | 9.3 | 13.7 | 12.9 | 10.1 | 5.9 | 1.4 | −1.2 | 4.5 |
| Daily max °C | 8.4 | 8.7 | 10.3 | 12.8 | 17.7 | 22.4 | 26.7 | 26.8 | 23.6 | 17.9 | 11.7 | 7.7 | 16.2 |
| Rain mm | 193 | 182 | 173 | 92 | 45 | 17 | 8 | 4 | 18 | 50 | 110 | 155 | 1047 |
| Rain days | 7 | 8 | 7.6 | 6.2 | 4.6 | 1.9 | 1.7 | 1.2 | 1.8 | 3.1 | 4.5 | 6.6 | 54.2 |

Year: 10.4 °C, half range 8.4 °C, 1047 mm on 54.2 days (threshold: 1.0 mm, from 0.25 mm counts × 0.8), mean daily range 11.7 °C.

**Wet and dry:** Winter is the wet season: November to March bring over four fifths of the year's precipitation, much of it as snow (Lodgepole, 2,053 m: 5.6 m a year, 41.6 days with snow). June to September are dry, a Mediterranean mountain climate.

**Caveats:** Temperatures and precipitation are PRISM's 30-year normals for the Giant Forest Visitor Center; rain days are Lodgepole's (5 km away, NOAA 1991–2020, days ≥ 0.01 in) times 0.8 for days ≥ 1.0 mm. No monthly snow-cover normal was found. Wind and fog in `vitals::climate` are estimates for a sheltered montane forest.

**Sources:**
- https://en.wikipedia.org/wiki/Sequoia_National_Park (climate boxes: Giant Forest Visitor Center from the PRISM Climate Group; Lodgepole, 2,053 m, NOAA 1991-2020 normals)
- https://prism.oregonstate.edu/ (PRISM 30-year normals)
- https://www.ncei.noaa.gov/products/land-based-station/us-climate-normals (Lodgepole, CA, 1991-2020)

## `base:ocean` — Ocean: the Azores (Ponta Delgada, São Miguel)

**Place:** Ponta Delgada, Sao Miguel, Azores, Portugal (mid-Atlantic islands). 37.74° N, 25.66° W, 35 m. Northern hemisphere. Köppen Csa (borderline Cfa/Csb).

**Why it fits:** A small island in the open North Atlantic, 1,400 km from Europe: its climate is the sea's. IPMA 1991–2020 normals at the Afonso Chaves Observatory.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 15.1 | 14.8 | 15.2 | 16 | 17.4 | 19.6 | 21.8 | 23 | 22.1 | 19.8 | 17.4 | 15.9 | 18.2 |
| Daily min °C | 12.7 | 12.1 | 12.6 | 13.3 | 14.6 | 16.8 | 18.9 | 19.9 | 19.1 | 17.1 | 14.9 | 13.4 | 15.5 |
| Daily max °C | 17.5 | 17.4 | 17.9 | 18.8 | 20.2 | 22.4 | 24.7 | 26 | 25 | 22.6 | 19.9 | 18.3 | 20.9 |
| Rain mm | 102.1 | 87.2 | 97.1 | 79.6 | 61 | 44.4 | 26 | 45.8 | 78.2 | 118 | 95.3 | 152.8 | 987.5 |
| Rain days | 13 | 10.8 | 11.7 | 10.2 | 8.7 | 6.6 | 4.8 | 6.5 | 9.4 | 13.7 | 11.8 | 15 | 122.2 |

Year: 18.2 °C, half range 4.1 °C, 987.5 mm on 122.2 days (threshold: 1.0 mm), mean daily range 5.4 °C, relative humidity 80.5%.

**Wet and dry:** Wet season October–March (Atlantic storms, December 153 mm), a short dry summer (July 26 mm). The year range is small (14.8–23.0 °C) and the day range smaller (about 5 °C). By the rule in the JSON: wet Jan, Feb, Mar, Oct, Nov, Dec; dry Jul, Aug.

**Caveats:** Rain days at ≥1.0 mm; at ≥0.1 mm the airport counts 178 a year (frequent drizzle). The Azores are subtropical-temperate: a cooler open-ocean choice near 45–50° N would be closer to today's 14 °C.

**Sources:**
- https://www.ipma.pt/pt/oclima/normais.clima/ (IPMA normals 1991-2020, Ponta Delgada Observatorio Afonso Chaves)
- https://en.wikipedia.org/wiki/Ponta_Delgada (climate box citing IPMA 1991-2020)

## `base:beach` — Beach: the Algarve coast (Faro, Portugal)

**Place:** Faro, Algarve, Portugal (Ria Formosa barrier-island beaches). 37.02° N, 7.97° W, 8 m. Northern hemisphere. Köppen Csa.

**Why it fits:** Faro's station sits on the Ria Formosa lagoon behind a chain of sandy barrier-island beaches. IPMA 1991–2020 normals.

| | Jan | Feb | Mar | Apr | May | Jun | Jul | Aug | Sep | Oct | Nov | Dec | Year |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Mean °C | 12.3 | 12.9 | 14.9 | 16.6 | 19.3 | 22.3 | 24.5 | 24.6 | 22.4 | 19.6 | 15.7 | 13.4 | 18.2 |
| Daily min °C | 8.3 | 8.8 | 10.8 | 12.4 | 15 | 17.8 | 19.6 | 19.9 | 18.3 | 15.8 | 11.9 | 9.7 | 14 |
| Daily max °C | 16.3 | 17 | 18.9 | 20.7 | 23.5 | 26.8 | 29.4 | 29.2 | 26.4 | 23.3 | 19.6 | 17.1 | 22.3 |
| Rain mm | 49.3 | 41.9 | 46.8 | 39 | 26.2 | 5.1 | 0.9 | 3 | 21.5 | 56 | 75.5 | 90 | 455.2 |
| Rain days | 6 | 5.4 | 5.6 | 5.3 | 3.8 | 0.8 | 0.2 | 0.2 | 1.9 | 6 | 6.6 | 7 | 48.8 |

Year: 18.2 °C, half range 6.2 °C, 455.2 mm on 48.8 days (threshold: 1.0 mm), mean daily range 8.3 °C, relative humidity 69.2%.

**Wet and dry:** Mediterranean: wet season October–March (December wettest, 90 mm), a near-rainless summer (June–August 9 mm in all, under one rain day a month). The sea is about 16–17 °C in January and 22–24 °C in August–September. By the rule in the JSON: wet Jan, Feb, Mar, Apr, Oct, Nov, Dec; dry May, Jun, Jul, Aug, Sep.

**Caveats:** Rain days at ≥1.0 mm; humidity is NOAA 1961–1990.

**Sources:**
- https://www.ipma.pt/pt/oclima/normais.clima/ (IPMA normals 1991-2020, Faro)
- https://en.wikipedia.org/wiki/Faro,_Portugal (climate box citing IPMA 'Faro 1991-2020'; humidity from NOAA 1961-1990)

## `base:deep_caves` — under ground (a rule, not a place)

**The rule:** away from its entrances, a cave's air sits at about the mean annual temperature of the
surface above it, at that elevation. Seasonal and daily swings die out in the first metres of rock, and
the rock's heat capacity filters out the rest.
- NPS, Carlsbad Caverns: the temperature in caves "tends to reflect the average annual temperature for the
  area at that given elevation".
- NPS, Great Basin (Lehman Caves): bedrock is "often just about the average year round surface
  temperature". Lehman Caves holds 10 °C (50 °F) all year.
- Badino (2004), *Cave temperatures and global climatic change*, Int. J. Speleology 33: 103–113: cave
  temperature is closely tied to the external climate.

**Exceptions:**
- *Cold traps:* a cave that takes in winter air runs colder. Carlsbad's Big Room is 13 °C under a 17 °C
  surface mean.
- *Depth:* the geothermal gradient warms deep passages. Carlsbad's deepest point, 315 m down, is 20 °C.
- *Entrances:* the zone near an entrance follows the outside air.

**Humidity:** 90–100%. Lehman Caves ranges 90–100%. Carlsbad is near 100% deep inside, and its Big Room
drops to 87.5% in winter, when cold, dry outside air flows in. There is no day, season or rain; only
dripping water.

**For the game:** `deep_caves` at a fixed 12 °C is the year mean of a temperate surface, roughly the
plains (12.8 °C) or the forest (9.3 °C). Strictly, a cave under the desert should be about 26 °C and one
under the tundra about −10 °C (in permafrost, ice caves). Its humidity should be about 0.95, not 0.6.

**Sources:**
- https://home.nps.gov/cave/learn/nature/weather.htm
- https://www.nps.gov/grba/learn/nature/airflow-and-cave-climate.htm
- https://digitalcommons.usf.edu/kip_articles/811

# The Cretaceous

No monthly numbers exist for these places. What the literature gives is listed under each biome with its
source. Every number marked *estimate* is ours: a reasoned choice where the literature stops, to be
replaced if a better source turns up. Formation coordinates are approximate present-day positions. All
five places were in the northern hemisphere.

**How the estimates were made.**
- *Temperature:* we started from the nearest quantitative proxy, the Kaiparowits's leaf-margin MAT of
  20 °C in southern Laramidia. The Gobi was a continental interior at about 40–45° N, so we gave it a
  larger seasonal range and a somewhat cooler mean.
- *Rain:* we followed the sediments. Active dunes with calcretes mean arid. Calcic and vertic paleosols
  mean semi-arid and seasonal. Ponded, annually flooded plains mean humid.
- *Clumped-isotope temperatures* from soil and lake carbonates record the warm season, when the
  carbonate forms. They are not the year mean.

## `animals:red_dunes` — the Djadokhta Formation, Mongolia (arid dune field)

**Place:** Djadokhta Formation, Bayan Zag (Flaming Cliffs) and Ukhaa Tolgod, southern Gobi, Mongolia. Age 80–72 Ma. Palaeolatitude about 40-45 N (reconstructions vary; Wikipedia infoboxes give 30.7 N for the Djadokhta and 40.5 N for the Barun Goyot, both unsourced).

Campanian, about 80-72 Ma (eolian deposition from about 80 Ma: Hasegawa et al. 2008). LITERATURE: an arid sand-dune desert of at least 160,000 km2 with large cross-bedded dunes and mature calcretes (Hasegawa et al. 2008, 2009); paleosols at Bayan Zag read as semi-arid with seasonal rain, hot dry summers and short, probably humid winters (Naugolnykh 2016); short-lived ponds and streams between dunes, and dune collapses after heavy rain (Dingus et al. 2008; Loope et al. 1999); the Okavango-Kalahari is the modern analogue (Jerzykiewicz 1998). NO published MAT or MAP for the Djadokhta was found. The nearest number is the younger, wetter Nemegt Formation (Maastrichtian, Owocki et al. 2020, from Tarbosaurus tooth enamel): a monsoonal climate with summer rain, MAP 775-835 mm, and an MAT reported as about 7.6 C on Wikipedia's summary but 'ten degrees above today's Mongolia' in the paper's abstract, so treat that MAT as unsettled. ESTIMATES (ours): MAT about 17 C (12-22), warmest month about 30 C, coldest about 5 C (half range about 12 C: a continental interior at about 40-45 N), MAP about 100-250 mm in a short summer wet season (2-3 months), and wet/dry months are a guess following the Nemegt's summer monsoon (Naugolnykh's 'humid winters' would put the rain in winter instead).

| | Value | Basis |
|---|---|---|
| Mean annual temperature | 17 °C (12–22) | estimate |
| Half range, summer to winter | 12 °C | estimate |
| Mean annual precipitation | 175 mm (100–250) | estimate |
| Wet season | 2 months (Jul, Aug) | estimate |
| Dry season | 10 months (Jan, Feb, Mar, Apr, May, Jun, Sep, Oct, Nov, Dec) | estimate |

**Sources:**
- https://en.wikipedia.org/wiki/Djadochta_Formation
- https://www2.jpgu.org/meeting/2008/program/pdf/E111/E111-P010_e.pdf
- https://journals.rcsi.science/0031-0301/article/view/166820
- https://doi.org/10.1016/j.jseaes.2008.11.010
- https://doi.org/10.1206/442.1
- https://doi.org/10.1016/j.palaeo.2019.05.012
- https://en.wikipedia.org/wiki/Nemegt_Formation

## `animals:scrubland` — the Barun Goyot Formation, Mongolia (semi-arid, between the dunes and the rivers)

**Place:** Barun Goyot (Baruungoyot) Formation, Nemegt Basin (Khulsan, Hermiin Tsav), southern Gobi, Mongolia. Age 75–70 Ma. Palaeolatitude about 40-45 N (estimate).

Late Campanian to early Maastrichtian, partly coeval with the Djadokhta. LITERATURE: red eolian and interdune sandstones, transitional between the Djadokhta's dunes and the Nemegt's river plains; in the 'lithobiotope' model the Djadokhta is the arid dune habitat, the Barun Goyot the semi-arid transition and the Nemegt the wetter plains (Jerzykiewicz et al. 2021); wetness increases up through the Baruungoyot into the Nemegt (Eberth 2018). No MAT or MAP for the Barun Goyot itself was found; the Nemegt above it had summer monsoon rain and MAP 775-835 mm (Owocki et al. 2020). ESTIMATES (ours): MAT about 17 C (12-22), half range about 11 C, MAP about 250-450 mm, a summer wet season of about 3 months and a dry winter-spring.

| | Value | Basis |
|---|---|---|
| Mean annual temperature | 17 °C (12–22) | estimate |
| Half range, summer to winter | 11 °C | estimate |
| Mean annual precipitation | 350 mm (250–450) | estimate |
| Wet season | 3 months (Jun, Jul, Aug) | estimate |
| Dry season | 7 months (Jan, Feb, Mar, Apr, Oct, Nov, Dec) | estimate |

**Sources:**
- https://en.wikipedia.org/wiki/Barun_Goyot_Formation
- https://en.wikipedia.org/wiki/Nemegt_Formation
- https://doi.org/10.1016/j.palaeo.2017.11.018
- https://doi.org/10.1139/cjes-2020-0148
- https://doi.org/10.1016/j.palaeo.2019.05.012

## `animals:floodplain` — the Cedar Mountain Formation, Utah (seasonal floodplain)

**Place:** Cedar Mountain Formation, Ruby Ranch Member type area and 'Lake Carpenter' near Arches, east-central Utah, USA. Age 116–110 Ma. Palaeolatitude about 34 N (Suarez et al. 2014); Wikipedia's infobox gives 40.2 N.

Ruby Ranch Member, late Aptian-early Albian (about 116-110 Ma; 'Lake Carpenter' ash 115.65 Ma). LITERATURE: a semi-arid to seasonal floodplain with calcrete and vertic paleosols, carbonate concentrated by dry-season evaporation, and dolomite showing strong evaporation; aridity from the rain shadow of the Sevier highlands ('Nevadaplano') to the west (Suarez et al. 2014, J. Sed. Res. 84: 975). MAP from paleosol chemistry (Suarez et al. 2020/2021, Geol. Soc. London Spec. Publ. 498): CALMAG 736-1,042 mm, CIA-K 795-1,275 mm, a little lower during the C10 carbon-isotope event; a thesis using carbon-isotope methods got much lower values (274 and 599 mm for the member: Univ. of Arkansas). Temperatures: clumped isotopes of lake carbonate 15-31 C (calcite 27-41 C, dolomite 19-21 C), palustrine 19.8-44.5 C; these are spring-summer water temperatures, not the year mean, and values above 50 C are diagenesis (Suarez et al. 2023, 2025). Palaeolatitude about 34 N (Suarez et al. 2014). No published MAT. ESTIMATES (ours): MAT about 21 C (17-25), half range about 8 C, MAP about 800 mm (the proxies' overlap, likely high), a summer wet season of about 4 months and a winter-spring dry season of about 6 (monsoon-like, as the paleosols suggest; the exact months are a guess). Note: Utahraptor comes from the older Yellow Cat Member, now dated Berriasian-Valanginian (about 139-134 Ma, Joeckel et al. 2023), also seasonally dry with vertic paleosols, outside the 75-125 Ma window.

| | Value | Basis |
|---|---|---|
| Mean annual temperature | 21 °C (17–25) | estimate |
| Half range, summer to winter | 8 °C | estimate |
| Mean annual precipitation | 800 mm (600–1100) | paleosol proxies, narrowed (estimate) |
| Wet season | 4 months (Jun, Jul, Aug, Sep) | estimate |
| Dry season | 6 months (Nov, Dec, Jan, Feb, Mar, Apr) | estimate |

**Sources:**
- https://geolsoc.figshare.com/collections/Multi-proxy_strategy_for_determining_palaeoclimate_parameters_in_the_Ruby_Ranch_Member_of_the_Cedar_Mountain_Formation/5087091
- https://studio.m-anage.com/gsa/2017AM/webprogram/Paper304788.html
- https://par.nsf.gov/biblio/10437392
- https://par.nsf.gov/biblio/10654206
- https://archives.datapages.com/data/sepm/journals/084/084011/975_i1527-1404-84-11-975.htm
- https://scholarworks.uark.edu/etd/1199/
- https://en.wikipedia.org/wiki/Cedar_Mountain_Formation
- https://doi.org/10.3390/geosciences13020032

## `animals:fern_forest` — the Kaiparowits Formation, Utah (humid, monsoonal forest)

**Place:** Kaiparowits Formation, Grand Staircase-Escalante, southern Utah, USA (wet coastal-plain forest of southern Laramidia). Age 76.6–74.5 Ma. Palaeolatitude mid-latitude southern Laramidia (value not looked up).

Late Campanian, about 76.6-74.5 Ma. LITERATURE: alluvial to coastal plain beside the Western Interior Seaway, 'extensively ponded and annually flooded', humid and subtropical 'similar to the present-day Gulf Coast or certain areas of Southeast Asia' (Maccracken et al. 2022, PLOS ONE); 'overall wet (humid), tropical to subtropical wet-dry setting, with episodic longer dry spells' (Titus et al. 2021, PeerJ). MAT 20 +/- 1 C from leaf-margin analysis (Miller et al. 2013); paleosol carbonate clumped isotopes give warm-season soil temperatures of about 35 +/- 4 C (Burgener et al. 2019). Tyrannosaur tooth isotopes point to a monsoonal setting, compared with Hanoi and Cuiaba, with cold-season highland runoff (Yamamura et al. 2021). Roberts (2007) is quoted as 'humid, subtropical (150 cm annual precipitation)', seen only in a search snippet: UNCONFIRMED. Flora: 80+ plant morphotypes, mostly angiosperms, with ferns, a lycopod and a horsetail in the understory, and cupressaceous, podocarp and pine conifers (charcoal). A purer fern-and-conifer forest would be Early Cretaceous, but no such formation with comparable climate numbers was found. ESTIMATES (ours): half range about 8 C (coldest month about 12 C, warmest about 28 C), MAP about 1,200-1,600 mm, a wet season of about 6 summer months and a dry season of about 4 winter months (monsoonal analogues); the months themselves are a guess.

| | Value | Basis |
|---|---|---|
| Mean annual temperature | 20 °C (19–21) | leaf-margin analysis (Miller et al. 2013) |
| Half range, summer to winter | 8 °C | estimate |
| Mean annual precipitation | 1400 mm (1200–1600) | estimate |
| Wet season | 6 months (May, Jun, Jul, Aug, Sep, Oct) | estimate |
| Dry season | 4 months (Dec, Jan, Feb, Mar) | estimate |

**Sources:**
- https://pmc.ncbi.nlm.nih.gov/articles/PMC8061582/
- https://pmc.ncbi.nlm.nih.gov/articles/PMC8782542/
- https://www.researchgate.net/publication/284916111_A_Late_Campanian_Flora_from_the_Kaiparowits_Formation_southern_Utah_and_a_brief_overview_of_the_widely_sampled_but_little-known_Campanian_vegetation_of_the_Western_Interior_of_North_America
- https://www.sciencedirect.com/science/article/abs/pii/S0031018218307144
- https://doi.org/10.1144/sp507-2020-52
- https://www.researchgate.net/publication/223755918_Facies_architecture_and_depositional_environments_of_the_Upper_Cretaceous_Kaiparowits_Formation_southern_Utah
- https://en.wikipedia.org/wiki/Kaiparowits_Formation

## `animals:crags` — high rock (an estimate)

**Place:** High rock in the Cretaceous world: the Sevier fold-thrust highlands west of the Utah basins (ESTIMATE). Age —. Palaeolatitude —.

ALL ESTIMATE. No palaeoclimate data for the highlands themselves. Suarez et al. (2014) tie the floodplain's dryness to the rain shadow of the Sevier highlands ('Nevadaplano'), and Yamamura et al. (2021) find cold-season highland runoff reaching the Kaiparowits rivers, so the uplands were real, wetter on their windward side and cooler. Rule used: the standard lapse rate, 6.5 C per km (the game's own temperature::air already applies 0.0065 C per block of altitude). Crags about 1-1.5 km above lowlands at 20-21 C give MAT about 12-14 C; we suggest 13 C at the crags' base, half range about 8 C, MAP about 700-1,000 mm with a summer wet season, frost and some snow on the tops in winter. If the game also lowers the air with the block's altitude, set this at the biome's base height so it is not counted twice.

| | Value | Basis |
|---|---|---|
| Mean annual temperature | 13 °C (11–15) | estimate |
| Half range, summer to winter | 8 °C | estimate |
| Mean annual precipitation | 850 mm (700–1000) | estimate |
| Wet season | 4 months (Jun, Jul, Aug, Sep) | estimate |
| Dry season | 0 months (—) | estimate |

**Sources:**
- https://archives.datapages.com/data/sepm/journals/084/084011/975_i1527-1404-84-11-975.htm

# Today's `climate.rs` against the type localities

| Biome | Today: °C / humidity / seasons | Type locality: year °C / half range | Real day range °C | Verdict |
|---|---|---|---|---|
| plains | 12 / 0.5 / 12 | 12.8 / 14.2 | 12.7 | Close. The seasons should be about 14. |
| forest | 11 / 0.65 / 10 | 9.3 / 8.7 | 8.0 | About 1.7 °C too warm, with slightly too much season. |
| birch_forest | 9 / 0.6 / 12 | 6.3 / 12.9 | 7.5 | About 3 °C too warm (2 °C against the countryside). The seasons are right. |
| taiga | 0 / 0.55 / 18 | −2.1 / 19.8 | 11.4 | Close. 2 °C colder and seasons of 20 would fit Fairbanks. |
| snowy_plains | −10 / 0.4 / 14 | −10.2 / 14.9 | 6.0 | The mean and seasons are right. Humidity 0.4 is too dry: the Arctic coast's day range is the smallest of all. |
| desert | 25 / 0.1 / 9 | 25.7 / 8.1 | 15.6 | The mean and seasons are right. 0.1 is right in spirit (the biggest day range here), but see the humidity note. |
| savanna | 26 / 0.3 / 4 | 27.0 / 1.6 | 9.6 | The mean is right. The seasons are too big: the llanos change by rain, not by temperature, and the warmest month (March) is in the dry season. Humidity 0.3 is too dry: the day range is a moderate 9.6 °C. |
| swamp | 19 / 0.9 / 8 | 25.1 / 3.4 | 12.6 | **6 °C too cold** for the Pantanal, the capybara's swamp (19 °C would be the Okefenokee). Seasons 8 is too big for the tropics. The Pantanal's day range is 12.6 °C, so it is less "humid" than 0.9 suggests. |
| mountains | 6 / 0.5 / 12 | 3.9 at 1,594 m / 8.8 | 9.9 | Davos is 2 °C colder at 1,594 m, but the game already lowers the air 0.65 °C per 100 blocks, so 6 °C at the biome's base is fine. The seasons are too big: mountain air is more oceanic than the plains (8.8). |
| ocean | 14 / 0.8 / 6 | 18.2 / 4.1 | 5.4 | **4 °C colder than the Azores.** That fits a cooler ocean near 45–50° N; keep 14 if the game's sea is temperate. The seasons should be smaller (about 4). |
| beach | 17 / 0.7 / 8 | 18.2 / 6.2 | 8.3 | Close. The seasons are slightly too big. |
| deep_caves | 12 / 0.6 / 0 | the surface year mean, humidity 90–100% | 0 | The temperature is fine for a temperate world. Humidity should be about 0.95. |
| red_dunes | 28 / 0.1 / 6 | ~17 (12–22) est. / ~12 est. | — | **Probably about 10 °C too hot, with too little season.** No proxy supports 28 °C. A 40–45° N continental dune field had hot summers and cold winters. |
| scrubland | 26 / 0.25 / 5 | ~17 est. / ~11 est. | — | Same as the dunes: too hot, too little season. |
| floodplain | 24 / 0.8 / 4 | ~21 (17–25) est. / ~8 est. | — | The mean is plausible. Humidity 0.8 is too wet for the Ruby Ranch's seasonal, calcrete-forming plain (0.5 would fit). Seasons 4 is too small. |
| fern_forest | 23 / 0.85 / 3 | 20 ± 1 (leaf margins) / ~8 est. | — | **3 °C too warm** against the one real proxy. It had a monsoonal dry season: the seasons should be larger. |
| crags | 15 / 0.5 / 6 | ~13 est. / ~8 est. | — | Fine as an estimate, if the game's altitude cooling isn't counted twice. |

**The biggest differences:**
1. The swamp is 6 °C too cold for the Pantanal.
2. The Gobi biomes (red_dunes, scrubland) are likely about 10 °C too hot.
3. The ocean is 4 °C below the Azores.
4. The tropical biomes' `seasons` are too large: the savanna's real half range is 1.6 °C and the Pantanal's
   3.4. Their year turns on rain, which is what the new rain seasons are for.

**Humidity and the day-night swing.** In `temperature::air`, the day swing (half the daily range in the
shade) is `4 + 8 × (1 − humidity)`, so 4 °C at humidity 1 and 12 °C at humidity 0. The real half daily
ranges here run from 2.7 °C (Azores) to 7.8 °C (Hermosillo). Fitting humidity to them would put almost
everything at 0.7–1.0, and even the desert at 0.53. The formula is the problem, not the humidity values:
- Its floor (4 °C at humidity 1) is wider than an island's or the Arctic coast's.
- Its ceiling (12 °C at humidity 0) is wider than a real desert in the shade.

A swing of about `2.5 + 6.5 × (1 − humidity)` fits these places with humidities close to today's:
- the desert at 0.1 gives a 16.7 °C daily range (real: 15.6);
- the Azores at 0.9 gives 6.3 (real: 5.4);
- Kassel at 0.77 gives 8 (real: 8.0).

That is a suggestion for the engine's numbers, not something this research changed.

**Rain, for the new seasons:**
- Rain all year, no dry season: forest, birch forest, plains (drier, cold winter), mountains (summer
  peak), tundra (a polar desert with a late-summer peak).
- A summer wet season and a dry season: savanna (April–November wet, December–March dry), swamp
  (October–March wet in the southern months, June–September dry), desert (July–September monsoon,
  April–June bone dry), taiga (June–September wet, May dry), and probably all the Cretaceous biomes
  (monsoonal).
- A winter wet season and a dry summer: beach (Faro: June–August almost rainless) and ocean (Azores:
  July–August dry).
