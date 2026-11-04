# Overworld food webs: our species, every biome's web, and the species the game needs

This is part 0 of the living-world plan (`notes/ECOLOGY.md`, "From real places, like the species")
for the overworld's animals: what the chihuahua, the cat and the capybara really eat and what eats
them; a food web for each overworld biome from its real place; the smallest species list, by the
design's three tiers, that makes every web whole; and the ecological rules of thumb that keep a
simulated web balanced. The machine-readable form is `overworld_web.json` beside this file.

**The short answer.** **Two new full creatures (wolf, jaguar) and fourteen simple ones, and four
of the simple ones can be wild forms of the farm animals.** The fourteen are hare, rodent, small
bird, deer, reindeer, fish, caiman, fox, coyote, eagle, owl, vulture, raven and gull. With five
food sources that are not creatures (insects, grubs, worms, shellfish, eggs), every overworld
biome then has plants, grazers, hunters and something for the dead. Every hunter has prey where it
lives, and every species but the top hunters has a predator. The farm animals fill four roles
(cow as bison, pig as boar or peccary, sheep as wild sheep, chicken as grouse or quail), and this
list assumes they do. The species reuse well: the fox lives in ten biomes, the owl, eagle, rodent
and small bird in all the land biomes, the hare in eight, and the deer in seven.

**How to read the marks.** A sentence with a link is sourced. **(est.)** marks this report's own
estimate or arithmetic. **(ref.)** marks a standard textbook value (typical adult masses, social
systems, which species live where) that was not re-checked against a source in this session.
Masses in the web tables are typical adults (ref.) unless linked.

---

## A. Our three species

### A1. Chihuahua (*Canis familiaris*, 2.5 kg, semi-wild)

The game's chihuahua is semi-wild, so its facts come from two places. Feral and free-ranging dogs
give its feeding *habits*. Small wild canids of its size give its hunting and its ecology: the
kit fox (1.6-2.7 kg) of the Sonoran and Chihuahuan deserts, and the swift fox (2-3 kg) of the
Kansas prairie. Both live in the chihuahua's two commonest biomes.

**Diet.** Free-ranging dogs are scavengers before they are hunters.
- **Human-derived food first.** Most dog populations "are highly dependent on human-derived food
  and gain a relatively small proportion of their diet from wild prey"
  ([Vanak & Gompper 2009, Mammal Review](https://wicri-demo.istex.fr/Wicri/Bois/explor/RenardV1/Site/fr/Main/Exploration/bibRecord.php?hk=000601)).
  In central India, human-derived food (handouts, garbage, crop residue and livestock carcasses)
  made up 83% of the dogs' diet by relative occurrence, and wild-caught food only 11%
  ([Vanak & Gompper 2009, J. Mammal.](https://bioone.org/journals/journal-of-mammalogy/volume-90/issue-5/09-MAMM-A-107.1/Dietary-Niche-Separation-Between-Sympatric-Free-Ranging-Domestic-Dogs-and/10.1644/09-MAMM-A-107.1.full)).
- **Carrion.** In rural Zimbabwe, dogs were "primarily scavengers of human waste and animal
  carcasses". In carcass trials they were the most successful vertebrate scavenger, taking 60% of
  the biomass ([Butler & du Toit 2002](https://cambridge.org/core/journals/animal-conservation-forum/article/diet-of-freeranging-domestic-dogs-canis-familiaris-in-rural-zimbabwe-implications-for-wild-scavengers-on-the-periphery-of-wildlife-reserves/7523A6957C93E6365D03DB46EBC773F0)).
  Radio-collared dogs there were "inefficient predators"
  ([Butler, du Toit & Bingham 2004](https://digitalcommons.usu.edu/wild_facpub/2046)).
- **Wild-canid diet.** The kit fox eats mainly rodents and rabbits: kangaroo rats, prairie dogs,
  cottontails and jackrabbits. It also eats carrion, large insects, lizards, snakes and
  ground-nesting birds, and cactus fruit when food is short
  ([ADW, kit fox](https://animaldiversity.org/accounts/Vulpes_macrotis/)). The swift fox eats
  small mammals, birds, reptiles, amphibians, insects, carrion, eggs, seeds, grain and fruit, and
  the mix "varies seasonally, depending on what is available"
  ([ADW, swift fox](https://animaldiversity.org/accounts/Vulpes_velox/)).
- **For the game (est.):** meat (rodents, young rabbits, birds) 40%, carrion 20%, insects 15%,
  fruit 15%, seeds 10%. `ECOLOGY.md` gives it meat and seeds. Real dogs eat carrion and fruit far
  more than seeds, so adding carrion and fruit is the change that matters.
- **By season (est., from the foxes):**
  - Summer: insects and young animals, at the peak of the grasshoppers and nestlings.
  - Autumn: fruit and berries.
  - Winter: rodents and carrion.
- **By habitat (est.):** in the desert, more kangaroo rats, lizards and cactus fruit. In the
  plains, rodents, cottontails and grasshoppers. In the forest, mice, voles, fruit and carrion.

**Intake (est.).**
- **Food:** about 260 kcal (1.1 MJ) a day. That is the NRC dog requirement of about
  130 kcal × kg^0.75 (ref.), at 2.5 kg. It comes to about 160-170 g of wet prey (about 6.5 kJ/g),
  or 6-7% of body mass a day.
- **Water:** the EPA's mammal equation gives about 0.23 L a day (DW = 0.099 × kg^0.90 L/day;
  [USEPA Wildlife Exposure Factors Handbook](https://www.epa.gov/node/97121)). On prey most of
  that comes with the food. Desert foxes of this size live largely without drinking (ref.).

**Range.** The kit fox's home range is typically about 3.2 km² (2.4-11.6 km²), shared by up to 3
foxes ([ADW](https://animaldiversity.org/accounts/Vulpes_macrotis/)). A truly feral dog pack in the
Apennines ranged much wider, close to a wolf pack's 197 km² there
([Ciucci et al. 1997, wolves](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/50959025);
the dog-pack figure is from Boitani et al., not checked here). **For the game: about 3 km², with
a foraging radius of about 1 km.**

**Activity.** The kit fox is nocturnal and occasionally crepuscular, and dens through the hot
day. The swift fox is nocturnal ([ADW](https://animaldiversity.org/accounts/Vulpes_velox/)).
Free-ranging dogs shift with people and heat (ref.). **For the game: crepuscular and night,
resting at midday in hot biomes.**

**Predators.**
- **Coyotes:** coyotes cause over 75% of kit fox deaths by predation. The rest come from bobcats,
  red foxes, badgers, feral dogs and large raptors
  ([ADW](https://animaldiversity.org/accounts/Vulpes_macrotis/)).
- **Raptors:** great horned owls, red-tailed hawks and goshawks are the raptors that most often
  strike small dogs and cats, mostly ones under 6-7 lb (about 3 kg)
  ([wildlife-damage-management.extension.org](https://wildlife-damage-management.extension.org/?p=754)).
  The 2.5 kg chihuahua is in that class.
- **Large cats and hyenas:** in Zimbabwe, leopards, lions and spotted hyaenas killed at least 6%
  of the dogs in a year ([Butler et al. 2004](https://digitalcommons.usu.edu/wild_facpub/2046)).
- **By biome:**

  | Biome | Its predators |
  |---|---|
  | Plains and desert | Coyote, golden eagle, great horned owl, bobcat (red-tailed hawk rarely) |
  | Forest and birch forest | Wolf, fox, eagle-owl or great horned owl, goshawk, lynx |
  | Savanna | Jaguar, puma, caracara, and a large owl at night |

**Prey sizes.** Kit fox prey runs from a 40 g kangaroo rat to a 2 kg jackrabbit (ref. masses).
That is about 0.01-0.8 of the fox's own mass, mostly below 0.2. This fits the rule that
carnivores under 21.5 kg mostly eat prey under half their own mass, invertebrates included
([Carbone et al. 1999](https://www.nature.com/articles/46266)).

### A2. Cat (*Felis catus* / African wildcat *F. lybica*, 4.5 kg)

**Diet.**
- **Wildcat:** the southern African wildcat "prefers to hunt small rodents". It shifts with prey
  abundance across seasons and years, and also eats other small mammals, birds, reptiles,
  amphibians, insects and other invertebrates. Its largest prey are hares, springhares and birds
  up to guineafowl size. In the Kalahari, males took more large mammals and females more birds and
  reptiles ([Herbst & Mills 2010](https://repository.up.ac.za/handle/2263/16378), via
  [Wikipedia](https://en.wikipedia.org/wiki/Southern_African_wildcat)).
- **Feral cats, worldwide:** of 2,084 prey species recorded, 47% are birds, 22% reptiles, 21%
  mammals, 6% insects and 3% amphibians
  ([Lepczyk et al. 2023](https://www.nature.com/articles/s41467-023-42766-6.pdf)). These are
  shares of species, not of the diet: by mass the diet is mostly rodents and rabbits.
- **Feral cats, by region:** across Australia, reptiles are eaten most in deserts and medium-sized
  mammals in the temperate southeast. Cats switch from rabbits to small mammals where rabbits are
  scarce ([Doherty et al. 2015](https://figshare.utas.edu.au/articles/journal_contribution/A_continental-scale_analysis_of_feral_cat_diet_in_Australia/22952837)).
- **For the game (est.):** meat 80% (rodents about 55, birds 15, reptiles and frogs 10), insects
  10%, carrion 5%, fish 5%.
- **By habitat (est., from those studies):** in the desert and savanna, reptiles and insects up to
  a third. In the forest, more birds in spring and summer. Near water, frogs and fish.

**Intake.**
- **Food (est.):** captive cats need about 153 kJ per kg a day for maintenance
  ([J. Anim. Physiol. Anim. Nutr. 2003](https://onlinelibrary-wiley-com-443.webvpn.zafu.edu.cn/doi/10.1046/j.1439-0396.2003.00431.x)),
  about 690 kJ at 4.5 kg. A hunting cat needs perhaps 1.5-2 times that, about 1.0-1.4 MJ a day.
  That is 160-210 g of wet prey (4-5% of body mass), or 7-10 mice a day.
- **Kills:** a feral cat in the Australian bush kills about 740 vertebrates a year
  ([Charles Darwin University](https://www.cdu.edu.au/news/news-archive/cute-killers-cats-kill-more-15-billion-australian-native-animals-year)):
  390 mammals, 225 reptiles and 130 birds
  ([Smithsonian](https://www.smithsonianmag.com/science-nature/australias-cats-kill-two-billion-animals-annually-180977235/)).
  That is about 2 vertebrates a day, with insects on top.
- **Water:** the EPA equation gives about 0.38 L a day
  ([USEPA](https://www.epa.gov/node/97121)). A wildcat on prey gets most of it from the prey, and
  drinks when there is water (ref.).

**Range.**
- **Wildcats:** in the Kalahari, females' home ranges were 6.1 km² and males' 9.8 km². One wildcat
  in the UAE used 52.7 km²; ranges grow where food is scarce
  ([Mongabay 2025](https://news.mongabay.com/2025/05/tabbys-likely-ancestor-earths-most-widespread-wildcat-is-an-enigma/),
  summarising Herbst's Kalahari work and the UAE study; seen in search results only).
- **Feral cats:** a meta-analysis found males' ranges of 22-3,232 ha and females' of 10-2,078 ha
  ([NZ J. Ecol. 2022](https://newzealandecology.org/nzje/3476)). Pet females in Sweden used
  30-40 ha and rarely went farther than 600 m from home
  ([WikiVet, after Liberg](https://en.wikivet.net/Special:MobileDiff/167216)).
- **For the game:** 0.4 km² near people, up to about 6 km² wild.

**Activity.** Free-ranging cats are nocturnal and crepuscular, with peaks around 21:00 and 05:00
([Appl. Anim. Behav. Sci. 2023, via WikiVet](https://en.wikivet.net/Timing_of_Feline_Activity)).
The wildcat is "largely nocturnal" and rests in cover by day
([Wikipedia](https://en.wikipedia.org/wiki/Southern_African_wildcat)).

**Predators.**
- **Coyotes:** cats were in 20% of urban coyote scats in Los Angeles and 4% of suburban ones, but
  in only 1.3% in Chicago and 4.5% in San Francisco
  ([phys.org 2020](https://phys.org/news/2020-03-urban-coyotes-lots-catsand-human.html); city
  figures via [LMU](https://digitalcommons.lmu.edu/cures_reports/12)).
- **Raptors:** great horned owls, red-tailed hawks and goshawks strike cats under about 3 kg,
  mostly kittens and small cats
  ([extension.org](https://wildlife-damage-management.extension.org/?p=754)).
- **By biome:**

  | Biome | Its predators |
  |---|---|
  | Plains and desert | Coyote, bobcat, golden eagle, great horned owl |
  | Forest, birch forest and taiga | Wolf, lynx, fox (on kittens), eagle-owl |
  | Savanna | Jaguar, puma, large raptors |

**Prey sizes.** Mostly 20-100 g mice and voles. The largest are hares, springhares and
guineafowl ([Wikipedia](https://en.wikipedia.org/wiki/Southern_African_wildcat)), at 1-2 kg
(ref.). That is about 0.005-0.4 of the cat's mass, mostly below 0.05.

### A3. Capybara (*Hydrochoerus hydrochaeris*, 50 kg)

**Diet.** It is a grazer: mainly grasses and aquatic plants, with occasional bark and fruit. It
is coprophagous, re-eating the day's food in the morning
([ADW](https://animaldiversity.org/accounts/Hydrochoerus_hydrochaeris/)).
- **Grasses and sedges:** 80% of the diet comes from five grasses
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/diet)). At Caño Limón (Colombian
  llanos), grasses, sedges, legumes and water hyacinths (Poaceae, Cyperaceae, Leguminosae,
  Pontederiaceae) made up 94% of the diet
  ([Forero-Montaña et al. 2003](https://tropicalstudies.org/rbt/attachments/volumes/vol51-2/29-Forero-Dieta.pdf)).
- **Aquatic plants:** estimates run from 0.6% to 87%, and more reeds and water hyacinths are
  eaten as the wet season advances
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/diet)).
- **The seasonal shift, in a flooded Venezuelan savanna**
  ([Barreto & Herrera 1998](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/52391845)):

  | Season | What it eats |
  |---|---|
  | Wet season | Mostly *Hymenachne amplexicaulis*, a richer grass, eaten selectively; reeds almost ignored |
  | Dry season | Reed beds (*Eleocharis*) and *Paratheria* grass patches in equal time, eaten longer and less choosily |

- **More species by season:** in the rains *Hymenachne*, *Leersia hexandra* and *Panicum laxum*;
  in the dry season *Reimarochloa*, *Paratheria*, *Sporobolus indicus* and *Axonopus*; sedges at
  the end of the rains. Broad-leaved plants are negligible
  ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)).
- **Crops:** it raids fields for grain, maize, melons, squash, bananas, sweet potatoes and manioc
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/diet)).
- **For the game:** grass 75%, water plants 20%, fruit and crops 5% (est.). Water plants could be
  their own category, or a "grass" (reeds, sedges) and "leaves" (water hyacinth) pair.

**Intake.**
- **Food:** about 3 kg of fresh forage a day
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/diet)), 6% of body mass wet.
  FAO's 70 g dry matter per kg^0.75 gives about 1.3 kg dry matter at 50 kg
  ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)), 2.6% of body mass. That is exactly the
  "2-3% dry matter for a grazer" that `ECOLOGY.md` assumes.
- **Water:** no measured figure was found. The EPA equation gives about 3.3 L a day
  ([USEPA](https://www.epa.gov/node/97121)) (est.). Fresh grass brings some of it.

**Range.**
- **Size:** 10-200 ha ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)). The average group uses
  5-20 ha. One study found a mean home range of about 79 ha around a 9 ha defended territory
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)).
- **What a range holds:** every range studied in the llanos held a grass patch, a stretch of
  water and some higher dry land
  ([Herrera & Macdonald 1989, via J. Mammal. 2013](https://bibliotecadigital.exactas.uba.ar/collection/paper/document/paper_00222372_v94_n1_p19_Corriale)).
- **For the game:** about 0.5 km², and it never goes far from water.

**Activity.** It is crepuscular: it rests by the water in the morning and in water or mud through
the heat, and grazes in the late afternoon, the evening and in bouts at night
([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)). Grazing starts after
16:00-17:00, and heavily hunted capybaras become strictly nocturnal
([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)).

**Social life and density.**
- **Groups:** about 10 adults (3-30)
  ([ADW](https://animaldiversity.org/accounts/Hydrochoerus_hydrochaeris/)). Up to 100 gather at
  dry-season water, and up to 40 in the wet season
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)).
- **Density:** 10/km² in the Guárico llanos and peaks of 200-350/km² in the Apure llanos. In the
  Pantanal, 3.2-14.8/km² ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)).

**Predators.**
- **Adults:** jaguars, pumas, anacondas, caimans and feral or stray dogs
  ([ADW](https://animaldiversity.org/accounts/Hydrochoerus_hydrochaeris/);
  [SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)). Felines and stray dogs
  kill the larger animals ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)).
- **Young:** foxes, ocelots, caimans, raptors and anacondas
  ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)). Newborns are taken by
  the crested caracara and the black vulture, by caimans and by dogs
  ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)).
- **Correction to the brief: the harpy eagle is a rainforest eagle**, not a bird of open llanos
  or Pantanal country. The raptors that take young capybaras in our two places are the caracara,
  the savanna hawk and the great black hawk (ref.).
- **In the Pantanal the jaguar eats mostly caiman and fish:** at Taiamã its scats held reptiles
  (mostly caiman) in 55%, fish in 46% and mammals (mostly capybara) in only 11%
  ([Mongabay 2021](https://news.mongabay.com/2021/12/meet-the-fishing-jaguars-that-have-made-this-patch-of-the-pantanal-their-own/)).
  In the southern Pantanal, 31.7% of jaguar kills were cattle, mostly calves, and peccaries were
  the main native prey
  ([Cavalcanti & Gese 2010](https://www.academia.edu/3122316)).

**How it escapes.**
- **Water:** alarmed capybaras jump into the nearest water and can stay under for 5 minutes.
- **Warning calls:** subordinate males give most of the alarm barks.
- **Defence:** against wild dogs the group huddles, with the young in the middle and the adults
  facing out ([SDZWA](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior)).
- **For the game:** a capybara within reach of water flees into it, which works against land
  hunters (jaguar, puma, dogs) but not against the caiman and the anaconda (est.).

### A4. The three side by side

| | Chihuahua | Cat | Capybara |
|---|---|---|---|
| Diet (game shares, est.) | meat 40, carrion 20, insects 15, fruit 15, seeds 10 | meat 80, insects 10, carrion 5, fish 5 | grass 75, water plants 20, fruit 5 |
| Food a day | 160-170 g wet, 6-7% (est.) | 160-210 g wet, 4-5% (est.) | 3 kg fresh, 6% wet, 2.6% dry |
| Water a day | 0.23 L, mostly from food (est.) | 0.38 L, mostly from food (est.) | about 3.3 L (est.) |
| Home range | about 3 km² (kit fox) | 0.4-6 km² | 0.05-2 km², about 0.8 typical |
| Active | dusk and night | dusk and night | late afternoon, evening, night |
| Social | pairs, small packs | alone | groups of about 10 (3-30), up to 100 in the dry season |
| Prey size / own mass | 0.01-0.8 | 0.005-0.4 | - |
| Main predators | coyote, eagle, owl, wolf; jaguar in the savanna | coyote, owl, eagle, lynx, wolf | jaguar, caiman, anaconda; raptors and foxes on the young |

---

## B. A food web for each overworld biome

Each web names 8-15 species a player would notice. Each biome's table lists only the animals;
"eats" pairs are in `overworld_web.json`. The game species each real animal maps to (section C)
is in brackets. How common each is: **C** common, **U** uncommon, **R** rare. Unless linked,
species lists, masses, social systems and abundances are standard natural history (ref.).

### Plains: a Kansas tallgrass prairie (Konza Prairie, Flint Hills)

**Plants.**
- **Grass:** big bluestem, Indiangrass, switchgrass, little bluestem.
- **Forbs:** coneflowers, sunflowers, leadplant (a legume).
- **Fruit and seeds:** wild plum and sumac thickets, and seeds from the grasses and sunflowers.
- **Trees:** gallery oak and hackberry woods along the streams.
- **What the grass carries:** Konza stocks bison at about 11.5 acres (4.7 ha) per animal unit. That
  is set so they remove 25% of the year's above-ground growth, and about 300 bison graze about
  949 ha ([Konza bison fact sheet](https://lter.konza.ksu.edu/outreach/BisonFact17.pdf)).

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Bison [cow as bison] | 450-900 kg | grass | wolves (historically), calves to coyotes | herd | C (managed), R (wild) |
| White-tailed deer [deer] | 60 kg | leaves, forbs, acorns | coyote (fawns), wolf (historically) | small groups | C |
| Eastern cottontail, black-tailed jackrabbit [hare] | 1.2-2.5 kg | grass, forbs | coyote, fox, bobcat, eagle, owl, cat | alone | C |
| Prairie vole, deer mouse [rodent] | 20-45 g | seeds, grass, insects | cat, chihuahua, fox, coyote, owl, hawk, snakes | alone, pairs | C |
| Thirteen-lined ground squirrel [rodent] | 0.15 kg | seeds, insects | hawks, badger, coyote, snakes | alone | C |
| Greater prairie-chicken [chicken as grouse] | 1 kg | seeds, leaves, insects | coyote, fox, hawks, owls | leks, flocks | U |
| Dickcissel, meadowlark [small bird] | 25-100 g | seeds, insects | cat, hawks, snakes | pairs, flocks | C |
| Grasshoppers, dung beetles [insects] | - | grass, dung | birds, rodents, chihuahua, cat | - | C |
| Coyote [coyote] | 13 kg | rodents, rabbits, fawns, fruit, carrion | wolf (historically) | pairs, family packs | C |
| Red fox [fox] | 5 kg | rodents, rabbits, birds, insects, fruit | coyote, eagle | pairs | U |
| Bobcat (later: lynx) | 9 kg | rabbits, rodents | coyote (kits) | alone | U |
| Red-tailed hawk, golden eagle (winter) [eagle] | 1.1-4.5 kg | rodents, rabbits | - | pairs | C |
| Great horned owl [owl] | 1.4 kg | rodents, rabbits, cats rarely | - | pairs | C |
| Turkey vulture [vulture] | 1.8 kg | carrion | - | roosting groups | C (summer) |

**Notes for the game.**
- **Decomposers:** dung beetles bury bison dung, and the fire-grazing cycle keeps the prairie open
  (ref.). In the game, dung and rot are the decomposers (`ECOLOGY.md`, part 8).
- **Wolves:** the plains wolf was gone from Kansas by about 1900. Keep it rare here.

### Forest: temperate deciduous (Białowieża, Poland; eastern US as the variant)

**Plants.**
- **Trees:** oak, hornbeam, lime, maple and ash, with spruce on the poorer ground.
- **Understorey:** hazel, bramble and raspberry.
- **Food from the trees:** acorns and beechmast in mast years, fruit and nuts from the hazel and
  wild apple.
- **Decomposers:** fungi in the leaf litter and dead wood.

**What the forest carries.** From 1798 to 1993 its wild ungulates ranged from under 2 to
14.4 animals a km², weighing 65-1,180 kg a km². Roe deer, wild boar and red deer were usually the
commonest ([Jędrzejewska et al. 1997](https://rcin.org.pl//Content/12705/PDF/BI002_26987_Cz-40-2_Acta-T42-nr36-398-451_o.pdf)).

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Red deer [deer] | 100-200 kg | grass, leaves, browse | wolf (its main prey) | herds | C |
| Roe deer [deer, small variant] | 25 kg | leaves, browse, forbs | lynx (its main prey), wolf, fox (fawns) | alone, small groups | C |
| Wild boar [pig as boar] | 80 kg | roots, mast, grubs, worms, carrion | wolf | groups of sows | C |
| European bison [cow as bison] | 500-800 kg | grass, browse | rarely wolves | herds | R |
| Brown hare [hare] | 4 kg | grass, forbs, bark in winter | fox, lynx, eagle, owl | alone | U |
| Bank vole, wood mouse, red squirrel [rodent] | 20 g-0.3 kg | seeds, mast, fungi, insects | cat, fox, owls, marten, chihuahua | alone | C |
| Jay, great tit, thrushes [small bird] | 20-170 g | seeds, insects, fruit | cat, goshawk, owl, marten | pairs, flocks | C |
| Hazel grouse [chicken as grouse] | 0.4 kg | buds, seeds, berries | fox, goshawk | pairs | U |
| Beetles, grubs, earthworms [tier 3] | - | dead wood, leaf litter | boar, birds, rodents, fox | - | C |
| Wolf [wolf] | 35-45 kg | red deer, boar, roe deer | - | pack of 4-8 | U |
| Eurasian lynx (later) | 20 kg | roe deer, hares | wolf (rarely) | alone | R |
| Red fox [fox] | 6 kg | voles, mice, hares, insects, fruit, carrion | wolf, lynx, eagle-owl | pairs | C |
| Goshawk, buzzard [eagle] | 0.9-1.2 kg | rodents, birds, squirrels | eagle-owl | pairs | U |
| Tawny owl, eagle-owl [owl] | 0.5-2.5 kg | rodents, birds; foxes and cats (eagle-owl) | - | pairs | C |
| Raven [raven] | 1.2 kg | carrion, eggs, small prey | - | pairs, winter roosts | U |

**Notes for the game.**
- **Wolves:** they killed on average 72 red deer, 16 roe deer and 31 wild boar a year per 100 km²,
  at 2.3 wolves per 100 km². They *limited* the deer but did not *regulate* them
  ([Jędrzejewski et al. 2002](https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_proquest_miscellaneous_18403333)).
- **The eastern US variant:** white-tailed deer, wild turkey (the chicken's role), coyote, black
  bear, bobcat and turkey vulture.

### Birch forest: southern Finland and the Moscow region (hemiboreal mixed forest)

**Plants.**
- **Trees:** birch and aspen, with spruce and pine.
- **Undergrowth:** bilberry and lingonberry under the trees, rowan and raspberry in the gaps.
- **Grass:** in the clearings.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Moose (later) | 350-600 kg | browse: birch, aspen, willow, pine twigs | wolf, bear (calves) | alone, cow and calf | C |
| Roe deer [deer] | 25 kg | browse, forbs | lynx, wolf, fox (fawns) | small groups | C |
| Wild boar [pig as boar] | 80 kg | roots, mast, grubs | wolf | groups | U |
| Mountain hare [hare] | 3 kg | grass, birch and willow twigs | fox, lynx, goshawk, eagle-owl | alone | C |
| Red squirrel, bank vole [rodent] | 25 g-0.3 kg | spruce seeds, fungi, berries | marten, goshawk, owls, fox, cat | alone | C |
| Black grouse, hazel grouse, capercaillie [chicken as grouse] | 0.4-4 kg | birch buds, berries, needles | fox, goshawk, marten | leks, flocks | C |
| Fieldfare, tits [small bird] | 20-100 g | insects, berries | cat, hawks, owls | flocks | C |
| Wolf [wolf] | 40 kg | moose, roe deer, boar | - | pack | U |
| Eurasian lynx (later) | 20 kg | hares, roe deer | - | alone | U |
| Red fox [fox], raccoon dog | 5-6 kg | voles, hares, birds, berries, carrion | wolf, lynx, eagle-owl | pairs | C |
| Goshawk [eagle] | 1.1 kg | grouse, squirrels, hares | eagle-owl | pairs | U |
| Ural owl, eagle-owl [owl] | 0.7-2.8 kg | voles, hares, birds | - | pairs | U |
| Raven, hooded crow [raven] | 0.5-1.2 kg | carrion, eggs, small prey | - | pairs, flocks | C |

**Notes for the game.**
- **The moose:** it is the big grazer here and in the taiga. Until it exists, the deer (roe and
  red) and the boar carry the role.

### Taiga: Siberian and Canadian boreal forest

**Plants.**
- **Trees:** spruce, larch, pine and fir, with willow and alder along the water.
- **Ground:** reindeer lichen and mosses, with bilberry, cloudberry and crowberry.
- **Seeds:** cone seeds feed the squirrels.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Moose (later) | 400-600 kg | browse, water plants in summer | wolf, bear | alone | U |
| Woodland caribou, forest reindeer [reindeer] | 110 kg | lichen (winter), sedges, leaves | wolf, bear, wolverine, lynx (calves) | herds | U |
| Wood bison (Canada) [cow as bison] | 600-900 kg | sedges, grass | wolf | herds | R |
| Snowshoe or mountain hare [hare] | 1.4-3 kg | twigs, bark, grass | lynx, fox, owls, goshawk, wolf | alone; numbers cycle about every 10 years | C (cycles) |
| Red-backed vole, red squirrel [rodent] | 25 g-0.3 kg | seeds, berries, fungi | owls, marten, fox, ermine | alone | C |
| Spruce grouse, capercaillie [chicken as grouse] | 0.5-4 kg | needles, buds, berries | fox, goshawk, lynx | flocks in winter | U |
| Beaver (later) | 20 kg | bark, water plants | wolf | family | U |
| Wolf [wolf] | 45-55 kg | moose, caribou, beaver, hares | - | pack | U |
| Canada or Eurasian lynx (later) | 10-20 kg | hares (the Canada lynx eats mostly hares) | wolf | alone | U |
| Brown bear (later) | 150-400 kg | berries, roots, fish, calves, carrion | - | alone | U |
| Red fox [fox] | 5 kg | voles, hares, birds, berries | wolf, lynx, eagle | pairs | C |
| Great grey or great horned owl [owl] | 1-1.4 kg | voles, hares | - | pairs | U |
| Goshawk, golden eagle [eagle] | 1-4.5 kg | grouse, hares, squirrels | - | pairs | U |
| Raven [raven], wolverine | 1.2 / 12 kg | carrion (wolf kills), eggs | - | alone | C / R |

**Notes for the game.**
- **The hare cycle:** Canada lynx kill 0.3-1.2 hares a day, most in the year after the hare peak.
  Coyotes kill most while hares are increasing
  ([O'Donoghue et al. 1998](https://bdu.siu.edu.ar/bdu/Record/todo:paper_00129658_v79_n4_p1193_ODonoghue)).
  At Kluane, hares fell from 14.7 to 0.2 a hectare in one decline
  ([UAlberta thesis](https://era.library.ualberta.ca/files/8910jv725)), and the 1989-90 peak was
  about 450 a square mile, about 1.7 a hectare
  ([Alaska Science Forum](https://www.gi.alaska.edu/alaska-science-forum/when-predators-become-dinner)).
- **The cat:** it is at the edge of its range here (0.2 in the game), as it should be.

### Snowy plains: Arctic tundra

**Plants.**
- **Grass:** sedges, cottongrass and grasses.
- **Shrubs:** dwarf willow and birch.
- **Ground:** mosses and lichens.
- **Fruit:** cloudberry and crowberry in late summer.
- **Growth:** short, all in June to August.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Reindeer or caribou [reindeer] | 80-150 kg | lichen (winter), sedges, willow leaves | wolf, bear (calves), golden eagle (calves) | large herds; they migrate | C (season) |
| Muskox (later) | 250-350 kg | sedges, willow | wolf | herd | U |
| Arctic hare [hare] | 4.5 kg | willow, sedges | Arctic fox, wolf, snowy owl, gyrfalcon | alone or bands | U |
| Lemmings, voles [rodent] | 50-80 g | sedges, mosses, grass | Arctic fox, snowy owl, ermine, skuas | alone; numbers cycle every 3-5 years | C (cycles) |
| Rock or willow ptarmigan [chicken as grouse] | 0.5 kg | willow buds, berries | gyrfalcon, Arctic fox | flocks | C |
| Snow bunting, Lapland longspur [small bird] | 30 g | seeds, insects | fox, skua, falcon | flocks | C (summer) |
| Arctic fox [fox, white variant] | 3.5 kg | lemmings, eggs, birds, carrion | wolf, red fox, golden eagle | pairs | C |
| Wolf [wolf] | 45 kg | caribou, muskox, hares | - | pack | R |
| Snowy owl [owl] | 2 kg | lemmings (3-5 a day, about 1,600 a year) | Arctic fox (eggs) | pairs | U (cycles) |
| Gyrfalcon, golden eagle [eagle] | 1.5-4.5 kg | ptarmigan, hares | - | pairs | R |
| Raven [raven] | 1.2 kg | carrion, eggs | - | pairs | U |
| Mosquitoes, flies [insects] | - | blood, nectar | birds | swarms | C (summer) |

**Notes for the game.**
- **Who eats the lemmings:** a snowy owl eats about 1,600 lemmings a year, 3-5 a day
  ([National Geographic](https://www.nationalgeographic.com/animals/birds/facts/snowy-owl)).
  Lemmings drive Arctic fox breeding
  ([J. Mammal. 2021](https://openpolar.no/Record/ftbioone:10.1093%2Fjmammal%2Fgyab115)), and a
  High Arctic web model found it mostly controlled from the top down
  ([food web model](https://openpolar.no/Record/ftdatacite:10.6084%2Fm9.figshare.c.3304824.v1)).
- **Not here:** neither the chihuahua nor the cat lives here, rightly.

### Desert: Sonoran (Arizona and Sonora)

**Plants.**
- **Cactus:** saguaro, prickly pear and cholla, which give fruit and pads.
- **Trees:** mesquite (its pods are seeds) and palo verde.
- **Shrubs:** creosote bush and bursage.
- **Annuals:** grasses and wildflowers after the winter and summer rains.
- **Seasons:** two seasons of growth, the winter rains and the July to September monsoon.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Desert mule deer [deer] | 60-90 kg | browse, cactus fruit, annuals | puma, coyote (fawns), jaguar (rare) | small groups | U |
| Javelina (collared peccary) [pig as peccary] | 20-25 kg | prickly pear, roots, mesquite pods | puma, coyote, jaguar, bobcat (young) | herds of 6-15 | C |
| Desert bighorn sheep [sheep as wild sheep] | 50-90 kg | grass, browse | puma, golden eagle (lambs) | bands | R |
| Black-tailed jackrabbit, desert cottontail [hare] | 1-2.5 kg | grass, forbs, cactus | coyote, bobcat, hawks, owls, rattlesnake | alone | C |
| Merriam's kangaroo rat, ground squirrels [rodent] | 40 g-0.15 kg | seeds; it needs no drinking water | kit fox, owls, rattlesnake, cat, chihuahua | alone | C |
| Gambel's quail [chicken as quail] | 0.17 kg | seeds, leaves | coyote, hawks, bobcat, cat | coveys | C |
| Cactus wren, doves [small bird] | 40-150 g | insects, seeds, cactus fruit | hawks, cat, snakes | pairs | C |
| Harvester ants, darkling beetles [insects] | - | seeds, debris | lizards, birds, foxes, chihuahua | - | C |
| Coyote [coyote] | 11 kg | rabbits, rodents, fruit, carrion, fawns | puma | pairs, families | C |
| Kit fox, gray fox [fox] | 2-4 kg | kangaroo rats, rabbits, insects | coyote (over 75% of kit fox predation), eagle | pairs | U |
| Bobcat, puma (later) | 9 / 55 kg | rabbits / deer, javelina | - | alone | U / R |
| Western diamondback, lizards (later) | 0.05-2 kg | rodents / insects | hawks, roadrunner, coyote | alone | C |
| Harris's hawk, red-tailed hawk [eagle] | 0.7-1.1 kg | rabbits, rodents; Harris's hawks hunt in groups | - | groups / pairs | C |
| Great horned owl [owl] | 1.4 kg | rodents, rabbits | - | pairs | C |
| Turkey vulture [vulture], raven [raven] | 1.2-1.8 kg | carrion | - | groups | C |

Sources for the species: [Arizona-Sonora Desert Museum fact sheets](https://www.desertmuseum.org/kids/oz/long-fact-sheets/index.php?print=y),
[ASDM predators and prey](https://www.desertmuseum.org/center/edu/docs/3-4_Hunters_predators.pdf),
the [NPS Sonoran handbook](https://www.nps.gov/parkhistory/online_books/natural/4a/nh4ah1.htm)
(mule deer on cactus fruit and annuals in winter), and
[ADW, kit fox](https://animaldiversity.org/accounts/Vulpes_macrotis/).

**Notes for the game.** A small rodent that never drinks (the kangaroo rat) and a fox that barely
does both live here. The forage goal's "water" target should be optional for desert species.

### Savanna: the Venezuelan llanos

**Plants.**
- **Grass:** tall *Trachypogon* and *Paspalum*, and *Axonopus* on the higher, drier savanna. In
  the flooded lows (bajíos and esteros), *Hymenachne*, *Leersia* and *Paspalum fasciculatum*.
- **Water plants:** water hyacinth and sedges.
- **Trees and palms:** moriche and llanera palms, with gallery forest along the rivers, which
  gives fruit.
- **Seasons:** the wet season is May to October and the dry season November to April.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Capybara [capybara] | 50 kg | grass, water plants | jaguar, puma, caiman, anaconda; caracara, vultures and dogs on the young | groups of 10, up to 100 in the dry season | C |
| White-tailed deer (llanos form) [deer] | 35-45 kg | forbs, browse, fruit | jaguar, puma | small groups | U |
| Collared peccary [pig as peccary], feral pig | 20-25 kg | roots, fruit, seeds | jaguar, puma | herds | U |
| Giant anteater (later) | 30 kg | termites, ants | jaguar, puma | alone | U |
| Florida cottontail [hare] | 1.2 kg | grass, forbs | fox, ocelot, owls, hawks, cat | alone | U |
| Cane mice, cotton rats [rodent] | 30-150 g | seeds, grass, insects | barn owl, foxes, hawks, cat, chihuahua, snakes | alone | C (after the rains) |
| Crested bobwhite [chicken as quail] | 0.15 kg | seeds, insects | foxes, hawks, cat | coveys | C |
| Seedeaters, flycatchers [small bird] | 10-50 g | seeds, insects | hawks, cat | flocks | C |
| Termites, grasshoppers [insects] | - | grass, dead wood | anteater, birds, foxes, chihuahua, cat | - | C |
| Spectacled caiman [caiman] | 25-40 kg | fish, birds, capybara young | jaguar, anaconda (young caiman) | many at dry-season pools | C |
| Jaguar [jaguar], puma (later) | 55-90 / 50 kg | capybara, peccary, deer, caiman, cattle | - | alone | R |
| Crab-eating fox [fox] | 6 kg | rodents, insects, fruit, crabs, frogs, carrion | jaguar, puma | pairs | C |
| Savanna hawk, crested caracara [eagle] | 0.9-1.3 kg | rodents, reptiles, carrion, newborns | - | pairs | C |
| Barn owl, great horned owl [owl] | 0.5-1.4 kg | rodents | - | pairs | U |
| Black vulture, turkey vulture [vulture] | 1.5-2 kg | carrion, capybara newborns | - | flocks | C |

**Notes for the game.**
- **The dry season:** the dry season packs capybaras, caimans and wading birds at the shrinking
  pools. That is the "waterholes" idea in `ECOLOGY.md`, and it is what really happens here.
- **Rodents:** they boom after the rains, so the cat's and the chihuahua's food here is seasonal.
- **Sources for the place:** the llanos' wildlife is best protected on private ranches such as
  Hato Piñero and Hato Masaguaral
  ([Eisenberg & Polisar 1999](https://flmnhbulletin.com/index.php/flmnh/article/view/flmnh-vol42-no3)).
  Capybaras are the dominant grazers of flooded savanna, judged by their effect on pasture height,
  biomass and quality ([Barreto & Herrera 1998](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/52391845)).

### Swamp: the Pantanal

**Plants.**
- **Water plants:** water hyacinth, *Salvinia* and water lilies.
- **Grass:** floodplain grasses and sedges.
- **Trees:** acuri and carandá palms, with gallery and island forest that gives fruit.
- **Seasons:** most of it floods from December to May.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Capybara [capybara] | 50 kg | grass, water plants | jaguar, caiman, anaconda | groups | C (3-15 a km²) |
| Marsh deer [deer, marsh variant] | 100 kg | water plants, grass | jaguar | alone, pairs | U |
| White-lipped peccary, feral pig [pig as peccary] | 30-80 kg | fruit, roots | jaguar | big herds / groups | C |
| South American tapir (later) | 250 kg | leaves, fruit, water plants | jaguar | alone | U |
| Rodents (marsh rats) [rodent] | 50-200 g | seeds, grass | foxes, owls, hawks, snakes | alone | C |
| Fish: piranha, pacu, catfish [fish] | 0.1-5 kg | insects, fruit, fish, detritus | caiman, jaguar, herons, giant otter | schools | C |
| Frogs (later) [frog] | 20-200 g | insects | caiman, herons, snakes | - | C |
| Yacare caiman [caiman] | 25-58 kg | fish, snails, birds, young mammals | jaguar | many at pools | C |
| Green anaconda (later) | 30-100 kg | capybara, caiman, birds | jaguar (rare) | alone | U |
| Jaguar [jaguar] | 90 kg (big here) | caiman, fish, capybara, peccary, cattle | - | alone (about 12 per 100 km² at Taiamã) | U |
| Giant otter (later) | 25 kg | fish | - | family | U |
| Crab-eating fox [fox] | 6 kg | crabs, frogs, rodents, fruit | jaguar | pairs | C |
| Jabiru, herons (later: heron) | 1-8 kg | fish, frogs, snails | - | colonies | C |
| Great black hawk, snail kite [eagle] | 0.4-1.2 kg | fish, frogs, snails, rodents | - | pairs | C |
| Black vulture [vulture] | 2 kg | carrion, dead fish | - | flocks | C |

**Notes for the game.**
- **Jaguar diet:** reptiles (mostly caiman) were in 55% of jaguar scats, fish in 46% and mammals
  in 11%. The density was 12.4 jaguars per 100 km², two to three times that elsewhere
  ([Mongabay](https://news.mongabay.com/2021/12/meet-the-fishing-jaguars-that-have-made-this-patch-of-the-pantanal-their-own/);
  [EurekAlert](https://www.eurekalert.org/news-releases/931638)).
- **Seasons:** rising water spreads the caimans across the land and raises the jaguar's kills of
  them ([Cavalcanti & Gese 2010](https://www.academia.edu/3122316)).
- **For the game:** the jaguar should take caiman and fish here, not only capybara.

### Mountains: the Alps (the Rockies as the variant)

**Plants.**
- **Grass:** alpine meadows of grasses, sedges and flowers, all in a short summer.
- **Shrubs:** dwarf shrubs such as alpenrose and bilberry.
- **Trees and lichen:** larch and spruce below the tree line, and lichen on the rock.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Alpine ibex, chamois [sheep as wild sheep] | 30-100 kg | grass, herbs, lichen in winter | wolf, lynx, golden eagle (kids) | herds, bachelor groups | C |
| Red deer [deer] | 100-200 kg | grass, browse | wolf | herds | U |
| Alpine marmot (later; meanwhile [rodent]) | 4-6 kg | grass, herbs | golden eagle (its main prey), fox | colonies | C (summer) |
| Snow voles [rodent] | 40 g | grass | owls, fox, ermine, kestrel | alone | C |
| Mountain hare [hare] | 3 kg | grass, twigs | golden eagle, fox, lynx | alone | U |
| Rock ptarmigan, black grouse [chicken as grouse] | 0.5-1.2 kg | buds, berries | golden eagle, fox | flocks | U |
| Alpine chough, snowfinch [small bird] | 30-200 g | insects, berries, scraps | falcons | flocks | C |
| Wolf [wolf] | 30-40 kg | chamois, red deer, ibex | - | pack | R |
| Eurasian lynx (later) | 20 kg | chamois, roe deer | - | alone | R |
| Red fox [fox] | 6 kg | voles, marmots, carrion | eagle, wolf, lynx | pairs | C |
| Golden eagle [eagle] | 4-6 kg | marmots, hares, kids, ptarmigan | - | pairs | U |
| Eagle-owl [owl] | 2.5 kg | voles, hares, birds | - | pairs | R |
| Bearded vulture [vulture], raven [raven] | 6 / 1.2 kg | bones and carrion / carrion | - | pairs | R / C |

**Notes for the game.**
- **The Rockies variant:** elk, bighorn sheep, mountain goat, yellow-bellied marmot, pika, puma,
  coyote, wolverine and grizzly bear.

### Ocean and beach: a temperate coast (North Sea and North Atlantic)

**Plants.**
- **In the water:** phytoplankton (most of the production), kelp and wrack seaweeds on rocks, and
  eelgrass in sheltered water.
- **On land:** marram and lyme grass on the dunes, and sea beet and sea kale on the strandline.

| Animal | Mass | Diet | Eaten by | Social | How common |
|---|---|---|---|---|---|
| Zooplankton (copepods) | - | phytoplankton | fish, mussels | swarms | C |
| Mussels, cockles, lugworms [shellfish] | - | plankton, detritus | gulls, oystercatchers, crabs, fish | beds | C |
| Shore crabs, sandhoppers [shellfish, insects] | - | carrion, detritus, seaweed | gulls, fish, foxes | - | C |
| Herring, sand eel, cod, flatfish [fish] | 0.05-10 kg | plankton, worms, fish | seals, gulls, terns, cod | schools | C |
| Squid (later; or a fish variant) | 0.1-1 kg | fish, crustaceans | seals, cod | schools | U |
| Harbour seal (later) | 80-130 kg | fish | orca (rare) | haul-out groups | U |
| Herring gull [gull] | 1 kg | fish, shellfish, carrion, eggs, scraps | fox (eggs) | colonies | C |
| Oystercatcher, terns [small bird / gull] | 0.1-0.5 kg | shellfish, sand eels | fox, gull (eggs) | colonies | C |
| Red fox [fox] | 6 kg | rabbits in the dunes, eggs, strandline carrion | - | pairs | U |
| Rabbit [hare], mice [rodent] | 0.02-1.5 kg | dune grass | fox, eagle, owl | colonies | C |
| White-tailed eagle [eagle] | 4-6 kg | fish, seabirds, carrion | - | pairs | R |
| Raven, crows [raven] | 0.5-1.2 kg | strandline carrion, shellfish | - | pairs | C |

**Notes for the game.** The tide line is the beach's carrion: dead fish and stranded animals. Gulls,
crabs, ravens and foxes clean it up.

---

## C. The species list the game needs

### C1. How the list was chosen

1. **Start from the webs (B).** Map each real animal to the fewest game species that do its job
   (same size class, same diet, same predators).
2. **Every biome needs every level:** plants, a grazer, a hunter, and something for the dead.
3. **Every hunter needs prey where it lives**, and every species but the top hunters needs a
   predator somewhere in its habitats.
4. **Our three species need their real predators and prey.** The chihuahua needs rodents, rabbits,
   insects and carrion, and is hunted by the coyote, eagle and owl. The cat needs rodents, birds
   and insects, and is hunted by the same three. The capybara needs grass and water plants, and is
   hunted by the jaguar and the caiman.
5. **Reuse one species across biomes** with local looks, as the game already does with habitats
   and the climate rules (Bergmann, Allen, Gloger): white hares and foxes in the snow, a small
   roe-type deer in the forests.

### C2. Tier 1: full creatures (modelled, tameable)

| Species | Biomes | Mass | Diet | Predators | Prey | Why it's needed |
|---|---|---|---|---|---|---|
| Chihuahua (built) | plains, savanna, desert, forest, birch forest | 2.5 kg | meat, carrion, insects, fruit, seeds | coyote, eagle, owl, wolf, jaguar | rodents, young hares, eggs, insects | ours |
| Cat (built) | plains, forests, savanna, desert, taiga | 4.5 kg | meat, insects, fish | coyote, owl, eagle, wolf, fox (kittens) | rodents, small birds, lizards, frogs, insects | ours |
| Capybara (built) | swamp, savanna, forest | 50 kg | grass, water plants | jaguar, caiman, anaconda; eagle, fox and vulture on the young | - | ours |
| **Wolf** (new) | forest, birch forest, taiga, snowy plains, mountains, plains (rare) | 35-55 kg | meat (deer, boar, reindeer, bison calves, hares), carrion | none (the top) | deer, boar, reindeer, wild sheep, hare, beaver | The top hunter of five cold and temperate biomes. Without it nothing eats the deer, reindeer and boar there. The dog's wild ancestor makes it a natural tame. |
| **Jaguar** (new) | swamp, savanna, desert (rare) | 55-100 kg | meat (capybara, caiman, peccary, deer), fish | none (the top) | capybara, caiman, boar/peccary, deer, fish | The capybara's and the caiman's predator; the top of both tropical webs |

**The farm animals and their wild roles.** The farm animals are full creatures already. Each has a
wild counterpart that fills a role in the webs, and making that counterpart a wild form of the
existing model (its own look, wild genes, wary of players) avoids four new species.

| Farm animal | Wild counterpart in our places | Role and biomes |
|---|---|---|
| Cow | Plains bison (Kansas), European bison (Białowieża), wood bison (Canada); the cow's own ancestor, the aurochs, lived in the European forests and plains until 1627 | The big grazer of the plains; rare in the forest and taiga |
| Pig | Wild boar (forests), collared peccary (desert, llanos), white-lipped peccary and feral pigs (Pantanal). Peccaries are not pigs but fill the same role | The omnivore that roots, eats mast and carrion; main prey of wolves and jaguars. Six biomes |
| Sheep | Mouflon (European mountains), Alpine ibex and chamois, bighorn and desert bighorn (Rockies, Sonora) | The mountain grazer, and rare in the desert |
| Chicken | Red junglefowl, its ancestor, lives in South and Southeast Asian forests, in none of our places. The local galliform takes the role: prairie-chicken (plains), wild turkey or hazel grouse (forest), black grouse and capercaillie (birch forest, taiga), ptarmigan (tundra, mountains), Gambel's quail (desert), crested bobwhite (llanos) | The ground bird of nine biomes; prey of foxes, coyotes, hawks and cats; lays eggs (tier 3 food) |

### C3. Tier 2: simple creatures (one model, few animations; they live, flee, breed, get eaten)

Some tier 2 creatures hunt (the fox, coyote, eagle, owl and caiman). They use the same forage goal
with prey as the target, so "simple" means few animations, not no behaviour.

**The core set: these fourteen make every web whole.**

| Species | Biomes | Mass | Diet | Predators | Prey | Why it's needed |
|---|---|---|---|---|---|---|
| **Hare** (cottontail, jackrabbit, brown, mountain, snowshoe, Arctic hare; white in the snow) | plains, forest, birch forest, taiga, snowy plains, mountains, desert, savanna, beach (rabbit) | 1.2-4.5 kg | grass, leaves, bark | coyote, fox, wolf, eagle, owl, jaguar; young to the cat and chihuahua | - | The mid-sized prey everywhere: the coyote's, fox's, eagle's and lynx's main food |
| **Rodent** (mice, voles, kangaroo rat, lemming, cotton rat, squirrel as a variant) | every land biome | 0.02-0.3 kg | seeds, grass, insects, fungi | cat, chihuahua, fox, owl, eagle (hawks), coyote | - | **Our two hunters' main food.** It fills the cat's 55% and the chihuahua's meat; the tundra's lemmings drive the fox and the owl |
| **Small bird** (sparrows, tits, wrens, buntings) | every land biome | 0.01-0.2 kg | seeds, insects, fruit | cat, eagle (hawks), owl, fox | insects (tier 3) | The cat's bird prey (15%), and a seed and insect eater in every biome |
| **Deer** (white-tailed, roe, red, mule, marsh deer; size by variant) | plains, forest, birch forest, savanna, swamp, mountains, desert | 25-200 kg | leaves, grass, fruit | wolf, jaguar, coyote (fawns) | - | The big grazer and browser of seven biomes, and the wolf's and jaguar's prey |
| **Reindeer** (caribou) | snowy plains, taiga | 80-150 kg | lichen, grass, leaves | wolf | - | The only big grazer the tundra has, and the taiga's. It eats lichen, so lichen is a food |
| **Fish** (local looks; also a catch, below) | ocean, swamp, beach, every river and lake | 0.1-5 kg | insects, plankton, fish, fruit | caiman, jaguar, gull, cat, eagle (sea eagle), heron | insects | The middle of both water webs; the caiman's and the gull's food |
| **Caiman** | swamp, savanna | 25-58 kg | fish, birds, capybara young | jaguar | fish, rodents, small birds, young capybaras | The water's ambush hunter; makes the capybara's "flee into water" a real choice |
| **Fox** (red, kit, gray, Arctic, crab-eating) | plains, forest, birch forest, taiga, snowy plains, mountains, desert, savanna, swamp, beach | 2-6 kg | meat, insects, fruit, carrion | coyote, wolf, eagle, jaguar | rodents, hares, small birds, eggs, insects | The small hunter of ten biomes and the chihuahua's and cat's rival. The tundra's main hunter |
| **Coyote** | plains, desert, forest, mountains | 9-16 kg | meat (rabbits, rodents, fawns), fruit, carrion | wolf, jaguar (rare) | hares, rodents, foxes, cats, chihuahuas, deer fawns | The main predator of small dogs and cats in their commonest biomes. Hunts alone or in a family |
| **Eagle** (golden eagle; red-tailed, Harris's and savanna hawks, goshawk, caracara, sea eagle as variants by size) | every land biome | 1-6 kg | meat | none (the top) | rodents, hares, small birds, grouse, young deer and sheep, chihuahuas and kittens (the eagle), fish (the sea eagle) | The daytime raptor. It makes open ground dangerous for small animals |
| **Owl** (great horned, eagle-owl, snowy, barn owl) | every land biome | 0.5-2.8 kg | meat | none (the top) | rodents, hares, small birds; cats and chihuahuas rarely | The night raptor. Our hunters are nocturnal, so it is the predator they meet |
| **Vulture** (turkey, black, bearded vulture) | savanna, swamp, desert, plains, forest (US variant), mountains | 1.5-6 kg | carrion | none | - | Something for the dead in the warm biomes |
| **Raven** (raven, crows) | forest, birch forest, taiga, snowy plains, mountains, desert, plains, beach | 0.5-1.2 kg | carrion, eggs, insects, scraps | eagle-owl, goshawk | eggs, small rodents | Something for the dead in the cold biomes, where there are no vultures. It follows wolves |
| **Gull** | beach, ocean | 1 kg | fish, shellfish, carrion, eggs | fox (eggs) | fish, shellfish | The coast's hunter and its carrion eater |

The vulture and the raven do the same job. If one model must do it, a "carrion bird" with a
vulture look in warm biomes and a raven look in cold ones would do (est.).

**Later: these add realism, but no web needs them to be whole.**

| Species | Biomes | Mass | Diet | Why |
|---|---|---|---|---|
| Lynx (Eurasian, Canada, bobcat) | forest, birch forest, taiga, mountains, plains, desert | 9-25 kg | hares, roe deer | The hare-and-roe specialist, and the cat's natural enemy. Eurasian lynx took 21-36% of spring roe deer in Białowieża |
| Puma | desert, mountains, savanna, swamp | 40-70 kg | deer, peccary, capybara | The desert's and the Rockies' top hunter |
| Brown bear | taiga, birch forest, mountains, forest | 150-400 kg | berries, roots, fish, calves, carrion | A big omnivore, and a scavenger of wolf kills |
| Moose | taiga, birch forest | 350-600 kg | browse, water plants | The boreal forest's big browser |
| Frog | swamp, forest, birch forest, plains, savanna (wet season) | 0.02-0.5 kg | insects | Insect eaters of wet ground; food for the caiman, heron, fox and cat |
| Lizard | desert, savanna | 0.01-0.3 kg | insects | The desert cat's reptile prey (up to a third of the diet in deserts) |
| Snake (rattlesnake, anaconda) | desert, plains, swamp, savanna | 0.5-100 kg | rodents / capybara, caiman | The anaconda is the capybara's third predator |
| Heron (jabiru, egrets) | swamp, savanna, beach | 1-8 kg | fish, frogs | The swamp's fish eater on land |
| Seal | ocean, beach | 80-130 kg | fish | The top hunter of the coast |

### C4. Tier 3: food sources in the world, not creatures

| Food source | Category | Where (biomes) | Found in | Eaten by | Why |
|---|---|---|---|---|---|
| **Insects** (grasshoppers, beetles, ants, termites; sandhoppers on the beach) | insects | every land biome; most in summer and the rains | tall grass, sand, termite mounds (savanna) | chihuahua, cat, fox, small bird, rodent, frog, lizard | 10-15% of our hunters' diets; Carbone's rule says small hunters live partly on them |
| **Grubs** (beetle larvae in rotten wood) | insects | forest, birch forest, taiga | logs and stumps | boar, fox, small bird, bear, chihuahua | `ECOLOGY.md`'s "beetles under logs"; a winter food |
| **Worms** (earthworms) | insects (or their own) | plains, forest, birch forest, swamp; after rain | wet soil | boar, fox, small bird, chihuahua | Food that follows the soil-water number |
| **Shellfish** (mussels, crabs, cockles) | fish | beach, ocean shallows, swamp (snails, crabs) | sand, rock, mud | gull, fox, raven, crab-eating fox | The coast's middle level, and its carrion cleaners |
| **Eggs** (in ground and tree nests, in spring) | meat | every land biome | nests by the grouse, small birds, gulls, caimans | fox, raven, chihuahua, coyote, gull | The game has eggs already; a big seasonal food for foxes and corvids |
| **Fish as a catch** | fish | any water | water blocks | cat, jaguar, fox, eagle | Fallback where no fish creatures are loaded |
| **Carrion** | carrion | wherever something died | carcasses (part 6) | vulture, raven, fox, coyote, wolf, chihuahua, boar, gull | Built into parts 6 and 8 |

**Plants as food** (the plant table, `vitals::plants`):
- **Grass:** in every land biome.
- **Leaves and browse:** in the forests, savanna trees and desert shrubs.
- **Fruit:** berries in the forests and taiga, cactus fruit in the desert, palm fruit in the
  savanna and swamp.
- **Seeds and mast:** acorns and beechmast in the forest, mesquite pods in the desert, grass seed
  in the plains and savanna.
- **Roots:** in every land biome.
- **Water plants:** water hyacinth and reeds in the swamp and savanna.
- **Lichen:** in the tundra and taiga, for the reindeer.
- **Fungi:** in the forests, for the boar, the rodent and the reindeer.

### C5. Who lives where (the core set)

| | plains | forest | birch | taiga | snowy | desert | savanna | swamp | mountains | ocean | beach |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **Plants** | grass, seeds, forbs | leaves, mast, fruit, fungi | browse, berries | browse, lichen, berries | sedges, lichen | cactus, pods, annuals | grass, water plants, palms | water plants, grass, palms | alpine grass, lichen | phytoplankton, kelp | seaweed, dune grass |
| **Grazers** | bison (cow), deer, hare, rodent, grouse | deer, boar, bison (rare), hare, rodent, grouse | deer, boar, hare, rodent, grouse | reindeer, boar, hare, rodent, grouse | reindeer, hare, rodent, ptarmigan | deer, peccary, wild sheep, hare, rodent, quail | **capybara**, deer, peccary, hare, rodent, quail | **capybara**, deer, peccary, rodent, fish | wild sheep, deer, hare, rodent, ptarmigan | fish, shellfish | shellfish, hare, rodent |
| **Small hunters** | **chihuahua**, **cat**, fox | **chihuahua**, **cat**, fox | **chihuahua**, **cat**, fox | **cat**, fox | fox | **chihuahua**, **cat**, fox | **chihuahua**, **cat**, fox, caiman | fox, caiman | fox | gull | gull, fox |
| **Top hunters** | coyote, eagle, owl, (wolf) | wolf, coyote, eagle, owl | wolf, eagle, owl | wolf, eagle, owl | wolf, owl, eagle | coyote, eagle, owl, (jaguar) | jaguar, eagle, owl | jaguar, eagle, owl | wolf, coyote, eagle, owl | gull | eagle |
| **For the dead** | vulture, raven | raven, vulture (US) | raven | raven | raven | vulture, raven | vulture | vulture | raven, vulture | shellfish, gull | gull, raven |
| **Tier 3** | insects, worms, eggs | grubs, worms, eggs | grubs, worms, eggs | grubs, eggs | insects, eggs | insects, eggs | insects (termites), eggs | worms, shellfish (snails), eggs | insects, eggs | shellfish, fish catch | shellfish, insects, eggs |

Every column has all four levels. Among the core set, every species but the eagle, owl, wolf,
jaguar, vulture and gull has a predator somewhere in its habitats.

---

## D. Ecological rules of thumb (with numbers)

### D1. Predator to prey: about 1 kg of a hunter per 100 kg of its prey

- **The rule:** "10,000 kg of prey supports about 90 kg of a given species of carnivore",
  whatever the carnivore's size. That is about 111 kg of prey for each kg of carnivore. It held
  for 25 carnivore species across all diets and habitats
  ([Carbone & Gittleman 2002, via Innovations Report](https://innovations-report.com/life-sciences/report-8728)).
- **Number versus mass:** the number of carnivores per prey biomass scales as 1/mass, so the
  biomass ratio stays constant ([same](https://synapsesocial.com/papers/W2014666871)).
- **For the game (est.):** a place's capacity for a hunter is 0.9% of its prey's biomass,
  divided by the hunter's mass. For example, 1,000 kg of deer and boar in a forest feeds about
  9 kg of wolf, so about 0.2 wolves. The game should round that to "a pack passes through
  sometimes", not "no wolves".
- **Several hunters on one prey:** this ratio is per carnivore species. Where several share the
  prey (wolf and lynx on deer, fox, owl and cat on rodents), each gets a share (est.).

### D2. What a hunter takes: prey size

- **Small hunters take small prey.** Carnivores under 21.5 kg mostly eat prey under half their own
  mass, much of it invertebrates. Above that size they switch to prey near their own mass,
  because an invertebrate diet cannot sustain a body over about 21.5 kg
  ([Carbone et al. 1999](https://www.nature.com/articles/46266)). Invertebrate feeders' intake
  rates are about a tenth of vertebrate feeders'.
- **The design matches.** `ECOLOGY.md` part 6 says "a twentieth of its mass up to its own; a pack
  up to three times". For small hunters (the chihuahua, cat, fox and owl) the real upper end is
  about half their mass. Wolves in packs take elk and moose at 4-10 times one wolf's mass (ref.),
  so "three times" is conservative but fine.

  | Hunter | Prey mass / own mass (ref., est.) |
  |---|---|
  | Cat | 0.005-0.4, mostly below 0.05 |
  | Chihuahua and fox | 0.01-0.8, mostly below 0.2 |
  | Coyote | 0.01-0.3 alone, up to 3-5 in a pack on deer |
  | Owl and eagle | 0.02-1.5 |
  | Wolf | 0.05 (hares) up to 10 (moose, in a pack) |
  | Jaguar | 0.05 (fish) up to about 3 (cattle) |
  | Caiman | 0.01-1 |

### D3. How much a hunter kills

| Hunter | Kill rate | Source |
|---|---|---|
| Wolf, Yellowstone, winter | 1.9 elk per wolf per month (1.6 early, 2.2 late winter); needs about 3.25 kg a day | [Smith et al. 2004](https://digitalcommons.mtu.edu/michigantech-p/13805); [NPS](https://www.nps.gov/articles/yellowstone-wolf-facts.htm) |
| Wolf, Białowieża | 72 red deer, 31 boar and 16 roe a year per 100 km², at 2.3 wolves per 100 km²: about 50 ungulates per wolf a year (est.) | [Jędrzejewski et al. 2002](https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_proquest_miscellaneous_18403333) |
| Eurasian lynx, Białowieża | 110-169 roe a year per 100 km², at 2.4-3.2 lynx: about 45-55 roe per lynx a year, one every 7 days (est.); 21-36% of spring roe | [Okarma et al. 1997](https://rcin.org.pl/dlibra/publication/28582/edition/12686/content) |
| Eurasian lynx, Scandinavia | 2.7-6.2 roe per 30 days (solitary females lowest, females with kittens highest) | [Andrén & Liberg 2015](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4373782/) |
| Canada lynx | 0.3-1.2 snowshoe hares a day | [O'Donoghue et al. 1998](https://bdu.siu.edu.ar/bdu/Record/todo:paper_00129658_v79_n4_p1193_ODonoghue) |
| Jaguar, Pantanal | one kill every 4.3 days (SD 4.4), longer after big prey | [Cavalcanti & Gese 2010](https://www.academia.edu/3122316) |
| Feral cat, Australia | about 740 vertebrates a cat a year (2 a day) | [CDU](https://www.cdu.edu.au/news/news-archive/cute-killers-cats-kill-more-15-billion-australian-native-animals-year) |
| Snowy owl | 3-5 lemmings a day, about 1,600 a year | [National Geographic](https://www.nationalgeographic.com/animals/birds/facts/snowy-owl) |

**For the game:** a big hunter kills every few days and eats for days (the wolf, lynx and jaguar,
one kill every 4-7 days alone). A small one kills several times a day (the cat, fox and owl, 2-10
small prey). So "hunters stop when full" (part 6) is the right rule, and a carcass should feed
its hunter for several game days.

### D4. About 10% of the energy reaches the next level

- **The rule:** about 10% of energy passes from one trophic level to the next on land. That is the
  "ten percent law" from Lindeman (1942), though his own data ranged from 0.1% to 37.5%. Losing
  half at each of three steps (death not by being eaten, dung, breathing) gives about an eighth
  ([Wikipedia, Ecological efficiency](https://en.wikipedia.org/wiki/Ecological_efficiency)).
- **In the sea:** transfer averages near 10% but varies widely, about 2-24% across systems
  (Pauly & Christensen 1995, via
  [FishBase Ecopath](https://fishbase.se/Ecopath/ecopathm.htm) and
  [Oldenburg](https://uol.de/en/fsuwi/climate-quiz/coast-1/answer-5-question)).
- **For the game (est.):** with plants as 1, grazers are about 0.1, hunters about 0.01 and top
  hunters about 0.001. This is the same as D1 one level down. Carbone's 0.9% is the
  prey-to-predator biomass ratio, which comes out near the 1% that two 10% steps would give from
  plants.

### D5. How many grazers a hectare of forage carries

- **The general rule:** grazer biomass rises steeply with plant production. Every property of the
  herbivore level (biomass, consumption, production) correlates with primary production across
  ecosystems. Livestock reach about **ten times** the biomass of wild herbivores for the same
  productivity, through herding and care
  ([Oesterheld, Sala & McNaughton 1992](https://link.springer.com/article/10.1038/356234a0)).
  In African reserves, large herbivore biomass rises with rainfall
  ([Coe, Cumming & Phillipson 1976](https://link.springer.com/article/10.1007/BF00317566)).
- **How much they eat:** wild land herbivores eat roughly a sixth of the year's plant production
  (median about 18%), and water herbivores about 51%, three times as much
  ([Cyr & Pace 1993](https://www.nature.com/articles/361148a0.pdf); the 18% median is via
  [visionmaker.us](https://visionmaker.us/info/parameter/24)). Konza manages its bison to remove
  25% ([Konza](https://lter.konza.ksu.edu/outreach/BisonFact17.pdf)).
- **By biome.** Each biome's game line uses that biome's own row. Values marked (est.) are typical
  ranges from standard references, not checked here.

  | Biome | Real carrying (sourced where linked) | Game line (est.) |
  |---|---|---|
  | Plains (Kansas) | managed bison: 1 animal unit per 4.7 ha removes 25% of growth ([Konza](https://lter.konza.ksu.edu/outreach/BisonFact17.pdf)), about 100 kg/ha; wild (est.): about 10-30 kg/ha of bison, deer and rabbits | a few big grazers per 10 ha; many rodents |
  | Savanna (llanos) | capybara 10/km² (Guárico) to 200-350/km² (Apure peaks); a flood-savanna meadow carried 1.6-3 per ha at best ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)), 80-150 kg/ha | capybaras up to 1-3 per ha by water in the rains; crowded at pools in the dry season |
  | Swamp (Pantanal) | capybara 3.2-14.8/km² ([FAO](https://www.fao.org/4/t0750e/t0750e0o.htm)) | capybaras 0.03-0.15 per ha, with deer and peccaries |
  | Forest (Białowieża) | ungulates 2-14 a km², 65-1,180 kg/km² ([Jędrzejewska et al. 1997](https://rcin.org.pl//Content/12705/PDF/BI002_26987_Cz-40-2_Acta-T42-nr36-398-451_o.pdf)), 0.7-12 kg/ha | about 1 deer or boar per 10 ha |
  | Birch forest | (est.) 3-10 kg/ha of moose, roe deer and hares | about 1 per 15 ha |
  | Taiga | hares 0.2-15 a hectare through the cycle ([UAlberta](https://era.library.ualberta.ca/files/8910jv725)); moose and caribou (est.) 0.1-2 a km² | hares rise and fall over a few game years; big grazers 1 per 50 ha |
  | Tundra | (est.) caribou 0.1-1 a km² averaged over the year, but in herds of thousands on migration; lemmings 1-100+ a hectare through the cycle | herds pass through; lemmings cycle |
  | Desert | (est.) 1-5 kg/ha: mule deer 0.5-2 a km², javelina, jackrabbits; kangaroo rats 5-50 a hectare | big grazers rare; rodents and rabbits common |
  | Mountains | (est.) chamois and ibex 2-10 a km² in summer pastures | herds on the high meadows in summer, lower in winter |

### D6. What happens without predators

- **Herbivores multiply and strip the plants.** On predator-free islands of 0.25-0.9 ha in Lago
  Guri, Venezuela, rodents, howler monkeys, iguanas and leaf-cutter ants were **10 to 100 times**
  as dense as on the mainland. Tree seedlings and saplings were "severely reduced", a trophic
  cascade ([Terborgh et al. 2001](https://reflectionsonpaperspast.com/2017/12/31/revisiting-terborgh-et-al-2001/);
  [WCS](https://library.wcs.org/en-us/doi/ctl/view/mid/33065/pubid/PUB11398.aspx)). Five years on,
  nearly all plant species were declining
  ([ScienceDaily 2006](https://www.sciencedaily.com/releases/2006/02/060228091342.htm)).
- **Then they crash.** On St. Matthew Island (Alaska), 29 reindeer introduced in 1944 with no
  predators grew to about **6,000 by 1963** and died off to **42 by 1966**. The lichen mats were
  about 1 cm deep, against 8-12 cm on a nearby island. Food, together with a hard winter, was the
  regulating mechanism
  ([Wikipedia](https://en.wikipedia.org/wiki/St._Matthew_Island);
  [Alaska Science Forum](https://www.gi.alaska.edu/alaska-science-forum/when-reindeer-paradise-turned-purgatory-0)).
  By 2005 the lichen had regrown to only 12% of the other island's.
- **Predators trim; food limits.** In Białowieża, wolves *limited* deer but did not *regulate*
  them: their share of the kills fell as deer grew more numerous. Wolf predation alone was a poor
  predictor of the deer's numbers, which also followed lynx, hunting and winters
  ([Jędrzejewski et al. 2002](https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_proquest_miscellaneous_18403333)).
- **For the game (est.):**
  - **Capacity is the main brake.** Spawning by capacity (part 7) and starving (part 7) do most of
    the balancing. Predators remove a slice, 10-35% of a prey's numbers a year, which stays under
    its yearly births.
  - **A check for plants:** a biome's grazers at capacity should eat no more than about 20-25% of
    its forage growth in their best season (Cyr & Pace's 18%, Konza's 25%).
  - **A check for hunters:** a biome's hunters at capacity should hold about 1% of the prey
    biomass (D1).
  - **Without hunters,** grazers can overshoot. Model overshoot as reduced regrowth (grass bitten
    to dirt) followed by starvation, like St. Matthew, not as unlimited growth.

### D7. The rules as numbers for the code

| Rule | Value | Source |
|---|---|---|
| Hunter biomass per prey biomass | 0.009 (90 kg per 10,000 kg) | Carbone & Gittleman 2002 |
| Transfer between levels | 0.10 (range 0.02-0.25) | Lindeman 1942; Pauly & Christensen 1995 |
| Share of plant growth eaten by wild grazers | 0.18 (median); up to 0.25 managed | Cyr & Pace 1993; Konza |
| Livestock / wild herbivore biomass at equal productivity | about 10 | Oesterheld et al. 1992 |
| Small-to-large prey switch | hunters over 21.5 kg | Carbone et al. 1999 |
| Small hunter's largest prey | about 0.5 × own mass | Carbone et al. 1999 |
| Grazer dry-matter intake | 2-3% of mass a day (capybara 2.6%) | FAO (capybara) |
| Small carnivore wet intake | 4-7% of mass a day (est.) | NRC; J. Anim. Physiol. Anim. Nutr. 2003 |
| Wolf intake | about 3.25 kg a day (about 7%) | NPS |
| Water | 0.099 × mass^0.90 L a day (mammals) | USEPA |
| Kills by big solitary hunters | one every 4-7 days | Cavalcanti & Gese 2010; Okarma et al. 1997 |
| Yearly share of a prey taken by one predator | 10-36% | Okarma et al. 1997; Jędrzejewski et al. 2002 |
| Herbivore density without predators | 10-100 × normal, then a crash | Terborgh et al. 2001; St. Matthew Island |

---

## What surprised us

1. **The semi-wild chihuahua is a scavenger first.** Free-ranging dogs live mostly on what people
   and carcasses leave: 83% human-derived food in India, and 60% of carcass biomass in Zimbabwe.
   They are "inefficient predators". In the game it should eat carrion and fruit before seeds,
   and hunt mostly rodents and insects.
2. **Our two hunters are below the 21.5 kg line**, so real ones eat small prey and many insects.
   Insects matter: a tier 3 food is not a nicety for them.
3. **In the Pantanal the jaguar eats mostly caiman and fish.** Mammals, capybara among them, were
   in only 11% of its scats at Taiamã. The capybara's predator there eats more of the capybara's
   predator.
4. **The harpy eagle is not in our capybara places.** The young capybara's raptors there are the
   caracara, the savanna hawk and the great black hawk; even vultures take newborns.
5. **Desert animals barely drink.** The kangaroo rat lives without drinking water, and kit foxes
   and wildcats get most of their water from prey. Water as a forage target should be optional
   for them, or they would crowd the few desert pools.
6. **The chicken has no wild ancestor in any of our places.** The red junglefowl is Asian. Its
   role goes to the local grouse and quail, which live in nine of our eleven biomes.
7. **Predators limit their prey but rarely set its numbers.** Food and winter do. A predator-free
   world does not explode for ever: it overshoots and crashes (St. Matthew: 29 to 6,000 to 42).
   The capacity-and-starving design already captures this. Hunters only need to take their 10-35%.
8. **One fox covers ten biomes.** The red, kit, gray, Arctic and crab-eating foxes are one game
   species with local looks. It does more for the webs than any other single addition.

## Sources

Our species
- Animal Diversity Web: [capybara](https://animaldiversity.org/accounts/Hydrochoerus_hydrochaeris/),
  [kit fox](https://animaldiversity.org/accounts/Vulpes_macrotis/),
  [swift fox](https://animaldiversity.org/accounts/Vulpes_velox/).
- FAO, [Wildlife utilization in Latin America, capybara section](https://www.fao.org/4/t0750e/t0750e0o.htm).
- San Diego Zoo Wildlife Alliance capybara fact sheet:
  [diet](https://ielc.libguides.com/sdzg/factsheets/capybaras/diet),
  [behaviour](https://ielc.libguides.com/sdzg/factsheets/capybaras/behavior).
- Barreto & Herrera 1998, Foraging patterns of capybaras in a seasonally flooded savanna of
  Venezuela, J. Trop. Ecol. ([record](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/52391845)).
- Forero-Montaña, Betancur & Cavelier 2003, capybara diet at Caño Limón, Rev. Biol. Trop. 51
  ([PDF](https://tropicalstudies.org/rbt/attachments/volumes/vol51-2/29-Forero-Dieta.pdf)).
- Corriale et al. 2013, J. Mammal. 94: 19, citing Herrera & Macdonald 1989
  ([record](https://bibliotecadigital.exactas.uba.ar/collection/paper/document/paper_00222372_v94_n1_p19_Corriale)).
- Herbst & Mills 2010, The feeding habits of the Southern African wildcat, J. Zool. 280: 403
  ([repository](https://repository.up.ac.za/handle/2263/16378);
  [Wikipedia summary](https://en.wikipedia.org/wiki/Southern_African_wildcat)).
- Lepczyk et al. 2023, A global synthesis of feral cat diets, Nat. Commun.
  ([PDF](https://www.nature.com/articles/s41467-023-42766-6.pdf)).
- Doherty et al. 2015, A continental-scale analysis of feral cat diet in Australia, J. Biogeogr.
  ([record](https://figshare.utas.edu.au/articles/journal_contribution/A_continental-scale_analysis_of_feral_cat_diet_in_Australia/22952837)).
- Woinarski et al., cats' kills per cat:
  [CDU](https://www.cdu.edu.au/news/news-archive/cute-killers-cats-kill-more-15-billion-australian-native-animals-year),
  [Smithsonian](https://www.smithsonianmag.com/science-nature/australias-cats-kill-two-billion-animals-annually-180977235/).
- Feral cat home range meta-analysis, [NZ J. Ecol. 2022](https://newzealandecology.org/nzje/3476);
  cat activity, [WikiVet](https://en.wikivet.net/Timing_of_Feline_Activity).
- Cat maintenance energy, [J. Anim. Physiol. Anim. Nutr. 87 (2003)](https://onlinelibrary-wiley-com-443.webvpn.zafu.edu.cn/doi/10.1046/j.1439-0396.2003.00431.x).
- Urban coyote diets: [phys.org 2020 (Los Angeles)](https://phys.org/news/2020-03-urban-coyotes-lots-catsand-human.html),
  [LMU](https://digitalcommons.lmu.edu/cures_reports/12).
- Vanak & Gompper 2009, Dogs as carnivores, Mammal Review
  ([record](https://wicri-demo.istex.fr/Wicri/Bois/explor/RenardV1/Site/fr/Main/Exploration/bibRecord.php?hk=000601));
  Vanak & Gompper 2009, J. Mammal. 90: 1058
  ([article](https://bioone.org/journals/journal-of-mammalogy/volume-90/issue-5/09-MAMM-A-107.1/Dietary-Niche-Separation-Between-Sympatric-Free-Ranging-Domestic-Dogs-and/10.1644/09-MAMM-A-107.1.full)).
- Butler & du Toit 2002, Anim. Conserv. 5: 29
  ([record](https://cambridge.org/core/journals/animal-conservation-forum/article/diet-of-freeranging-domestic-dogs-canis-familiaris-in-rural-zimbabwe-implications-for-wild-scavengers-on-the-periphery-of-wildlife-reserves/7523A6957C93E6365D03DB46EBC773F0));
  Butler, du Toit & Bingham 2004, Biol. Conserv.
  ([record](https://digitalcommons.usu.edu/wild_facpub/2046)).
- Ciucci et al. 1997, wolf pack home range, central Italy
  ([record](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/50959025)).
- Raptor attacks on small pets,
  [wildlife-damage-management.extension.org](https://wildlife-damage-management.extension.org/?p=754).

Places and webs
- Konza Prairie [bison fact sheet](https://lter.konza.ksu.edu/outreach/BisonFact17.pdf).
- Jędrzejewska et al. 1997, ungulates of Białowieża 1798-1993, Acta Theriol. 42
  ([PDF](https://rcin.org.pl//Content/12705/PDF/BI002_26987_Cz-40-2_Acta-T42-nr36-398-451_o.pdf)).
- Jędrzejewski et al. 2002, Kill rates and predation by wolves in Białowieża, Ecology
  ([record](https://bvbvf1.bib-bvb.de/vufind/PrimoRecord/cdi_proquest_miscellaneous_18403333)).
- Okarma et al. 1997, lynx predation on roe and red deer in Białowieża, Acta Theriol. 42
  ([record](https://rcin.org.pl/dlibra/publication/28582/edition/12686/content)).
- Andrén & Liberg 2015, lynx and roe deer, PLoS ONE
  ([PMC](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4373782/)).
- O'Donoghue et al. 1998, Functional responses of coyotes and lynx to the snowshoe hare cycle,
  Ecology 79: 1193 ([record](https://bdu.siu.edu.ar/bdu/Record/todo:paper_00129658_v79_n4_p1193_ODonoghue)).
- Hare densities: [UAlberta thesis](https://era.library.ualberta.ca/files/8910jv725),
  [Alaska Science Forum](https://www.gi.alaska.edu/alaska-science-forum/when-predators-become-dinner).
- Snowy owl, [National Geographic](https://www.nationalgeographic.com/animals/birds/facts/snowy-owl);
  Arctic fox and lemmings, [J. Mammal. 2021](https://openpolar.no/Record/ftbioone:10.1093%2Fjmammal%2Fgyab115);
  High Arctic web model, [figshare](https://openpolar.no/Record/ftdatacite:10.6084%2Fm9.figshare.c.3304824.v1).
- Arizona-Sonora Desert Museum:
  [fact sheets](https://www.desertmuseum.org/kids/oz/long-fact-sheets/index.php?print=y),
  [predators and prey](https://www.desertmuseum.org/center/edu/docs/3-4_Hunters_predators.pdf);
  [NPS Sonoran handbook](https://www.nps.gov/parkhistory/online_books/natural/4a/nh4ah1.htm).
- Eisenberg & Polisar 1999, mammals of north-central Venezuela
  ([record](https://flmnhbulletin.com/index.php/flmnh/article/view/flmnh-vol42-no3)).
- Cavalcanti & Gese 2010, Kill rates and predation patterns of jaguars in the southern Pantanal,
  J. Mammal. ([copy](https://www.academia.edu/3122316)).
- Taiamã jaguars:
  [Mongabay 2021](https://news.mongabay.com/2021/12/meet-the-fishing-jaguars-that-have-made-this-patch-of-the-pantanal-their-own/),
  [EurekAlert](https://www.eurekalert.org/news-releases/931638).
- Yellowstone wolves: Smith et al. 2004
  ([record](https://digitalcommons.mtu.edu/michigantech-p/13805)),
  [NPS wolf facts](https://www.nps.gov/articles/yellowstone-wolf-facts.htm).

Rules
- Carbone, Mace, Roberts & Macdonald 1999, Energetic constraints on the diet of terrestrial
  carnivores, Nature 402: 286 ([article](https://www.nature.com/articles/46266)).
- Carbone & Gittleman 2002, A common rule for the scaling of carnivore density, Science 295: 2273
  ([summary](https://innovations-report.com/life-sciences/report-8728);
  [abstract](https://synapsesocial.com/papers/W2014666871)).
- Coe, Cumming & Phillipson 1976, Oecologia 22: 341
  ([record](https://link.springer.com/article/10.1007/BF00317566)).
- Oesterheld, Sala & McNaughton 1992, Nature 356: 234
  ([article](https://link.springer.com/article/10.1038/356234a0)).
- Cyr & Pace 1993, Nature 361: 148 ([PDF](https://www.nature.com/articles/361148a0.pdf)).
- Lindeman 1942 and ecological efficiency,
  [Wikipedia](https://en.wikipedia.org/wiki/Ecological_efficiency); Pauly & Christensen 1995 via
  [FishBase Ecopath](https://fishbase.se/Ecopath/ecopathm.htm).
- Terborgh et al. 2001, Ecological meltdown in predator-free forest fragments, Science 294: 1923
  ([WCS](https://library.wcs.org/en-us/doi/ctl/view/mid/33065/pubid/PUB11398.aspx);
  [review](https://reflectionsonpaperspast.com/2017/12/31/revisiting-terborgh-et-al-2001/);
  [2006 follow-up](https://www.sciencedaily.com/releases/2006/02/060228091342.htm)).
- St. Matthew Island reindeer (Klein 1968):
  [Wikipedia](https://en.wikipedia.org/wiki/St._Matthew_Island),
  [Alaska Science Forum](https://www.gi.alaska.edu/alaska-science-forum/when-reindeer-paradise-turned-purgatory-0).
- USEPA 1993, Wildlife Exposure Factors Handbook, water equation
  ([EPA](https://www.epa.gov/node/97121)).

Not checked in this session, used as (ref.): NRC 2006 (dog energy needs), typical adult masses and
social systems, and which species live in which place.
