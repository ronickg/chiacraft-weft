# Names: the lexicon's sheet

Written from `mods/base/lore/lexicon.toml` (`python3 tools/names/names.py sheet > notes/NAMES.md`); do not edit by hand. `python3 tools/names/names.py check` fails while it is stale. Why it is built this way: `reports/Naming and invented words.md`. New names come from `gen` and `derive`, never freehand.

## Saying them

- One spelling per sound: *k* and *g* always hard, *s* never *z*, *h* always said, *ch* as in *church*.
- Vowels as in Spanish or Italian: *a* in *father*, *e* in *bed*, *i* in *machine*, *o* in *more*, *u* in *rule*; *ai* in *eye*, *au* in *cow*, *ei* in *day*.
- Stress (capitals below): Old Eskenai and its daughters on the first syllable, Dwarvish on the last root.
- Invented names are never translated. Russian writes them by the table at the end; French and English as written.

## The tongues

- **Old Eskenai** (`old`, from old; stress on the first syllable). The ancients' tongue, carved on altars, relics, runes and gates. Dead, so it never changes.
- **Elvish** (`elvish`, from old through the soft drift; stress on the first syllable). Old Eskenai worn smooth by thousands of years of song: open, flowing, often ending in a vowel or -n, with b d g and l between vowels and ei common. The ancients' pupils, so their words are the easiest to trace back.
- **Dwarvish loans and mountain names (frost)** (`dwarvish_loan`, from old through the frost drift; stress on the first syllable). The ancients' words as the dwarves borrowed them from the mountain halls, and the names of mountain and cold ruins.
- **Worn speech (humans, traders)** (`worn`, from old through the worn drift; stress on the first syllable). Plain English with the old words worn down: every trader calls a ruin a dokan. Also the coasts', plains', swamps' and deserts' ruins, until a desert or sea people of their own is added.
- **Dwarvish** (`dwarvish`, from dwarvish; stress on the last root). Their own tongue, not descended from the ancients'. Short, closed, stressed at the end, with lots of b d g r and kv: the sound of hammering, and of words said in a hall of stone. Head first ('hold-mountain'), plural -ok.

Which tongue names a ruin, by where it stands: forest: Elvish; mountain, cold: Dwarvish loans and mountain names (frost); coast, plains, swamp, desert: Worn speech (humans, traders).

## Old Eskenai's roots

| root | means | Soft | Frost | Worn | say | ru |
|---|---|---|---|---|---|---|
| **esk** | star | eska | esk | esh | ESK | эск |
| **kep** | fall, come down | keba | chep | kep | KEP | кеп |
| **nai** | be born, begin | nei | ne | nai | NAI | най |
| **rai** | light, shine | rei | re | rai | RAI | рай |
| **nau** | night, dark | nau | no | nau | NAU | нау |
| **dau** | fire, burn | dau | do | dau | DAU | дау |
| **skir** | ice, cold | sila | skir | skir | SKIR | скир |
| **omu** | water, sea | omu | om | omo | O-mu | ому |
| **hes** | sky, air | hesa | hes | es | HES | хес |
| **bor** | mountain, height | bola | bor | bor | BOR | бор |
| **nu** | deep, below | nu | nu | no | NU | ну |
| **duk** | stone | duga | duk | dok | DUK | дук |
| **tem** | iron, metal | tema | tem | tem | TEM | тем |
| **tauk** | gate, a way through | tauga | tok | tauk | TAUK | таук |
| **aru** | line, flow (ley) | alu | ar | aro | A-ru | ару |
| **kaun** | keep, guard | kaun | kon | kaun | KAUN | каун |
| **gom** | beast, creature | goma | gom | gom | GOM | гом |
| **dren** | tree, wood | dren | dren | dren | DREN | дрен |
| **has** | dry, sand | hasa | has | as | HAS | хас |
| **pok** | rot, mire | poga | pok | pok | POK | пок |
| **pem** | world | pema | pem | pem | PEM | пем |
| **maun** | mind, knowing | maun | mon | maun | MAUN | маун |
| **sen** | word, mark, rune | sen | sen | sen | SEN | сен |
| **ta** | make, build | ta | ta | ta | TA | та |
| **ris** | break, ruin | risa | ris | ris | RIS | рис |
| **eka** | old, first | ega | ek | eka | E-ka | эка |
| **gai** | great | gei | ge | gai | GAI | гай |
| **pai** | seed, egg, young | pei | pe | pai | PAI | пай |
| **rem** | watch, see | rema | rem | rem | REM | рем |
| **wom** | voice, call | woma | wom | wom | WOM | вом |
| **ber** | go, cross | bela | ber | ber | BER | бер |
| **taru** | sleep, wait | talu | tar | taro | TA-ru | тару |
| **kru** | bone | kru | kru | kro | KRU | кру |
| **ema** | life, blood | ema | em | ema | E-ma | эма |
| **tun** | hollow, cave | tun | tun | ton | TUN | тун |

Endings: **-ar** one who (*kaunar, keeper*); **-an** place of (*dukan, stone place (a ruin)*); **-ek** a made thing (*senek, an inscription*); **-o** done (*kepo, fallen*).

## How roots join

The modifier comes first and the head last. If three consonants would meet, or two of the same, *e* goes between. If two vowels meet, the first plain vowel drops; a diphthong's second half becomes *w* or *y*.

| build | joined |
|---|---|
| hes + dau | hesdau |
| esk + nai | eskenai |
| esk + tem | esketem |
| aru + ema | arema |
| dau + aru | dawaru |

## The drifts

Each applied in order, in SCA² syntax (`target/replacement/environment[/exception]`: `_` the sound changed, `#` a word's edge, capitals categories, `(X)` optional).

Categories: `V` = a e i o u ai au ei; `P` = a e i o u; `C` = p t k b d g m n s h f r w y l v ch sh ng kv; `F` = e i ei; `G` = k g; `K` = m n r s t k p; `B` = b d g.

**Soft** (Elvish, and the forest ruins)

1. s plus p, t or k at the start loses the stop: skir → sir `sp,st,sk/s,s,s/#_`
2. a word ending in a consonant other than n takes -a: duk → duka `/a/C_#/n_#`
3. p t k become b d g between vowels: duka → duga `p,t,k/b,d,g/V_V`
4. r becomes l between vowels: aru → alu `r/l/V_V`
5. ai becomes ei: nai → nei `ai/ei/_`

**Frost** (the dwarves' loans, and mountain and cold ruins)

1. in words of two or more syllables, a final single vowel drops: aru → ar `P//V(C)(C)(C)_#`
2. au becomes o, ai becomes e: kaun → kon, tauk → tok `au,ai/o,e/_`
3. k becomes ch (as in church) before i or e, but not after s: kepesk → chepesk `k/ch/_F/s_`
4. b d g become p t k at the end of a word `b,d,g/p,t,k/_#`

**Worn** (humans and traders, coasts, plains, swamps, deserts; the latest era)

1. h at the start of a word drops: has → as `h//#_`
2. in words of three or more syllables, a vowel in the second syllable between single consonants drops, where the first can close a syllable: ekaru → ekru `P//#(C)(C)VK_CV`
3. and where b, d or g and r are left to start a syllable: abaru → abru `P//#(C)(C)VB_rV`
4. u becomes o (but au stays): duk → dok, nu → no `u/o/_`
5. n becomes ng before k or g `n/ng/_G`
6. a final sk becomes sh: kepesk → kepesh `sk/sh/_#`

One word in every form:

| Old Eskenai | means | Soft | Frost | Worn |
|---|---|---|---|---|
| Eskenai | star-born (the ancients) | Eskenei | Eskene | Eskenai |
| Esketem | star-iron (star steel) | Eskedema | Esketem | Esketem |
| Kepesk | fall-star (a meteorite) | Kebeska | Chepesk | Kepesh |
| kaunar | keeper | kaunala | konar | kaunar |
| Hesdau | sky-fire (lightning) | Hesdau | Hesdo | Esdau |
| dukan | stone place (a ruin) | dugan | dukan | dokan |

## The names

Every name with its build, meaning and how to say it. For names from Old roots, the same word in every drift.

### people

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Alunei** | the elves' own name | aru + nai | born of the flow (ley) | Elvish | A-lu-nei | Алунэй | Alunei · Arune · Arnai | English: Elves; home: aludren |
| **Dugema** | the elves' word for the dwarves | duk + ema | stone-blood | Elvish | DU-ge-ma | Дугема | Dugema · Duchem · Dokma |  |
| **Dundok** | the dwarves' own name | dund + -ok | the kins | Dwarvish | DUN-dok | Дундок | — | English: Dwarves |
| **Eskenai** | the ancients | esk + nai | star-born | Old Eskenai | ES-ke-nai | Эскенай | Eskenei · Eskene · Eskenai | English: the Ancients |
| **Eskene** | the dwarves' word for the ancients | eskene | star-born | Dwarvish | ES-ke-ne | Эскене | — |  |
| **Eskenei** | the elves' word for the ancients | esk + nai | star-born | Elvish | ES-ke-nei | Эскенэй | Eskenei · Eskene · Eskenai |  |

### realm

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Aludren** | an elven forest realm (the first home) | aru + dren | wood of the flow | Elvish | A-lu-dren | Алудрен | Aludren · Arudren · Arodren | home: aludren |
| **Gundbor** | the mountain kingdom | gund + bor | hold of the mountain | Dwarvish | gund-BOR | Гундбор | — |  |

### place

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Alema** | a ley spring in the realm | aru + ema | flow-life | Elvish | A-le-ma | Алема | Alema · Arem · Arma | home: aludren |
| **Arema** | a ley spring | aru + ema | flow-life | Old Eskenai | A-re-ma | Арема | Alema · Arem · Arma |  |
| **Grodsund** | Gundbor's city | grod + sund | forge of gold | Dwarvish | grod-SUND | Гродсунд | — |  |
| **Peidren** | a sacred grove | pai + dren | seed-wood | Elvish | PEI-dren | Пэйдрен | Peidren · Pedren · Paidren | home: aludren |
| **Taleska** | Aludren's city | taru + esk | waiting for the stars | Elvish | TA-les-ka | Талеска | Taleska · Taresk · Taresh | home: aludren |
| **Tokumb** | the great gate | tok + umb | gate of the deep | Dwarvish | to-KUMB | Токумб | — |  |
| **Umbsund** | a deep gold mine | umb + sund | deep of gold | Dwarvish | umb-SUND | Умбсунд | — |  |
| **Venduk** | the oath-stone (vend + duk: dd becomes d) | vend + duk | oath stone | Dwarvish | ven-DUK | Вендук | — |  |

### dimension

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Eskan** | the furthest dimension, the ancients' home, where Old Eskenai is still spoken | esk + -an | star-place | Old Eskenai | ES-kan | Эскан | Eskan · Eskan · Eskan |  |
| **Nopem** | a night dimension | nau + pem | night-world | Dwarvish loans and mountain names (frost) | NO-pem | Нопем | Naubema · Nopem · Naupem |  |

### ruin

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Drenduga** | a forest ruin | dren + duk | wood-stone | Elvish | DREN-du-ga | Дрендуга | Drenduga · Drenduk · Drendok |  |
| **Eskerem** | a star-watching ruin on a peak | esk + rem | star-watch | Old Eskenai | ES-ke-rem | Эскерем | Eskelema · Eskerem · Eskerem |  |
| **Ristok** | a mountain ruin with a dead gate | ris + tauk | broken gate | Dwarvish loans and mountain names (frost) | RIS-tok | Ристок | Ristauga · Ristok · Ristauk |  |

### boss

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Askaun, the Sand Keeper** | the desert's guardian | has + kaun | dry-keeper | Worn speech (humans, traders) | AS-kaun | Аскаун | Haskaun · Haskon · Askaun |  |
| **Borkon, the Height-Keeper** | the mountains' guardian | bor + kaun | mountain-keeper | Dwarvish loans and mountain names (frost) | BOR-kon | Боркон | Borkaun · Borkon · Borkaun |  |
| **Drenkaun, the Wood-Warden** | the forest's guardian | dren + kaun | tree-keeper | Elvish | DREN-kaun | Дренкаун | Drenkaun · Drenkon · Drengkaun |  |
| **Ekaskir, the Old Ice** | the tundra's guardian | eka + skir | old ice | Dwarvish loans and mountain names (frost) | E-kas-kir | Экаскир | Egaskila · Ekaskir · Ekaskir |  |
| **Nogom** | a deep-cave guardian | nau + gom | night-beast | Dwarvish loans and mountain names (frost) | NO-gom | Ногом | Naugoma · Nogom · Naugom |  |
| **Nokaun, Keeper of the Deep** | the ocean's guardian | nu + kaun | deep-keeper | Worn speech (humans, traders) | NO-kaun | Нокаун | Nugaun · Nukon · Nokaun |  |
| **Pokgom, the Mire Beast** | the swamp's guardian | pok + gom | rot-beast | Worn speech (humans, traders) | POK-gom | Покгом | Pokgoma · Pokgom · Pokgom |  |

### person

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Emalei** | the warden-queen | ema + rai | life-light | Elvish | E-ma-lei | Эмалэй | Emalei · Emare · Emrai | home: aludren |
| **Kvor Vendbrok** | the king | kvor · vend + brok | lord; oath-old | Dwarvish | KVOR vend-BROK | Квор Вендброк | — | byname: Old Oath |
| **Reiwoma** | the eldest, a seer | rai + wom | light-voice | Elvish | REI-wo-ma | Рэйвома | Reiwoma · Rewom · Raiwom | home: aludren |

### title

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Dundbrok** | the title of the eldest | dund + brok | kin-old | Dwarvish | dund-BROK | Дундброк | — |  |

### relic

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Kepesk** | a meteorite heart; traders say Kepesh | kep + esk | fall-star | Old Eskenai | KE-pesk | Кепеск | Kebeska · Chepesk · Kepesh |  |
| **Maunsen** | a tablet that teaches runes | maun + sen | mind-rune | Old Eskenai | MAUN-sen | Маунсен | Maunsen · Monsen · Maunsen | English: Mind-Rune |

### rune

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Aru** | one of the first four runes | aru | flow | Old Eskenai | A-ru | Ару | Alu · Ar · Aro |  |
| **Dau** | one of the first four runes | dau | fire | Old Eskenai | DAU | Дау | Dau · Do · Dau |  |
| **Skir** | one of the first four runes | skir | ice | Old Eskenai | SKIR | Скир | Sila · Skir · Skir |  |
| **Tauk** | one of the first four runes | tauk | gate | Old Eskenai | TAUK | Таук | Tauga · Tok · Tauk |  |

### spell

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Hesdau** | the lightning spell's name on its tome; Hesdo to dwarves, Esdau to traders | hes + dau | sky-fire | Old Eskenai | HES-dau | Хесдау | Hesdau · Hesdo · Esdau | English: Lightning |

### weapon

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Brumesk** | a star-steel hammer | brum + esk | hammer of the star | Dwarvish | bru-MESK | Брумеск | — |  |
| **Hesalu** | an elven bow | hes + aru | sky-thread | Elvish | HE-sa-lu | Хесалу | Hesalu · Hesar · Esro |  |
| **Naulisa** | an elven sword | nau + ris | night-breaker | Elvish | NAU-li-sa | Наулиса | Naulisa · Noris · Nauris |  |
| **Vorktem** | a dwarven axe | vork + tem | axe of iron | Dwarvish | vork-TEM | Ворктем | — |  |

### material

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **Chepesk** | the dwarves' word for meteorite iron | chepesk | fall-star | Dwarvish | CHE-pesk | Чепеск | — |  |
| **Esketem** | star steel's lore name | esk + tem | star-iron | Old Eskenai | ES-ke-tem | Эскетем | Eskedema · Esketem · Esketem | English: Star Steel |

### word

| name | what | build | means | tongue | say | ru | Soft · Frost · Worn | notes |
|---|---|---|---|---|---|---|---|---|
| **dokan** | what everyone calls a ruin | duk + -an | stone place | Worn speech (humans, traders) | DO-kan | докан | dugan · dukan · dokan |  |

## The elves' homes

- **Aludren** (`aludren`, dialect: soft): Old-growth or birch forest with ley nodes, where the ley lines run strongest (the report's pick; could move to the giant groves once they exist).

More homes and regional dialects go in `[people.elvish.homes]`; a dialect is a drift that extends `soft`.

## Dwarvish

| root | means | from | ru |
|---|---|---|---|
| **gund** | hold, hall | native | гунд |
| **kvor** | lord, king | native | квор |
| **brum** | hammer | native | брум |
| **vork** | axe | native | ворк |
| **vend** | oath | native | венд |
| **umb** | deep, under | native | умб |
| **sund** | gold | native | сунд |
| **grod** | forge | native | грод |
| **dund** | kin | native | дунд |
| **brok** | old | native | брок |
| **bor** | mountain | loan of bor (frost) | бор |
| **esk** | star | loan of esk (frost) | эск |
| **tem** | iron | loan of tem (frost) | тем |
| **duk** | stone | loan of duk (frost) | дук |
| **tok** | gate | loan of tauk (frost) | ток |
| **kon** | keep | loan of kaun (frost) | кон |
| **chepesk** | meteorite iron (fall-star) | loan of kep + esk (frost) | чепеск |
| **eskene** | the ancients (star-born) | loan of esk + nai (frost) | эскене |

Endings: **-ok** plural (*dundok, the kins*).

Bynames and true names: players mostly meet a dwarf's plain English byname (Old Oath, Ironhand); the true name, in Dwarvish, is learned through standing with the hold.

## Against the generic

- x, z, q and j used to look exotic (Xyrthos, Zyrael) `[xzqj]`
- th: stock fantasy, and Tolkien's `th`
- ae: stock fantasy (Aeth-, Ael-) `ae`
- y as a vowel `y(?![aeiou])`
- doubled letters `([a-z])\1`
- apostrophes, hyphens and accents (an epithet is set off with a comma) `['\-\u00c0-\u024f]`
- stock endings -ia, -ion, -oth, -wyn, -ryn `(ia|ion|oth|wyn|ryn)$`
- stock openings Aeth-, Ael-, Vael- `^(aeth|ael|vael)`
- the names language models keep producing `^(elara|lyra|kael|kaelen|thorne|eldoria|voss|silas)$`
- reads as Tolkien or generic elf: -iel, -wen, -dor, -ion, -las, -ndil `(iel|wen|dor|ion|las|ndil)$` (elvish)
- reads as Tolkien: gal-, cel-, mith- `^(gal|cel|mith)` (elvish)
- El- at the start of an elf's personal name `^el` (elvish; person)
- Tolkien's Khuzdul: Kh-, Khaz-, -dûm, -zul `^kh|(dum|dûm|zul)$` (dwarvish, dwarvish_loan)
- the Norse dwarf list (and Tolkien's) `^(durin|thorin|balin|dvalin|dwalin|dain|nain|thrain|thror|oin|gloin|ori|nori|dori|fili|kili|bifur|bofur|bombur|fundin)$` (dwarvish, dwarvish_loan)
- grand English mash-ups: an English name must be concrete and local (Bywater, not Stormwatch) `^(storm|shadow|blood|ember)|(forge|fall|heim|gard)$` (English names)
- grand titles `\b(the )?(eternal|unbroken)\b` (English names)

Bad-word lists (funny, rude or loaded in a language players read): en 40, ru 34, fr 21, de 18, mi 1 words.

## Russian letters

ai ай · au ау · ei эй · ch ч · sh ш · ng нг · kv кв · sk ск · a а · e е · e (at the start) э · i и · o о · u у · k к · g г · h х · w в · y й · p п · t т · b б · d д · m м · n н · s с · r р · f ф · l л · v в
