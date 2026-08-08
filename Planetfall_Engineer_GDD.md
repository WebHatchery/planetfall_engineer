# Planetfall Engineer

**Master Game Design Document**  
**Working genre:** 3D real-time planetary-engineering puzzle game  
**Player role:** Chief Hydrological Engineer, Planetary Restoration Programme

## 1. High Concept

*Planetfall Engineer* is a mission-based engineering puzzle game about restoring hostile alien worlds. Players reshape terrain, build flow-control infrastructure, and react to evolving environmental hazards to route water, lava, gases, and exotic fluids through the remnants of ancient planetary machinery.

Every mission is a self-contained restoration contract: make a basin habitable, contain a geothermal eruption, deliver clean water to a dormant biomechanical complex, or activate an ancient rune-powered climate engine. Fluid systems run in real time, but the player can pause, inspect, and issue a precise plan before committing to it.

The central tension is simple: the planet was built to move immense forces, and the player must learn to guide rather than fight them.

## 2. Player Fantasy

> **“I am the engineer who makes a dead world breathe again.”**

The player is not a soldier, ruler, or factory owner. They are a systems-minded field engineer with a powerful planetary toolkit. They read a landscape like a circuit diagram, uncover the intent of vanished alien terraformers, and turn catastrophic flows into working ecosystems.

Success should feel earned through insight: a carefully placed spillway stops a flood; a diverted lava stream powers an ancient pump; steam condenses into the first rainfall over a barren valley.

## 3. Design Pillars

1. **Fluid puzzles with readable consequences.** Fluids must feel dynamic and physical while remaining understandable enough to plan around.
2. **Terrain is part of the circuit.** The landscape is an active engineering material, not scenery.
3. **Controlled real-time pressure.** Events unfold live, while pause-and-plan turns urgency into deliberate problem solving.
4. **Ancient systems, modern ingenuity.** Alien ruins create mystery and special rules; the player applies clear engineering logic to master them.
5. **Many valid solutions.** Missions reward robust systems, clever rerouting, efficient builds, and safe recovery—not one hidden “correct” layout.
6. **Restoration is visible.** Each successful mission should tangibly transform a hostile place into something more stable, alive, and beautiful.

## 4. Core Gameplay Loop

1. **Survey:** Read mission goals, terrain, fluid sources, weather forecasts, ruin functions, and protected areas.
2. **Plan:** Pause, sketch routes, set priorities, and choose where to place terrain edits and structures.
3. **Build:** Deploy conduits, pumps, gates, storage, power, purification, and ancient-interface devices within a limited budget.
4. **Run and observe:** Advance time; watch flows, pressure, power, contamination, and structural integrity.
5. **Adapt:** Respond to leaks, changing flow rates, storms, seismic shifts, and newly uncovered systems.
6. **Stabilise:** Meet the primary restoration target and keep the system stable through a verification window.
7. **Evaluate:** Receive a mission grade based on safety, efficiency, environmental recovery, resource use, and optional objectives.

## 5. Mission Structure

Each mission is a compact engineering scenario, generally 15–35 minutes on first completion. Missions have a clear beginning, a defined restoration brief, and a final state that can be verified.

### Mission anatomy

- **Briefing:** Narrative context, primary target, known threats, available equipment, and optional objectives.
- **Survey phase:** Free camera exploration, overlays, fluid-path previews, and optional scanning of unknown ruins.
- **Engineering phase:** Real-time operation with pause-and-plan available at all times.
- **Escalation:** One or more planned disruptions, such as a vent surge, storm front, thaw, or ruin activation.
- **Verification:** A short stability period once the primary objective is met.
- **Debrief:** Results, unlocks, replay challenges, environmental transformation summary, and lore discoveries.

Missions can include fixed elements for authored puzzle clarity and limited procedural variation—source intensity, storm timing, buried relic locations, or secondary leak positions—for replay value.

## 6. Fluid Simulation Philosophy

Fluid behaviour should privilege **clarity, predictability, and expressive interaction** over scientific completeness. The simulation represents volume, height, direction, pressure, temperature, contamination, and material state at a scale readable from the game camera.

### Design rules

- Fluids follow visible gradients; terrain contours and directional overlays communicate likely paths.
- Enclosed systems build pressure, requiring rated pipes, relief paths, valves, or expansion storage.
- Every material has a small set of legible properties rather than an opaque simulation model.
- Interactions create durable opportunities and risks: lava can create land, but destroys ordinary equipment; contamination can be filtered, diluted, isolated, or accidentally spread.
- The game clearly signals impending failure before it becomes irreversible, except in explicitly flagged high-risk challenges.
- Simulation fidelity may be abstracted away from the player; gameplay-facing behaviour must remain consistent.

### Player-readable overlays

Flow direction, depth, pressure, temperature, contamination, power, structural load, terrain grade, watershed boundaries, and predicted overflow paths are available as toggleable overlays.

## 7. Fluid Types and Interactions

| Material | Primary use | Main hazards | Key interactions |
|---|---|---|---|
| Water | Habitation, irrigation, cooling, transport | Flooding, erosion, contamination | Cools lava; becomes steam under heat; freezes in cold regions |
| Brine | Mineral extraction, thermal transfer | Salinity, corrosion, soil damage | Can be desalinated; lowers freeze point; contaminates freshwater |
| Lava | Geothermal power, land formation, ruin activation | Extreme heat, ignition, destruction | Cooled by water into rock; vaporises water; can seal channels |
| Steam | Atmospheric seeding, pneumatic power | Pressure bursts, heat | Condenses to water; drives turbines; causes humidity and rain effects |
| Cryofluid | Freezing, ice construction, cooling | Brittle structures, runaway freezing | Solidifies water; fractures hot materials; can preserve leaks temporarily |
| Acid | Resource processing, alien-biome cleansing | Corrosion, toxic exposure | Dissolved by neutraliser; attacks ordinary pipes; activates some relics |
| Toxic slurry | Waste containment, soil remediation | Persistent contamination | Settles in basins; can be separated, purified, or safely vitrified by lava |
| Nutrient solution | Ecological recovery, bio-dome supply | Algal bloom, contamination sensitivity | Converts restored basins into living terrain; requires clean water balance |
| Aetheric condensate | Late-game alien terraforming medium | Unstable resonance effects | Powers rune devices; changes gravity/flow behaviour in special zones |

Interactions should generate obvious, memorable outcomes: water meeting lava creates steam and new rock; steam crossing a cold condenser produces a rain-fed reservoir; acid erodes a blocked channel but can destroy its containment.

## 8. Terrain and Engineering Mechanics

The player uses terrain as a first-class tool.

- **Excavate:** Cut channels, retention basins, trenches, spillways, and foundations.
- **Raise and grade:** Build berms, levees, ramps, catchments, and protective high ground.
- **Compact and reinforce:** Stabilise slopes against erosion, seismic movement, or thermal damage.
- **Seal:** Line channels and basins to prevent infiltration or contamination.
- **Break and clear:** Remove weak rock, ice plugs, debris dams, and selected ruin obstructions.
- **Create terrain through chemistry:** Cool lava into rock, freeze water into temporary dams, or use acid to open specific geological barriers.

Terrain work has costs, time, and consequences. Steep channels accelerate flow but increase erosion; broad basins increase safety but consume space; deep excavation can expose hazards or forgotten infrastructure.

## 9. Construction System

Construction uses a clear build palette organised by function: flow control, transport, pressure and power, processing, terrain support, habitat support, and ancient interfaces.

### Rules

- Devices snap to valid surfaces, foundations, pipes, terrain nodes, or rune sockets as appropriate.
- Placement previews show footprint, connection points, slope constraints, expected flow direction, power demand, and hazards.
- Structures have material ratings for pressure, heat, cold, corrosion, and seismic stress.
- Build capacity is limited by mission budget, supply drops, available power, and/or fabrication time.
- Blueprints can be queued while paused; construction begins only when the player resumes or explicitly approves it.
- Critical structures can be upgraded rather than replaced, allowing recovery from an under-designed first attempt.

## 10. Pause-and-Plan Real-Time Gameplay

Time normally advances continuously. The player may pause at any moment to inspect overlays, rotate the camera, queue construction, issue gate and pump settings, set automation rules, and compare predicted outcomes.

Pause is a core expression tool, not a penalty. Some advanced challenge modes may restrict pausing or grade its use, but the standard campaign embraces thoughtful planning. Fast-forward is available when systems are stable; slow motion supports watching complex interactions unfold.

## 11. Dynamic Events and Disasters

Events create changing constraints rather than arbitrary punishment. Each is forecast through mission intel, sensors, visual tells, or a visible countdown when appropriate.

- Geothermal surges and lava overflows
- Flash floods, cloudbursts, and seasonal thaw
- Dust storms that reduce solar collection and visibility
- Seismic tremors that crack pipes, shift terrain, or open vents
- Ice shelf collapse and sudden cryofluid release
- Acid rain and corrosion waves
- Ancient-machine awakenings that change flow rules or power availability
- Biological bloom, contamination outbreaks, and invasive alien growth
- Meteor impacts in designated late-game scenarios

The player should usually be able to mitigate, reroute, or recover from a disaster. Catastrophic failure is reserved for neglected safety systems, clear mission boundaries, or optional high-stakes conditions.

## 12. Progression

Progression combines campaign unlocks, player mastery, and narrative discovery.

- New devices arrive when a new environmental problem has made their value intuitive.
- New fluids and interactions are introduced one or two at a time.
- Ancient ruins gradually reveal why the worlds failed and what the original terraformers were attempting.
- Completion unlocks optional contracts, challenge modifiers, cosmetic restoration landmarks, and sandbox components.
- Grades reward different styles—efficient, resilient, ecological, exploratory—without requiring perfection to progress.

## 13. Campaign Structure

The campaign follows a planetary restoration expedition across a connected but varied star sector. The opening worlds teach core flow engineering; later regions combine prior systems and introduce ancient rune networks with planet-scale stakes.

| Act | Theme | Player learning |
|---|---|---|
| I — First Landing | Barren volcanic world | Terrain, water routing, basic pressure, lava containment |
| II — The Frozen Archive | Ice moon and buried facilities | Phase change, freezing, thaw, storage, timed release |
| III — Poisoned Inheritance | Acidic industrial world | Separation, corrosion, purification, safe waste handling |
| IV — The Drowned Engines | Oceanic ruin world | Tides, floating infrastructure, massive reservoirs, redundancy |
| V — Glyphs of Genesis | Ancient terraforming nexus | Multi-fluid systems, rune logic, planetary-scale restoration |

Suggested launch campaign: 24–30 principal missions, with 8–12 optional contracts and a final multi-stage restoration operation.

## 14. Biome Ideas

- **Ashfall Basin:** Lava rivers, ash storms, scarce water, exposed geothermal infrastructure.
- **Glacier Vault:** Subsurface aquifers, movable ice dams, fragile ruins under permafrost.
- **Caustic Delta:** Acid rivers, corroded industrial relics, toxic marshes, neutralisation puzzles.
- **Azure Expanse:** Shallow alien sea, tide gates, floating collectors, drowned rune arrays.
- **Red Dune Reach:** Evaporation, sand burial, hidden aquifers, water-conservation challenges.
- **Verdant Scar:** A world beginning to recover; ecosystems become an engineering variable.
- **Obsidian Crown:** Volcanic mountain ring surrounding an ancient climate engine.
- **The Genesis Vault:** Final alien megastructure where each previous discipline combines.

## 15. Objectives

### Primary objectives

- Store a target volume of lava or water safely.
- Deliver a specified fluid to a facility at a required rate and purity.
- Fill, drain, or stabilise a basin.
- Generate and sustain a power target.
- Restore irrigation to a bio-dome or seed zone.
- Contain contamination below a threshold.
- Activate, repair, or safely shut down an ancient terraforming machine.
- Protect a research camp, evacuation route, or rare ecosystem.

### Optional objectives

- Use fewer resources or structures.
- Avoid terrain damage or preserve a landmark.
- Recover lost survey archives.
- Complete before a weather deadline.
- Maintain ecological health above a threshold.
- Achieve full containment with no emergency vents.
- Use a featured fluid interaction in a non-prescribed way.

## 16. Buildings and Devices

| Category | Examples |
|---|---|
| Flow control | Channels, culverts, floodgates, valves, weirs, spillways, check dams |
| Transport | Pipes, insulated conduits, elevated aqueducts, pumps, siphons, tunnels |
| Storage | Reservoirs, pressure tanks, lava crucibles, ice vaults, settling ponds |
| Power | Geothermal taps, steam turbines, flow turbines, solar fields, batteries |
| Processing | Filters, desalinators, neutralisers, condensers, separators, vitrifiers |
| Terrain support | Retaining walls, reinforcement anchors, sealant liners, foundations, bridges |
| Monitoring | Flow meters, pressure sensors, weather stations, seismic probes, survey drones |
| Ecology | Bio-dome inlets, irrigation towers, nutrient injectors, soil restorers |
| Ancient interfaces | Rune relays, glyph gates, resonance stabilisers, conduit keys, climate spires |

Most devices should be understandable in isolation and combine into systems. A player who can describe what a device does can reason about when to use it.

## 17. Win and Fail Conditions

### Winning

A mission succeeds when all mandatory objectives are met and the relevant systems remain within safe operating limits for a verification period. Narrative missions may add a final activation, departure, or restoration visualisation.

### Failing

Failure occurs when a hard safety boundary is crossed: protected personnel are lost, a key facility is destroyed, contamination exceeds an irreversible threshold, an ancient reactor catastrophically destabilises, or the mission runs out of an explicitly limited resource/time allowance.

Where possible, the game offers **soft failure states**—repair costs, reduced grade, lost optional objectives, temporary evacuation—so experimentation remains enjoyable. Players can restart, revert to a mission checkpoint, or continue in recovery mode where the design permits.

## 18. UI/UX Concepts

- **Cinematic engineering camera:** Smooth orbit, zoom from planetary vista to device-level view, terrain clipping and cutaway support.
- **Persistent mission strip:** Objective progress, time controls, alerts, budget/capacity, and quick access to overlays.
- **Layered overlays:** One-click access to flow, pressure, heat, contamination, power, structural risk, terrain grade, and objectives.
- **Readable alert hierarchy:** Informational, caution, urgent, and critical signals differentiated by colour, shape, sound, and screen placement.
- **Inspect panel:** Select anything to see inputs, outputs, limits, predicted issues, connected systems, and contextual actions.
- **Planning tools:** Ghost placement, build queue, route preview, measurement, annotation pins, and optional player-drawn flow plans.
- **Accessibility:** Full remapping, scalable UI, subtitle support, colourblind-safe overlay palettes, reduced motion, pause accessibility, and non-colour hazard indicators.

The interface should feel like a refined expedition-control system: technical enough to support mastery, never so dense that it obscures the landscape.

## 19. Art Direction

The visual identity blends grounded planetary engineering with monumental, rune-carved alien infrastructure.

- **Worlds:** Stylised realism with strong material readability—water glints, lava radiates heat, acid looks corrosive, ice is translucent and stressed.
- **Ancient ruins:** Geometric, weathered megastructures with restrained luminous glyphs; alien technology is imposing but interpretable.
- **Restoration:** Colour and life return visibly as missions succeed: clear water, mosses, vegetation, atmosphere, and changed skies.
- **Player equipment:** Practical expedition machinery, modular and legible from a distance, with clear operating states.
- **Composition:** Grand before-and-after vistas frame each mission; readable topography and flow paths take priority over visual noise.

## 20. Audio Direction

Audio reinforces scale, material behaviour, and engineering confidence.

- Fluids have distinct sonic signatures: heavy lava, rushing water, hissing steam, cracking ice, corrosive acid.
- Devices communicate state through restrained mechanical audio and escalating warning tones.
- Music is atmospheric, patient, and adaptive—sparse during planning, more rhythmic under system stress, expansive on restoration success.
- Ancient machines use tonal, harmonic motifs rather than generic “magic” sounds.
- Mix prioritises warnings and nearby physical activity; accessibility options include separate alert, ambience, and music controls.

## 21. Replayability

- Mission grades and medals across efficiency, resilience, ecology, and exploration.
- Optional objectives and alternate unlock paths.
- Challenge modifiers: restricted build kit, severe weather, no-pause, limited excavation, or altered source conditions.
- Multiple viable layouts and emergent recovery stories.
- Daily/weekly curated contracts can be considered after launch if they do not distract from authored content.
- Sandbox maps with adjustable sources, terrain, weather, and victory rules.

## 22. Level Editor

A post-launch-friendly editor can extend the game’s life significantly. It should use the same understandable building blocks as the campaign.

### Creator features

- Sculpt terrain; paint material zones; place fluid sources, ruins, buildings, hazards, and protected areas.
- Define objectives, budgets, unlocks, timed events, environmental thresholds, and briefing text.
- Test-play from the editor with all overlays and simulation controls.
- Package custom missions with thumbnail, difficulty guidance, tags, and optional narrative text.
- Browse, rate, bookmark, and share community missions through a curated in-game hub.

Campaign-only spoiler assets and unstable experimental systems can be restricted as needed.

## 23. Multiplayer Considerations (Future)

Multiplayer is not part of the initial release and should not compromise the single-player puzzle experience. The best future fit is cooperative engineering rather than competition.

- 2–4 players share a mission map and system state.
- Players can specialise informally: terrain, flow routing, power, monitoring, or emergency response.
- Shared pause requires a clear vote/host rule; an asynchronous planning mode may avoid friction.
- Co-op objectives should favour communication, not simultaneous busywork.
- Competitive modes are optional and lower priority: efficiency challenges, mirrored puzzle races, or leaderboards rather than direct sabotage.

## 24. Modding Potential

The game’s systemic nature supports modding well after its core tools and data rules are stable.

- Community missions and terrain packs are the first priority.
- Data-defined devices, fluids, objectives, biomes, and event sets can enable deeper creation.
- Visual/audio replacement packs and localisation support broaden participation.
- Clear versioning, dependency declarations, validation, and safe sandboxing are essential for a healthy ecosystem.
- Official content should retain a clear distinction from community content for campaign integrity and supportability.

## 25. MVP Scope

The MVP proves the player fantasy: read a landscape, route dangerous fluids, and visibly restore a world.

### Include

- One volcanic biome with 6–8 authored missions.
- Water, lava, steam, and one contamination mechanic.
- Terrain excavation, raising, channels, basic reinforcement, and simple sealing.
- Core devices: pipes, pumps, gates, reservoirs, spillways, turbines, sensors, filters, and one ancient interface.
- Pause, fast-forward, overlays, construction previews, alerts, save/restart, and mission grading.
- One escalating disaster type per mission family.
- Introductory narrative framing, briefing/debriefing, and a strong restoration before/after payoff.

### Explicitly defer

- Full campaign, all biomes, multiplayer, public level editor, deep mod API, procedural world generation, and complex ecosystem simulation.
- More than a small, fully tested set of fluid interactions.

## 26. Stretch Goals

- Additional fluid families: acid, cryofluid, brine, nutrient solution, aetheric condensate.
- Dynamic weather and seasonal systems.
- Larger multi-stage regional maps with persistent restoration state.
- Advanced automation, programmable control logic, and player templates.
- Wildlife/ecosystem restoration chains.
- Photo mode, cinematic mission replays, and restoration timelapses.
- Full level editor, mod support, and co-op missions.
- New Game Plus contracts with remixed systems and more extreme events.

## 27. Key Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Fluid behaviour becomes hard to read | Build overlays, previews, clear material rules, and generous warning lead time from the first prototype |
| Simulation creates performance or stability issues | Constrain the gameplay model by region, detail level, and authored mission scale; test worst-case scenarios early |
| Terrain freedom breaks authored puzzles | Design broad solution spaces with protected boundaries, budgets, and natural constraints rather than brittle exact solutions |
| Too many fluids overwhelm new players | Introduce one new material or interaction at a time; keep early missions focused |
| Real-time stress alienates puzzle players | Make pause central, provide slow motion and forgiving recovery, reserve no-pause for optional challenges |
| Buildings feel like generic city-builder clutter | Keep the build set compact, purpose-led, and visibly connected to flow problems |
| Ancient lore competes with engineering clarity | Use ruins to create readable mechanics and mission context, not opaque arbitrary rules |
| Scope expands across biomes and systems | Lock the vertical slice and MVP device/fluid list before campaign production |

## 28. Implementation Roadmap

### Phase 1 — Foundations and paper design

- Define a concise fluid property matrix and interaction rules.
- Prototype terrain-to-flow readability, pressure behaviour, pause controls, and the core overlay suite.
- Establish mission grammar, success/failure boundaries, and building placement language.
- Test with small greybox puzzles until multiple satisfying solutions consistently emerge.

### Phase 2 — Vertical slice

- Produce one polished volcanic mission sequence that includes survey, terrain work, water routing, lava containment, a dynamic event, ancient activation, and a restoration payoff.
- Validate the core UX, camera, audio feedback, art target, difficulty curve, and replay loop.
- Use playtests to remove ambiguity before adding content breadth.

### Phase 3 — MVP production

- Build the full first biome, six to eight missions, core device set, four-fluid interaction set, save flow, progression, and grading.
- Create reusable mission-authoring tools and a content checklist.
- Tune performance, simulation edge cases, accessibility, onboarding, and failure recovery.

### Phase 4 — Campaign expansion

- Add biomes in an order that compounds lessons: frozen systems, contamination, tides, then rune-driven endgame systems.
- Add devices only where their first use has a clear problem to solve.
- Develop narrative discoveries and restoration visuals alongside mission production.

### Phase 5 — Launch readiness

- Complete balancing, broad usability testing, difficulty settings, localisation preparation, QA for extreme system states, and final accessibility pass.
- Build replay challenges and sandbox support from stable campaign systems.
- Finalise onboarding, campaign pacing, polish, and player-facing documentation.

### Phase 6 — Post-launch

- Prioritise quality-of-life improvements, new authored contracts, balance updates, and community-requested tools.
- Evaluate editor, modding, and cooperative play only after the core single-player simulation and content pipeline are reliable.

## 29. Success Criteria

*Planetfall Engineer* succeeds when players can look at an unstable alien landscape, form a plan, execute it with understandable tools, survive a surprise, and feel genuine pride when the world visibly changes because of their engineering.

The game should be approachable in its first hour, intellectually rich after dozens of missions, and memorable for the moment a player turns a planetary disaster into the beginning of a living world.
