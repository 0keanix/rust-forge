# Testing Bevy systems headlessly

Names below were checked against bevy 0.20.1 on docs.rs. For another version, confirm each one in that version's
docs first.

## Build a headless App

- Start from `App::new()`, add `MinimalPlugins` (task pools, frame count, `TimePlugin` and a schedule runner; no
  window, no renderer), then add only the plugin under test. A test that needs `DefaultPlugins` is testing
  rendering, not a gameplay rule; split the rule out instead.
- Never call `App::run` in a test: the schedule runner would loop. Call `App::update` once per frame you want
  to simulate.
- Add what the plugin under test expects from elsewhere yourself: `StatesPlugin` (`bevy::state::app::StatesPlugin`)
  before any `init_state`, and resources such as `ButtonInput<KeyCode>` (see below).
- Write a small helper `fn app() -> App` per test module that builds this setup, so each test states only what
  differs.
- Spawn entities with `app.world_mut().spawn(...)` and keep the returned entity id; set tunables with
  `app.insert_resource(...)` so a test does not depend on the default numbers.

## Advance time deterministically

- Real time makes tests flaky. Insert the `TimeUpdateStrategy` resource with
  `TimeUpdateStrategy::ManualDuration(Duration::from_millis(...))`: each `update` then advances `Time` by exactly
  that duration.
- Do not assume the very first `update` advances time by the full step; run one warm-up `update` after setup, read
  the starting value, and assert on changes from there.
- Simulate a period by looping `update` a known number of times, and choose step and tunables so expected values
  are exact or compare floats with a tolerance.
- For rules in `FixedUpdate`, step by a duration that is a whole multiple of the fixed timestep, so the number of
  fixed runs per `update` is known.

## Send input

- `MinimalPlugins` has no input plugin. Add the resource directly with `app.init_resource::<ButtonInput<KeyCode>>()`
  and drive it: `app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft)`, then
  `release(...)` when the test wants the key up.
- Without the input plugin nothing clears the "just pressed" flags between frames; call `clear()` on the resource
  after the frame that should see the edge, or test `pressed` instead of `just_pressed`.
- Prefer systems that read an intent (a component or message such as "sprinting") produced by a small input
  system; then most tests set the intent directly and only one test covers the key mapping.
- To inject a message, call `app.world_mut().write_message(...)` before `update`.

## Assert on state

- Read components with `app.world().entity(id).get::<T>()` and resources with `app.world().resource::<T>()` after
  the `update` calls; assert on the values the rule promises (stamina dropped by drain rate times elapsed time,
  stayed at zero, regenerated only after the delay).
- Test one rule per test and include the boundary: exactly at zero, just before and just after a delay ends.
- To query many entities, use `app.world_mut().query::<&T>()` and iterate it over `app.world()`.
- If an assertion fails by exactly one frame, check system ordering in the plugin before changing the test; it is
  usually a missing `.before`/`.after` or `.chain()`.
