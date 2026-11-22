# testkit

Scripted runs, checks and test commands for chiacraft (never shipped). The script steps are listed
in `src/script.rs` (`COMMANDS`); how to run a script is in the project's `CLAUDE.md` ("Trying it in
the game").

## Compare mode: like-for-like look shots

Every look compared against a reference (Bliss, vanilla Minecraft) runs in compare mode, so no
script has to remember the list of controls (Ronald, 11-22: "make the game easier to compare by
like pausing time no clouds etc").

- In a script: `compare on` after the first `loaded`, then `hour 12` before each shot (the clock
  set, a second, frozen, the sky settled 8 s). Or launch with `--config testkit:compare_mode=1`,
  which turns it on at the first `loaded`.
- In the chat: `/compare on`, `/compare off` (the settings go back as they were and time runs).
- It sets: time frozen; weather clear and pinned; no clouds of any kind (soft clouds, their
  shadows, the pack's cirrus `base:look_cirrus_off`, Weft's own clouds by `core:cloud_distance`
  0); fixed exposure (`base:look_auto_exposure` 0); bloom off; no wind and no mist (`/air 0 0 0
  mist`); no mob spawning; the HUD hidden; GUI scale 2, FOV 70. Scripted runs already pin the
  shader clock.
- One control back at a time, for the step that tests it: `compare on exposure=1`, `bloom=2`,
  `clouds=1`, `mist=0.3`, `wind=1`, or any setting by full name (`base:look_leaf_sun=3`).
- Its pair on the Minecraft side is `~/mcshaders/bliss_ctl.sh` on the PC: Bliss with volumetric
  clouds off (vanilla clouds already off), auto exposure off at 0.02237 and bloom 0, shot from
  the same spot and FOV at the same hours (`bliss_mcw.txt`: /tp 165.5 200 168.5, yaw 300, pitch
  12; hours 05-19). A compare is always "our compare mode against `bliss_ctl`".
- Many variants in one launch: after `compare on` and `hour`, `set_config <key> <value>`, `wait
  2`, `shot` for each variant (settings the shader pack reads change live).
