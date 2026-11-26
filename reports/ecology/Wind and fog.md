# Wind and fog: the same real places

Research for the weather's air (`mods/base/vitals/src/air.rs`, the numbers in
`vitals::climate::BIOMES`). Each biome keeps the real place of `Biome climates.md`; this adds its mean
wind and its fog days per season, and when in the day its fog lies. Gathered 2026-11-03 from airport
records, the weather services and papers. Several airport figures are tallies of raw hourly
reports, made for this report, not published climatologies; the Cretaceous rows are estimates from
modern stand-in stations.

**How to read it.** Seasons are spring, summer, autumn, winter (Mar–May, Jun–Aug, Sep–Nov, Dec–Feb).
The Pantanal is in the southern hemisphere and is given in its own seasons, as the game shifts it.
Wind is the mean in m/s at about 10 m. A fog day is a day with fog seen at least once: under 1 km for
most sources, under 0.8 km for the US airport reports, the agency's own rule for DWD, IPMA and
Roshydromet. The gust factor is the peak gust over the mean.

| biome → place | wind m/s (Sp / Su / Au / Wi) | prevailing | fog days (Sp / Su / Au / Wi) | when | gusts | in the game |
|---|---|---|---|---|---|---|
| plains → Manhattan, Kansas (KMHK) | 3.8 / 3.0 / 3.0 / 3.1 | S–SSW; N–NNE in winter | 5.3 / 5.8 / 6.3 / 8.4 | radiation fog 04–08 h, gone by mid morning; winter's lasts longer | 1.7 | dawn |
| forest → Kassel (DWD 02532) | 2.5 / 2.0 / 2.4 / 3.0 | SW | 5.8 / 4.9 / 16.1 / 11.3 (1971–2000) | at sunrise, most in autumn; Nov–Feb into the day | ~1.7 (est.) | dawn |
| birch_forest → Moscow (27612) | 1.5 / 1.1 / 1.4 / 1.6 | SW and W; NW and N in summer | ~1.5 / ~2.3 / ~3 / ~1.6 (8 a year) | late night to mid morning (est.) | ~1.7 (est.) | dawn |
| taiga → Fairbanks (PAFA) | 2.3 / 2.0 / 1.5 / 1.1 | N–NE; WSW in summer | 2.0 / 2.7 / 7.3 / 7.8 | 04–10 h in summer and autumn; winter ice fog at any hour | 1.6–1.7 | ice |
| snowy_plains → Utqiaġvik (PABR) | 5.3 / 5.3 / 6.1 / 6.0 | E–ENE | 26.9 / 46.4 / 29.1 / 17.9 | summer sea fog at night and morning; freezing and ice fog through the day | 1.3 | sea |
| desert → Hermosillo (MMHO) | 2.3 / 2.6 / 1.6 / 1.4 | SW–WSW; E in autumn | about 0 (est.) | a few shallow fogs 04–07 h | 1.5–2.0 (est.) | dawn, 0.2 a season |
| savanna → Arauca (SKUC) | 1.5 / 1.2 / 1.2 / 1.6 | E to NE trades | 1.2 / 2.7 / 1.4 / 0.1 (a lower bound) | 06–08 h | 2.5–3 in squalls (est.) | dawn |
| swamp → Nhumirim, the Pantanal (Corumbá, SBCR) | 3.5 / 2.9 / 2.9 / 3.6 | E and NE; SW in cold fronts | Corumbá 1.5 / 1.7 / 1.3 / 1.1; Nhumirim est. 3–6 a season | pre-dawn to mid morning, peak 06–08 h | 1.8–1.9 | dawn, 3 / 3.5 / 5.5 / 5 |
| mountains → Davos (MeteoSwiss) | 2.4 / 2.7 / 2.2 / 1.7 | NE down the valley | est. 1–2 / 1 / 2–3 / 2–3 | night and early morning (est.) | ~1.7 (est.) | dawn |
| ocean → Ponta Delgada (IPMA) | 3.1 / 2.5 / 2.7 / 3.4 | WNW | 1.4 / 0.7 / 0.3 / 0.4 | any hour (est.) | ~2.1 | sea |
| beach → Faro (IPMA) | 4.1 / 3.7 / 3.4 / 3.8 | WSW | 1.0 / 1.6 / 2.1 / 2.3 (1971–95) | late night to early morning (est.) | ~2.0 | dawn |
| red_dunes → Djadokhta, Mongolia | est. 4.5 / 3.5 / 3.3 / 2.8 | NW–W | est. under 1 a season | dawn, if any | est. 2.5–3 | dawn, 0.3 each |
| scrubland → Barun Goyot, Mongolia | est. 3.8 / 3.2 / 2.9 / 2.5 | W–NW | est. 0.5 / 1 / 0.5 / 0 | dawn, over ponds | est. 2.3–2.8 | dawn |
| floodplain → Cedar Mountain, Utah | est. 2.5 / 2.5 / 2.0 / 1.8 | W–SW in storms | est. 2 / 1 / 2 / 5 | dawn | est. ~3 | dawn |
| fern_forest → Yixian, Liaoning | est. 2.8 / 2.0 / 1.8 / 1.8 | NW–SW in spring, S–SE in summer | est. 2 / 4 / 6 / 2 | dawn over the lake | est. ~2.2 | dawn |
| crags → the Sevier highlands, Utah | est. 4.5 / 3.7 / 3.7 / 3.7 | W–SW | est. 5 / 1 / 3 / 6 | early morning, or cloud on the peaks | est. ~2.5 | dawn |

Deep caves have no wind or fog.

## Notes

- **Moscow**: the station reads sheltered (22% calms); open ground nearby averages about 2.5 m/s.
  The game keeps the station's, fitting a birch forest's shelter.
- **Hermosillo**: 0.1–0.3 fog-coded days a season; summer's low-visibility days are monsoon dust and
  rain. The game gives it 0.2–0.3 a season.
- **Arauca**: the airport barely reports between 00 and 05 h, so night fog is missed; a looser count
  gives 2.3 / 4.1 / 2.5 / 0.4.
- **The Pantanal**: no Nhumirim series was found. Corumbá's airport counts include smoke from burning
  (Aug–Oct); Nhumirim, inland among the lagoons, should see more radiation fog on calm humid nights in
  autumn and early winter. The game takes 3 / 3.5 / 5.5 / 5.
- **Davos**: MeteoSwiss's open data has no fog; at 1,594 m valley stratus is mostly cloud, so 5–10 days
  a year is an estimate.
- **Ponta Delgada**: few fog days at sea level; the islands' real hazard is low cloud on the mountains.
- **Faro**: modern reports count about 3.5 a year (2002–2018), lower than the 1971–95 normals the game uses.
- **Gust factors**: none published for Kassel, Davos or Moscow; 1.7 is typical for open, flat or valley
  ground. The game draws gusts in the shader pack rather than from these.
- **The Cretaceous**: wind and fog scaled from modern stand-ins: Dalanzadgad (Mongolia), Canyonlands
  Field (Moab), Chaoyang (Liaoning) and Bryce Canyon (2,310 m), adjusted for a lake in humid forest
  (Yixian, more fog, less wind), inversion mist (Cedar Mountain, winter discounted) and cloud on high
  ground (Sevier, reduced).

## Sources

1. Iowa Environmental Mesonet, archived hourly airport reports 2005–2024 (KMHK, PAFA, PABR, MMHO):
   https://mesonet.agron.iastate.edu/request/download.phtml
2. DWD Kassel (02532), daily data and weather phenomena: https://opendata.dwd.de/climate_environment/CDC/observations_germany/climate/daily/
   ; Kalb & Schirmer (DWD) on fog at Kassel: https://dwdbib.dwd.de/retrosammlung/download/pdf/508238 ;
   direction: https://www.windfinder.com/windstatistics/kassel_calden
3. Roshydromet Moscow (WMO 27612): http://www.pogodaiklimat.ru/climate/27612.htm
4. Fairbanks: https://akclimate.org/city_summaries/fairbanks-december-2011/ ; ice fog, Kumai & O'Brien
   1965: https://apps.dtic.mil/sti/pdfs/AD0676811.pdf
5. NWS Alaska Aviation Weather Unit, MVFR/IFR climatology: https://webarchive.library.unt.edu/web/20130214133036/http://aawu.arh.noaa.gov/misc/papers/MVFR_IFR_Climatology_Report.pdf
6. NCEI global hourly and daily summaries (Arauca 80099099999, Corumbá 83552099999, and the Cretaceous
   stand-ins): https://www.ncei.noaa.gov/access/services/data/v1 ; https://www.climatestotravel.com/climate/colombia/arauca ;
   https://www.climatestotravel.com/climate/brazil/corumbá ; friagem: https://periodicos.ufpe.br/revistas/rbgfe/article/view/242487
7. MeteoSwiss Davos: https://data.geo.admin.ch/ch.meteoschweiz.ogd-smn/dav/ogd-smn_dav_m.csv
8. IPMA normals 1971–2000, Ponta Delgada: https://www.ipma.pt/bin/file.data/climate-normal/cn_71-00_PONTA_DELGADA_SAO_MIGUEL.pdf ;
   Faro: https://www.ipma.pt/bin/file.data/climate-normal/cn_71-00_FARO.pdf ; https://www.windfinder.com/windstatistics/ponta_delgada ;
   https://www.windfinder.com/windstatistics/faro ; Guerreiro et al. 2020, fog at Portuguese airports:
   https://pdfs.semanticscholar.org/a264/6694f7fb47dc0e79fc437f381c5f3101d6b4.pdf
