# Fossils, DNA and cloning

A design for a late-game loop: find fossils, dig them out, prepare them, read their genome, make
an egg, hatch it and raise it. Research and design only; nothing here is built. Written
2026-11-06 from Ronald's idea note (`notes/ideas/fossils-and-dna.md`, on
`chihuahua-papillon-model` at 4574fed4).

Ronald's words: "I used to play Zoo Tycoon, and I kind of liked the sort of DNA thing that there
was. Like, it's very rudimentary, but I think if we could do something similar, but give it more
thought behind it ... for more like the late game where you can start cloning or doing more cool
things with dinosaurs ... we would probably also need to sort of add fossils and stuff to the world
as well."

## In short

- **The loop:** spot fossils on rock faces (the rock tells you which ancient place, and so which
  species) → brush them out → prepare them at a bench → read them at a sequencer into a **genome
  vial** (a % per species, plus the colour and build alleles the finds have shown) → at 50% make an
  egg at a synthesizer, the gap filled from a living relative → incubate it with the egg system
  as it is → it hatches tame and imprinted → raise and breed it.
- **Why it can go deeper than Zoo Tycoon cheaply:** the game already has a real genome per animal
  (33 slots of colours, patterns, build and temperament), gene pools per place, breeding that
  passes genes on, eggs that need warmth, the species' real formations in their rows, rock strata
  with ores in their faces, and amber.
- **The catch:** the dinosaurs already live behind the amber gateway. So cloning gives what the
  gateway can't: species alive nowhere (the ice age first, with real DNA), animals designed from
  the alleles you found, animals tame from birth in your base, and a museum and journal.
- **The fiction (Ronald, 11-06: "Jurassic Park style"):** DNA survives in fossils and in amber,
  and the gaps are filled from a living relative. Amber is a main source: blood-fed insects in
  amber are the richest dinosaur find, and the amber players already mine for the gateway can
  hold them.
- **Against grind:** the vial is never used up, failed eggs cost only materials, the rock names
  the species, half your finds go to your target species, 50% in about a dozen finds, a hatch is
  certain at 100%, three core stations.
- **Late game:** digging and the museum from the Stone and Bronze Ages; the lab in the Electric
  Age, where `notes/PROGRESSION.md` already puts "Genetics machines".
- **First build:** fossils in the world with a brush and a display (phase 1), then the plain lab
  end to end for species already in the game (phase 2).
- **Decided** (Ronald, 11-06, section 11): Jurassic Park fiction; the ice age first (dire wolf,
  Irish elk), T. rex as the late capstone; clones under 75% can't breed; every other
  recommendation taken. Phase 1's build brief: `notes/briefs/Fossils phase 1.md`.

## 1. How other games do it

Sources inline. A few wikis wouldn't load; claims resting only on guides are marked
"unverified".

### Zoo Tycoon: Dinosaur Digs (2002)

- **No fossils and no DNA.** Dinosaurs are bought as eggs; a scientist (a new keeper) incubates
  and hatches them. Some species unlock through research (T. rex, Apatosaurus, the woolly
  mammoth). A recovery team catches escapees.
  ([Wikipedia](https://en.wikipedia.org/wiki/Zoo_Tycoon:_Dinosaur_Digs),
  [ZTWildlife wiki](https://ztwildlife.fandom.com/wiki/Zoo_Tycoon:_Dinosaur_Digs))
- Reviews: a novelty bolted onto the zoo game, few new systems (Metacritic 67).
- **The dig and the DNA Ronald remembers are probably Zoo Tycoon 2: Extinct Animals.**

### Zoo Tycoon 2: Extinct Animals (2007)

1. Build the Extinct Animal Research Lab.
2. Walk the zoo in first person with a sonar gun that turns green near buried bones, and dig.
   Better sonar and more species come with the zoo's star rating. A paleontologist can search for
   you.
3. In the lab, fit the bones together like a jigsaw.
4. A short DNA minigame (pressing prompted keys), then clone it, normal or "Super" (what Super
   changes: unverified).
([Gameswelt review](https://www.gameswelt.de/zoo-tycoon-2/test/wiederbelebung-der-dinosaurier-2688),
[Worthplaying](https://worthplaying.com/game/2836-zoo-tycoon-2-extinct-animals/))

Liked: the minigames broke up running the zoo, the first few times. Hated: no challenge, and
annoying by the tenth animal. **Lesson:** a detector plus digging by hand fits a first-person
block game. Assembling a skeleton is a great moment once per species, not every time.

### Jurassic Park: Operation Genesis (2003)

- Fossil teams go to 9 dig sites, each with three species. Sites wear down through 6 grades from
  "excellent" to "exhausted", and an exhausted one still gives a little.
- **50% DNA to make a species; more DNA gives a longer life, 100% the full lifespan.**
- Amber is a rare jackpot with a lot of DNA, of an unknown species.
([Wikipedia](https://en.wikipedia.org/wiki/Jurassic_Park:_Operation_Genesis))

**Lesson:** the 50% unlock, with every point past it still worth something. Sites that wear down
but never die keep players moving.

### Jurassic World Evolution 1, 2 and 3

- **The loop:** research opens dig sites; the Expedition Centre sends a team that returns fossils,
  minerals and treasure; the Fossil Centre extracts a fossil's genome share by its quality, or
  sells it. Duplicates past 100% sell automatically. Selling funds the next dig.
  ([JWE wiki](https://jurassicworld-evolution.fandom.com/wiki/Expedition_item),
  [TechRaptor](https://techraptor.net/gaming/guides/jurassic-world-evolution-2-how-to-unlock-all-dinosaurs-sandbox-mode))
- **50% genome to incubate. Viability starts at the genome %, and each gene mod lowers it**
  (players' reports, [Steam](https://steamcommunity.com/app/648350/discussions/1/1697174779859363693)).
- **Gene mods** change attack, defence, lifespan, resilience and looks; slots open with genome %
  ([Gamepressure](https://gamepressure.com/jurassic-world-evolution/genome-modifications/z9b0eb)).
  Hybrids are unlocked by the story, not designed by players.
- **JWE3 (2025)** added breeding in nests with inherited traits and markings, and control over
  sex and looks at synthesis. Breeding with inherited looks was its most praised feature (92% on
  Steam at launch, against 86% and 90% for the first two).
  ([PCGamesN](https://www.pcgamesn.com/jurassic-world-evolution-3/steam-release-reception),
  [allthings.how](https://allthings.how/jurassic-world-evolution-3-breeding-guide-nests-genes-juveniles))
- Hated: IGN called unlocking genes "mandatory robotic actions"; gene editing wasn't freeform; the
  simulation is shallow, and waiting felt like a mobile game
  ([Wikipedia](https://en.wikipedia.org/wiki/Jurassic_World_Evolution)). Players hated losing a
  costly, heavily modded egg to a failure roll
  ([Steam](https://steamcommunity.com/app/648350/discussions/1/1743357605574421053)).

**Lesson:** the 50% shape works. Make each point past it visibly matter. Keep failure cheap. And
the biggest lift came from breeding with inherited looks, which chiacraft's genome already does.

### Ark: Survival Evolved and Ascended

Checked against [ark.wiki.gg](https://ark.wiki.gg):
- **No fossils to speak of.** "Dinosaur Bone" came only from Fossil Piles in two past events and
  can't be had now ([Fossil Pile](https://ark.wiki.gg/wiki/Fossil_Pile)). No DNA-from-fossils loop.
- **The Tek Cloning Chamber** copies a living tame for Element Shards and power: cost by species
  and level, time 40 minutes to about 4 days; imprint resets; a power cut loses everything
  ([Cloning Chamber](https://ark.wiki.gg/wiki/Cloning_Chamber)). Players: "No dodo is worth 38
  element"; worth it only to copy a perfect bred animal
  ([Steam](https://steamcommunity.com/app/346110/discussions/0/1290691308587228764)).
- **Breeding:** each stat from the stronger parent 55% of the time (as ours); 3 mutation rolls at
  2.5% per baby, each +2 levels and a colour change; no new mutations past a counter of 20. Eggs
  incubate only in a temperature range (raptor 20-28 °C, rex 32-34 °C), and imprinting care adds
  up to 20% stats ([Breeding](https://ark.wiki.gg/wiki/Breeding),
  [Mutations](https://ark.wiki.gg/wiki/Mutations)).
- **Traits** (Ascended, a paid add-on): one random trait per wild creature, not inherited; a Gene
  Scanner extracts it to implant in a young one ([Traits](https://ark.wiki.gg/wiki/Traits)).
  Called pay-to-win.
- Hated: mutation stacking takes weeks and spreadsheets.

**Lesson:** temperature-gated eggs fit our temperature system. Copying a tame is wanted, but not
if it costs more than catching a new one.

### Fossils and Archaeology Revival (Minecraft mod)

1. Fossil blocks in stone and sandstone drop bio-fossils (and bones, skulls, plant fossils, relic
   scraps); deepslate gives Paleozoic shale fossils; permafrost gives frozen meat.
2. A Sifter turns sand and gravel into finds, rarely amber.
3. The Analyzer turns a bio-fossil into DNA (with "a high chance" of junk), frozen meat into
   mammal DNA, dino meat into DNA every time.
4. The Culture Vat makes an egg or embryo from DNA and "bio-goo"; 10% of the time it breaks and
   spawns a "Failuresaurus".
5. The egg hatches by a heat source and imprints on whoever is near.
([fossilsarcheology wiki](https://fossilsarcheology.wiki.gg/wiki/Getting_Started_(Guide)),
[Analyzer](https://fossilsarcheology.wiki.gg/wiki/Analyzer),
[Culture Vat](https://fossilsarcheology.wiki.gg/wiki/Culture_Vat))

About 9.4 million downloads. **JurassiCraft**, by contrast, chains about 9 machines (cleaning
station, grinder, sequencer, combinator, synthesizer, embryo machine, calcifier, incubator...),
and even its guides call that a chore
([holy.gg](https://www.holy.gg/en/post/jurassicraft-dinosaur-mod-minecraft)). **Lesson:** fossils
where players mine anyway, and a short chain of machines.

### Minecraft's archaeology (1.20)

- Suspicious sand and gravel spawn only in some structures. Hold the brush on one: 4 dust stages,
  4.8 seconds. Stop and it holds a few seconds, then slowly reverts. Breaking or moving it loses
  the loot. ([Minecraft wiki](https://minecraft.wiki/w/Suspicious_Sand))
- A sniffer egg is 6.7% per block in warm ocean ruins and hatches in 10-20 minutes
  ([Sniffer Egg](https://minecraft.wiki/w/Sniffer_Egg)).
- Liked: the brush's feel and the reveal. Hated: too few sites, buried trail ruins easy to miss,
  and loot that's mostly pottery sherds: "shallow" and "a failed mechanic"
  ([GuruGamer](https://gurugamer.com/pc-console/digging-deeper-why-minecrafts-archaeology-system-needs-a-major-expansion-26467),
  [Mojang feedback](https://feedback.minecraft.net/hc/en-us/community/posts/13162717760013/comments/13540611804429)).

**Lesson:** the brush is the right verb. A find must feed something, and dig spots belong where
players already go.

### Prehistoric Kingdom

Dig teams at sites around the world bring back DNA that fills a species' genome; a full genome
makes the animal in the nursery; a genetics lab sets colour and sex; most species breed. Early
access, 91% positive on Steam.
([Wikipedia](https://en.wikipedia.org/wiki/Prehistoric_Kingdom),
[Steam](https://store.steampowered.com/app/666150)) **Lesson:** looks genetics are cheap and
loved; keep them apart from stat grind.

### Others worth a line

- **Stardew Valley:** fossils from artifact spots and bone nodes fill a museum display; a
  dinosaur egg (about 0.6% per artifact spot) hatches into a dinosaur that lays more. Treasure
  Totems later eased the low odds.
  ([TheGamer](https://www.thegamer.com/stardew-valley-ginger-island-fossils-location-guide/))
  **Lesson:** a hatched extinct animal that keeps giving is a great payoff; low odds need a
  bad-luck fix.
- **Terraria:** desert fossil through the Extractinator, a Sturdy Fossil 10% of the time; it can be
  fed from a chest by wire ([Extractinator](https://terraria.wiki.gg/wiki/Extractinator)).
  **Lesson:** let bulk steps be automated.
- **No Man's Sky: Relics (2025):** dig bones out of the ground, assemble skeletons any way you
  like for a base museum; digging can wake guardians at the rarest finds
  ([official](https://www.nomanssky.com/2025/03/no-mans-sky-relics/)). **Lesson:** a skeleton on
  display is a goal in itself.
- **Jurassic World Alive (mobile):** random DNA per fusion, disliked
  ([GamePress](https://gamepress.gg/jurassicworldalive/node/208001)). **Lesson:** no random
  roll on top of a grind currency.

### Across the games

| Works | Players hate |
|---|---|
| A threshold (50%), then every point past it matters (JPOG, JWE) | Long machine chains (JurassiCraft's 9) and "mandatory robotic actions" (JWE) |
| Digging as a physical act with a detector or careful reveal (ZT2, Minecraft, NMS) | A random roll on top of a grind (JWA, JWE's hatch failure after a big spend) |
| Rare jackpots (JPOG amber, sniffer eggs, Stardew's egg) | Cloning that costs more than catching (Ark) |
| Extras turned into something: a display, a museum (NMS, Stardew) | Minigames repeated for every animal (ZT2) |
| Eggs that need warmth (Ark, FA:R, Minecraft) | Digs with no payoff, at few, hidden sites (Minecraft) |
| Breeding with inherited looks (JWE3's best-loved feature) | Waiting with nothing to do; multi-day timers (JWE, Ark) |
| A short chain: mine, analyse, grow (FA:R) | Very low odds with no bad-luck help (Stardew before totems) |

## 2. The real science

For flavour, and to know where the game bends the truth. Sources inline.

### DNA doesn't last

- **The half-life.** Allentoft et al. (2012) dated 158 moa leg bones from 600 to 8,000 years old.
  DNA in them lost half its strands about every 521 years, in ground at about 13 °C. Age explained
  only 39% of the differences; burial explained much of the rest.
  ([Scientific American](https://www.scientificamerican.com/article/dinos-dna-demise-genetic-material-512-year-half-life/),
  [The Conversation](https://theconversation.com/moa-bones-reveal-dna-half-life-but-jurassic-park-remains-fiction-10067))
- **Cold keeps it.** At −5 °C the strands would still be too short to read after about 1.5 million
  years, and every bond broken by 6.8 million. Real finds have since gone further.
- **The oldest DNA:**
  - Mammoth teeth from Siberian permafrost, 0.7 to about 1.2 million years old (van der Valk et
    al., *Nature* 2021) ([Nature](https://media.nature.com/original/magazine-assets/d41586-021-00348-w/d41586-021-00348-w.pdf)).
  - Sediment at Kap København, Greenland, about 2 million years old: an open forest with
    mastodons and reindeer, its DNA stuck to mineral grains (Kjær et al., *Nature* 2022)
    ([GEUS](https://pub.geus.dk/en/publications/a-2-million-year-old-ecosystem-in-greenland-uncovered-by-environm/)).
- **The first extinct animal's DNA** was the quagga's, from dried muscle on a museum skin, 1984
  ([PubMed](https://pubmed.ncbi.nlm.nih.gov/6504142/)).
- **So:** ice-age animals (under about a million years, best frozen) can give real DNA. Anything
  from the age of dinosaurs, 66 million years and older, can't.

### What dinosaur bones do keep

- **Soft tissue and collagen.** Schweitzer's *T. rex* (2005) gave flexible, vessel-like tubes.
  *Brachylophosaurus* (2009) gave collagen fragments that group it with birds. *Lufengosaurus*,
  195 million years old, kept collagen in its rib canals (2017). *Edmontosaurus* collagen was
  confirmed again in 2025.
  ([PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC4771714),
  [U of T](https://www.utoronto.ca/news/good-ribbance-u-t-researcher-finds-dino-rib-bones-reveal-remnants-195-million-year-old-protein),
  [Sci.News](https://www.sci.news/paleontology/original-collagen-edmontosaurus-fossil-bone-13631.html))
- **A disputed claim.** *Hypacrosaurus* cartilage (2020) showed cell nuclei that took DNA stains.
  Critics say nothing shows the stain wasn't from microbes
  ([The Conversation](https://theconversation.com/has-dinosaur-dna-been-found-an-expert-explains-what-we-really-know-133017)).
- **Colour.** Pigment bodies (melanosomes) keep their shapes, and the shapes tell the colour.
  *Sinosauropteryx* had a ginger and white striped tail and a bandit mask; *Psittacosaurus* was
  dark above and pale below, as forest animals are; *Borealopelta*, an armoured nodosaur, was
  reddish-brown and countershaded, so even tonne-weight armoured animals were hunted.
  ([The Conversation](https://theconversation.com/we-discovered-this-dinosaur-had-stripes-and-that-tells-us-a-lot-about-how-it-lived-86170),
  [Bristol](https://dinocolour.blogs.bristol.ac.uk/?p=380),
  [ScienceDaily](https://www.sciencedaily.com/releases/2017/08/170803122725.htm))
- **Feathers.** Velociraptor had quill knobs on its forearm, so wing feathers (Turner et al.,
  *Science* 2007) ([Phys.org](https://phys.org/news109516799.html)).
- **Proteins from enamel** now reach 21-24 million years (a rhino from the Canadian Arctic, 2025)
  ([York](https://www.york.ac.uk/news-and-events/news/2025/research/rhino-tooth-protein-evolution/)).
- **So:** a dinosaur find gives shape, colour, feathers, some protein and family ties. Enough to
  guide a reconstruction, never enough to clone.

### Amber

- A 1993 *Nature* paper claimed weevil DNA from 120-million-year-old amber. Nobody could repeat
  it; modern DNA had got in. Even resin a few decades old held no readable DNA (Penney et al.
  2013). ([PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC1688388),
  [Manchester](https://www.manchester.ac.uk/about/news/the-final-nail-in-the-jurassic-park-coffin/))
- What amber does keep is shape in three dimensions. A 99-million-year-old piece of Burmese amber
  holds a young feathered dinosaur's tail, eight bones and all (Xing et al. 2016)
  ([National Geographic](https://www.nationalgeographic.com/animals/article/feathered-dinosaur-tail-amber-theropod-myanmar-burma-cretaceous)).
- **So:** amber gives feathers, scales and insects, and their colour. Never DNA.

### Bringing animals back, today

- **Cloning needs living cells.** The black-footed ferret Elizabeth Ann (2020) was cloned from
  cells frozen in 1988; she couldn't breed. Przewalski's horse clones came from cells frozen in
  1980. The bucardo, cloned in 2003 from the last one's skin, died within minutes of a lung defect,
  after some 200 embryos and 7 pregnancies.
  ([CBS](https://www.cbsnews.com/news/endangered-ferrets-cloned-animal-frozen-1980s),
  [National Geographic](https://www.nationalgeographic.com/science/article/news-bucardo-pyrenean-ibex-deextinction-cloning))
- **De-extinction means editing a living relative.** Colossal Biosciences (mostly company claims,
  not yet peer-reviewed):
  - Woolly mice (March 2025): 8 edits in 7 mouse genes, for long wavy hair.
  - "Dire wolves" Romulus, Remus and Khaleesi (born October 2024 and January 2025): grey wolf
    cells with about 20 edits in 14 genes, carried by dog mothers. Five dire-wolf variants were
    left out because they cause deafness or blindness. Colossal's own chief scientist later said
    "our animals are gray wolves".
  - The mammoth: Asian elephant stem cells (2024), aiming at a calf from an elephant surrogate by
    late 2028.
  - The thylacine: over 300 edits in a dunnart cell line; the dunnart as surrogate.
  - The dodo: pigeon germ cells grown in the lab (2025), the Nicobar pigeon as the source genome,
    gene-edited chickens as surrogates.
  - The South Island giant moa (2025), with the Ngāi Tahu Research Centre.
  ([Colossal](https://colossal.com/the-return-of-the-dire-wolf/),
  [Live Science](https://www.livescience.com/animals/extinct-species/our-animals-are-gray-wolves-colossal-didnt-de-extinct-dire-wolves-chief-scientist-clarifies),
  [The Register](https://www.theregister.com/2025/03/04/us_scientists_woolly_mouse_genes/),
  [SingularityHub](https://singularityhub.com/2024/03/12/colossal-creates-elephant-stem-cells-for-the-first-time-in-quest-to-revive-the-woolly-mammoth/))
- **Back-breeding** selects old traits out of living animals over generations: the Tauros
  programme toward the aurochs, the Quagga Project toward the quagga (since 1987). Look-alikes,
  as their founders say.
  ([Rewilding Europe](https://rewildingeurope.com/tauros-programme/),
  [Wikipedia](https://en.wikipedia.org/wiki/Quagga_Project))
- **The dinosaur path real scientists talk about** starts from a bird. Bhullar and Abzhanov (2015)
  turned chicken embryos' beaks back toward a snout by blocking two signals; none hatched. Jack
  Horner's "chickenosaurus" never got its tail.
  ([Yale](https://news.yale.edu/2015/05/12/tweaking-beak-retracing-bird-s-beak-its-dinosaur-origins-laboratory))
  Birds are living theropod dinosaurs; crocodilians are the other living archosaurs
  ([UCMP](https://www.ucmp.berkeley.edu/diapsids/archosy.html)).
- **So:** the honest version of cloning is a reconstruction: a relative's genome edited toward the
  evidence, carried by a surrogate, labelled as what it is.

### How fossils form and where they are

- Bones become fossils when buried fast: minerals fill their pores (permineralisation) or replace
  them; a dissolved bone leaves a mould, filled later as a cast; footprints, burrows and dung are
  trace fossils ([TMDC](https://docs.tmdinosaurcenter.org/staff/prep-lab-procedures/how-fossils-are-formed-fossilization-process)).
- They lie in sedimentary rock: sandstone, mudstone, shale, limestone. Never in igneous rock.
  Badlands and deserts show them because erosion keeps cutting bare rock with no soil over it.
- **The Djadokhta Formation** in the Gobi, 75-71 million years old, is wind-blown dune sandstone.
  Many of its animals were buried alive when wet dunes collapsed. The Flaming Cliffs gave the
  first confirmed dinosaur eggs (1920s). The "Fighting Dinosaurs", a Velociraptor and a
  Protoceratops locked together, were found there in 1971. Citipati's "Big Mamma" sits brooding on
  her nest; the "egg thief" was the eggs' parent. Protoceratops eggs were soft and leathery
  (*Nature* 2020).
  ([Wikipedia](https://en.wikipedia.org/wiki/Djadochta_Formation),
  [AMNH](https://www.amnh.org/explore/ology/ology-cards/038-fighting-dinosaurs?view=modal),
  [Sci.News](https://www.sci.news/paleontology/eggs-earliest-dinosaurs-soft-leathery-shells-08549.html))
- **Exceptional sites:** the Burgess Shale (about 508 million years, soft bodies), the La Brea tar
  pits (over 5.5 million ice-age bones)
  ([ROM](https://www.rom.on.ca/news-releases/fossil-paradise-discovery-burgess-shale-charles-d-walcott),
  [IUGS](https://iugs-geoheritage.org/geoheritage_sites/la-brea-tar-pits-rancho-la-brea-late-quaternary-asphalt-seeps-and-paleontological-site/)).
- **Permafrost mummies:** the mammoth calves Lyuba (about 41,800 years, Yamal, 2007), Yuka (about
  28,000 years; her nuclei stirred in mouse eggs in 2019 but never divided), Nun cho ga (Klondike,
  2022) and Yana (2024); the wolf pup Zhùr (about 57,000 years); cave lion cubs. A 52,000-year-old
  mammoth's skin kept its chromosomes' 3D shape (*Cell* 2024). Thawing river banks and mine cuts
  expose them.
  ([Australian Museum](https://australian.museum/learn/news/blog/the-story-of-baby-lyuba/),
  [ScienceAlert](https://www.sciencealert.com/scientists-have-reawakened-nuclei-from-an-extinct-mammoth-who-died-28-000-years-ago),
  [NBC](https://www.nbcnews.com/news/world/mammoth-baby-50000-years-siberia-permafrost-rcna184325),
  [ScienceDaily](https://sciencedaily.com/releases/2024/07/240711111343.htm))
- **Digging one out:** in the field, tissue over the bone, then burlap soaked in plaster (a field
  jacket), then undercut and lift. In the lab, an air scribe and picks under a microscope, and
  glue to harden fragile bone.
  ([ROM](https://www.rom.on.ca/blogs/summerasaurus-part-iii-jacketing-process),
  [NPS](https://home.nps.gov/joda/learn/nature/fossil-preparation-laboratory.htm))

## 3. What already exists to build on

Most of the loop's hard parts are already in the game. That's the main reason this can go deeper
than Zoo Tycoon without a lot of new machinery.

| Need | What exists | Where |
|---|---|---|
| A genome to fill | Every animal carries a `Genome`: up to 16 loci of its species' own (colours, patterns, size) and 17 core traits (shade, warmth, marks, eye colour, head, neck, tail, legs, girth, horns, temperament), two copies each, 33 slots. Babies take one copy from each parent; 1% of alleles mutate. | `mods/animals/core/src/coats.rs`, `mods/animals/genetics/*.json` |
| Wild gene pools | Each species has a pool per place that evolves (selection by camouflage, drift, mutation 0.2%, migration). | `mods/animals/core/src/evolution.rs` |
| Stats and levels | Nine stats (health, stamina, food, damage, speed, weight, oxygen, torpor, fortitude) with wild and tamed points; levels 1-150. Young take each stat's points from one parent, the stronger 55% of the time. | `species.rs`, `breeding.rs` |
| Eggs | Eggs hold their young from laying. They develop at 25-40 °C; a nest pulls the air toward a brooding parent's 37 °C and halves the time. A placed egg hatches as the placer's: taming by raising. | `mods/animals/src/eggs.rs`, `animals_core::life::EGG_AIR` |
| Per-species eggs | Shell, shape, colour and clutch layout per species, from the fossil record (Citipati's paired rings, Protoceratops' soft buried clutch, the sauropod's round chalk-white eggs). In progress. | session "Eggs: a distinct look per species", `reports/Eggs by species.md` on `claude/eggs-per-species-5f7d28` |
| Where each species lived | Every Cretaceous species' row already names its real formation: the Djadokhta (and Bayan Mandahu) dunes in Mongolia, the Cedar Mountain floodplain in Utah, the Yixian lakes in China. | comments in `mods/animals/core/src/species.rs` |
| A family tree | Each species has a family and the groups above it (`Family { name, above }`). Too shallow for relatives today: birds aren't under Dinosauria. | `species.rs` |
| Rock to put fossils in | Rock faces show their column's rocks: the hot dry land's strata (sandstone, ochre, cream and red sandstone, banded clay) on mesas and canyon walls, slate in the cold, granite high up, limestone on low cliffs, karst towers. Faces already carry ores (coal in banded clay, ironstone in red sandstone). | `base_api::land::{rock_set, strata, face_ore}` |
| Amber | Amber ore in seams (three per cube below y 64 in the overworld, six in the Cretaceous). Amber blocks build the gateway. | `mods/animals/src/gateway.rs` |
| The dinosaurs, alive | The Cretaceous, a dimension behind an amber gateway, holds the raptors, Utahraptor, protoceratops, ankylosaurs, sauropod, iguanodont, citipati and more, alive and breeding. | `mods/animals/src/cretaceous.rs`, `notes/ANIMALS.md` |
| Ages | Tier 4 (Electric and digital) lists "Genetics machines" on the beasts path; tier 2 "breeding lines". | `notes/PROGRESSION.md` (dev) |
| Lore | Star-born ancients built the ley network and gates to their worlds, then fell. Ley lines carry magic; ley nodes on a 512-block grid; a dowsing rod and later ley sight show them. | `notes/PROGRESSION.md`, `vitals::ley` (dev) |
| Study | "Learning by study (examining things fills the journal and opens recipes; the first finder names a species)". | `notes/PROGRESSION.md` |

### The one big question this raises

The dinosaurs are already alive behind a gateway that any Stone Age player can build from amber.
So cloning can't just be "the way to get dinosaurs". In Zoo Tycoon and Jurassic World Evolution
it is the only way; here it isn't. It has to give things the gateway doesn't:

1. **Species alive nowhere.** Animals known only from fossils: the ice-age animals of the
   overworld's cold (with real, honest DNA), and dinosaurs of formations the Cretaceous doesn't
   have. This is what makes a fossil worth finding.
2. **A chosen animal.** The lab lets you pick its colours, build and temperament from the alleles
   your finds have shown, instead of taking what the wild pool gives. Breeding lines start from
   your design.
3. **Your own from the first breath.** A clone hatches imprinted and tame, in your overworld base,
   with no knockout and no trip. Copying a living animal (a prized tame) is the simple version.
4. **Knowledge.** Every species' first fossil fills its journal page and names it; a full genome
   is a deed the tribe shares. Fossils on display make a museum.

The recommended design leans on all four, with 1 and 2 as the heart.

## 4. The fiction: Jurassic Park style (decided)

Ronald chose it on 11-06 ("Jurassic Park style"), over an honest-only version and a ley-line
explanation. So in the game:

- **DNA survives** in fossil bone, teeth, eggs and above all in amber. How much a find gives
  depends on what it is and how well it was kept (its quality), not on real chemistry.
- **Amber is the richest source** for dinosaurs, as in Operation Genesis (a rare jackpot). A
  biting insect that fed on a dinosaur and was caught in resin holds that animal's blood. Amber
  already exists (ore in the rock, the gateway's blocks), so players who mine it for the gateway
  start finding inclusions long before the lab.
- **Gaps are filled from a living relative**, as section 5.5 describes: birds and crocodilians for
  dinosaurs (not frogs: the game has closer relatives, and the choice is part of the play), wolves
  and deer for the ice-age mammals.
- **Ice-age finds** are the easy case: frozen remains give a lot of DNA, as real permafrost
  mummies do. They come first among the new species.
- **The real science (section 2) stays as flavour**: journal pages, item descriptions and names
  can use it (the Djadokhta's buried dunes, Big Mamma on her nest, real melanosome colours), but it
  doesn't set the rules.
- **No ley lines in the explanation.** They play no part in DNA. The wyvern's dragon bones still
  lie at ley nodes (the wyvern is the one fantasy animal), as a place, not a mechanic.
- **Clones are labelled** on their screen: "Cloned: 72% genome, gaps from a spectacled caiman",
  "True-bred", or "Copy of Ash".

## 5. The recommended loop, step by step

Working names throughout. Names for anything invented (the lab's machines)
go through `tools/names/names.py` and `mods/base/lore/lexicon.toml` before they reach the game.

### 5.1 Finding

**Fossil-bearing rock.** One fossil block per host rock, drawn as bone showing in that rock:
`fossil_red_sandstone`, `fossil_ochre_sandstone`, `fossil_banded_clay`, `fossil_limestone`, and
later `permafrost` with remains in it. The rock tells the player which ancient place it comes from,
and so which species can be in it. The species' rows already name their formations:

| Rock in the game | Real formation | Species that can be in it |
|---|---|---|
| red, ochre and cream sandstone (the strata on mesas and canyon walls) | Djadokhta and Bayan Mandahu, Mongolia: wind-blown dune sandstone; animals buried alive by collapsing wet dunes | Velociraptor (both), Protoceratops, Citipati (and nests), Pinacosaurus, Kryptobaatar, Priscagama, Gobipteryx |
| banded clay (the strata's clay layers) | Cedar Mountain, Utah: floodplain mudstone | Utahraptor, Gastonia, Iguanacolossus, Cedarosaurus, the neosuchian, Ceratodus (teeth) |
| limestone (low cliffs, river banks) | Yixian, China: fine lake beds (shale and ash in reality) | Psittacosaurus, Confuciusornis, Lycoptera; feather and skin impressions |
| permafrost (new: frozen ground a few blocks under the tundra and snowy plains, showing in river banks there) | Yukon and Siberian permafrost, where thawing banks expose mummies | ice-age species, once they exist (phase 5) |
| amber ore (exists: seams below y 64 in the overworld, more in the Cretaceous) | Burmese and Baltic amber | inclusions: blood-fed insects (a dinosaur's DNA), feathers, plain insects, plants |
| amber nodules (new, phase 4): lumps in the banded clay's coal seams (real: amber turns up with lignite and coal) | resin from ancient conifer forests | the same inclusions, more often |

Granite, slate and the karst towers hold none. Igneous rock never holds fossils, and the game's
slate is the cold's rock, not a lake bed. That rule is something a player can learn and use.

**Where they show.** Mostly on rock faces, because that is where erosion opens real fossil beds:
badlands, cliffs, canyon walls. They already work this way for ores (`base_api::land::face_ore`:
coal in banded clay, ironstone in red sandstone). Fewer lie buried in the same rocks behind the
face, and those are better preserved (section 5.2).

**Bone beds and skeleton beds.** Fossils come in knots of 1-3 blocks. Rarely, a skeleton bed: a
buried structure of 6-20 fossil blocks in the shape of one animal lying down, with a skull. It's
the find that mounts a full skeleton. The Djadokhta's "Fighting Dinosaurs" (a Velociraptor and a
Protoceratops locked together, found in 1971) is a fitting one-off.

**Nests.** In the red beds, a fossil clutch: eggs in the layout the eggs session found per
species (Citipati's paired rings with the parent's bones over them, the "Big Mamma" pose;
Protoceratops' soft eggs buried together).

**Searching.** Three steps, so a player is never just wandering:
1. The journal says which rock holds which species from the first find on ("Velociraptor: the red
   dune beds").
2. A rock hammer (Stone Age): tapping a face shows fossil blocks within a few blocks behind it as
   a faint outline for a few seconds. Cheap, slow, short reach.
3. A survey staff (tier 3, steam): shows buried fossils and amber within 16 blocks, like the
   dowsing rod's feel.

### 5.2 Digging

- **Quality.** Every find has a quality from 0 to 100. Exposed finds on a face start at 30-70
  (weathered). Buried finds start at 60-100 (protected). Digging in is the trade for better finds.
- **Pick:** breaks the block into bone fragments (quality capped at 20) and rock. Fragments still
  count for something (section 5.4), and grind into bone meal (real: fossil phosphate is mined for
  fertiliser).
- **Brush** (Stone Age: a stick and fibre or feathers): hold use on the block for 4-8 seconds
  (softer rock is faster). Keeps the find's quality. Stopping keeps the progress for a while;
  there's no random breakage. Minecraft's brush is the model, with less waiting.
- **Chisel** (Bronze): twice as fast as the brush, 10 points of quality lower.
- **Plaster jacket** (Bronze: burnt lime or gypsum with cloth or fibre, real practice): needed to
  lift fragile finds: skulls, eggs, nests and impressions. Without one, they break into fragments.
  This is a clear rule, not a dice roll.
- **What comes out:** an unprepared find, "bone in red sandstone". The rock names the place; the
  species shows when it's prepared.

### 5.3 Preparing

- **Preparation bench** (Bronze station; needs a chisel and brush on it). A timed job like the
  furnace, 60-120 seconds. In: an unprepared or jacketed find. Out: the prepared fossil (species,
  part, quality) and some sand or gravel.
- **Air scribe** (Steam, an add-on to the bench, runs on compressed air): half the time, and the
  quality cap rises by 10. An acid bath for limestone matrix later (real practice).
- **Kinds of find:** fragment, bone, tooth or claw, skull, egg, nest, skin or feather impression,
  track (a footprint: journal and display only), coprolite (fossil dung: tells the diet, journal
  only), amber inclusion.
- **The first of a species** opens its journal page and names its finder ("the first finder names
  a species", `notes/PROGRESSION.md`), with some XP.
- **Display.** A fossil stand for one find, a case for a few, and a mounted skeleton from a
  species' full set (skull plus four bones, say). The mounted skeleton is the species' own box
  model in its standing pose, painted as bone (the `blocky-model` skill can repaint a texture).
  That's a museum for little new art.
- **Stacking.** Prepared fossils stack by species, kind and quality band (steps of 10), so a
  player who digs in the Bronze Age doesn't carry a hundred loose items until the Electric Age.

### 5.4 Extracting: from fossil to genome

- **The sequencer** (Electric, tier 4, the "Genetics machines" of the beasts path).
- In: a prepared fossil (consumed). Out: its share added to a **genome vial** for its species.
- **The genome vial** is an item, not a hidden counter. It holds its species, its completeness
  (0-100%) and the alleles it has seen (section 5.5). Two vials of a species merge at the
  sequencer. As an item it can be stored, traded, carried through portals between linked worlds,
  and lost on death like anything else. The journal shows each species' best vial.
- **Making an egg never uses up the vial.** It is knowledge. A failed egg costs materials and
  time, never genome. This is the most important rule against grind.

**What a find adds** (all numbers are starting points, in `animals_core::fossils`):

| Find | Share of the genome | Also |
|---|---|---|
| fragment | 1% | |
| bone | 5% | 1 allele seen |
| tooth or claw | 6% | 1 allele |
| skull | 12% | 2 alleles; the build traits' range (head, horns) |
| fossil egg | 10% | 1 allele; the egg's look for synthesis |
| nest (a clutch) | 15% | 2 alleles |
| skin or feather impression | 0% | 2 colour alleles (real: melanosomes show colour) |
| blood-fed insect in amber | 15% of one species | 2 alleles; the richest dinosaur find |
| feather in amber | 4% | 3 colour alleles |
| plain insect or plant in amber | 0% | journal and display only |
| ice-age hair | 6% | 2 colour alleles |
| ice-age bone or tusk | 8% | 1 allele |
| frozen body (a mummy) | 30% | 4 alleles; the jackpot, as Lyuba and Yuka were |

Share added = the find's share × its quality (0-1), and for ice-age finds × the cold factor:
1.0 in permafrost, 0.5 thawed (a river bank in summer). Amber's quality is high (70-100): resin
seals what it holds.

**Whose blood an insect holds.** Amber found in the Cretaceous holds a species living in that
biome. Amber in the overworld holds a species of the formations, weighted by the target rule
below. The species shows when the amber is cut (phase 1: at once; later at the preparation
bench).

Shares add straight up to 100%. No diminishing returns: a player can count how far they are.

**Bad luck protection.** A find in a rock picks its species from that formation. The journal lets
a player mark one species as their target, and half of all finds in its formation are that
species. So the player who wants a Utahraptor isn't buried in Gastonia bones.

**How long it takes.** An average find is about 6% × 0.7 quality = about 4%. So:
- 50% (the first egg) takes about 10-12 finds, a few good digging trips;
- 100% takes about 22-25 finds;
- a skeleton bed or a nest near a node can jump 15-20% at once.

### 5.5 The genome: completeness, gaps and alleles

The game's genome has 33 slots (section 3). A vial knows two things about them:

**Completeness** decides what is possible, with three clear steps (Zoo Tycoon's percentage, made
to mean something):

| Completeness | What it allows |
|---|---|
| under 50% | Nothing yet. The journal shows the bar. |
| 50-74% | An egg can be made, with the gap filled from a relative. The animal can't breed (sterile). |
| 75-99% | The animal can breed. Its young carry its genes, gap-fill effects included. |
| 100% | No gap: no filler needed, no filler effects. "True-bred" on its screen. |

Sterile below 75% has real roots: cloned animals often fail or can't breed (the bucardo died in
minutes; the cloned ferret Elizabeth Ann couldn't breed). It gives the last stretch of digging a
reason without making it compulsory.

**Alleles seen.** Each find reveals alleles for one or more loci, drawn from the species' gene
file (where every population starts). Colour comes from impressions, amber and hair; build traits
from skulls. A vial lists what it has seen: "Redness: sandy, rufous. Size: 0.35-0.70". More finds
mean more choice at the lab. Loci never seen take the plain form (the first allele, the trait's
mean).

**Gap fillers.** Below 100%, the gap (100% − completeness) is filled from a living relative's
genome. The player takes a sample: a sample kit (a needle and a vial) used on a tamed or
knocked-out animal, or a fresh egg of it. Which relative fits comes from a small family tree in
`animals_core::fossils`, built on the existing `Family` rows:

```
Archosauria
├── Crocodylia: caimans (living); the neosuchian (in the Cretaceous)
└── Dinosauria
    ├── Ornithischia: ceratopsians, ankylosaurs, iguanodonts
    └── Saurischia
        ├── Sauropoda: Cedarosaurus
        └── Theropoda: dromaeosaurs, oviraptorids
            └── Avialae: Confuciusornis, enantiornithines
                └── Aves: every living bird (sparrow, gull, owls, eagle, raven, vulture, hawk)
Mammalia: canids, felids, deer (fillers for ice-age mammals)
```

How far the relative sits from the species sets how strong its effect is and how likely the egg
is to hatch:

| Relative | Example | Effect strength | Hatch chance |
|---|---|---|---|
| same family | grey wolf for a dire wolf | ×0.25 | 95% |
| same order or group | a living Protoceratops for Psittacosaurus; a bird for a dromaeosaur | ×0.5 | 85% |
| same big group | a bird for an ankylosaur (both dinosaurs) | ×1.0 | 70% |
| the other side of the archosaurs | a caiman for any dinosaur | ×1.5 | 55% |
| further | a mammal for a dinosaur | not allowed | |

Hatch chance rises with completeness: the table's figure at 50%, reaching 100% at 100%
completeness, whatever the relative. So a player who finishes a genome never loses an egg. Real
cloning is far worse (the bucardo: about 200 embryos, one birth). The game is kind on purpose:
JWE's players hated losing a costly egg to a roll, and a failed egg here still costs a day and
materials. The lab shows the chance before starting.

**What each kind of filler does.** Scaled by the gap's share and the strength above, so a 90%
genome barely shows its filler and a 50% one shows it plainly:

| Filler | Stats (wild points moved) | Genes | Life |
|---|---|---|---|
| bird | +speed, +stamina, −health | temperament toward skittish; shade lighter; marks finer | grows 10% faster |
| crocodilian | +health, +fortitude, −speed | temperament toward bold; shade darker; longer head | grows 20% slower; lives 15% longer |
| same-family mammal | small shifts toward the relative | toward the relative's colours | none |

The shifts go into the genome itself (the core traits and the young's wild points), so they pass
to the young of fertile clones. Breeding can select them back out over generations, as the real
Quagga Project and Tauros select for lost traits.

**Copying a living animal.** A sample from one of your own tamed animals makes a vial at 100%
with that animal's exact genome: a copy, not a reconstruction. Ark's cloning chamber is the model,
without its resource grind. The cost rises with the animal's level. It gives the prized-tame
backup players ask for, and no new alleles.

### 5.6 Synthesis: making the egg

- **The synthesizer** (Electric, tier 4). In:
  - a genome vial of 50% or more (not used up);
  - a filler sample if under 100% (used up);
  - shell material: bone meal, limestone dust or eggshell;
  - a yolk: any fresh egg, or meat and sugar;
  - power, about one in-game day.
- **The lab screen.** Before starting, the player picks:
  - each seen locus's two alleles from the vial's list;
  - each quantitative trait (size, build, temperament) within the range the finds have shown;
  - the sex.
  Unseen loci stay plain. This is JWE's gene modding, built on our real genome instead of a
  menu of bonuses.
- **Out:** a synthetic egg with its young decided, as a laid egg is (`eggs.rs` `Egg`), drawn
  with its species' egg look.
- **The clone's level.** Not the wild 1-150. It is 10 at 50% completeness up to 40 at 100%, with
  its points spread evenly, then moved by the filler. A clone is designed, not strong. Breeding
  lines make it strong. A copy of a living animal keeps that animal's level and points.

### 5.7 Incubating

Synthetic eggs use the egg system as it is: they develop between 25 and 40 °C, and a nest pulls
the air toward 37 °C and halves the time. Hatch times are the species' own (about 1.4-2 days for
the dinosaurs, 5.5 for the wyvern).
- **The incubator** (Steam, tier 3; an electric one in tier 4 holds four eggs): holds 37 °C
  anywhere and counts as a nest. A fire next to a nest already works for the early game.
- **Mammals need a surrogate**, as real de-extinction does: the embryo goes into a tamed female
  of the filler species, who carries it for the species' gestation. Grey wolf for a dire wolf, red
  deer for an Irish elk, jaguar for a cave lion. For an animal with no living relative in the game
  (the mammoth: there are no elephants), an artificial womb at tier 5.
- **The hatch roll** happens when it would hatch. A failed egg "didn't develop", and the journal
  says why in one line ("filled from a distant relative: try a closer one or more evidence").

### 5.8 Hatching, raising and traits

- It hatches as the placer's own, imprinted, as raised eggs already do. It grows, eats and bonds
  with the existing systems. Nothing new to learn here.
- Its screen says what it is: "Reconstructed: 72%, gaps from a spectacled caiman" or
  "True-bred" or "Copy of Ash".
- **Breeding clones** (75% and up) mixes their genes as any breeding does. The filler's shifts
  are in the genes, so a line drifts back toward the wild form or away from it as the player
  chooses.
- **Few founders.** Clones from one vial are close kin. A real conservation problem; worth a
  mild inbreeding cost later (lower health in the young of close kin) so finds from many places
  matter. Not in the first phases.
- **Hybrids.** Not in the plan (Ronald took the recommendation). Mixing species breaks the
  genome's design: each species has its own loci.
- **The wyvern.** Fantasy, so no fossils in the strata. A late capstone (taken): "dragon bones"
  that lie only at ley nodes in the crags, read by the sequencer like any find.

## 6. Blocks, items, stations and tools

| Thing | Kind | Age | Made from or found | Job |
|---|---|---|---|---|
| fossil blocks (4 rocks, plus permafrost later) | block | found | rock faces, buried near them | the finds |
| skeleton bed, fossil nest | structure | found | rare, in the red beds and clay | big finds |
| bone fragments | item | 0 | a fossil broken with a pick | 1% each; bone meal |
| brush | tool | 0 | stick, fibre or feathers | careful digging |
| rock hammer | tool | 0 | stone head, stick | shows fossils behind a face |
| chisel | tool | 1 | bronze | faster digging |
| plaster jacket | item | 1 | lime or gypsum, cloth or fibre | lifts skulls, eggs, nests, impressions |
| unprepared find | item | 1 | dug | to the bench |
| preparation bench | station | 1 | planks, bronze tools | finds to fossils |
| prepared fossil | item | 1 | the bench | display, journal, the sequencer |
| fossil stand, display case, mounted skeleton | blocks | 1 | wood, glass, a species' full set | the museum |
| air scribe | bench add-on | 3 | steam age | faster, better prep |
| survey staff | tool | 3 | steam | finds buried fossils and amber |
| incubator | machine | 3 | steam; electric version at 4 | holds 37 °C |
| sample kit | tool | 3 | glass, iron needle | samples from living animals |
| sequencer | machine | 4 | electric | fossils into genome vials |
| genome vial | item | 4 | the sequencer | a species' genome record |
| synthesizer | machine | 4 | electric | vial, filler, shell, yolk into an egg |
| artificial womb | machine | 5 | future age | mammals with no living relative |

## 7. Where the numbers live

The shared crates rule (`CLAUDE.md`): numbers in `mods/animals/core` or `vitals`, read by the
mod, checked by tests, shown on a sheet.

- **`animals_core::fossils`** (new): the formations and their species; the find kinds with
  shares, alleles and jacket needs; quality ranges (exposed 30-70, buried 60-100, pick cap 20,
  chisel −10, air scribe +10; amber 70-100); the cold factor; amber's inclusion odds; the thresholds (50, 75, 100); the family
  tree and the relatives table (effect strength, hatch chance); the filler effects; the clone's
  level (10-40); timings (brush 4-8 s, prep 60-120 s, synthesis one day).
- **The share of fossils in a face**: also in `animals_core::fossils`, placed by the animals
  mod's own decoration reading `base_api::land::rock_set` and `strata`, as amber ore is placed by
  animals today. Start at about 0.6% of matching face blocks, in knots of 1-3 (ironstone is 5%,
  gold 0.3%): a canyon wall of 2,000 face blocks shows about a dozen. Checked like
  `face_ores_come_in_their_shares`.
- **The sheet.** `mods/animals/SPECIES.md` gets a fossils part per species: its formation, its
  rocks, its finds' alleles, the closest filler in the game, and how many average finds reach 50%
  and 100%.
- **Tests that tie it together:** every species with fossils has a formation whose rock exists in
  the strata; every fossil species has a filler in the game at "same big group" or closer; 50% is
  reachable in 15 average finds or fewer; a genome made at the lab survives a save as an egg does.
- **Progression tags** (`mods/base/progression/src/content.rs`, dev): every new item, block and
  station with its age and path (beasts), as the brief rule asks.

## 8. Progression and gating

The loop starts early and pays off late. Fossils are found from the first canyon. DNA waits for
the Electric Age. That is Minecraft's archaeology and Stardew's museum early, Zoo Tycoon and JWE
late.

| Age | What the player can do |
|---|---|
| 0 Stone | Spot fossils on faces. Brush them out or break them for fragments and bone meal. Rock hammer. |
| 1 Bronze | Chisel, plaster jackets, preparation bench. Identify species, fill the journal, build displays and a museum. |
| 2 Iron | Bigger digs: skeleton beds, nests (they need jackets and patience). Mounted skeletons. Breeding lines of living animals (already planned). |
| 3 Steam | Air scribe, survey staff, incubator, sample kit. |
| 4 Electric | Sequencer, genome vials, synthesizer. First clones at 50%. The lab screen. |
| 5 Future | Artificial womb (the mammoth). Faster synthesis. |
| 6 Star | The wyvern's dragon bones, if Ronald wants it. |

An age key could ask for a mounted skeleton (the Iron Age's knowledge gate) or a hatched clone
(the Electric Age's beasts gate), which makes the loop part of the main path without forcing every
player deep into it.

## 9. What to avoid

From the games in section 1 and their players:

- **Losing progress to chance.** The vial is never used up. A failed egg costs only materials.
  Fossils never "break" at random while digging; only the pick (a choice) breaks them.
- **Random species from every find.** The rock names the formation; the target rule makes half
  the finds the species you want.
- **A sell-and-repeat economy.** JWE's expeditions turn into a money loop. Here a fossil is worth
  its share and its display, nothing to sell to a market.
- **Long waits with nothing to do.** Prep and synthesis run like the furnace, beside other work.
  The brush takes seconds, not minutes.
- **Endless percentages.** 50% is the goal that matters and is reachable in a few trips. 75% and
  100% are clear steps with clear rewards (breeding, true-bred).
- **Hidden numbers.** The vial shows its percentage and alleles. The lab shows the hatch chance
  and what the filler will shift before starting.
- **A huge menu of fake genes.** The lab edits the genome the game already has (colours, build,
  temperament), not a list of +10% bonuses.
- **Fossils as inventory junk.** They stack by species, kind and quality band, and displays hold
  them.
- **A long machine chain.** The core is three stations: the preparation bench, the sequencer and
  the synthesizer. The incubator is optional (a nest by a fire works), and the rest are upgrades.
  JurassiCraft's nine machines are the warning.
- **Clicking through bulk.** The sequencer takes fossils from the storage network (tier 4 has one),
  so a chest of finds runs on its own, as Terraria lets its Extractinator be fed by wire.
- **Minigames every time.** No puzzle per fossil. The one special moment, mounting a species'
  first skeleton, happens once (ZT2's jigsaw was fun once and tiresome by the tenth).
- **A cloned animal worse than a caught one.** Copying a tame costs less than catching and
  raising a new one of its level, or nobody uses it (Ark's "No dodo is worth 38 element").

## 10. Build phases, smallest playable first

Each phase is playable alone and adds one thing.

1. **Fossils and amber finds in the world.** Fossil blocks in the strata's sandstones, banded clay
   and limestone faces, by formation. Brush and pick. A brushed find drops a fossil item with its
   species; amber ore now and then holds an inclusion (a blood-fed insect names its species). The
   journal notes each species' first find. Placing a find puts it on display. No DNA yet.
   Brief: `notes/briefs/Fossils phase 1.md`.
2. **The lab, plain.** Sequencer, genome vial, synthesizer, incubator, for species already in the
   game (no new models). No lab screen yet: the egg takes the plain wild form. Gap fillers from the
   family tree, with stat shifts. Thresholds 50/75/100. This is the Zoo Tycoon loop, playable end
   to end.
3. **Design.** Alleles seen, the lab screen, impressions and amber feathers for colour, copying a
   living tame with the sample kit.
4. **Digging deeper.** Quality, preparation bench and air scribe, plaster jackets, rock hammer
   and survey staff, skeleton beds and nests, mounted skeletons, amber nodules in the coal seams.
5. **The ice age** (Ronald: "Ice age first"). Permafrost, mummies, surrogate mothers. First
   species: the dire wolf (grey wolf surrogate) and the Irish elk (red deer surrogate), each a new
   model, gene file and species row. The mammoth with the artificial womb after.
6. **The capstones.** T. rex (Ronald: "T. rex as the late capstone"), with Triceratops beside it
   if wanted: Hell Creek's river sandstones and mudstones as a new, rarer bed deep in the strata,
   skeleton beds only, alive nowhere. The wyvern's dragon bones at ley nodes in the crags.

Phase 1 is small and early-game; phases 2-3 are the late-game heart Ronald asked for. Doing 1 then
2 first gets the whole loop in hand quickly; 4 makes it feel deep.

## 11. Decisions

Ronald answered on 11-06 (relayed by Planner):

| # | Question | Answer |
|---|---|---|
| 1 | How the fiction works | "Jurassic Park style": DNA survives in amber and fossils, gaps filled from a living relative. Not the ley-line explanation (section 4). Amber gets a bigger role. |
| 2 | What cloning gives when the dinosaurs live behind the gateway | Taken: species alive nowhere and designed animals as the heart, plus tame from birth and a museum. |
| 3 | Which lost species first | "Ice age first": dire wolf and Irish elk first, T. rex as the late capstone. |
| 4 | Clones under 75% | "Can't breed". |
| 5 | Copying a living tame | Taken: own tames only, cost by level. |
| 6 | Hybrids | Taken: none. |
| 7 | Fossils early, DNA late | Taken: digging and display from the Stone and Bronze Ages, the lab in the Electric Age. |
| 8 | The wyvern's dragon bones | Taken: a late capstone, at ley nodes in the crags. |
| 9 | Clone levels | Taken: 10 at 50% up to 40 at 100%; a copy keeps its animal's level. |

Left for Planner or the worker to decide, with a lead:
- **Insects in amber:** which insect. A biting midge or a mosquito both fit (both are known from
  Cretaceous amber); the name and look go through the art and names tools.
- **Ceratodus teeth and the fish** (lungfish, Lycoptera): fossils of them can wait for phase 2;
  phase 1 may skip fish.
