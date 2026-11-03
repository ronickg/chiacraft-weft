# Cretaceous food webs

Research for the Cretaceous part of the living-world loop (`notes/ECOLOGY.md`, "From real places,
like the species", parts 4-7). It covers what our four Cretaceous species ate and what ate them
(A), a real food web for each of the dimension's five biomes (B), the smallest species list by the
design's three tiers that makes every web whole (C), and what the herbivores ate and when (D). The
data is in `cretaceous_web.json`, next to this file.

How to read it:

- **Fossil evidence** means a specimen shows it: gut contents, tooth marks, a nest, a bonebed.
- **(inference)** means it follows from anatomy, size or living relatives, but no fossil shows it.
- **(estimate)** means a number worked out here, with the method given.
- **(game)** means a choice for the game that the record doesn't support, said openly.

## The places, and where the game bends them

| Biome | Real place | Age | What it was |
|---|---|---|---|
| Red dunes | Djadokhta Formation: Bayn Dzak (Flaming Cliffs), Tugrikin Shireh, Ukhaa Tolgod, Mongolia | Campanian, ~75-71 Ma | Wind-blown dunes in a warm semi-arid climate, with short-lived ponds and streams between dunes; compared to the Kalahari |
| Scrubland | Bayan Mandahu Formation (Inner Mongolia) and the Djadokhta's interdune flats; the Barun Goyot for the drier edge | Campanian, ~75-71 Ma | The same kind of land as the dunes but flatter and drier, with more ankylosaurs and lizards. Bayan Mandahu has the "sister" species: *Protoceratops hellenikorhinus*, *Velociraptor osmolskae*, *Kryptobaatar mandahuensis* |
| Floodplain | Cedar Mountain Formation, Yellow Cat Member, eastern Utah | Now dated Berriasian-Valanginian, ~139-134 Ma (Joeckel et al.); once thought Barremian, ~125 Ma | An open, seasonally flooded plain on clay soils that swelled and cracked (Vertisols), with strongly seasonal, possibly monsoonal rain, river forests of conifers and ferns, and wildfires |
| Fern forest | Yixian Formation, Jehol Biota, Liaoning, NE China | Barremian, ~125 Ma | A humid conifer forest with ginkgoes, cycads, ferns and horsetails round volcanic lakes, with regular dry spells. Cool: mean annual temperature ~10 °C |
| Crags | No formation | - | Fossils come from basins where sediment piled up, almost never from high rock, which erodes. Every Mesozoic upland fauna is lost. The crags' web is built from what lived on the edges, plus the wyvern |

**Why the Yixian for the fern forest.** The Cedar Mountain's wetter members don't fit well: the
Poison Strip and Ruby Ranch are semi-arid, and the Mussentuchit (~100 Ma) is a coastal plain whose
leaf flora is all flowering plants. The Yixian is the best-studied wet conifer and fern forest of
the Early Cretaceous. Its gut contents settle more of who ate whom than any other formation's, and
it is Early Cretaceous like the Utahraptor's floodplain. The cost: it was cold, while the game's
Cretaceous is warm everywhere. A warmer alternative is the Wessex Formation (Isle of Wight,
Barremian), but its land was a fire-swept fern "prairie" more than a forest.

**The bends, said once.** The game puts three faunas into one dimension:

- **Time:** the Yellow Cat is ~139-134 Ma, the Yixian ~125 Ma and the Djadokhta ~75-71 Ma.
  Utahraptor and Velociraptor are ~60 million years apart, nearly as far as Velociraptor is from
  us. *Protoceratops* never met an iguanodont or a sauropod like these.
- **Place:** Utah and Mongolia were a continent apart, though North America and Asia shared
  faunas over Beringia at times.
- **Grass:** there was no grass in any of these places. The oldest grass known is a few bits from
  the Albian of China, ~113-100 Ma, caught on a hadrosauroid's teeth (Wu et al. 2018), and grass
  was still rare at the end of the Cretaceous (phytoliths in Indian titanosaur dung; Prasad et al.
  2005). The game's grass blocks there are a bend. The real ground cover was ferns, horsetails, low cycadophytes,
  and later small flowering herbs. In the diets below, "grass" is 0 and low ferns and horsetails
  count as "leaves". See D for how the game can map this.
- **Climate:** the Yixian was cool with cold winters, and the game makes it warm.

## A. Our four species

### What they ate, and how we know

| | Velociraptor mongoliensis | Velociraptor osmolskae | Utahraptor ostrommaysi | Wyvern (fantasy) |
|---|---|---|---|---|
| Mass | 14-20 kg (Paul 1988; Turner et al. 2007); up to ~28 kg in some estimates | As *V. mongoliensis* (known from skull bones only) | ~280-500 kg; 280 kg for the largest specimen measured (Molina-Pérez & Larramendi 2019), up to 391-777 kg (Pintore et al. 2024). The game's 300 kg is on the low side of most estimates | 5000 kg (game). Real models: the azhdarchids *Quetzalcoatlus northropi* and *Hatzegopteryx* (~10-11 m wingspan, ~200-250 kg) |
| Direct evidence | **Fighting Dinosaurs** (MPC-D 100/25, Tugrikin Shireh, 1971): a Velociraptor with its sickle claw in a *Protoceratops*'s throat and its arm in the *Protoceratops*'s beak, buried by a collapsing dune or sandstorm. **An azhdarchid pterosaur bone in its gut** (MPC-D 100/54; Hone et al. 2012), unetched by acid, so eaten just before it died and probably scavenged | **Tooth-marked *Protoceratops* jaws with shed Velociraptor-like teeth** at Bayan Mandahu (Hone et al. 2010): late-stage carcass feeding, by scavenging or after a kill | **The Stikes Quarry block** (9 t, eastern Utah): an adult, several juveniles and hatchlings, perhaps twice as many as first counted, with at least one iguanodont, all caught in a quicksand-like dewatering structure, probably while going after the trapped iguanodont (Utah Geological Survey). The *Utahraptor* type was found with *Gastonia* | None (fantasy). Azhdarchids: no gut contents; *Hatzegopteryx*'s short, robust neck and wide gape suggest it was the apex predator of Haţeg island, taking young dinosaurs and animals the size of a horse (Naish & Witton 2017) |
| Inferred diet | Small vertebrates (lizards, mammals, young dinosaurs) and *Protoceratops*; much scavenging. Its bite was weak (~300 N; Sakamoto 2022). It pinned prey under its weight and gripped with the sickle claws, as hawks do, and may have eaten them alive (Fowler et al. 2011) | As its cousin | Large herbivores: iguanodonts, *Gastonia* juveniles, young sauropods. Heavy legs suited ambush, not long chases | Azhdarchids as "terrestrial stalkers": they walked the ground taking small animals and young dinosaurs, like giant ground hornbills or storks (Witton & Naish 2008, 2015). Big eagles take prey up to about their own mass |
| Prey size (vs own mass) | ~0.002× (a 30 g lizard) to ~5× (an adult *Protoceratops*, 62-104 kg). Tackled prey bigger than itself **alone**: there is no sign of a group | As its cousin | Alone ~0.01× to ~3× (a young iguanodont). In a group, Kirkland et al. (1993) suggested prey of 1-2 t (5-15×) | ~0.001× (a 5 kg animal) to ~0.6× (a 3 t iguanodont) (game, from azhdarchids and eagles) |
| Hunting | **Alone**. Many Velociraptor fossils, none found with another (Wikipedia, summarising the literature). Possibly most active at night or twilight: its scleral rings are like nocturnal birds' (Schmitz & Motani 2011), and *Protoceratops* was cathemeral | Alone (inference) | **In a group of mixed ages**, from the Stikes block: the strongest group evidence for any dromaeosaur, though it may be a trap that collected animals over time | Alone (game; as eagles and azhdarchids) |
| What preyed on them | Adults: nothing known. The Djadokhta has no resident large predator; tyrannosaur and other large remains are fragments of animals passing through (Longrich 2010). Young and eggs: probably large lizards, adult Velociraptor and azhdarchids (inference) | As its cousin | Adults: perhaps a large allosauroid (~10 m, known only from teeth and bones in the Yellow Cat). Young: the allosauroid, crocodyliforms, adult Utahraptor (inference) | None (apex). Eggs and young: Utahraptor groups (game) |

**The pack question.** The evidence on dromaeosaur packs:

- **Deinonychus:** found with *Tenontosaurus* carcasses, which started the pack-hunting idea
  (Ostrom 1969; Maxwell & Ostrom 1995). Roach & Brinkman (2007) read the same sites as a
  free-for-all, like Komodo dragons or crocodiles. Frederickson et al. (2020) found in tooth carbon
  isotopes that young Deinonychus ate different prey from adults. So the young weren't fed by
  their parents and didn't hunt in a wolf-like pack. A loose mob of all ages is still possible.
- **Trackways:** six large dromaeosaurs walking side by side in Shandong, China (Li et al. 2008).
  This is the only direct sign that dromaeosaurs moved in groups.
- **Utahraptor:** the Stikes block points to groups of mixed ages.
- **Velociraptor:** no sign of a group.

What it means for the game: the Utahraptor's `Social::Pack` stands; a "pack" there is a loose
group of mixed ages (a family, or a mob at a carcass). The two Velociraptors have no support for
`Social::Pack`. They are better as `Solitary`, or as pairs that gather at a carcass (a choice for
Ronald; the "pack hunter" signature is the game's).

### Daily food and water (estimates)

Method: field metabolic rate (FMR) from the allometry of free-living animals in Nagy, Girard &
Brown (1999). The equations, in kJ a day for mass M in grams, as commonly cited from that review:

- mammals: 4.82 M^0.734;
- birds: 10.5 M^0.681;
- reptiles: 0.196 M^0.889.

Two bounds:

- **"endotherm":** the mean of the mammal and bird rates. Theropods and sauropods sit nearest
  this: Wiemann et al. (2022) read bird-like metabolic rates from the chemistry of their bone.
- **"mesotherm":** the geometric mean of the mammal and reptile rates. Grady et al. (2014) put
  dinosaurs here from their growth rates. Wiemann et al. found ceratopsians and hadrosaurs near
  lizards' rates, so ornithischians sit nearer this.

To turn energy into food: whole prey gives ~6 kJ/g metabolised, wet weight. Fern and conifer
browse gives ~8 kJ/g dry matter, which is ~18 kJ/g gross at ~45% digestibility, in the range of
Hummel et al. (2008).

| Species | Mass | Mesotherm | Endotherm | Game value | Water (estimate) |
|---|---|---|---|---|---|
| Velociraptor | 15 kg | 0.40 kg meat/day (2.6%) | 1.1 kg/day (7.2%) | **5%** of its mass in meat a day; about one lizard-and-mammal day, or a *Protoceratops* carcass every 2-3 weeks | ~2-3% a day; most from prey (meat is ~70% water), drinking at interdune ponds when there are any |
| Utahraptor | 300 kg | 4.5 kg (1.5%) | 8.9 kg (3.0%) | **2.5%** (~7.5 kg/day, as a lion). A 1-2 t iguanodont feeds a group of five for weeks | ~2-3% |
| Wyvern | 5000 kg | 44 kg (0.9%) | 65 kg (1.3%) | **1.5%** (~75 kg/day, a little over the endotherm figure because flying costs more): one *Protoceratops* a day, or an iguanodont a month | ~1-2% |

Comparison: wolves eat ~3 kg a day at ~40 kg (~7%), and lions ~7 kg a day at ~190 kg (~3.5%). So
the endotherm figures fit living hunters of those sizes. The herbivores and small animals'
figures are in C and in the JSON.

## B. A food web for each biome

Numbers of PBDB collections give a rough idea of how common each animal was. For the Djadokhta,
they count Djadokhta collections naming each genus (PBDB, below). Collections aren't individuals,
but the order agrees with the literature.

### Red dunes: the Djadokhta Formation

*Protoceratops* is "extremely common and the most frequent occurrence". Bayn Dzak has been called
the "*Protoceratops* fauna" (Longrich 2010). PBDB: *Protoceratops* 23 collections, *Velociraptor*
13, *Pinacosaurus* 12, *Zalambdalestes* 11, the lizard *Adamisaurus* 10, *Kryptobaatar* 5. Fully
aquatic animals are almost absent, and plants are nearly unknown: only fossil roots (*Radicites
gobiensis*; Naugolnykh 2016).

| Animal | Mass | Diet | Eaten by | Social life | How common |
|---|---|---|---|---|---|
| *Protoceratops andrewsi* | 62-104 kg adult | Low browse, chopped and sheared with a strong bite (Button & Zanno 2019; low browsing, Tanoue et al. 2009) | Velociraptor (fossil); scavenging mammals and insects on carcasses (borings) | Groups of mixed ages; small herds at Tugrikin Shireh (Hone et al. 2014); a nest of 15 juveniles; buried soft-shelled eggs, ≥12 a clutch (Norell et al. 2020) | Dominant |
| *Velociraptor mongoliensis* | 15 kg | Meat, carrion | Nothing known (adults) | Alone | Common |
| *Pinacosaurus grangeri* | ~1.9 t adult (Paul 2010) | Low plants; may have eaten pulpy fruit, perhaps insects (speculative) | Velociraptor on juveniles (inference) | Juveniles in groups, killed together by sandstorms; adults alone | Common, mostly juveniles |
| *Citipati osmolskae* / *Oviraptor* | 75-110 kg (*Citipati*) | Omnivore or herbivore (inference) | Velociraptor on young and eggs (inference) | Nests of up to 22 eggs, brooded like birds' (Norell et al. 1995, 2018). *Oviraptor* was no egg thief: it sat on its own eggs | Common at Ukhaa Tolgod |
| *Kryptobaatar dashzevegi* (multituberculate) | ~30-50 g (estimate) | Omnivore, leaning to animal food; hopped like a jerboa | Velociraptor, troodontids, lizards (inference) | Probably burrowing (inference) | The commonest mammal (Kielan-Jaworowska et al. 2003) |
| *Zalambdalestes lechei* (eutherian) | ~50-100 g (estimate, ~5 cm skull) | Insects | As above | Alone (inference) | Common |
| Lizards: *Adamisaurus*, *Priscagama*, *Gobinatus*, others | 10-100 g; *Estesia* (Barun Goyot), a monitor-like lizard ~1.5-2 m | Insects; *Estesia* eggs and small vertebrates (inference) | Velociraptor, oviraptorids, troodontids | Alone | "Rather abundant and diverse" |
| *Shuvuuia deserti* (alvarezsaur) | ~1 kg (estimate) | Ants and termites, dug out (inference) | Velociraptor | Alone; night-active (Choiniere et al. 2021) | Uncommon |
| *Saurornithoides*, *Byronosaurus* (troodontids) | ~5-15 kg | Small vertebrates, perhaps plants (inference) | Velociraptor | ? | Uncommon |
| *Gobipteryx minuta* (enantiornithine bird) | ~0.2-0.5 kg (estimate) | Insects, seeds (inference) | Velociraptor, troodontids | Nested on the ground; embryos in eggs are known | Uncommon |
| *Gobiosuchus* (small land crocodyliform) | ~5-10 kg (estimate) | Small vertebrates, insects (inference) | Velociraptor | Alone | Uncommon |
| *Zangerlia* (land turtle) | ~2-5 kg (estimate) | Plants (inference) | Velociraptor (on young) | Alone | Rare |
| Azhdarchid pterosaur | ~50-200 kg | Small animals stalked on the ground (Witton & Naish 2008) | Scavenged by Velociraptor (fossil) | ? | Rare (bones) |
| Carrion insects | - | Carcasses: borings, pupal chambers and gnawed joints in *Protoceratops* and Velociraptor skeletons (Saneyoshi et al. 2011; Kirkland & Bader 2010) | Lizards, mammals | - | Common traces |

**The dead:** Velociraptor (scavenged a pterosaur and *Protoceratops*), small mammals (broad
gnawing borings in *Protoceratops* ribs) and insects (borings and pupal chambers). All three are
fossil evidence: the whole "something for the dead" level is real here.

**Plants** (all inference except the roots): low shrubs (Fastovsky 1997); shrubby drought-tolerant
conifers (cheirolepidiaceans, *Brachyphyllum*-type); *Ephedra*-like gnetaleans; ferns and
horsetails round interdune ponds; by the Campanian, small flowering plants too. The modern
analogues are North African and West Asian deserts (Naugolnykh 2016).

### Scrubland: Bayan Mandahu, the Barun Goyot, the Djadokhta's flats

Bayan Mandahu's fauna is the Djadokhta's with sister species: *Protoceratops hellenikorhinus*,
*Velociraptor osmolskae*, *Kryptobaatar mandahuensis*. It adds the dromaeosaur *Linheraptor*, the
oviraptorids *Machairasaurus* and *Wulatelong*, the alvarezsaur *Linhenykus*, troodontids, and
many *Pinacosaurus*. Bayan Mandahu and the Alag Teeg have most of the juvenile *Pinacosaurus* known
(at least 24 skeletons, more than any other ankylosaur's). The drier Barun Goyot adds the
ankylosaurs *Saichania* and *Tarchia*, *Bagaceratops*, and the lizard *Estesia*.

| Animal | Mass | Diet | Eaten by | Social life | How common |
|---|---|---|---|---|---|
| *Pinacosaurus* (*P. grangeri*, *P. mephistocephalus*) | ~1.9 t | Low plants | Velociraptor on juveniles | Juvenile groups | Dominant here |
| *Protoceratops hellenikorhinus* | ~100 kg (estimate; a longer skull than *P. andrewsi*'s) | As *P. andrewsi* | Velociraptor: the tooth-marked jaws are from here | Groups | Common |
| *Velociraptor osmolskae* | ~15 kg | Meat, carrion | - | Alone | Uncommon |
| *Linheraptor* | ~25 kg (estimate) | Meat | - | ? | Uncommon |
| Oviraptorids (*Machairasaurus*, *Wulatelong*) | 20-40 kg (estimate) | Omnivore (inference) | Dromaeosaurs on young | Nests | Common |
| *Kryptobaatar*, *Zalambdalestes*, other small mammals | 30-100 g | Insects, seeds | Everything larger | Burrows | Common |
| Lizards | 10-100 g | Insects | Dromaeosaurs, troodontids, oviraptorids | - | Abundant |
| *Linhenykus* (alvarezsaur) | <1 kg | Ants and termites (inference) | Dromaeosaurs | - | Rare |
| Termites and ants | - | Dead plant matter | Alvarezsaurs, lizards, mammals | Colonies | Inference |

For the game, this is where *Velociraptor osmolskae* lived: its row could add scrubland, and
*Protoceratops* lives here less often than on the dunes.

### Floodplain: the Yellow Cat Member, Cedar Mountain Formation

PBDB's Yellow Cat collections: *Utahraptor* 4; *Gastonia*, *Iguanacolossus*, *Cedarosaurus*,
*Moabosaurus*, *Falcarius*, *Nedcolbertia*, *Yurgovuchia* and the lungfish *Ceratodus* 2 each; the
rest 1. *Gastonia* is "among the most common dinosaur fossils in the Cedar Mountain Formation", with
ten skulls by 2014. The member splits in two:

- **Lower:** *Falcarius* (a bonebed of hundreds of individuals at Crystal Geyser),
  *Iguanacolossus*, *Mierasaurus*, *Yurgovuchia*.
- **Upper:** *Utahraptor*, *Gastonia*, *Hippodraco*, *Cedrorestes*, a sail-backed iguanodont,
  *Cedarosaurus* and *Moabosaurus*.

| Animal | Mass | Diet | Eaten by | Social life | How common |
|---|---|---|---|---|---|
| Iguanodonts: *Iguanacolossus fortis* (lower), *Hippodraco*, *Cedrorestes*, the sail-backed form (upper) | *Iguanacolossus* ~9 m, ~1-5 t (5 t, Paul); *Hippodraco* ≥4.5 m | Browse up to ~4 m: ferns, horsetails, conifers, cycadophytes (inference) | Utahraptor (the Stikes block); the allosauroid | Herds (inference from relatives' bonebeds) | Common |
| *Gastonia burgei* (polacanthid ankylosaur) | ~1.9 t, 5 m | Low plants | Utahraptor on young; armour against it (Kirkland) | Bonebeds suggest groups; one formed in a drought or flood | Most common |
| *Cedarosaurus weiskopfae* (brachiosaurid), *Moabosaurus* (turiasaur) | ~10 t (estimate) | High browse: conifers (*Araucaria*), horsetails, ferns | Utahraptor and the allosauroid on young | Herds (inference) | Common |
| *Utahraptor ostrommaysi* | ~300 kg | Meat, carrion | Allosauroid (inference) | Mixed-age groups | Common |
| Large allosauroid (teeth and bones) | ~10 m, ~2-3 t (estimate) | Meat | - | Alone | Uncommon |
| *Falcarius utahensis* (therizinosaur) | ~100 kg (estimate) | Plants, perhaps some meat | Utahraptor, allosauroid | Mass bonebed: groups | Common (lower) |
| *Nedcolbertia* (ornithomimosaur), *Martharaptor* | ~20-50 kg | Omnivores (inference) | Utahraptor | ? | Uncommon |
| Crocodyliforms (a skull; indeterminate) | ~2-3 m, ~100-150 kg (estimate) | Fish, turtles, animals at the water | Utahraptor on young | Alone | Present (a skull) |
| *Glyptops*, *Naomichelys* (turtles) | 1-5 kg | Omnivores | Crocodyliforms | - | Present |
| Lungfish *Ceratodus*, *Semionotus*, *Amia* | 1-10 kg | Snails, insects, plants, fish | Crocodyliforms, Utahraptor (inference), pterosaurs | - | Common in the channels and lakes |
| *Cifelliodon* (mammaliaform), *Toxolophosaurus* (lizard), a choristodere | 50 g-1 kg | Insects, plants, hard food (*Toxolophosaurus* had crushing teeth) | Small theropods | - | Uncommon |
| Pterosaurs (indeterminate), birds (*Aquatilavipes* tracks: a wader) | - | Fish, small animals | - | - | Uncommon |

**Plants:** none described from the Yellow Cat itself. From the formation and the region:

- conifers;
- the tree fern *Tempskya* and conifer wood (Poison Strip);
- fern, lycopsid and moss spores and conifer pollen (Tschudy et al.);
- charophyte algae in the ponds.

The Yellow Cat's land was an open floodplain "neither forest nor grassland", with river forests.
The game's ferns and horsetails over the ground fit it. At its new age, ~139-134 Ma, flowering
plants were barely starting: no flowers in the floodplain (inference). The Ruby Ranch and
Mussentuchit floras, which have them, are 20-40 million years younger.

### Fern forest: the Yixian Formation (Jehol Biota)

Insects had their largest Mesozoic radiation here: insects lead PBDB's Yixian list, with
*Confuciusornis* (14 collections) the commonest vertebrate there. *Psittacosaurus* is the commonest
dinosaur of the Jehol, known from hundreds of specimens. Gut contents are what make this web.

| Animal | Mass | Diet | Eaten by | Social life | How common |
|---|---|---|---|---|---|
| *Psittacosaurus lujiatunensis* | ~20-30 kg adult (*P. mongoliensis* >20 kg) | Tough plants, ground fine with gastroliths even as hatchlings (Wang et al. 2025) | *Repenomamus* (a juvenile in its gut, Hu et al. 2005; one attacking an adult, 2023); *Yutyrannus* (inference) | Juveniles in groups of the same age, without adults (Zhao et al. 2014) | Dominant |
| *Jinzhousaurus yangi* (iguanodontian) | ~7 m, ~1-2 t (estimate) | Browse | *Yutyrannus* | ? | Uncommon |
| *Dongbeititan dongi* (titanosauriform sauropod) | ~10 t (estimate) | High browse | *Yutyrannus* on young | ? | Rare |
| *Liaoningosaurus* (tiny ankylosaur) | ~1 m juvenile specimen | Plants; perhaps fish (Ji et al. 2016, *Journal of Geology*; debated) | - | - | Rare |
| *Jeholosaurus* (small ornithopod) | ~5-10 kg | Plants, perhaps omnivore | Dromaeosaurs, *Sinosauropteryx* | - | Common (Lujiatun) |
| *Yutyrannus huali* (feathered tyrannosauroid) | ~1.4 t (Xu et al. 2012) | Meat | - | Three found together | Apex |
| *Sinocalliopteryx*, *Sinosauropteryx* (compsognathids) | 1-20 kg | *Sinocalliopteryx*: two *Confuciusornis* and an ornithischian in its gut, a dromaeosaur leg in another (Xing et al. 2012). *Sinosauropteryx*: a lizard, mammal jaws | Bigger theropods | Alone | Common |
| *Sinornithosaurus*, *Microraptor* (small dromaeosaurs) | 1-5 kg | *Microraptor*: a bird, fish, a mammal and a lizard swallowed whole in four specimens (Jiufotang Formation, slightly younger; O'Connor et al. 2019) | *Sinocalliopteryx* | - | Common |
| *Repenomamus* | 4-6 kg (*R. robustus*), 12-14 kg (*R. giganticus*) | Meat, including young *Psittacosaurus* | Theropods | - | Uncommon |
| *Confuciusornis sanctus* (early bird) | ~0.2-1.5 kg (estimate) | Fish remains in one specimen; seeds or plants proposed | *Sinocalliopteryx* (fossil), *Microraptor*, pterosaurs | Flocks; many killed together by eruptions | Abundant |
| Lizards (*Dalinghosaurus*, *Yabeinosaurus*), frogs (*Liaobatrachus*) | 10-500 g | Insects | Compsognathids, *Microraptor* (fossil) | - | Common |
| Choristoderes (*Hyphalosaurus*, *Monjurosuchus*), turtles (*Manchurochelys*) | 0.5-5 kg | Fish | - | - | Common in lakes |
| Fish: *Lycoptera* (dominant), *Peipiaosteus*, *Protopsephurus* | 10 g-2 kg | Insect larvae, plankton | Birds, choristoderes, *Microraptor*, pterosaurs | Shoals | Abundant |
| Insects (beetles, flies, mayflies, wasps, lacewings, scorpionflies) | - | Everything | Lizards, mammals, birds, frogs | - | Abundant |

**Plants:**

- **Conifers:** Podocarpaceae, Pinaceae, Cupressaceae, Sciadopityaceae and Taxodiaceae,
  dominant in the petrified wood (from the formation's petrified-wood studies).
- **Ginkgoes:** *Ginkgo apodes*.
- **Cycads and seed ferns.**
- **Ground cover:** ferns, horsetails and lycopods.
- **Gnetaleans:** *Ephedra*-like.
- **Early flowering plants:** *Archaefructus*, a water plant, *Hyrcantha* and *Archaeamphora*.
- **Pondweed:** *Potamogeton* (PBDB).

### Crags: no real place, and an honest web

No Cretaceous formation preserves high rock. What lived on the crags is inference from the edges
and from living mountain webs:

- **Plants:** in the clefts and on ledges, drought-tolerant conifers (araucarians and
  cheirolepidiaceans), cycads and ferns.
- **Small animals:** lizards, small mammals, insects, and birds nesting on cliffs as many birds
  do today (inference).
- **The hunter:** the wyvern hunts out from it into every other biome.
- **The dead:** its kills feed carrion insects, and rot.

The web is whole only because the wyvern's prey lives next door: a check of "every hunter has
prey there" must count its range, not just the crags.

| Animal | Mass | Diet | Eaten by | Social life |
|---|---|---|---|---|
| Wyvern | 5000 kg | Meat (from all biomes), fish, carrion | - | Alone; nests on ledges |
| Early birds (*Gobipteryx*, game) | ~0.3 kg | Insects, seeds | (none here at a wyvern's scale) | Cliff colonies (game) |
| Lizards | 10-100 g | Insects | Birds? | - |
| Small mammals | 30 g | Insects, seeds | Lizards (large ones) | - |
| Insects, carrion insects | - | Plants, carcasses | Lizards, mammals, birds | - |

## C. The species list the game needs

The rule, from part 6: a hunter takes prey from 1/20 of its mass to its own, or three times its
own in a pack. Real animals break it both ways, and so do the rows below. The small hunters take
far smaller prey: a Velociraptor eats 30 g lizards, 1/500 of its mass. A Velociraptor also took
prey ~5× its mass alone. A species-level "prey from x to y" (in the JSON as `prey_mass_ratio`)
would be closer to the record than one rule for all.

### Tier 1: full creatures (modelled, tameable)

| Species | Biomes | Mass | Diet | Predators | Prey | Why it's needed |
|---|---|---|---|---|---|---|
| *Velociraptor mongoliensis* (have) | red dunes 1.0, scrubland 0.4 | 15 kg | meat 0.7, carrion 0.25, insects 0.05 | wyvern (game) | protoceratops, kryptobaatar, lizards, birds, citipati young and eggs | - |
| *Velociraptor osmolskae* (have) | red dunes 1.0; **add scrubland 0.6** (Bayan Mandahu was its home) | 15 kg | as above | wyvern | as above, pinacosaurus juveniles | - |
| *Utahraptor ostrommaysi* (have) | floodplain 1.0, scrubland 0.5, fern forest 0.3 | 300 kg | meat 0.8, carrion 0.15, fish 0.05 | wyvern; crocodyliform on young | iguanodont, gastonia young, sauropod young, psittacosaurus, crocodyliform, turtles | - |
| Wyvern (have) | crags 1.0, others 0.33 | 5000 kg | meat 0.8, fish 0.1, carrion 0.1 | - | every herbivore from protoceratops up, utahraptor, raptors, crocodyliform | - |
| ***Protoceratops andrewsi*** (new) | red dunes 1.0, scrubland 0.6 | 85 kg | leaves 0.85, seeds 0.1, fruit 0.05 | raptors (fossil), wyvern | - | The dunes' herbivore: most of what lived there, and the raptors' food. Herds, buried egg clutches; a tame herd animal (like sheep) |
| ***Iguanodont*** (new; the row *Iguanacolossus fortis*, standing for the Yellow Cat's iguanodonts and the Yixian's *Jinzhousaurus*) | floodplain 1.0, fern forest 0.6 | ~4000 kg (1-5 t) | leaves 0.85, seeds 0.1, fruit 0.05 | utahraptor (fossil: Stikes), wyvern | - | The floodplain's and forest's big herbivore: what the Utahraptor was trapped hunting. The obvious tame beast of burden. It could start as tier 2 |

### Tier 2: simple creatures that live, flee, breed and get eaten

| Species | Biomes | Mass | Diet | Predators | Prey | Why |
|---|---|---|---|---|---|---|
| *Pinacosaurus grangeri* (ankylosaur model) | scrubland 1.0, red dunes 0.5 | 1900 kg | leaves 0.8, fruit 0.1, roots 0.05, insects 0.05 | raptors on young, wyvern | - | The scrubland's big herbivore; young in groups give the raptors prey their size |
| *Gastonia burgei* (same model, its own row) | floodplain 1.0, scrubland 0.3, fern forest 0.3 | 1900 kg | leaves 0.9, fruit 0.05, seeds 0.05 | utahraptor on young, wyvern | - | The floodplain's commonest animal; armoured, so adults are hard prey |
| *Cedarosaurus weiskopfae* (sauropod; stands for *Dongbeititan* in the forest) | floodplain 1.0, fern forest 0.5 | ~10000 kg | leaves 0.95, seeds 0.05 | utahraptor and wyvern on young | - | High browse (the conifers); a giant carcass for the dead |
| *Citipati osmolskae* (oviraptorid) | red dunes 1.0, scrubland 0.7 | 90 kg | leaves 0.4, seeds 0.25, fruit 0.1, meat 0.15, insects 0.1 | raptors on young, wyvern | lizards, kryptobaatar, eggs | The dunes' omnivore; brooding nests the player sees |
| *Psittacosaurus lujiatunensis* | fern forest 1.0, floodplain 0.3, scrubland 0.3 | 25 kg | leaves 0.6, seeds 0.3, fruit 0.1 | utahraptor, wyvern (and *Repenomamus*, real) | - | The forest's small, common herbivore: prey for the Utahraptor where it is rare |
| *Kryptobaatar dashzevegi* (small mammal model; multituberculates lived in all three formations) | red dunes 1.0, scrubland 1.0, fern forest 0.5, floodplain 0.5, crags 0.3 | 0.04 kg | insects 0.45, seeds 0.3, fruit 0.1, meat 0.15 | raptors, citipati, birds? | insects, eggs | The small-animal level everywhere: the raptors' everyday food |
| *Adamisaurus magnidentatus* (lizard model) | red dunes 1.0, scrubland 1.0, crags 0.6, floodplain 0.5, fern forest 0.5 | 0.05 kg | insects 0.9, leaves 0.05, fruit 0.05 | raptors, citipati, birds, kryptobaatar? | insects | Insect eaters in every biome; prey for young raptors |
| *Confuciusornis sanctus* (early bird; a second row *Gobipteryx minuta* for the Gobi) | fern forest 1.0, floodplain 0.6; *Gobipteryx*: crags 1.0, scrubland 1.0, red dunes 0.5 | 0.5 kg | *Confuciusornis*: fish 0.4, seeds 0.4, insects 0.2 | raptors, utahraptor young | fish, insects | Flocks in the air; the crags' small life |
| Crocodyliform (Neosuchia, indeterminate; the Yellow Cat's) | floodplain 1.0 (in water), fern forest 0.4 | 150 kg | fish 0.6, meat 0.3, carrion 0.1 | utahraptor, wyvern | fish, turtles, young dinosaurs at the water | The water's hunter; a predator for young Utahraptors, as caimans are for capybaras |

### Tier 3: food sources in the world

| Source | Biomes | Category | Eaten by | Why |
|---|---|---|---|---|
| Beetles and grubs in rotting wood and dung | all but red dunes | insects | lizards, kryptobaatar, birds, citipati | Insects for the insect eaters |
| Termite and ant nests | red dunes, scrubland | insects | lizards, kryptobaatar, (alvarezsaurs) | The dry biomes' insects |
| Carrion insects on carcasses | all | eat carrion; give insects | lizards, kryptobaatar | Real in the Djadokhta: the borings; they speed rot |
| Fish (lungfish *Ceratodus*, *Semionotus* and *Amia* in the floodplain; *Lycoptera* in the forest lakes) | floodplain, fern forest | fish | crocodyliform, birds, wyvern, utahraptor | Fish as a catch in water |
| Eggs in nests (protoceratops, citipati, raptors) | where they nest | meat | raptors, kryptobaatar, lizards, citipati | Eggs are real food and nests are built already |
| Carcasses | all | carrion | raptors, utahraptor, wyvern, insects | The dead |

The plants (the plant table, not creatures) are in D.

### Left out, and what stands in for them

| Real animal | Stood in for by | Note |
|---|---|---|
| Yellow Cat allosauroid, *Yutyrannus* | wyvern | Real apex hunters of the floodplain and forest. Optional tier 2 if the wyvern is too rare to keep herbivores in check |
| Azhdarchid pterosaurs | wyvern | The real model for the wyvern; Velociraptor's scavenged meal |
| *Repenomamus* | - | A mammal that ate baby dinosaurs: optional, for the forest |
| *Falcarius*, *Nedcolbertia*, troodontids, alvarezsaurs, compsognathids, small dromaeosaurs | Velociraptor, Utahraptor | Middle-sized theropods; optional |
| Turtles, choristoderes, frogs | crocodyliform, fish | Water's small life; optional |
| *Zalambdalestes* | kryptobaatar | The insect-eating small mammal |

## D. What the herbivores ate, and the seasons

### The plants

There were no grasses, and before ~120 Ma almost no flowering plants. What herbivores ate:

| Plant | Where | What it gave | Food value | Seasons (inference) |
|---|---|---|---|---|
| Horsetails (*Equisetum*) | wet ground everywhere: pond edges in the dunes, the floodplain, the forest | leaves (shoots) | **High**: energy like modern browse, among the best foods for big herbivores (Hummel et al. 2008) | Shoots in the wet season; persists in the dry where the ground stays wet |
| Ground ferns | floodplain, forest, scrubland hollows | leaves | Medium; some ferns (*Angiopteris*-type marattialeans) good (Hummel et al. 2008) | Flush with rain, dies back in drought |
| Tree ferns (*Tempskya*) | floodplain | leaves | **Poor** (Hummel et al. 2008) | All year |
| Cycads and bennettitaleans | all but the wettest | leaves, seeds | Leaves **poor** and tough; seeds in cones (inference) | Seeds after the rains |
| *Araucaria*-type conifers | floodplain, forest, crags | leaves (needles), seeds (big cones) | Needles **high energy but slow to ferment**: food for the biggest guts (sauropods) | Cones ripen late in the wet season (inference) |
| Cheirolepidiaceans (*Brachyphyllum*, *Frenelopsis*) | red dunes, scrubland, crags | leaves | Low (inference); drought-tolerant scale-leaved shrubs and trees | All year |
| Podocarps, pines, cypresses | forest | leaves, seeds | Podocarps **poor** (Hummel et al. 2008) | Cones in autumn (inference) |
| *Ginkgo* | forest | leaves, fleshy seeds (fruit) | Leaves **good** (Hummel et al. 2008); fleshy seeds | Leaves in spring and summer, seeds in autumn, deciduous (as the living one) |
| *Ephedra*-like gnetaleans | red dunes, scrubland, forest | fruit (fleshy cone bracts), seeds | Medium (inference) | Fruit after the rains |
| Early flowering plants (*Archaefructus*, pondweed) | forest lakes | leaves, seeds | Medium (inference) | Summer |
| Fern rhizomes | floodplain, forest | roots | Speculative food for diggers | Year round |

So: horsetails, *Araucaria*, *Ginkgo* and good ferns are the best foods, and cycads, tree ferns and
podocarps the poorest (Hummel et al. 2008). For the game: a plant row's food value can come from
this table. The dry biomes' shrubs keep animals alive in the dry season but don't fatten them.

**Who ate what:**

- **Protoceratops:** a low browser with a beak and shearing teeth and a strong bite, for tough,
  fibrous plants (Button & Zanno 2019; Tanoue et al. 2009). Shrubs, ferns and horsetails by
  ponds, cycadophytes.
- **Ankylosaurs:** low plants, soft or pulpy fruit, with a big hindgut. Small teeth, replaced
  slowly.
- **Iguanodonts:** up to ~4 m, everything in reach (inference).
- **Sauropods:** high conifers, with horsetails and ferns low down (Hummel et al. 2008).
- **Psittacosaurus:** tough plants, perhaps seeds and nuts, ground with gastroliths.

### The seasons

| Biome | Real seasonality | The loop |
|---|---|---|
| Red dunes and scrubland (Djadokhta) | Semi-arid with seasonal rain (Naugolnykh 2016): hot, dry summers and probably short, humid winters (as summarised in the formation's literature). Ponds between the dunes came and went. Sandstorms and collapsing dunes killed *Protoceratops* and young *Pinacosaurus* in groups | The rain comes in **winter and spring**; ponds fill, shrubs and horsetails flush; *Protoceratops* and *Citipati* nest in spring; raptors feed on the young. In the dry summer, the ponds go and animals gather at those left (inference). This fits the raptors' `breeds: WET`, `peak: SPRING`, if the dunes' wet season is winter-spring, not summer |
| Floodplain (Yellow Cat) | **Strongly seasonal, possibly monsoonal** rain with very dry spells. Vertisols swelled when wet and cracked up to 1.5 m deep when dry. The floodplain turned into shallow lakes in the rains (Joeckel et al., University of Nebraska). A short wet season, wildfires, waterlogged bogs (summary in the Utahraptor's literature). A *Gastonia* bonebed formed in a drought or flood | **Monsoon rain** (summer, by what a monsoon is: inference) → lakes and a flush of ferns and horsetails → iguanodont, Gastonia and sauropod herds spread → Utahraptor hunts them. **Dry season** → cracked ground and fire; herds gather at the last water, and some die there → carrion for the Utahraptor and the crocodyliforms. The clearest real case of "rain → plants → herbivores → raptors" of the three |
| Fern forest (Yixian) | Humid, with regular dry spells (growth rings in the petrified wood). ~10 °C mean annual temperature and cold winters (Amiot et al. 2011). Eruptions, wildfires and lake gas killed animals in masses | Growth in a wet summer, slow in a dry spell. The game's forest is warmer than the real one: in its winter, plants rest, *Ginkgo* drops its leaves (inference) |
| Crags | - | As the dunes, but colder (game) |

## Sources

Papers and museum pages, as read for this report:

- Hone, Tsuihiji, Watabe & Tsogtbaatr 2012. Pterosaurs as a food source for small dromaeosaurs. *Palaeogeography, Palaeoclimatology, Palaeoecology*. https://doi.org/10.1016/j.palaeo.2012.02.021; summary https://www.smithsonianmag.com/science-nature/a-dinosaurs-pterosaur-lunch-114418691/
- Hone et al. 2010. Tooth-marked *Protoceratops* at Bayan Mandahu. *Palaeogeography, Palaeoclimatology, Palaeoecology*. Summary: https://www.livescience.com/11006-velociraptor-frozen-time-scavenging-larger-dinosaur.html
- The Fighting Dinosaurs: Barsbold 1974, Osmólska 1993, Unwin, Perle & Trueman 1995, Carpenter 1998, as summarised at https://en.wikipedia.org/wiki/Fighting_Dinosaurs
- *Velociraptor*: mass, bite force, the "raptor prey restraint" model (Fowler et al. 2011, *PLoS ONE*, https://doi.org/10.1371/journal.pone.0028964), sociality, *V. osmolskae*. https://en.wikipedia.org/wiki/Velociraptor
- Schmitz & Motani 2011. Nocturnality in dinosaurs inferred from scleral ring and orbit morphology. *Science*. https://doi.org/10.1126/science.1200043
- Frederickson, Engel & Cifelli 2020. Ontogenetic dietary shifts in *Deinonychus*. *Palaeogeography, Palaeoclimatology, Palaeoecology*. Summary: https://edition.cnn.com/2020/05/08/us/raptor-pack-hunting-questions-scn-trnd and https://shareok.org/items/cb2a83a7-5bfe-403a-b70e-a45a73fec48c
- Li et al. 2008. Behavioral implications of dromaeosaurid trackways (Shandong). *Naturwissenschaften*. https://doi.org/10.1007/s00114-007-0310-7
- Roach & Brinkman 2007. A reevaluation of cooperative pack hunting and gregariousness in *Deinonychus*. *Bulletin of the Peabody Museum of Natural History* 48.
- *Utahraptor*: size estimates, the Yellow Cat's age, paleoecology. https://en.wikipedia.org/wiki/Utahraptor
- Utah Geological Survey, the Utahraptor megablock. https://geology.utah.gov/popular/dinosaurs-fossils/megablock
- The Djadokhta Formation: environment, fauna, abundance (Jerzykiewicz et al. 1993; Dashzeveg et al. 2005; Longrich 2010; Naugolnykh 2016). https://en.wikipedia.org/wiki/Djadochta_Formation; Dashzeveg et al. 2005, *American Museum Novitates* 3498, https://bioone.org/journals/american-museum-novitates/volume-2005/issue-3498/0003-0082(2005)498[0001:NSSDEA]2.0.CO;2/New-Stratigraphic-Subdivision-Depositional-Environment-and-Age-Estimate-for-the/10.1206/0003-0082(2005)498[0001:NSSDEA]2.0.CO;2.full
- Naugolnykh 2016. Upper Cretaceous paleosols of the Bain-Dzak section, southern Mongolia. *Paleontological Journal* 50. https://doi.org/10.1134/S0031030116120133
- Saneyoshi et al. 2011. Trace fossils on dinosaur bones from Gobi eolian deposits. *Palaeogeography, Palaeoclimatology, Palaeoecology* 311. https://doi.org/10.1016/j.palaeo.2011.07.024; summary https://www.smithsonianmag.com/science-nature/eaters-of-the-dinosaur-dead-45672019/
- *Protoceratops*: size, herds, nests, soft eggs (Norell et al. 2020, *Nature*, https://doi.org/10.1038/s41586-020-2412-8), diet. https://en.wikipedia.org/wiki/Protoceratops
- *Pinacosaurus*: juvenile groups (Currie et al. 2011, *Acta Palaeontologica Polonica*, https://app.pan.pl/article/item/app20100055.html). https://en.wikipedia.org/wiki/Pinacosaurus
- *Citipati*: brooding (Norell et al. 1995; Norell et al. 2018, *American Museum Novitates* 3899, https://bioone.org/journals/american-museum-novitates/volume-2018/issue-3899/3899.1/A-Second-Specimen-of-Citipati-Osmolskae-Associated-With-a-Nest/10.1206/3899.1.full). https://en.wikipedia.org/wiki/Citipati
- *Kryptobaatar* (Kielan-Jaworowska et al. 2003). https://en.wikipedia.org/wiki/Kryptobaatar
- Choiniere et al. 2021. Evolution of vision and hearing modalities in theropod dinosaurs (*Shuvuuia* nocturnal). *Science* 372.
- The Cedar Mountain Formation: members, ages, fauna tables. https://en.wikipedia.org/wiki/Cedar_Mountain_Formation; Kirkland et al. 1999, distribution of vertebrate faunas, https://www.miketaylor.org.uk/tmp/PDF/Kirkland%20et%20al%201999%20-%20distrib%20of%20Cedar%20Mtn%20Fm%20vert%20faunas.pdf
- Joeckel et al. Yellow Cat paleosols and monsoonal climate. University of Nebraska: https://research.unl.edu/blog/soil-study-lends-clues-to-ancient-climate; Geological Society, Berriasian-Hauterivian chronostratigraphy: https://geolsoc.figshare.com/collections/Chronostratigraphy_and_terrestrial_palaeoclimatology_of_Berriasian_Hauterivian_Strata_of_the_Cedar_Mountain_Formation_Utah_USA/4697138
- *Gastonia*. https://en.wikipedia.org/wiki/Gastonia_(dinosaur)
- McDonald et al. 2010. New basal iguanodonts from the Cedar Mountain Formation (*Iguanacolossus*, *Hippodraco*). *PLoS ONE*. https://pmc.ncbi.nlm.nih.gov/articles/PMC2989904
- A mid-Cretaceous angiosperm macroflora from the Cedar Mountain Formation (Mussentuchit). *Journal of Paleontology* 2016. https://www.cambridge.org/core/product/03B8EE5F8918C6206DDDBFEB9DDE3ACA
- The Yixian Formation: age, climate, flora, fauna. https://en.wikipedia.org/wiki/Yixian_Formation; Zhou 2006, Evolutionary radiation of the Jehol Biota, *Geological Journal*, https://zendy.io/title/10.1002/gj.1045
- Amiot et al. 2011. Oxygen isotopes of East Asian dinosaurs reveal exceptionally cold Early Cretaceous climates. *PNAS*. https://doi.org/10.1073/pnas.1011369108
- *Repenomamus* (Hu et al. 2005, *Nature*, https://doi.org/10.1038/nature03102; the 2023 *Scientific Reports* find). https://en.wikipedia.org/wiki/Repenomamus
- *Microraptor* gut contents (O'Connor et al. 2019, *Current Biology*). https://www.nationalgeographic.com/science/article/new-fossil-lizard-found-inside-microraptor-dinosaur
- *Sinocalliopteryx* gut contents (Xing et al. 2012, *PLoS ONE*). https://www.smithsonianmag.com/science-nature/stomach-contents-preserve-sinocalliopteryx-snacks-23453368/
- Zhao, Benton, Xu & Sander 2014. Juvenile-only clusters of *Psittacosaurus*. *Acta Palaeontologica Polonica*. https://www.doi.org/10.4202/APP.2012.0128; gastroliths in hatchlings (Wang et al. 2025): https://www.eurekalert.org/news-releases/1121542
- Xu et al. 2012. A gigantic feathered dinosaur from the Lower Cretaceous of China (*Yutyrannus*). *Nature*. https://doi.org/10.1038/nature10906
- Hummel et al. 2008. In vitro digestibility of fern and gymnosperm foliage: implications for sauropod feeding ecology and diet selection. *Proceedings of the Royal Society B* 275. https://pmc.ncbi.nlm.nih.gov/articles/PMC2600911
- Witton & Naish 2008. A reappraisal of azhdarchid pterosaur functional morphology and paleoecology. *PLoS ONE*. https://doi.org/10.1371/journal.pone.0002271; and their 2015 reply, *Acta Palaeontologica Polonica*, https://www.app.pan.pl/article/item/app000052013.html
- Naish & Witton 2017. Neck biomechanics indicate that giant Transylvanian azhdarchid pterosaurs were short-necked arch predators. *PeerJ*. https://peerj.com/articles/2908/
- Wiemann et al. 2022. Fossil biomolecules reveal an avian metabolism in the ancestral dinosaur. *Nature* 606. https://www.nature.com/articles/s41586-022-04770-6
- Grady et al. 2014. Evidence for mesothermy in dinosaurs. *Science* 344. https://experts.arizona.edu/en/publications/evidence-for-mesothermy-in-dinosaurs/
- Nagy, Girard & Brown 1999. Energetics of free-ranging mammals, reptiles, and birds. *Annual Review of Nutrition* 19. https://doi.org/10.1146/annurev.nutr.19.1.247. The intercepts used here are as commonly cited from it; I couldn't reread them in the paper itself, so check them before the numbers go into a table.
- Wu, You & Li 2018. Dinosaur-associated Poaceae epidermis and phytoliths from the Early Cretaceous of China. *National Science Review* 5.
- Prasad et al. 2005. Dinosaur coprolites and the early evolution of grasses and grazers. *Science* 310. https://doi.org/10.1126/science.1118806
- The Paleobiology Database: genera per formation, read for the counts above. Djadokhta: https://paleobiodb.org/data1.2/occs/taxa.txt?formation=Djadokhta&show=size&rank=genus; Yellow Cat: https://paleobiodb.org/data1.2/occs/taxa.txt?formation=Cedar%20Mountain&member=Yellow%20Cat&rank=genus; Yixian: https://paleobiodb.org/data1.2/occs/taxa.txt?formation=Yixian&rank=genus&show=class
