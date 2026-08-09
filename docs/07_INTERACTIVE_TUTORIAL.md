# Interactive Tutorial Specification

## 1. Outcome

On first play of L01, the tutorial MUST cause the player to perform—not merely
read—the core loop:

1. pan, zoom, and rotate the 3D engineering camera;
2. move the survey cursor and inspect terrain;
3. select and recover a visible planetary fabrication deposit;
4. pause, spend finite stock on terrain and a committed channel plan;
5. run simulation and see water change route;
6. operate a floodgate in response to a visible beacon-flood warning;
7. hold the objective stable and finish without free or regenerating stock.

The tutorial is event-driven. It advances from admitted commands and simulation
events, never from elapsed real time or a click on “next” alone.

## 2. Tutorial state

```text
TutorialState
  tutorial_id: string
  current_step_id: string
  completed_step_ids: ordered set
  hint_level: 0..2
  ticks_in_step: u32          // simulation ticks; frozen while paused
  real_seconds_in_step: f32  // hint presentation only; not authoritative/hash
  dismissed_prompt: bool
  skip_confirmed: bool
```

`completed_step_ids`, `current_step_id`, and `skip_confirmed` are saved. Camera
movement progress does not need an exact camera position after load; a completed
camera step stays completed. Hint timing is presentation state and MUST NOT
affect replays or hashes.

## 3. Step definition

Each data-authored step contains:

```text
id, title_text_id, body_text_id
completion_predicate
allowed_command_kinds[]
focus_target (optional UI/world ID)
pause_policy (preserve | force_pause | force_run_1x)
hint_after_seconds [10, 25]
checkpoint_on_complete
on_enter_effects[]
on_complete_effects[]
```

Focus treatment highlights but does not intercept unrelated inspect/camera
input. Only state-changing commands outside `allowed_command_kinds` may be
tutorial-locked. A locked command returns `tutorial_locked` and the prompt
explains the currently available action.

## 4. L01 step sequence

### T01 — `tutorial_l01_welcome`

- Enter: simulation paused; meltwater source disabled.
- Prompt: restoration brief in at most 55 words; identify objective basin.
- Allowed: camera, overlay, select, inspect, dismiss prompt, tutorial skip.
- Complete: briefing prompt dismissed.
- Focus: objective basin world zone, then release focus.

### T02 — `tutorial_l01_move_camera`

- Prompt: “Pan, zoom, and rotate the engineering view to inspect the marked basin.”
- Complete only after the camera target moves at least 3 cells, zoom changes by
  at least one discrete level, and yaw changes by one 90-degree quarter. Any
  supported path is valid: bindings, drag/wheel, or visible camera controls.
- The world remains live 3D during rotation; stepped cliffs, water surface, and
  objective markers must remain depth-correct and the basin must be pickable.
- Hint 1 points to controls legend. Hint 2 offers a clickable “Center basin”
  sequence that pans, zooms, then rotates; using all three assisted controls
  completes the step and records `assisted=true` for QA only.
- Checkpoint: no.

### T03 — `tutorial_l01_move_cursor`

- Prompt: select the marked first runoff cut at (8,10).
- Complete from accepted `MoveSurveyCursor` or `SelectCell` with target in the
  runoff focus zone. Pointer selection MUST use §11's 3D screen ray; keyboard
  selection moves the same world-space reticle. Both are equivalent.
- Hint 1 animates the survey reticle without moving it. Hint 2 expands the valid
  focus zone to any of the three marked cuts.

### T04 — `tutorial_l01_inspect_grade`

- Enter: enable terrain-grade overlay and open inspector for selected cell.
- Prompt identifies elevation and predicted downhill arrows.
- Complete when grade overlay has been visible and the inspector open for one
  rendered frame, then the player dismisses the explanation.
- This is the only informational acknowledgement required between actions.

### T05 — `tutorial_l01_select_deposit`

- Prompt: select the marked `wreckage_cache` at (5,6).
- Complete from accepted `MoveSurveyCursor` or `SelectCell` on that deposit.
- Inspector states `28 fabU AVAILABLE TO RECOVER` and explains that mission
  fabrication is finite and local; it must not call the stock credits or money.

### T06 — `tutorial_l01_recover_deposit`

- Prompt: “Tap RECOVER to reclaim 28 fabU from the marked wreckage.”
- Complete only from accepted `RecoverDeposit(wreckage_cache)` and its
  `deposit_recovered` event. Prompt dismissal cannot advance it.
- HUD changes from `FAB 0` to `FAB 28`; the world marker becomes visibly
  depleted and remains selectable with `DEPLETED` inspect text.
- Checkpoint on complete.

### T07 — `tutorial_l01_pause_plan`

- Prompt: pause simulation before shaping the first barrier.
- Source remains disabled, so the player cannot miss a live consequence.
- Complete from `SetTimeControl(Paused)`.

### T08 — `tutorial_l01_raise_first`

- Prompt: raise runoff cut (8,10) once to form the first barrier.
- Complete on an accepted terrain raise.

### T09 — `tutorial_l01_raise_remaining`

- Prompt first focuses runoff cut (14,10), then (20,10). The player must select
  and raise each once through visible map and `RAISE` targets.
- Complete after two accepted terrain raises at those exact authored cells.
- The same stable step ID stores a `completed_targets` bitset so save/load after
  the second barrier cannot grant or repeat fabrication spending.

### T10 — `tutorial_l01_place_channel`

- Prompt focuses the visible `CHANNEL` palette entry and pad (23,9).
- Player selects `CHANNEL`, places the 3D ghost, and taps visible `COMMIT`.
- Complete only when `CommitPlan` creates the channel. HUD must preview 2 `fabU`
  and finish at `FAB 11`, reflecting the three prior 5 `fabU` raises.

### T11 — `tutorial_l01_run_and_observe`

- Enter: enable the 20 `vU`/tick meltwater source and prompt the player to tap 1x.
- Complete when time is running; the visible route and dam objective update live.

### T12 — `tutorial_l01_beacon_warning`

- Prompt: observe the 25% inlet while routed water begins backing toward the
  protected beacon. All camera, inspect, overlay, pause, and time controls stay
  available; unrelated construction is unlocked.
- Complete only from the authoritative advisory `threshold_crossed` event for
  the beacon. Enter T13 paused so its ten-tick hard-failure grace period cannot
  expire behind the explanation.

### T13 — `tutorial_l01_open_gate`

- Prompt: “Select the inlet floodgate, then tap FULL before the beacon floods.”
- Complete from accepted `ConfigureDevice` setting the authored gate to 100%.
- The prompt names both the visible `FULL` control and the protected target.

### T14 — `tutorial_l01_stabilize`

- Prompt: keep time running while the dam fills and holds for 100 ticks.
- All normal L01 commands unlocked. Complete on mission success.
- Debrief states the causal chain: recovered wreckage funded three barriers and
  a channel; the open gate then protected the beacon and filled the dam.

## 5. L02 first-use power onboarding

L02 adds a short event-driven onboarding strip, saved and replayed under
`tutorial_l02_power`. It does not repeat camera or placement basics:

1. `tutorial_l02_inspect_grid`: tap the visible `POWER` overlay and inspect
   `geothermal_tap`; complete when `SUPPLY 3 eU / DEMAND 0 eU` is visible.
2. `tutorial_l02_bootstrap_pump`: build and enable the 3 `eU` pump; complete on
   its first powered transfer from authored geothermal supply.
3. `tutorial_l02_place_turbine`: place a flow turbine in `turbine_run`; complete
   on commit after its 14 `fabU` preview was visible.
4. `tutorial_l02_observe_generation`: run until `power_generated` records at
   least 1 `eU`; explain that measured tick-N flow powers tick N+1.
5. `tutorial_l02_manage_demand`: enable reservoir release while the pump runs;
   complete after the player observes one deterministic brownout and either
   restores turbine supply or toggles a consumer to clear the deficit.

The first brownout pauses once and names allocation order: safety, transport,
process, interface, then stable machine ID. It is a forecast and recovery
lesson, not an immediate failure. Replay may skip this strip, but skipping
never grants power, fabrication, devices, or altered settings.

## 6. Skip, replay, and reset

- “Skip tutorial” requires one confirmation and is available from T01 onward.
- Skip completes tutorial predicates, unlocks the full L01 build kit, enables
  the source, leaves simulation paused, and does not place or edit anything.
- Skip does not recover deposits or grant fabrication; L01 remains playable
  from its authored zero stock through the visible `RECOVER` action.
- Campaign completion does not require `tutorial_complete` when skip confirmed.
- Replaying L01 defaults tutorial off but offers “Replay guided tutorial.”
- Reset Mission clears tutorial state. Reset Checkpoint preserves steps through
  the checkpoint and returns world/commands to that checkpoint snapshot.
- A player can reopen the current prompt and controls legend at any time.

## 7. Presentation and accessibility

- Prompts occupy no more than 30% of screen width and never cover their focus.
- Focus uses outline, icon, and optional pulse; it cannot rely on color alone.
- Reduced-motion replaces pulse/camera tween with a static outline and line.
- Control text is generated from current action bindings, never hard-coded keys.
- Screen-reader-ready text strings include target name, state, and required
  action even if the visual prompt uses an icon.
- Simulation auto-pauses for blocking tutorial explanations and restores the
  prior time mode only where a step explicitly says so.

## 8. Automated tutorial tests

- A full reference command stream reaches T14 and mission success with the
  exact 28 recovered / 17 spent / 11 remaining fabrication ledger.
- Every step ignores unrelated UI-only commands without advancing.
- Every state-changing command not allowed by the current step is rejected with
  `tutorial_locked` and leaves state/hash unchanged.
- Mouse-equivalent and keyboard-equivalent command streams complete T02/T03;
  pointer streams cover all four yaw quarters and at least two zoom levels.
- Save/load at the start of every step restores that exact step and world hash.
- Hint timers do not affect authoritative state or replay hash.
- Reset checkpoint from T09 restores the deposit depleted, exact remaining
  stock, and source disabled without duplicating the 28 `fabU` yield.
- Skip from T01, T08, and T12 produces a playable paused mission with no free
  placements, recovery, or fabrication mutation.
- L02 power onboarding passes with normal, skipped, save/load-at-each-step, and
  intentional-brownout streams; current supply, next-tick turbine output,
  allocation order, and cleared deficit hashes repeat exactly.
