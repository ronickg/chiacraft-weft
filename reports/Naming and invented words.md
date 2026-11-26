# Naming and invented words

What Ronald asked (11-05): "read up on how to create words for your game ... there's specific ways one
can do it ... to make sure our game is unique and doesn't feel like it's all AI generated names. You
know, like I know Tolkien was very good at doing that." Added the same day, through Planner: he loves
Lord of the Rings-style peoples, "dwarven kingdoms in the mountains, elves in a specific biome, maybe
more". So the ancients' tongue is the root, and each people gets its own sound.

Part 1 is the research, with a source for each claim. Part 2 is the method for chiacraft: the
star-born ancients' naming language, the daughter tongues of the elves, the dwarves and the later
peoples, example names, a style guide, and a proposed tool. Nothing here is built yet. The roots and
names are a first draft for Ronald to change.

## The short version

- Good invented names are made from a **small sound system** plus **roots with meanings**, and then
  **aged by regular sound changes**. Tolkien did this, and so does David J. Peterson today. That is
  why their names feel like one language and not a pile of cool syllables.
- Names sound generic or "AI" when they have no system: stock sounds (Elara, Kael, Eldoria),
  apostrophes, x and z for strangeness, and grand English mash-ups ("Stormforge").
- chiacraft gets one root tongue, the ancients' (we call them the **Eskenai**, "star-born"). The
  **elves** speak a softened daughter of it, and the **dwarves** have their own unrelated tongue that
  borrowed the ancients' words for star, iron, stone and gate. **Humans and traders** speak plain
  English with worn-down old words. One root word therefore turns up in three regional forms, and a
  player can learn to see they are the same word.
- Most things stay plain English: items, blocks, animals, tools. Invented words are kept for what the
  ancients left behind (relics, runes, ruins, bosses, gates, dimensions) and for the peoples' own
  names. English parts are translated into ru and fr. Invented parts are kept as they are, written in
  Cyrillic letters for ru.
- Proposed tool: one lexicon file as the single source of truth, plus a small Python generator,
  sound-change applier and check, like the existing sheets.

---

# Part 1: Research

## 1. How Tolkien did it

**The languages came first.** Tolkien said the invention of languages was "the foundation", and that
the stories were made to give the languages a world, not the other way round. He also said "To me a
name comes first and the story follows" ([Sound and language in Middle-earth](https://en.wikipedia.org/wiki/Sound_and_language_in_Middle-earth);
the letter number is cited differently by different sources: [Goodreads quotes](https://www.goodreads.com/work/quotes/6739549)).

**Sound changes from one ancestor.** Quenya and Sindarin both come from an older Primitive Elvish,
through changes applied in order. Helge Fauskanger's survey lists them in rough time order, from
Primitive Quendian through Common Eldarin to Quenya ([Ardalambion, qevolution.pdf](https://www.ardalambion.org/qevolution.pdf)).
Eldamo has a page for each single change, such as initial *d-* becoming *l-* in Quenya ([Eldamo](https://eldamo.org/content/words/word-3325625207.html)),
and says the order is approximate because Tolkien kept revising. Two examples:
- primitive *galadā* "tree" became Quenya **alda** and Sindarin **galadh** ([Eldamo, √GALAD](https://eldamo.org/content/words/word-588112985.html);
  [elfdict](https://elfdict.com/w/alda)). Quenya dropped the first *g*. Sindarin softened *d* to *dh*
  and lost the last vowel.
- primitive *ndorē* "land" became Quenya **nórë** and Sindarin **dôr**, the *-dor* of Gondor and
  Mordor ([elfdict, ndorē](https://www.elfdict.com/w/nore)).

The two languages are clearly related, yet each sounds like itself. This is the main trick to copy.

**Phonaesthetics: sounds chosen for feeling.** In the talk "A Secret Vice" (1931) Tolkien separated
languages built for use from languages built for beauty. He named Greek, Finnish and Welsh as
languages with a "very characteristic" and beautiful word-form ([A Secret Vice](https://en.wikipedia.org/wiki/A_Secret_Vice);
the 2016 edition by Fimi and Higgins adds his "Essay on Phonetic Symbolism": [Glasgow eprints](https://eprints.gla.ac.uk/170107)).
In the lecture "English and Welsh" (1955) he made the famous point that "cellar door" sounds
beautiful once you set aside its meaning ([English and Welsh](https://en.wikipedia.org/wiki/English_and_Welsh);
the phrase is older than Tolkien: [Tolkien Gateway](https://tolkiengateway.net/wiki/Cellar_door)).
He took his tastes into the work. Quenya was patterned on Finnish (later also Latin and Greek), and
Sindarin was meant to stand to Quenya as medieval Welsh stands to Latin, with Welsh-like mutations and
plurals made by vowel change ([Elvish languages of Middle-earth](https://en.wikipedia.org/wiki/Elvish_languages_of_Middle-earth)).

**Place names come from roots with meaning.** *Gondor* is "land of stone" (Sindarin *gond* "stone",
akin to Quenya *ondo*), and Gondolin's older form *Ondolindë* is "Rock of the Music of Water"
([Encyclopedia of Arda, O](https://www.encyclopedia-of-arda.com/lexicon-O.html)). *Imladris* is built
on *imlad*, a deep valley with a flat floor, and Tolkien gave it the English name *Rivendell*
([Encyclopedia of Arda, L](https://www.encyclopedia-of-arda.com/lexicon-L.html)). Because a root comes
back in many names (*gond*, *dor*, *-ndil*), readers start to recognise it without being taught.

**Each people sounds different.** Elvish is meant to sound pleasant. Mordor's Black Speech is meant to
sound harsh. One linguist counted it at about 63% consonants, against 52% and 55% in Elvish samples,
with more *u*, fewer long *i*, and far more *b*, *d* and *g*. Shire names (Hobbiton, Bywater) are plain
English, while Bree-land names are Welsh-tinged and feel a little foreign ([Sound and language in Middle-earth](https://en.wikipedia.org/wiki/Sound_and_language_in_Middle-earth)).
The effect is built from sound choices you can count.

## 2. Modern craft for games and fiction

**David J. Peterson** (Dothraki and High Valyrian for *Game of Thrones*, Chakobsa for *Dune*, and
others) wrote *The Art of Language Invention* (2015). It covers sounds, words, how a language changes
over time, and writing systems ([Penguin Random House](https://www.penguinrandomhouse.com/books/316207/the-art-of-language-invention-by-david-j-peterson/9780143126461/);
[Wikipedia](https://en.wikipedia.org/wiki/David_J._Peterson)). He works much as Tolkien did. For
Dothraki he imagined the language a thousand years earlier and evolved it forward, and he calls the
evolution of his languages "simulated" ([Cleaver interview](https://www.cleavermagazine.com/brian-burmeister-interviews-david-j-peterson-author-of-the-art-of-language-invention/);
[Science Friday](https://www.sciencefriday.com/segments/from-dothraki-to-valyrian-building-the-languages-of-game-of-thrones/)).
His starting point was the roughly 50 words George R. R. Martin had used, mostly names. Peterson kept
them and built the language around them ([Dothraki language](https://en.wikipedia.org/wiki/Dothraki_language)).
The lesson for us: names the game already has (Meteoric Iron, Starfall) can stay, and the language
grows around them.

**Mark Rosenfelder's *Language Construction Kit*** began as web pages and became a book in 2010
([zompist.com](https://www.zompist.com/kit.html); [the book](https://www.zompist.com/lckbook.html)).
It opens with a short section on a "naming language", for writers who only need names for towns,
people and mountains ([LibraryThing](https://www.librarything.com/work/9696661)). The kit insists the
order of steps matters: sounds first, then words. Working backwards "will lead to an inconsistent if
not incoherent work" ([zompist.com](https://www.zompist.com/kit.html)).

**A naming language** is a light sketch of a language: only the sounds, the syllable rules, the
spelling and a few dozen roots and endings needed to build names, with no grammar for sentences.
Martin O'Leary's well-known essay and generator described it as "a light sketch of a language,
focusing purely on the parts which are necessary to produce names" (mewo2.com, *Generating naming
languages*; the page was offline when checked on 11-05). A Reddit guide does the same by hand
([r/conlangs resources](https://lr.eu.psf.lt/r/conlangs/wiki/resources)). This is the right size for
chiacraft.

**Word generators** build words from sound classes and syllable patterns:
- *Awkwords*: simple, with patterns and weights. *Lexifer* (William S. Annis): uses real-language
  sound frequencies and a "cluster table" for what happens when two sounds meet. *Vulgar(lang)*: a
  paid generator for a fuller language. *Logopoeist*: full statistics, command line only
  ([Colin Gorrie's guide](https://colingorrie.pages.dev/articles/conlang-word-generator)).
- *Gleb*: generates whole random sound systems ([Linguifex tools guide](https://linguifex.com/wiki/Guide:Conlanging_tools)).
- The point that carries over: weight the sounds. Real languages use some sounds far more than
  others, and a generator that picks every sound equally gives words that feel fake.

**Sound-change appliers** take a word list and a list of rules and give the aged words:
- Rosenfelder's *SCA²* writes a rule as `target/replacement/environment`. For example `c/g/V_V` means
  "c becomes g between vowels". The `-p` option prints which rule changed each word ([SCA² help](https://zompist.com/scahelp.html)).
- *Lexurgy* (Graham Hill, Kotlin, a web app and a command line) is the most powerful today
  ([GitHub](https://github.com/def-gthill/lexurgy)). *ASCA* is a newer one in Rust ([r/conlangs](https://lr.eu.psf.lt/r/conlangs/comments/1h2ryxf/introducing_asca_a_brand_new_sound_change_applier)).

## 3. How games did it

| Game | What they did | What to take |
|---|---|---|
| **Skyrim**, the Dragon language | Led by Emil Pagliarulo, with the script by Adam Adamowicz. The sound came from a reading of *Beowulf*. Its 34 runes are drawn as claw scratches. Each shout is three words of power, over 60 words in all ([Game Informer](https://gameinformer.com/b/features/archive/2011/01/20/skyrim-s-dragon-shouts?PostPageIndex=2)) | Words that do something in play get learned. Short one-syllable roots that combine. The look of the script comes from the lore (claws) |
| **Elder Scrolls**, Ayleid | Mostly compounds of adjective plus noun or noun plus noun. Only fragments are translated, such as *Av molag anyammis*, "from fire, life". Daedric is a script used for several languages, not a language ([UESP Ayleid](https://en.m.uesp.net/wiki/Lore:Ayleid_Language); [UESP alphabets](https://content2.uesp.net/wiki/Lore:Elven_Alphabets)) | A ruin tongue can be partly understood. Fans enjoy decoding a consistent system |
| **Zelda**, Hylian and Sheikah | In *Breath of the Wild* they are letter-for-letter ciphers of English. Older games used Japanese kana ciphers ([Zelda Wiki](https://zeldawiki.wiki/wiki/Sheikah_Language); [Nintendo Life](https://www.nintendolife.com/features/how-amateur-scholars-are-translating-zelda-tears-of-the-kingdoms-secret-language)) | A cipher gives a script but no language. Fine for signs, but it adds no meaning |
| **Minecraft**, the Enchanting Table | Tom Hall's Standard Galactic Alphabet from *Commander Keen* (1990), showing words picked at random from a fixed list. They have no effect on the enchantment ([NameHero](https://www.namehero.com/gaming-blog/minecraft-enchanting-table-language-everything-you-need-to-know/)) | What to avoid for runes: chiacraft's runes should mean something |
| **Destiny**, the Hive | Names track the lore. Aurash took the king's form and became Auryx, then Oryx. Xi Ro became Xivu Arath ([Destinypedia](https://www.destinypedia.com/Oryx)) | A name can change as the story changes, like our ages |
| **Elden Ring** | Invented given names with plain English titles ("Godrick the Grafted"). Some come from real old languages: *Scadutree* from Old English *sceadu* "shadow", *Siofra* from Irish ([Gamepressure](https://www.gamepressure.com/newsroom/we-mispronounce-many-names-from-elden-ring-all-because-of-inspira/z86fff)) | The invented name plus English epithet pattern works for bosses |
| **Horizon Zero Dawn** | The machines have plain English names after animals (Watcher, Thunderjaw, Tallneck). The old world's systems keep their technical names (GAIA, HADES) ([Wikipedia](https://en.wikipedia.org/wiki/Horizon_Zero_Dawn)) | Plain English for common creatures. The ancient layer keeps its own names |
| **Subnautica** | The alien glyphs are translated for the player by the PDA. In *Subnautica 2* the script (Axum) is logographic, and reading it is gated behind scanning a "Rosetta Stone" ([Steam forum](https://steamcommunity.com/app/264710/discussions/0/1483235412204162891/?ctp=9); [allthings.how](https://allthings.how/rosetta-stone-in-subnautica-2-location-scan-reward-and-decoded-inscriptions/)) | Understanding the ancients' writing can be a gate, which fits our Knowledge gate |
| **No Man's Sky** | Almost everything is generated from one seed. Players may rename discoveries within a content filter ([Wikipedia](https://en.wikipedia.org/wiki/No_Man%27s_Sky)) | Procedural names without roots carry no meaning, so nothing links them. Our player-made names (maps, tribes) need the same filter |
| **Valheim** | Plain English items, with bosses named from Norse: Eikthyr after the stag Eikþyrnir, Moder from "mother" ([Valheim wiki](https://valheim.fandom.com/wiki/Eikthyr); [G-Portal](https://www.g-portal.com/en/blog/valheim-bosses-guide)) | Plain items, invented boss names. But its names are borrowed real Norse, where ours are our own |

## 4. What makes names feel generic or "AI-generated"

What people notice:
- **The same few names.** Writers and developers report that language models keep producing
  *Elara* and *Lyra* ("90% of all the times" in one Gemini thread), plus *Kael*, *Kaelen*, *Thorne*,
  *Voss*, *Silas* ([Google AI forum](https://discuss.ai.google.dev/t/gemini-cannot-come-up-with-unique-name-for-characters/94830);
  [AOL/TODAY](https://www.aol.com/articles/ai-generated-baby-name-rolls-195619325.html)). One player
  found that searching *Eldoria* turns up almost only AI-written content ([itch.io](https://itch.io/post/11060499)).
  This evidence is anecdotal, but the pattern is consistent.
- **The apostrophe.** Diana Wynne Jones's *Tough Guide to Fantasyland* already joked that names in
  Fantasyland feel incomplete without one, as in *Gna'ash* ([Goodreads repost](https://www.goodreads.com/author_blog_posts/140339-revisiting-the-tough-guide-to-fantasyland?tab=author)).
  Readers cannot tell how to say it, and it is usually decoration ([Tor.com](https://tor.com/2013/08/27/apostrophes-in-science-fiction-and-fantasy-names);
  [Springhole](https://springhole.net/writing/creating-sf-names.htm)).
- **Stock pieces:** *x*, *z*, *y* and *th* used to look exotic (Xyrthos, Zyrael), the endings
  *-ia*, *-ion*, *-or*, *-oth*, *-wyn*, *-ryn*, and the openings *Aeth-*, *Ael-*, *Vael-*, *El-*.
  Also Norse pastiche (*-heim*, *-gard*) and English mash-ups (*Stormforge*, *Shadowfen*,
  *Bloodfall*, *Emberhold*), and grand titles (*the Eternal*, *the Unbroken*).
- **No family resemblance.** Each name is made alone, so two names from the same land share nothing.

The cure, from everything in sections 1 and 2:
1. **A small sound system per people.** Fewer sounds and strict syllable rules make every name from
   that people sound related.
2. **Meaning from roots.** Every invented name is built from roots with meanings, so it can be
   explained and reused, and players start to read it.
3. **Different systems per people and era, linked by sound changes.** Related names show their
   kinship (*Esketem*, *Eskedema*). Unrelated peoples sound unrelated.
4. **Names come from the lexicon, not from a prompt.** No name gets in without a derivation. This is
   also the best defence against AI sameness when Claude helps: generate from our sounds, then check.

## 5. Plain words and invented ones, and translation

Tolkien himself mixed the two. Hobbits say *Rivendell*, while elves say *Imladris*. For translators he
wrote the *Guide to the Names in The Lord of the Rings*. Its rule: names in English form ("Dead
Marshes") should be translated, Elvish names left unchanged, and the hard cases are names he had
already turned into English-sounding ones, like *Rivendell* ([Translating The Lord of the Rings](https://en.wikipedia.org/wiki/Translating_The_Lord_of_the_Rings)).

Game localisation into Russian uses the same tools. A study of 1,000 names from three fantasy games
found personal names mostly transliterated or transcribed, and meaningful names often translated word
for word (calqued) ([Bulletin of Chelyabinsk State University](https://journals.csu.ru/index.php/BulletinCSU/article/download/2597/2270)).
So for chiacraft (en, ru, fr): English parts are translated, invented parts are written in Cyrillic
letters by a fixed table, and the English gloss next to them is translated too.

---

# Part 2: The method for chiacraft

## 6. The family tree

```
                 Old Eskenai  (the ancients' tongue: carved on altars, relics, gates)
                 /                 |                          \
     Elvish (soft drift)   Dwarvish loans (frost drift)   Worn speech (humans, traders, coasts)
     the forest elves      inside the dwarves' own,       plus plain English
                           unrelated tongue
```

- **Old Eskenai** is what the ancients carved. It never changes, because it is dead. Altars, relics,
  runes and gates use it.
- **Elvish** is the ancients' tongue, softened by long use, as Sindarin and Quenya are of Primitive
  Elvish. The elves were the ancients' pupils, and they live where the ley lines run.
- **Dwarvish** is not related, as Tolkien's Khuzdul is unrelated to Elvish. The dwarves dug into the
  ancients' mountain halls and borrowed the words for what they found there: star, iron, stone, gate,
  keep. Those loans went through the "frost" changes. This is like English, a Germanic language full
  of French loans.
- **Worn speech** belongs to humans, traders and settlements, and to ruins on coasts and plains. It is
  mostly plain English, with old words worn down: every trader calls a ruin a *dokan*.

Which tongue names a place follows who lives near it. Forest ruins get elvish names. Mountain and
cold ruins get dwarvish forms. Coasts, plains, swamps and deserts get worn forms, until a desert or
sea people of their own is added.

## 7. Old Eskenai: sounds and syllables

**Sounds** (spelled as written, one letter for one sound):

| | |
|---|---|
| Stops | p t k, b d g |
| Nasals | m n (and ng before k or g) |
| Others | s h f r, w y (rare) |
| Vowels | a e i o u |
| Diphthongs | **ai** (as in *eye*), **au** (as in *cow*): the tongue's signature |

**Not in Old Eskenai:** *l*, *z*, *v*, *x*, *q*, *c*, *j*, *th*, apostrophes, doubled letters. The
missing **l** is its fingerprint ("the ancients had no l"). Elvish gains *l* later, which makes the
link visible.

**Syllables:** (C)(r)V(C). Starts allowed: one consonant, or *pr tr kr br dr gr*, or *sk st* at the
start of a word. Ends allowed: *m n r s t k*, and *sk* or *st* at the end of a word only. A word ends
in a vowel, *m n r s t k* or *sk*, never in *b d g*.

**Stress:** always the first syllable of the first root: **ES**-ke-nai, **BOR**-kaun.

**Weights for the generator** (so it sounds like a language, not dice): vowels *a* 26%, *e* 20, *i*
18, *o* 14, *u* 12, *ai* 5, *au* 5; consonants *k t s r n* common, *m d b g* middling, *h p f w y*
rare.

**Joining roots:**
1. The modifier comes first and the head last, as in English: *hes* "sky" + *dau* "fire" = **Hesdau**,
   "sky-fire", lightning.
2. If three consonants would meet, or two of the same, put *e* between: *esk* + *nai* = **Eskenai**,
   *esk* + *tem* = **Esketem**.
3. If two vowels meet, the first plain vowel drops (*aru* + *ema* = **Arema**). A diphthong's second
   half becomes *w* or *y* (*dau* + *aru* = **Dawaru**).

## 8. The roots (35) and endings

| Root | Meaning | Root | Meaning | Root | Meaning |
|---|---|---|---|---|---|
| esk | star | kep | fall, come down | nai | be born, begin |
| rai | light, shine | nau | night, dark | dau | fire, burn |
| skir | ice, cold | omu | water, sea | hes | sky, air |
| bor | mountain, height | nu | deep, below | duk | stone |
| tem | iron, metal | tauk | gate, a way through | aru | line, flow (ley) |
| kaun | keep, guard | gom | beast, creature | dren | tree, wood |
| has | dry, sand | pok | rot, mire | pem | world |
| maun | mind, knowing | sen | word, mark, rune | ta | make, build |
| ris | break, ruin | eka | old, first | gai | great |
| pai | seed, egg, young | rem | watch, see | wom | voice, call |
| ber | go, cross | taru | sleep, wait | kru | bone |
| ema | life, blood | tun | hollow, cave | | |

**Endings:** **-ar** "one who" (*kaunar*, keeper), **-an** "place of" (*dukan*, stone place, a
ruin), **-ek** "a made thing" (*senek*, an inscription), **-o** "done" (*kepo*, fallen).

Roots were checked against English, Russian, French and German for bad meanings, and several first
picks were dropped: *sau* (German "sow"), *ham*, *nosk* (Russian носки "socks"), *wai* (Maori
"water"), *gau*. Short words always collide with something somewhere. The check is only for collisions
that are funny, rude or loaded.

## 9. The three drifts (sound-change sets)

Each set is applied in order, like SCA². Each rule is one plain sentence and one SCA²-style line.

**Soft drift: Elvish, and the forest ruins**
1. *s* plus *p t k* at the start loses the stop: *skir* → *sir*. (`sp,st,sk/s,s,s/#_`)
2. A word ending in a consonant other than *n* takes *-a*: *duk* → *duka*.
3. *p t k* become *b d g* between vowels: *duka* → *duga*. (`p,t,k/b,d,g/V_V`)
4. *r* becomes *l* between vowels: *aru* → *alu*. (`r/l/V_V`)
5. *ai* becomes *ei*: *nai* → *nei*.

**Frost drift: the dwarves' loans, and mountain and cold ruins**
1. In words of two or more syllables, a final single vowel drops: *aru* → *ar*.
2. *au* becomes *o*, *ai* becomes *e*: *kaun* → *kon*, *tauk* → *tok*.
3. *k* becomes *ch* (as in *church*) before *i* or *e*, but not after *s*: *kepesk* → *chepesk*.
4. *b d g* become *p t k* at the end of a word.

**Worn drift: humans and traders, coasts, plains, swamps, deserts; the latest era**
1. *h* at the start of a word drops: *has* → *as*.
2. In words of three or more syllables, a vowel in the second syllable between single consonants
   drops: *ekaru* → *ekru* (then *ekro* by rule 3).
3. *u* becomes *o* (but *au* stays): *duk* → *dok*, *nu* → *no*.
4. *n* becomes *ng* before *k* or *g*.
5. A final *sk* becomes *sh*: *kepesk* → *kepesh*.

The same word in all four forms:

| Old Eskenai | Meaning | Elvish | Dwarvish loan | Worn |
|---|---|---|---|---|
| Eskenai | star-born (the ancients) | Eskenei | Eskene | Eskenai |
| Esketem | star-iron (star steel) | Eskedema | Esketem | Esketem |
| Kepesk | fall-star (a meteorite) | Kebeska | Chepesk | Kepesh |
| kaunar | keeper | kaunala | konar | kaunar |
| Hesdau | sky-fire (lightning) | Hesdau | Hesdo | Esdau |
| dukan | stone place (a ruin) | dugan | dukan | dokan |

A few words come through a drift unchanged. That is normal (English *wind* is still *wind*) and makes
the system feel less mechanical.

## 10. Twenty names of the ancients' world

| Name in game | What | Built from | Meaning | Form |
|---|---|---|---|---|
| **Eskenai** | the ancients | esk + nai | star-born | Old |
| **Esketem** | star steel's lore name | esk + tem | star-iron | Old |
| **Kepesk** | a meteorite heart (relic) | kep + esk | fall-star | Old; traders say *Kepesh* |
| **Eskan** | the furthest dimension, their home | esk + -an | star-place | Old |
| **Nopem** | a night dimension | nau + pem | night-world | Frost |
| **Ekaskir, the Old Ice** | tundra boss | eka + skir | old ice | Frost (unchanged) |
| **Borkon, the Height-Keeper** | mountain boss | bor + kaun | mountain-keeper | Frost |
| **Drenkaun, the Wood-Warden** | forest boss | dren + kaun | tree-keeper | Soft (unchanged) |
| **Nokaun, Keeper of the Deep** | ocean boss | nu + kaun | deep-keeper | Worn |
| **Askaun, the Sand Keeper** | desert boss | has + kaun | dry-keeper | Worn |
| **Pokgom, the Mire Beast** | swamp boss | pok + gom | rot-beast | Worn (unchanged) |
| **Nogom** | a deep-cave boss | nau + gom | night-beast | Frost |
| **dokan** | what everyone calls a ruin | duk + -an | stone place | Worn |
| **Drenduga** | a forest ruin | dren + duk | wood-stone | Soft |
| **Ristok** | a mountain ruin with a dead gate | ris + tauk | broken gate | Frost |
| **Eskerem** | a star-watching ruin on a peak | esk + rem | star-watch | Old |
| **Maunsen** | relic: a tablet that teaches runes | maun + sen | mind-rune | Old |
| **Dau, Skir, Aru, Tauk** | the first four runes | roots | fire, ice, flow, gate | Old |
| **Hesdau** | the lightning spell (tome name) | hes + dau | sky-fire | Old; *Hesdo*, *Esdau* |
| **Arema** | a ley spring | aru + ema | flow-life | Old |

The pattern is visible in the list: every guardian is a *-kaun* or a *-gom*, which is the point. A
player who has met Borkon guesses at once that Nokaun is a guardian too.

## 11. The elves

**Where:** one biome of old forest. My pick is the old-growth or birch forests with ley nodes. They
could move to the giant groves once those exist. Elves live where the ley lines run strongest.

**Their tongue:** Old Eskenai through the soft drift (section 9). They keep the ancients' word order
(modifier first) and first-syllable stress.

**How it sounds and why:** open and flowing. Words often end in a vowel or *-n*. *b d g* and *l*
appear between vowels, and *ei* is common. This is the ancients' speech worn smooth by thousands of
years of song. They are the ancients' pupils, so their words are the easiest to trace back. That is the
Quenya and Sindarin relationship, made our own. **Not allowed** (it would read as Tolkien or as generic
elf): *-iel*, *-wen*, *-dor*, *-ion*, *-las*, *-ndil*, *gal-*, *cel-*, *mith-*, *El-* at the start of
a personal name, and *th*.

| Name | What | Old form | Meaning |
|---|---|---|---|
| **Alunei** | the elves' own name | arunai | born of the flow (ley) |
| **Aludren** | their forest realm | arudren | wood of the flow |
| **Taleska** | their city | taresk | waiting for the stars |
| **Reiwoma** | the eldest, a seer | raiwom | light-voice |
| **Emalei** | the warden-queen | emarai | life-light |
| **Hesalu** | a bow | hesaru | sky-thread |
| **Naulisa** | a sword | nauris | night-breaker |
| **Peidren** | a sacred grove | paidren | seed-wood |
| **Alema** | a ley spring in the realm | arema | flow-life |
| **Dugema** | their word for the dwarves | dukema | stone-blood |
| **Eskenei** | their word for the ancients | eskenai | star-born |

## 12. The dwarves

**Where:** kingdoms in the mountains, built into or beside the ancients' mountain halls.

**Their tongue:** their own, not descended from the ancients'. It differs in every system-level
choice, so it never sounds like a drift of Eskenai:

| | Old Eskenai and Elvish | Dwarvish |
|---|---|---|
| Roots | one or two syllables | one syllable, heavy |
| Vowels | a e i o u, ai au | **a e o u** only, no diphthongs; no native *i* |
| Consonants | no *l*, *v* | adds **v** and **kv**; no *w*, *y*, *f* |
| Clusters | light | heavy word ends: *-rn -rk -rd -nd -nk -mb -st -sk* |
| Word order | modifier first | **head first** ("hold-mountain") |
| Stress | first syllable | **last** root |
| Plural | rare | **-ok** |

**Native roots (to start):** *gund* hold, hall; *kvor* lord, king; *brum* hammer; *vork* axe; *vend*
oath; *umb* deep, under; *sund* gold; *grod* forge; *dund* kin; *brok* old.

**Loans from the ancients**, through the frost drift: *bor* mountain, *esk* star, *tem* iron, *duk*
stone, *tok* gate (from *tauk*), *kon* keep (from *kaun*), *Chepesk* meteorite. Loans keep their *i*
(which native words lack), so a careful player can tell a loan from a native word, as with French
words in English.

**How it sounds and why:** short, closed, stressed at the end, with lots of *b d g r* and *kv*. It is
the sound of hammering and of words said in a hall of stone. It follows Tolkien's contrast between
Elvish and Dwarvish but avoids his shapes. **Not allowed:** *Kh-*, *Khaz-*, *-dûm*, *-zul*, and names
from the Norse dwarf list (Durin, Thorin, Balin). Tolkien's Khuzdul has a Semitic shape built on three
consonants. Ours is a stacking, end-stressed shape.

**A Tolkien touch:** Tolkien's dwarves kept their tongue private and used outer names among others.
Ours could too. Outside their holds a dwarf goes by a plain English byname ("Old Oath"), and the true
name is learned through standing with the hold, which fits the Knowledge gate.

| Name | What | Built from | Meaning |
|---|---|---|---|
| **Dundok** | the dwarves' own name | dund + -ok | the kins |
| **Gundbor** | the mountain kingdom | gund + bor (loan) | hold of the mountain |
| **Grodsund** | its city | grod + sund | forge of gold |
| **Kvor Vendbrok** | the king (byname "Old Oath") | kvor; vend + brok | lord; oath-old |
| **Dundbrok** | title of the eldest | dund + brok | kin-old |
| **Brumesk** | a star-steel hammer | brum + esk (loan) | hammer of the star |
| **Vorktem** | an axe | vork + tem (loan) | axe of iron |
| **Tokumb** | the great gate | tok (loan) + umb | gate of the deep |
| **Umbsund** | a deep gold mine | umb + sund | deep of gold |
| **Venduk** | the oath-stone | vend + duk (loan) | oath stone (*dd* becomes *d*) |
| **Chepesk** | their word for meteorite iron | loan of *kepesk* | fall-star |
| **Eskene** | their word for the ancients | loan of *eskenai* | star-born |

## 13. Other peoples who could follow

- **Humans** (plains, rivers, coasts): plain English, humble and concrete like the Shire's (Bywater,
  not Stormwatch), plus worn old words (*dokan*, *Kepesh*, *Esdau*). They are the "common speech", so
  their names translate into ru and fr.
- **A desert people:** a fourth drift of Eskenai, sketched as long vowels written doubled, *k* → *h*
  between vowels, final *-r* dropped. They would take over the desert's worn names.
- **A sea or island people:** their own unrelated tongue, made like the dwarves', with a few loans.
- **Beast-folk or a raider people**, if one is ever wanted: an unrelated tongue tuned harsh the way
  Black Speech was (more consonants, more *u*, more *b d g*), and never borrowed from Tolkien's.
- **Natives of the dimensions:** each gets a sound system with no shared roots, so the other worlds
  feel truly foreign. The exception is the ancients' home, Eskan, where Old Eskenai is spoken alive.

## 14. Style guide

**When to invent and when to stay English**

| Thing | Rule | Example |
|---|---|---|
| Items, blocks, tools, food, materials | English | Stone Axe, Meteoric Iron, Star Steel |
| Common animals and monsters | English | Raven, Mammoth |
| Relics, the ancients' things | invented + English gloss | Maunsen (Mind-Rune) |
| Bosses | invented name + English epithet | Borkon, the Height-Keeper |
| Runes | invented root, meaning on hover | Dau: fire |
| Spells | English function name, old name on the tome | Lightning (Hesdau) |
| Ruins, gates, dimensions | invented, in the local tongue | Ristok, Eskan |
| Peoples | English in the UI, own name in lore | Elves (Alunei), Dwarves (Dundok) |
| Human places | humble English compounds | Saltmarsh Ford |
| Player-made names | free, with a content filter | |

**Spelling and saying**
- One spelling per sound: *k* is always hard (never *c*), *g* is always hard, *s* is never *z*, *h*
  is always said.
- Vowels as in Spanish or Italian: *a* in *father*, *e* in *bed*, *i* in *machine*, *o* in *more*, *u*
  in *rule*; *ai* in *eye*, *au* in *cow*, *ei* in *day*; *ch* in *church*.
- Stress: Eskenai and Elvish on the first syllable, Dwarvish on the last.
- No apostrophes, hyphens or accents in names. An epithet is separated with a comma.
- Capitalise names. Roots in the glossary are lower case.

**Rules against the generic**
1. No invented word without a derivation in the lexicon.
2. None of: *x*, *z*, *q*, *y* as a vowel, *th*, *ae*; *-ia*, *-ion*, *-oth*, *-wyn*, *-ryn*;
   *Aeth-*, *Ael-*, *Vael-*; Elara, Lyra, Kael, Thorne, Eldoria and the like.
3. No English mash-ups of grand words for proper names: Storm-, Shadow-, Blood-, -forge, -fall,
   -heim, -gard. An English name must be concrete and local.
4. Lengths vary: one to four syllables, not always three.
5. No two names of the same kind one letter apart.
6. New names reuse roots. Add a root only when no root fits, and log its meaning.
7. Check new roots and names for bad meanings in en, ru, fr and de.

**Translation (en, ru, fr)**
- Keep the name and its epithet in separate lang keys (`boss.base.borkon` = "Borkon",
  `boss.base.borkon.title` = "the Height-Keeper"), so each language can order and inflect them:
  "Боркон, Страж Высот", "Borkon, le Gardien des Hauteurs".
- English parts are translated. Invented names are never translated: in fr they stay as written, and
  in ru they are written by one fixed table kept in the lexicon: a а, e е (э at the start), i и, o о,
  u у, ai ай, au ау, ei эй, k к, g г, h х, ch ч, sk ск, w в, y й, ng нг, kv кв. So Eskenai is
  Эскенай and Borkon is Боркон.
- Russian declines names that end in a consonant (Боркона). Prefer UI strings where the name stands
  alone ("Boss: Borkon").
- The lexicon's meanings and stress marks are the translators' notes.

**In play:** lore pages and relics can teach roots one at a time, with a glossary in the journal. A
player who knows *kaun* = keep can read an altar. That ties the language to the Knowledge gate, the
way Skyrim's words of power and Subnautica's Rosetta Stone tie reading to play.

## 15. The proposed tool (not built)

A small, stdlib-only Python tool, kept in the repo with one data file as the single source of truth,
following the pattern of the vitals and animals sheets.

**Files**
- `mods/base/lore/lexicon.toml`: the lexicon. It sits with the game data because the journal glossary
  and lang keys may read it later. It holds the peoples, the sounds and weights per people, the
  syllable rules, the roots and endings, the drift rules in SCA² form, the Cyrillic table, the banned
  patterns, the bad-word list, and every name with its build.
- `tools/names/names.py`: the commands.
- `notes/NAMES.md`: a generated sheet with all roots and names in every form, with meanings and
  pronunciation.

A name entry looks like this:

```toml
[name.borkon]
kind = "boss"            # boss, ruin, relic, rune, spell, dimension, people, place, weapon...
people = "dwarvish_loan" # which drift or tongue gives the shown form
build = ["bor", "kaun"]  # roots and endings, joined by the rules
form = "Borkon"          # what the game shows; the check re-derives it
gloss = "mountain-keeper"
lang_key = "boss.base.borkon"
```

**Commands**
- `names.py gen --people elvish -n 40`: candidate roots from a people's weighted sounds and syllable
  rules, minus ones too close to existing roots, banned patterns or listed bad words. A person picks
  the roots, and the generator only offers.
- `names.py derive eskenai --steps`: applies each drift and shows every rule that fired, like SCA²'s
  `-p`.
- `names.py check`: fails if a name's build uses an unknown root, if re-deriving gives a different
  form (a deliberate exception needs `irregular = "why"`), if a form breaks its people's syllable
  rules, if it matches a banned pattern or bad word, if two names of one kind are one edit apart, or
  if a lang key named in the lexicon is missing from en, ru or fr. It runs with the other checks.
- `names.py sheet > notes/NAMES.md`: regenerates the sheet (and the check fails while it is stale, as
  with SHEET.md).

The rules stay in SCA² syntax so that a hard case can be checked in Lexurgy without porting. When
Claude helps with names, it uses `gen` and `derive` and never makes names up freehand.

## 16. What to decide next (Ronald)

1. The ancients' name (*Eskenai*), and whether the roots feel right. Swap any that don't. This is the
   moment, before names land in lang files.
2. Which forest biome is the elves' home.
3. Whether dwarves have outer English bynames and hidden true names.
4. Whether to build the tool now (about a day, a worker branch) or after the first bosses are named.
