# Coat genetics

One file per species, named as its row in `src/species.rs` (`dog.json` for every dog breed, the
chihuahua's among them; `raptor.json` and `osmolskae.json` for Velociraptor's two species, the
same loci, different frequencies): the loci that
decide how an animal looks, their alleles in dominance order
(the first one present is the one that shows; a `codominant` locus shows both, a `xlinked` one is
carried on the X, so males have one copy), each allele's frequency in the wild, and quantitative
traits (`quantitative`: two values, the mean shows, babies get one of each parent's with a little
drift). A species with breeds lists each breed's own frequencies under `breeds` (a chihuahua is
mostly fawn and sable, rarely merle): wild or newly met animals of a breed are drawn from those;
babies take only their parents' alleles, so crossing breeds mixes them. Babies take one allele of each locus from each parent, so they look like them, and hidden
recessives can surprise (two black chihuahuas with a chocolate puppy).

The phenotype rules (which genotype is which colors and patterns) and the coat renderer are in the
game's `src/coats.rs` and, for trying coats out in the viewer, the skill's `coats.js`
(`.claude/skills/blocky-model/scripts/coats.js`): the same rules, change both together. They
paint on the layers each creature's build writes (`layers.py`), copied here into
`layers/<model>/` and embedded in the mod: what is coat and what is fixed, its light, its
position, where white, points, a mask and patterns show. Every file ends with `Sz`, the size gene (quantitative: small to big for its kind, `coats::size`
with the species' sex difference and spread in `src/species.rs`).

These files are where every population starts: in the game each species has a gene pool per
place that evolves from it (`src/evolution.rs`: natural selection by camouflage, drift,
mutation, migration), and wild animals spawn from their place's pool. A new species needs a gene
file here
(listed in `coats.rs`'s `FILES`), its model's layers (`MODEL_LAYERS`) and a phenotype rule.
