# SPEC.md: Offline Voxel Medieval Open-World RPG

## Summary

A cross-platform, offline, single-player open-world medieval RPG with a procedurally generated, deterministic world, stylized voxel art, and a destructible environment. Villages and their people react to the player's decisions and to destruction, and the world state persists across saves. The game runs on Windows, Linux, macOS, Android, iOS (phone and tablet) and web from one codebase.

## Problem

The draft does not state who has this problem or why it matters now. See Q1.

## Goals

- G-1: Same seed produces byte-identical world data on every supported platform (verified by a fixture test).
- G-2: Every main quest is completable with each playable class (verified by a test matrix: main quests × classes).
- G-3: Frame budgets met per render tier (see NFR-1).
- G-4: No progress lost after a crash: the last autosave loads on next launch (see NFR-5).
- G-5: Writers change dialogue and quest text by editing `.yarn` files without code changes.
- G-6: Gameplay rules run in unit tests without starting the engine.

## Non-goals

- Multiplayer and any online service. The game requires no network.
- User-generated content.
- Voice acting.
- Photorealism. The art direction is stylized voxel art on every tier.

## Users and scenarios

Personas (the draft names only the first two; the rest are inferred and marked):

- **Player**: plays offline on any supported device, chooses a class, explores, fights, destroys, decides.
- **Writer**: edits `.yarn` dialogue and quest content as text. No code access required.
- **Developer** (inferred): implements modules, runs unit and fixture tests.

Main flows:

1. Player starts a new game with a seed, picks warrior or mage.
2. Player walks out of the starting village; the world streams in around them.
3. Player talks to a villager; a Yarn dialogue runs and records a choice.
4. Player starts a main quest; its objectives appear in the journal.
5. Player switches between first and third person with the view switch.
6. Player sets fire to a wooden structure; it burns, loses support, and collapses; the village reacts and the world records it.
7. A key building is threatened; the quest either protects it or adapts (see FR-13).
8. Player closes the game mid-session; autosave preserves state.
9. Player relaunches after a crash; the last valid save loads.
10. Player on a phone uses touch controls for the same actions as keyboard or gamepad.

## Requirements

**FR-1: The system must generate the same world from the same seed.**

- Given a seed S, when the world is generated twice on any platform, then both outputs are identical.
- Given two different seeds, when generated, then the outputs differ.

**FR-2: The system must generate regions, roads, landmarks and villages.**

- Given a generated world, when inspected, then it contains regions of type forest, farmland, mountains and coast; roads; and landmarks of type ruins, camp, castle and shrine; and generated villages.

**FR-3: Villages must contain houses, workplaces and a gathering place.**

- Given a generated village, when inspected, then it has at least one house, one workplace and one gathering place.

**FR-4: Each person must have a name, role, personality, daily routine, memory of the player, and a reaction to reputation.**

- Given a villager, when queried, then all six fields are present.
- Given the player's reputation changes, when the villager is next addressed, then the villager's reaction reflects the new reputation.
- Given the player interacts with a villager, when the interaction ends, then the villager's memory of the player is updated.

**FR-5: The player must choose a class at game start: warrior or mage.**

- Given a new game, when the player is asked for a class, then exactly warrior and mage are offered.
- Given a class is chosen, when the game starts, then the class cannot be changed in that save (see Q5).

**FR-6: Each class must differ in combat and abilities.**

- Given the warrior, when equipped, then weapons, armor and stamina apply.
- Given the mage, when equipped, then spells, mana and positioning apply.
- Given a class ability, when used, then it can act on the world (see FR-11).

**FR-7: Every main quest must be completable with every class, in different ways.**

- Given any main quest and any playable class, when the quest's stages are played, then the quest reaches the completed state.
- Given two classes, when the same main quest is completed, then the completion path differs in at least one objective or outcome.

**FR-8: Quests must have three weights: main, side and small tertiary.**

- Given a main quest, when defined, then it is an ordered story with acts.
- Given a side quest, when defined, then it has its own story and rewards.
- Given a tertiary quest, when defined, then it is either generated or authored.

**FR-9: Each quest must have stages, objectives, a state and a journal entry.**

- Given a quest, when its state is read, then it is exactly one of: not started, active, completed, failed.
- Given an objective is completed, when the quest updates, then the journal records the change.
- Given a quest reaches a rewarded state, when the reward is granted, then it is granted once only: a second completion attempt grants nothing.

**FR-10: Dialogue must use Yarn Spinner `.yarn` scripts.**

- Given a `.yarn` file is edited, when the game next loads dialogue, then the change appears without a code change or rebuild.
- Given a script, when run, then it can read and write world state (reputation, quest stage).
- Given a script, when run, then it can call typed commands: give item, start quest, change mood (see Interfaces).
- Given a script contains a choice, when the player selects an option, then the choice is recorded (see FR-12).

**FR-11: The world must be destructible.**

- Given a wooden structure, when fire reaches it, then it burns.
- Given a stone structure, when force is applied above its threshold, then it breaks.
- Given a structure loses support, when support is checked, then it collapses into pieces.
- Given a collapse, when the pieces settle, then the world state records the destroyed structure.

**FR-12: Decisions must be recorded with context and have consequences.**

- Given the player makes a decision, when recorded, then the record includes the decision, the context (scene, time, participants), and the affected entities.
- Given a recorded decision, when evaluated, then the state of at least one of: quests, prices, people, villages, ending is changed.

**FR-13: Quest items, key buildings and key people must be protected or the quest must adapt.**

- Given a quest item, when a destruction event would destroy it, then the event is blocked or the quest adapts (see Q12).
- Given a key building, when destroyed, then the affected quest adapts.
- Given a key person, when killed or destroyed, then the affected quest adapts.

**FR-14: Villages must react to destruction, and the world must remember it.**

- Given a structure in a village is destroyed, when the village is next simulated, then at least one villager reaction is triggered.
- Given the player saves and reloads, when the world loads, then destroyed structures remain destroyed.

**FR-15: Debris must be bounded.**

- Given a destruction event, when debris is spawned, then the active debris count does not exceed the tier cap (see NFR-1 and Q11).

**FR-16: The system must provide first and third person views with one camera rig and a view switch.**

- Given the game is running, when the player triggers the view switch, then the view changes between first and third person without a reload.
- Given either view, when the camera moves, then it uses the same rig.

**FR-17: The system must choose a render tier from the device, with manual override.**

- Given a device, when the game starts, then a tier (Low, Mid or High) is selected automatically.
- Given the player sets a tier manually, when set, then the manual tier is used and persists in settings.

**FR-18: Touch controls must map to the same actions as keyboard and gamepad.**

- Given touch input, when the player uses the stick, look drag or action buttons, then the same actions fire as with keyboard or gamepad.
- Given any touch target, when measured, then it is at least 48 dp.

**FR-19: Saves must be autosaved, crash-safe, versioned, and backed up before upgrade.**

- Given the game is running, when autosave triggers, then the state is written without corrupting the previous save.
- Given a save file, when read, then it carries a version number.
- Given a save from an older version, when loaded, then a backup of the original is written before migration.

**FR-20: UI must be built from atoms, molecules and organisms that only show state and emit intents.**

- Given a UI component, when inspected, then it reads state from its inputs and emits intents; it holds no game logic.

**FR-21: Modules must communicate only through events or interfaces.**

- Given module A needs data from module B, when implemented, then it uses an event or interface from B, not a direct reference to B's internals.

**FR-22: Gameplay rules must be pure and engine-independent.**

- Given a gameplay rule, when tested, then it runs in a unit test without starting the engine.

<!-- module: offline-saves -->

### FR-23: Offline play

The system must work completely without a network.

- Given no internet connection, when the player starts the game, then everything works and no error is shown
- Given the game is running, when it needs data, then it reads local files only

### FR-24: Saving

The system must never lose the player's progress.

- Given a world, when the player plays, then it autosaves at an interval and on quit
- Given the game closes unexpectedly, when the player opens it again, then they lose at most the last autosave interval and the save is not corrupt
- Given several worlds, when the player opens the list, then each shows its name, last played time and size

### FR-25: Versions

The system must keep old saves working.

- Given a save from an older version, when the player opens it, then it is upgraded after a backup is made
- Given a save from a newer version, when the player opens it, then the game explains it cannot open it and changes nothing

<!-- module: voxel-world -->

### FR-26: Streaming

The system must show a large world that loads around the player without stalls.

- Given the player moves, when new areas come into range, then they load without the frame rate dropping below the target
- Given the view distance setting, when the player changes it, then the world shows that distance and memory use stays within the budget
- Given distant terrain, when it is far away, then a simpler version is shown and swaps in without a visible pop

### FR-27: Editing

The system must let the player break and place voxels.

- Given a target within reach, when the player breaks it, then it is removed, feedback plays and lighting updates
- Given a free target position, when the player places a block, then it appears and cannot overlap the player or other solids
- Given an edit, when the game saves, then the edit is still there on the next load

### FR-28: Materials

The system must give each voxel kind rules for how it behaves.

- Given a voxel kind, when it is defined, then it has a name, a color or material, hardness, sound and whether it is solid, transparent or emissive
- Given an emissive voxel, when it is in the world, then it lights its surroundings

<!-- module: world-generation -->

### FR-29: Seeds

The system must generate the same world from the same seed.

- Given a seed, when a world is created twice, then both worlds are identical
- Given no seed, when a world is created, then a random seed is used and shown so it can be shared

### FR-30: Terrain and biomes

The system must create varied terrain.

- Given a new area, when it is generated, then it has terrain, water, biomes and caves that blend at their borders
- Given two neighbouring chunks, when they are generated at different times, then they join without gaps or cliffs at the seam

### FR-31: Structures

The system must place structures and resources in a believable way.

- Given a biome, when it is generated, then its structures and resources appear at the defined rarity
- Given a structure, when it is placed, then it fits the terrain and never overlaps another structure

<!-- module: medieval-world -->

### FR-32: Regions and landmarks

The system must make a world with places that have a character of their own.

- Given a new world, when it is generated, then it has regions (for example forest, farmland, mountains, coast) with their own look, weather and creatures
- Given a region, when the player explores it, then it holds landmarks such as ruins, camps, castles and shrines placed with purpose
- Given two settlements, when the world is generated, then roads connect them following the terrain

### FR-33: Exploring

The system must reward exploring without forcing it.

- Given the player discovers a place, when they arrive, then it appears on the map with a name
- Given the player travels far, when they choose to, then fast travel to discovered places is available under the rules set for it

### FR-34: Time and weather

The system must change over time.

- Given the game clock, when a day passes, then light, weather and what people do change with it

<!-- module: villages-npcs -->

### FR-35: Villages

The system must generate villages that feel inhabited.

- Given a village site, when it is generated, then it has houses, a gathering place and workplaces that fit its size and region
- Given a village, when the player visits, then its name, buildings and inhabitants are the same on every visit

### FR-36: People

The system must give each person an identity.

- Given a villager, when they are created, then they have a name, a role, a personality and a short history
- Given a day, when time passes, then each person follows a routine (work, eat, rest) and is found in the right place
- Given the player talks to a person again, when they return, then the person remembers what happened between them

### FR-37: Reputation

The system must make people react to the player.

- Given the player's deeds, when they enter a village, then people greet, avoid or trade differently depending on their standing

<!-- module: dialogue -->

### FR-38: Writing

The system must let writers create conversations without touching code.

- Given a Yarn script, when a writer adds or edits a node, then the game plays it the next time without a code change
- Given a script with an error, when the game loads it, then the error names the file and line and the game does not crash

### FR-39: Playing

The system must show conversations clearly.

- Given a person who has something to say, when the player talks to them, then the line, the speaker and the choices appear
- Given a choice, when the player picks it, then the conversation continues down that branch
- Given a line, when the player wants to go faster, then they can skip the typing effect and advance

### FR-40: State

The system must connect conversations with the world.

- Given a variable such as reputation or a quest stage, when a node checks it, then it shows the options that apply
- Given a command in a script (give item, start quest, change mood), when the line runs, then the game does it once

### FR-41: Languages

The system must support more than one language.

- Given a chosen language, when a conversation plays, then the lines appear in that language and fall back to the original when missing

<!-- module: quests -->

### FR-42: Kinds of quests

The system must support quests of different weight.

- Given the main story, when the player advances, then main quests unfold in order and drive the plot
- Given a person or place, when the player meets them, then side quests offer their own stories and rewards
- Given the world, when the player does small things (deliver, hunt, find), then minor quests give short goals that can be generated or authored

### FR-43: Progress

The system must track what the player has done.

- Given a quest, when it starts, then it has stages, objectives and a state (not started, active, completed, failed)
- Given an objective, when the player completes it, then the quest advances and the player is told
- Given several active quests, when the player opens the journal, then each shows its goal, where to go and what is left

### FR-44: Rewards and failure

The system must give outcomes that matter.

- Given a finished quest, when the player completes it, then rewards (items, experience, reputation) are given once
- Given a quest that can fail, when its condition is met (a person dies, time runs out), then it fails with a clear message and consequences

<!-- module: choices-consequences -->

### FR-45: Decisions

The system must offer decisions that matter.

- Given a moment of choice, when the player decides, then the game records the decision with its context
- Given a decision, when the player considers it, then the options are clear and do not hide that they have consequences

### FR-46: Consequences

The system must make the world respond.

- Given a recorded decision, when the player returns later, then quests, people, prices or places reflect it
- Given reputation with a group, when the player acts, then it rises or falls and changes how the group treats them
- Given the end of the story, when it arrives, then the ending reflects the key decisions made

### FR-47: World state

The system must keep one source of truth for what happened.

- Given a save, when the player loads it, then all decisions and their effects are restored

<!-- module: character-classes -->

### FR-48: Creating a character

The system must let the player choose who they are.

- Given a new game, when the player creates a character, then they choose a name, an appearance and a class (warrior, mage or others)
- Given a class, when it is shown, then the player sees what it is good at, how it plays and its starting equipment

### FR-49: Playing the class

The system must make each class feel different.

- Given a warrior, when they fight, then they rely on weapons, armor and stamina
- Given a mage, when they fight, then they rely on spells, mana and positioning
- Given any class, when the player explores, then they can use class abilities on the world (for example breaking, burning, shielding, moving things)

### FR-50: Growth

The system must let characters grow.

- Given experience, when the player levels up, then they earn points to improve abilities, attributes or skills
- Given equipment and skills, when the player changes them, then stats update at once and show what changed

<!-- module: combat-destruction -->

### FR-51: Combat

The system must make fighting responsive and readable.

- Given an attack, when the player uses it, then it connects with clear feedback (animation, sound, hit effects, camera) and a visible result
- Given an enemy, when it attacks, then the player can read the attack early enough to respond
- Given damage, when it is dealt, then it follows the rules of weapon, spell, armor and resistance

### FR-52: Destruction

The system must let the world break in ways that matter.

- Given a destructible structure, when it takes enough damage, then it breaks into pieces that fall and leave debris
- Given a structure that loses support, when it is broken, then the unsupported part collapses
- Given a spell or weapon, when it hits, then it can affect the world according to its type (fire burns wood, force breaks stone)

### FR-53: Consequences and cost

The system must keep destruction under control.

- Given a village building, when the player destroys it, then people react and the world remembers it
- Given a very destructive moment, when many pieces fall, then the frame rate stays within the budget by limiting active debris

<!-- module: game-input -->

### FR-54: Devices

The system must be playable with keyboard and mouse or a gamepad.

- Given a connected gamepad, when the player uses it, then the game switches to it and shows its button glyphs
- Given keyboard and mouse, when the player uses them, then the game switches back and shows key glyphs

### FR-55: Remapping

The system must let players change every control.

- Given the controls screen, when the player picks an action and presses a key or button, then it is bound and saved
- Given a conflict, when two actions share a key, then the player is told and can resolve it

### FR-56: Options

The system must offer options that help different players.

- Given the settings, when the player changes sensitivity, inversion, dead zones or hold versus toggle, then it applies at once
- Given a one-handed or limited-mobility setup, when the player binds actions, then every action can be reached

<!-- module: first-person -->

### FR-57: Movement and look

The system must let the player move and look in first person with control that responds immediately.

- Given the game is running, when the player moves the mouse, then the view turns with no added delay and no smoothing unless enabled
- Given the player presses a movement key, when they hold it, then the character walks, sprints, crouches or jumps as bound
- Given the frame rate changes, when the player moves, then the speed stays the same

### FR-58: Interaction

The system must let the player act on what they look at.

- Given a block or object within reach, when the player looks at it, then it is highlighted and a prompt shows the action
- Given a highlighted target, when the player presses the action key, then the action happens once and gives feedback

### FR-59: Comfort

The system must let players tune the view to feel comfortable.

- Given the settings, when the player changes field of view, sensitivity or head bob, then the change applies at once and is remembered
- Given "reduce motion" is on, when the player moves, then head bob, sway and camera shake are off

<!-- module: frame-budget -->

### FR-60: Frame rate

The system must hold a steady frame rate on the target hardware.

- Given the reference scene, when the player moves through it, then the game holds the target frame rate (proposed: 60 fps with 1% lows above 45 fps)
- Given a heavy moment such as loading chunks or an explosion, when it happens, then no single frame takes longer than 33 ms

### FR-61: Loading and memory

The system must start fast and stay within its memory.

- Given a cold start, when the game opens, then the main menu shows within the target time (proposed: 5 seconds)
- Given a long session, when the player plays for an hour, then memory use stays flat

### FR-62: Measuring

The system must let the team see where time goes.

- Given a debug overlay, when it is on, then it shows frame time, the cost of each render pass, chunks in flight and memory
- Given a change to rendering, when it is made, then the frame time before and after is reported

<!-- module: crafting-inventory -->

### FR-63: Inventory

The system must let the player carry and organise items.

- Given an item is collected, when there is room, then it is added to the inventory and the hotbar shows it
- Given a full inventory, when the player collects an item, then it stays in the world and the player is told

### FR-64: Crafting

The system must let the player make new items from others.

- Given the player has the ingredients, when they craft a recipe, then ingredients are used and the result is added
- Given missing ingredients, when the player opens a recipe, then it shows what is missing

### FR-65: Tools

The system must make tools matter.

- Given a tool, when the player uses it on a voxel, then speed and drops depend on the tool and the material
- Given a tool with durability, when it is used, then it wears out and breaks at zero

<!-- module: lighting-effects -->

### FR-66: Light and atmosphere

The system must light the world so it feels real and readable.

- Given the time of day, when it changes, then sun, sky, fog and color grade change together without a pop
- Given a light source in the world, when it is on, then it lights and shadows its surroundings
- Given fog or haze, when the player looks into the distance, then far terrain fades toward the horizon color

### FR-67: Cinematic effects

The system must offer film-like effects the player can control.

- Given the graphics settings, when the player turns on volumetric light, ray-traced light, depth of field, motion blur or color grading, then each one applies and can be turned off on its own
- Given an effect, when it is shown in settings, then it shows its quality levels and what it costs in frame time

### FR-68: Quality tiers

The system must run on a range of hardware.

- Given hardware that cannot run an effect, when the game starts, then the effect is off or lowered and the player is told
- Given the lowest tier, when the game runs, then it still looks coherent and meets the frame rate target

<!-- module: device-quality -->

### FR-69: Choosing a tier

The system must pick a sensible quality for each device.

- Given the first launch, when the game inspects the device, then it picks a tier (Low, Mid, High) from its graphics capability and memory
- Given the player changes the tier, when they confirm, then it applies and is remembered for that device
- Given the game runs slowly, when the frame rate stays below the target for several seconds, then it suggests a lower tier and does not change it without asking

### FR-70: One look on every device

The system must look like the same game everywhere.

- Given any tier, when a scene is shown, then it uses the same palette, models and light colors
- Given the lowest tier, when the game runs, then lighting is still polished: soft shadows, ambient occlusion, fog and glow

### FR-71: Power and heat

The system must be kind to batteries and heat.

- Given a mobile device, when the player turns on the frame limiter, then the game caps at the chosen rate (proposed: 30 or 60 fps)
- Given the device gets hot or low on battery, when the system reports it, then the game offers to lower quality

<!-- module: third-person -->

### FR-72: Following the character

The system must keep the character in view while the player moves and looks around.

- Given the character moves, when the player moves the mouse or stick, then the camera orbits around them at a set distance
- Given a wall or block behind the character, when the camera would enter it, then the camera moves closer and returns when there is room
- Given the camera is very close, when it overlaps the character, then the character fades instead of blocking the view

### FR-73: Aiming

The system must let the player aim precisely.

- Given the player aims, when they hold the aim input, then the camera moves over the shoulder and the crosshair shows the real target
- Given the player changes shoulder, when they press the shoulder key, then the camera swaps sides smoothly

### FR-74: Changing view

The system must let the player switch between camera modes.

- Given third person, when the player presses the view key, then the game switches to first person without moving the character
- Given a saved preference, when the game starts, then it opens in the last used view

<!-- module: touch-controls -->

### FR-75: On-screen controls

The system must be playable with touch alone.

- Given a touch device, when the game starts, then it shows a move stick, a look area and the main action buttons
- Given the player drags on the look area, when they move their finger, then the view turns at the chosen sensitivity
- Given the player uses several fingers, when they move and look at once, then both work without interfering

### FR-76: Switching input

The system must handle more than one input at a time.

- Given a keyboard, mouse or gamepad is connected to a touch device, when the player uses it, then touch controls hide and the matching glyphs show

<!-- module: performance -->

### FR-77: Response time

The system must answer fast enough that people do not wait to do their work.

- Given a common action, when a person does it, then the result shows within the target below (proposed: 1 second for 95% of requests)
- Given a screen with data, when it opens, then its main content shows within the target below (proposed: 2.5 seconds on an average connection)
- Given an action that takes longer, when it runs, then the person sees progress and can do something else

### FR-78: Perceived speed

The system must feel fast even when work takes time.

- Given a screen that loads data, when it waits, then it shows the layout of what is coming instead of a blank page
- Given a change the person makes, when it is likely to succeed, then it shows at once and is corrected if the server refuses it

### FR-79: Size and load

The system must stay fast as data and people grow.

- Given a very long list, when it is shown, then it loads in parts and stays fast to scroll
- Given many people using it at once, when the load grows up to the target below (proposed: ten times today's), then response times stay within their targets
- Given a request for too much data, when it is made, then it is limited or paged and says so

## Non-functional requirements

**NFR-1: Frame budgets by render tier (proposed).**

- Low: 30 fps on a mid-range phone.
- Mid: 60 fps on a tablet or integrated GPU.
- High: 60 fps on a desktop GPU.
- While streaming, no frame exceeds twice the tier's frame budget (the draft's rule).

**NFR-2: Platforms.** Windows, Linux, macOS, Android, iOS (phone and tablet) and web, from one codebase.

**NFR-3: Offline.** Every feature works with no network connection.

**NFR-4: Art direction.** One stylized voxel art direction on every tier. Lighting features by tier: soft shadows, ambient occlusion, fog and glow on all tiers; bounce light on High only. Exact per-tier feature split is proposed (see Q13).

**NFR-5: Save safety.** Autosave is crash-safe. A crash during write must not corrupt the previous valid save. Saves are versioned. A backup is created before any upgrade.

**NFR-6: Touch targets.** Minimum 48 dp.

**NFR-7: Localization.** Dialogue is localized. When a translation is missing, the original language is shown.

**NFR-8: Testability.** Gameplay rules have unit tests independent of the engine. Quest completability is covered by a main quest × class test matrix. World generation has fixture tests keyed by seed.

Security, privacy, accessibility beyond touch target size: not stated in the draft. See Q14.

## Data

| Entity         | Fields                                                                                       | Relations                                                                     |
| -------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| World          | seed, generation version                                                                     | has Regions, Roads, Landmarks, Villages                                       |
| Region         | type (forest, farmland, mountains, coast), bounds                                            | belongs to World                                                              |
| Landmark       | type (ruins, camp, castle, shrine), position                                                 | belongs to World                                                              |
| Village        | name, position, reputation                                                                   | has Buildings, People                                                         |
| Building       | kind (house, workplace, gathering place), structure state, support state, key flag           | belongs to Village; has Pieces                                                |
| Piece          | material (wood, stone, other), position, state                                               | belongs to Building                                                           |
| Person         | name, role, personality, routine, memory of player, reputation reaction                      | belongs to Village                                                            |
| Player         | class (warrior, mage), reputation, mood, position                                            | owns Inventory, Quest states                                                  |
| Class          | name, weapons, armor, stamina or mana, abilities                                             | referenced by Player                                                          |
| Quest          | weight (main, side, tertiary), state (not started, active, completed, failed), rewarded flag | has Acts/Stages, Objectives, Rewards, Journal entries                         |
| Stage          | order, state                                                                                 | belongs to Quest; has Objectives                                              |
| Objective      | description, state                                                                           | belongs to Stage                                                              |
| Reward         | content, granted flag                                                                        | belongs to Quest                                                              |
| JournalEntry   | quest, text, timestamp                                                                       | belongs to Quest                                                              |
| Decision       | id, context (scene, time, participants), affected entity references                          | recorded by Dialogue or Action; affects Quest, Price, Person, Village, Ending |
| DialogueScript | file path (.yarn), language, version                                                         | runs against World state                                                      |
| Save           | format version, world state snapshot, timestamp, backup reference                            | one per slot (see Q15)                                                        |

Inventory and crafting exist as modules in the draft base list. Their fields are not specified. See Q7.

## Interfaces

**Yarn commands (typed):**

- `giveItem(itemId: string, count: int) -> void`
- `startQuest(questId: string) -> bool`: returns false if the quest is already active, completed or failed.
- `changeMood(personId: string, mood: enum) -> void`
- Reads: `getReputation(villageId | global) -> int`, `getQuestStage(questId) -> int`
- Writes: `setQuestStage(questId, stage: int) -> void`
- Full command list: see Q8.

**Events (names proposed):**

- `DecisionRecorded(decisionId, context)`
- `QuestStateChanged(questId, oldState, newState)`
- `ObjectiveCompleted(questId, objectiveId)`
- `StructureDamaged(buildingId, pieceId, cause)`
- `StructureCollapsed(buildingId, pieceIds)`
- `VillageReacted(villageId, reason)`
- `RewardGranted(questId, rewardId)`
- `SaveWritten(slot, version)`

**Screens and UI organisms (names proposed):**

- HUD, Dialogue, Journal, Settings (includes render tier override), Save/Load.

**Input actions (shared by keyboard, gamepad and touch):** move, look, action, view switch, and class ability actions (names proposed).

## Edge cases

- **Save write interrupted.** Load the previous valid save. Never load a partially written file.
- **Save version newer than the game.** Refuse to load and report the version. Do not overwrite.
- **Save version older than the game.** Back up the original, then migrate. If migration fails, keep the backup and do not overwrite the original.
- **Quest item is inside a structure being destroyed.** Block the destruction or adapt the quest (FR-13). Pick the rule in Q12.
- **Key person killed.** Affected quest adapts; the quest must still be completable with each class (FR-7).
- **Destruction exceeds debris cap.** Stop spawning debris at the cap; the structure still collapses logically.
- **Reward granting requested twice.** Grant once; the second request is a no-op.
- **Choice invalidates an active quest.** Quest state changes through QuestStateChanged; the journal records the change.
- **Translation missing for a line.** Show the original language line.
- **Yarn script error.** Report the file and line; the rest of the game continues with the affected dialogue unavailable.
- **Manual render tier higher than the device can sustain.** Use the override; the frame budget may be missed. Report in settings (see Q13).
- **Device switches between touch and keyboard mid-session.** Same actions apply; no mode change required.
- **Seed produces different world on a different platform.** Treat as a bug; fixture tests must catch it.
- (FR-24) A full disk stops saving with a clear message and keeps the last good save
- (FR-25) Backups are limited in number so they do not fill the disk
- (FR-26) Moving very fast (falling, teleport) still loads the destination before the player is shown inside empty space
- (FR-27) Breaking a block that holds up others follows the rules set for support and falling blocks
- (FR-28) Transparent and liquid voxels render in the right order and do not hide what is behind them
- (FR-29) Chunks generated in a different order give the same result
- (FR-30) The spawn point is always on safe, open ground
- (FR-32) The same seed always places the same landmarks and roads
- (FR-32) Generated places never block the main story paths
- (FR-33) Places generated far from the player are not built until needed
- (FR-35) A village changed by the player (destroyed house, rebuilt wall) keeps those changes
- (FR-36) A person blocked by the player or by a change in the world finds another route or waits
- (FR-36) A person who dies is gone from routines, quests and dialogue, and the world reacts in a consistent way
- (FR-39) Leaving a conversation midway (damage, distance, pause) ends it cleanly and keeps the variables set so far
- (FR-40) Reloading a save restores the variables and the nodes already visited
- (FR-41) Long lines and different alphabets fit in the dialogue box without clipping
- (FR-42) Generated minor quests never repeat the same target twice in a row
- (FR-43) Doing quest steps out of order or skipping ahead never breaks the quest
- (FR-43) A quest that points to a person or place that no longer exists is repaired or closed with an explanation
- (FR-45) A decision made by accident can be understood afterwards in the journal or a history log
- (FR-46) Conflicting decisions resolve by clear precedence rules, never by a random outcome
- (FR-47) Flags are named and listed in one place so writers and designers can check them
- (FR-48) The class chosen is kept in the save and can appear in dialogue checks
- (FR-49) Every main quest can be finished with every class, possibly in different ways
- (FR-50) Rebuilding a character (respec) follows clear rules and never loses items
- (FR-52) Debris settles and is removed or merged after a time so that physics does not run forever
- (FR-52) Quest items, key buildings and people are protected from being lost for good, or the quest adapts
- (FR-53) On weaker devices, destruction is smaller in scale but follows the same rules
- (FR-54) Unplugging a gamepad during play pauses the game and shows a message
- (FR-55) Reserved keys such as the pause key cannot be left unbound
- (FR-57) Looking straight up or down stops at the pole and never flips the view
- (FR-57) Losing window focus releases the mouse and stops movement
- (FR-58) Reach is measured from the eyes, and blocks behind other blocks cannot be targeted
- (FR-60) Alt-tabbing, resizing and changing monitor never drop the game to a slow state
- (FR-61) A world with many edits loads in the same time as a new one within a small margin
- (FR-63) Dropping, splitting and stacking items follow the same rules everywhere
- (FR-64) A recipe is atomic: it either completes or changes nothing
- (FR-66) Entering a cave or building blends the lighting over about a second
- (FR-67) "Reduce motion" turns motion blur, shake and bloom bursts off at once
- (FR-68) Changing a tier applies without restarting where possible
- (FR-69) A browser without the needed graphics features starts in the lowest tier or explains what is missing
- (FR-70) Light and shadow settings never change gameplay, such as what is visible to the player
- (FR-71) Backgrounding the app pauses rendering and audio and resumes without a reload
- (FR-72) In tight spaces the camera never shows the inside of a block or the void
- (FR-73) The crosshair never points at the character's own body
- (FR-75) A finger sliding off a button never leaves an action stuck on
- (FR-59) Controls stay out of system gesture zones (edges, home bar) and notches
- (FR-76) Rotating the device keeps the game running and re-lays out the controls
- (FR-77) A slow connection or an old device: the main task still works, only slower
- (FR-77) A request takes too long: it ends with an error the person can retry, not with an endless wait
- (FR-78) A fast change is shown and then refused by the server: the person is told and the screen returns to the real state
- (FR-79) One account has a hundred times more data than the average: it does not slow down the others

## Open questions

- **Q1.** What problem does this game solve, for whom, and why now? The draft does not say.
- **Q2.** World size and number of villages?
- **Q3.** Which classes beyond warrior and mage, if any?
- **Q4.** Which decisions are permanent, and which can be reversed?
- **Q5.** Can the player change class after the start?
- **Q6.** How many endings?
- **Q7.** What do inventory and crafting do? What items exist and how are they obtained?
- **Q8.** What is the full list of Yarn commands, with parameter types?
- **Q9.** Which languages must be supported at launch?
- **Q10.** Minimum supported devices per platform (OS version, RAM, GPU)?
- **Q11.** How much of the world can be destroyed on each tier? Debris cap per tier?
- **Q12.** When a quest item sits inside a destructible structure, does destruction get blocked or does the quest adapt? Which per case?
- **Q13.** Exact lighting features per tier beyond the split in NFR-4. Is the tier shown in settings with its effect on the budget?
- **Q14.** Privacy, security and accessibility requirements (beyond touch target size)? Is there any data leaving the device, even crash reports?
- **Q15.** Number of save slots? Autosave frequency and retention of backups?
- **Q16.** What is the reputation scale (range, per-village or global, thresholds)?
- **Q17.** What does a villager's memory of the player store, and how long is it kept?
- **Q18.** What does "mood" mean for a person: which values and how do they decay?
- **Q19.** Does world generation change between game versions? If so, how are existing saves handled for the same seed?
- **Q20.** Which view (first or third person) is the default?
- **Q21.** How does the player get the mage and warrior ability lists and their balance? Who owns balancing data?
- Q22 (FR-23): Is there any optional online feature (cloud saves, sharing seeds) or is the game fully offline?
- Q23 (FR-24): How often does autosave run and how many backups are kept?
- Q24 (FR-26): How large is the world and is it infinite, bounded or made of separate levels?
- Q25 (FR-27): Can everything be destroyed or only some materials, and does the world rebuild?
- Q26 (FR-28): How small is a voxel (block size or micro voxel) and how many kinds are needed at launch?
- Q27 (FR-30): Which biomes, how large, and how do they influence the look and the gameplay?
- Q28 (FR-31): Which structures and resources exist at launch and how rare are they?
- Q29 (FR-32): How big is the world and how many regions and landmark types exist at launch?
- Q30 (FR-33): Is there a map from the start or is it earned by exploring?
- Q31 (FR-34): How long is a game day in real time, and do seasons exist?
- Q32 (FR-35): How many villages and people must be active at once on the lowest device?
- Q33 (FR-36): Which parts of a person are generated and which are written by hand for story characters?
- Q34 (FR-38): Who writes the scripts and which tool do they use to preview them?
- Q35 (FR-41): Which languages are needed at launch, and is dialogue voiced?
- Q36 (FR-42): How many main acts and side quests are planned for launch?
- Q37 (FR-42): How are minor quests generated: from templates, from world state, or by hand?
- Q38 (FR-45): Can decisions be undone, and is there any permanent choice the player should be warned about?
- Q39 (FR-46): How many endings and key decisions are planned?
- Q40 (FR-48): Which classes exist at launch beyond warrior and mage, and can the player mix them?
- Q41 (FR-50): Is the progression by levels, by use, by skill trees, or by equipment?
- Q42 (FR-51): Is combat real time, with pause, or turn-based, and is there a lock-on?
- Q43 (FR-52): Which materials break, which burn and which resist, and does the world rebuild?
- Q44 (FR-54): Which gamepads and layouts must be supported at launch?
- Q45 (FR-57): Which speeds and jump height define the feel (blocks per second, jump in blocks)?
- Q46 (FR-59): What are the default and allowed ranges for field of view?
- Q47 (FR-60): What is the target hardware, resolution and frame rate for each quality tier?
- Q48 (FR-61): What are the memory and install-size limits?
- Q49 (FR-63): How many slots and what stack sizes are there?
- Q50 (FR-64): Are recipes known from the start or discovered?
- Q51 (FR-67): Which effects are part of the identity of the game and which are options?
- Q52 (FR-68): What is the minimum hardware and the frame rate promised on it?
- Q53 (FR-69): What are the minimum devices and the frame rate promised on each tier?
- Q54 (FR-71): What are the memory, download and install-size limits on each platform?
- Q55 (FR-72): Which distance and height feel right by default, and can players change them?
- Q56 (FR-74): Is switching views allowed everywhere or locked in some places?
- Q57 (FR-75): Which actions get buttons (jump, break, place, sprint, crouch), and which sit in a menu?
- Q58 (FR-59): Is there an aim-assist or auto-target for touch, and how strong?
- Q59 (FR-77): Which actions are the most important and which time is acceptable for each?
- Q60 (FR-77): Which devices and connections must it work well on?
- Q61 (FR-79): How many people, records and requests per day must it handle at launch and in a year?

## Milestones

Proposed delivery order, each a shippable vertical slice:

1. **M1: Deterministic world on desktop.** Seed-based generation (FR-1, FR-2), one region rendered at High tier, camera with view switch (FR-16), save and load of world state (FR-19 basic). Desktop only.
2. **M2: Village and people.** Villages with houses, workplaces, gathering place (FR-3); people with the six fields (FR-4); Yarn dialogue with read/write state and typed commands (FR-10).
3. **M3: Class and first quest.** Class selection (FR-5), class combat basics (FR-6), one main quest completable with both classes (FR-7), quest model and journal (FR-8, FR-9).
4. **M4: Choices and consequences.** Decision recording (FR-12), reputation and reactions (FR-4), state restored on load.
5. **M5: Destruction.** Wood burns, stone breaks, structures collapse (FR-11); village reactions and persistence (FR-14); debris cap (FR-15); protection and quest adaptation (FR-13).
6. **M6: Platform spread.** Render tier selection and override (FR-17), touch controls (FR-18), Linux, macOS, Windows, then Android, iOS, web.
7. **M7: Localization and hardening.** Localized dialogue with fallback (NFR-7), save migration and backup (NFR-5), frame budget verification per tier (NFR-1).
