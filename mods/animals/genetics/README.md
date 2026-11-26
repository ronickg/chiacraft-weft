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
shared crate `core/src/coats.rs` (`animals_core`), which the game uses and the website runs as
WASM (`web/`, `web/build.sh`): one copy of the rules. They
paint on the layers each creature's build writes (`layers.py`), copied here into
`layers/<model>/` and embedded in the mod: what is coat and what is fixed, its light, its
position, where white, points, a mask and patterns show. Every file has `Sz`, the size
gene (quantitative: small to big for its kind, `coats::size` with the species' sex difference
and spread in `src/species.rs`), and ends with `Cu` and `Fi`, curiosity and fierceness, which
with the core's temperament (`Tp`) make its personality (`coats::personality`, genome version 3:
an older genome draws them at its next upkeep).

**The core** (`core.json`): traits every species carries after its own loci, in genes 16-31
(`Genome::core`): size, shade, warmth, how far white reaches, the marks' size, density,
contrast, edge and stretch, eye color, and build (head, neck, tail, legs, girth, horns). They
decide how each individual wears its species' colors and patterns. Each has a heritability
(`h`): the rest of its spread is the animal's own chance (from its seed), not passed on. A gene
file may set for them:

- `traits`: their mean and spread for this species (`{"Ms": {"mean": 0.55, "sd": 0.15}}`);
- `sexes`: what being male or female shifts (`{"male": {"Ct": 0.12}}`: a male's starker display);
- `young`: what a young one's look shifts, fading as it grows, and a `pattern` it is born with;
- `eyes`: its eye colors, along the eye trait, painted on the iris its model's layers mark (a
  species' genes may decide it instead: a colorpoint cat's blue);
- `habitats`: per habitat (`grassland`, `forest`, `taiga`, `snow`, `desert`, `mountains`), the
  frequencies wild ones there start from, as a breed's: a species' kinds in different country;
- `mark_scale`: model units to a pattern's (a big model's marks drawn bigger);
- `build`: per build trait (`Hd`, `Nk`, `Tl`, `Lg`, `Gt`, `Ho`), the model's `bones` it scales,
  along which `axes` and how far (`range`: a bone `1 + range` times as big at the trait's top,
  `1 - range` at its bottom). A head's trait scales its jaw and eyelids too (child bones keep their
  size); a chain (a neck, a tail) scales every segment along it; a bone whose boxes sit in a
  `_tilt` child (rotated elements) is scaled through that child.

Scars aren't genes: an animal keeps what life left on it in its genome's `scars` (`src/scars.rs`),
painted on its coat's layers, never passed on.

A species' own locus with a core trait's id replaces it (its `Dk`, its `Sz`); never reuse a core
id for a locus of alleles. Add a species' new loci after its old ones, or saved genomes read the
wrong genes.

These files are where every population starts: in the game each species has a gene pool per
place that evolves from it (`src/evolution.rs`: natural selection by camouflage, drift,
mutation, migration), and wild animals spawn from their place's pool. A new species needs a gene
file here
(listed in `coats.rs`'s `FILES`), its model's layers (`MODEL_LAYERS`) and a phenotype rule.
