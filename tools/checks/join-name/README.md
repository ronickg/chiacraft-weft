# A guest keeps its name across a content switch

The bug (Phase 6, aa03294f): bob joined alice's world as "" although his `core:name` is `bob`.
Settings' ids came from sorting every setting by name, key bindings included (`core:key.<action>`),
so a world whose data package registers a key (the bell's client half) moved `core:name`'s id;
the game's values stayed at the old ids, and bob's game read the slot beside it. Now a package's
settings and keys sort after all the others (`registry::freeze`), and switching content sets the
packages' values again by name (`config::content_changed`).

Two copies, from the folder holding `target/release/weft`, about a minute:

```sh
rm -f name-ticket.txt name-mark-*
mkdir -p run-name-alice/config run-name-bob/config
echo "name = alice" > run-name-alice/config/core.txt
echo "name = bob" > run-name-bob/config/core.txt
./target/release/weft --run-dir run-name-alice --world name-alice-1 --packages examples/packages --script tools/checks/join-name/alice.txt &
./target/release/weft --run-dir run-name-bob --script tools/checks/join-name/bob.txt
```

Look for, in alice's log: `bob joined (account identity …)` (not ` joined`), and no
`joined with no name` warning; in bob's: `player name 'bob'` at start and `joining as 'bob'`.
`name-alice.png`: "bob joined the game" in the chat.
