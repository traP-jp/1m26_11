"""Host-side tests for the production button firmware state machine."""

import unittest

from firmware.button_firmware import (
    BUTTONS,
    CONTROLS,
    DEBOUNCE_MS,
    LONG_PRESS_MS,
    POLL_INTERVAL_MS,
    PRESSED,
    RELEASED,
    ButtonState,
    canonical_frame,
)


def linear_ticks_diff(new, old):
    return new - old


def wrapping_ticks_diff(period):
    half_period = period // 2

    def ticks_diff(new, old):
        return ((new - old + half_period) % period) - half_period

    return ticks_diff


class ButtonDriver:
    """Drive raw transitions at exact millisecond offsets."""

    def __init__(
        self,
        control="up",
        initial_level=RELEASED,
        started_at=0,
        ticks_diff=linear_ticks_diff,
        tick_period=None,
    ):
        self.now = started_at
        self.level = initial_level
        self.tick_period = tick_period
        self.state = ButtonState(control, initial_level, self.now, ticks_diff)

    def _tick(self, delta_ms):
        self.now += delta_ms
        if self.tick_period is not None:
            self.now %= self.tick_period

    def sample_after(self, delta_ms, level=None):
        self._tick(delta_ms)
        if level is not None:
            self.level = level
        return self.state.poll(self.level, self.now)

    def advance_after_startup(self):
        return self.sample_after(DEBOUNCE_MS)

    def begin_transition(self, level, after_ms=1):
        return self.sample_after(after_ms, level)

    def confirm_transition(self, after_ms=DEBOUNCE_MS):
        return self.sample_after(after_ms)


class CanonicalFrameTests(unittest.TestCase):
    def test_all_gpio_control_mappings_and_runtime_constants(self):
        self.assertEqual(
            BUTTONS,
            (
                (2, "up"),
                (3, "down"),
                (4, "left"),
                (5, "right"),
                (6, "red"),
                (7, "yellow"),
                (8, "green"),
            ),
        )
        self.assertEqual(CONTROLS, tuple(control for _, control in BUTTONS))
        self.assertEqual(POLL_INTERVAL_MS, 1)
        self.assertEqual(DEBOUNCE_MS, 20)
        self.assertEqual(LONG_PRESS_MS, 700)

    def test_canonical_frames_are_compact_ascii_json_lines(self):
        for _, control in BUTTONS:
            for gesture in ("short_press", "long_press"):
                frame = canonical_frame(control, gesture)
                self.assertEqual(
                    frame,
                    '{{"v":1,"control":"{}","gesture":"{}"}}\n'.format(
                        control, gesture
                    ),
                )
                frame.encode("ascii")
                self.assertNotIn(" ", frame)
                self.assertEqual(frame.count("\n"), 1)
                self.assertTrue(frame.endswith("\n"))

    def test_unknown_frame_values_are_rejected(self):
        with self.assertRaises(ValueError):
            canonical_frame("unknown", "short_press")
        with self.assertRaises(ValueError):
            canonical_frame("up", "tap")


class DebounceTests(unittest.TestCase):
    def setUp(self):
        self.button = ButtonDriver()
        self.assertIsNone(self.button.advance_after_startup())

    def test_transition_is_not_confirmed_at_19_ms(self):
        self.assertIsNone(self.button.begin_transition(PRESSED))
        self.assertIsNone(self.button.confirm_transition(19))
        self.assertEqual(self.button.state.stable_level, RELEASED)
        self.assertIsNone(self.button.state.press_started_ms)

    def test_transition_is_confirmed_at_exactly_20_ms(self):
        self.assertIsNone(self.button.begin_transition(PRESSED))
        self.assertIsNone(self.button.confirm_transition(20))
        self.assertEqual(self.button.state.stable_level, PRESSED)
        self.assertEqual(self.button.state.press_started_ms, self.button.now)
        self.assertFalse(self.button.state.armed)

    def test_press_bounce_does_not_create_an_event_or_start_early(self):
        self.assertIsNone(self.button.begin_transition(PRESSED))
        self.assertIsNone(self.button.sample_after(7, RELEASED))
        self.assertIsNone(self.button.sample_after(5, PRESSED))
        self.assertIsNone(self.button.sample_after(19))
        self.assertEqual(self.button.state.stable_level, RELEASED)
        self.assertIsNone(self.button.sample_after(1))
        self.assertEqual(self.button.state.stable_level, PRESSED)

        self.assertIsNone(self.button.begin_transition(RELEASED, after_ms=100))
        self.assertEqual(
            self.button.confirm_transition(),
            canonical_frame("up", "short_press"),
        )

    def test_release_shorter_than_debounce_does_not_end_or_duplicate_press(self):
        self.assertIsNone(self.button.begin_transition(PRESSED))
        self.assertIsNone(self.button.confirm_transition())

        self.assertIsNone(self.button.begin_transition(RELEASED, after_ms=100))
        self.assertIsNone(self.button.sample_after(8, PRESSED))
        self.assertIsNone(self.button.sample_after(6, RELEASED))
        self.assertIsNone(self.button.sample_after(19))
        self.assertEqual(self.button.state.stable_level, PRESSED)
        self.assertEqual(
            self.button.sample_after(1),
            canonical_frame("up", "short_press"),
        )
        self.assertTrue(self.button.state.armed)
        self.assertIsNone(self.button.sample_after(100))

    def test_press_shorter_than_debounce_is_ignored(self):
        self.assertIsNone(self.button.begin_transition(PRESSED))
        self.assertIsNone(self.button.sample_after(19, RELEASED))
        self.assertIsNone(self.button.sample_after(DEBOUNCE_MS))
        self.assertEqual(self.button.state.stable_level, RELEASED)
        self.assertIsNone(self.button.state.press_started_ms)


class GestureTests(unittest.TestCase):
    def make_press(self, raw_hold_ms, control="up"):
        button = ButtonDriver(control=control)
        self.assertIsNone(button.advance_after_startup())
        self.assertIsNone(button.begin_transition(PRESSED))
        self.assertIsNone(button.confirm_transition())

        # Since both edges use the same debounce duration, the elapsed time
        # between confirmed transitions equals the elapsed time between raw
        # transitions.
        self.assertGreaterEqual(raw_hold_ms, DEBOUNCE_MS)
        self.assertIsNone(
            button.begin_transition(
                RELEASED, after_ms=raw_hold_ms - DEBOUNCE_MS
            )
        )
        return button, button.confirm_transition()

    def test_699_ms_is_short_press(self):
        _, frame = self.make_press(699)
        self.assertEqual(frame, canonical_frame("up", "short_press"))

    def test_700_ms_is_long_press(self):
        _, frame = self.make_press(700)
        self.assertEqual(frame, canonical_frame("up", "long_press"))

    def test_holding_button_emits_no_repeat(self):
        button = ButtonDriver()
        self.assertIsNone(button.advance_after_startup())
        self.assertIsNone(button.begin_transition(PRESSED))
        self.assertIsNone(button.confirm_transition())

        for elapsed_ms in (100, 600, 700, 5_000):
            self.assertIsNone(button.sample_after(elapsed_ms))

        self.assertIsNone(button.begin_transition(RELEASED))
        self.assertEqual(
            button.confirm_transition(), canonical_frame("up", "long_press")
        )
        self.assertIsNone(button.sample_after(1_000))

    def test_rapid_distinct_presses_each_emit_once(self):
        button = ButtonDriver(control="red")
        self.assertIsNone(button.advance_after_startup())
        frames = []

        for _ in range(5):
            self.assertIsNone(button.begin_transition(PRESSED, after_ms=1))
            self.assertIsNone(button.confirm_transition())
            self.assertIsNone(button.begin_transition(RELEASED, after_ms=30))
            frames.append(button.confirm_transition())

        self.assertEqual(
            frames,
            [canonical_frame("red", "short_press")] * 5,
        )

    def test_buttons_keep_independent_press_times(self):
        up = ButtonDriver(control="up")
        down = ButtonDriver(control="down")
        self.assertIsNone(up.advance_after_startup())
        self.assertIsNone(down.advance_after_startup())

        self.assertIsNone(up.begin_transition(PRESSED))
        self.assertIsNone(up.confirm_transition())
        self.assertIsNone(down.begin_transition(PRESSED, after_ms=100))
        self.assertIsNone(down.confirm_transition())

        self.assertIsNone(down.begin_transition(RELEASED, after_ms=100))
        self.assertEqual(
            down.confirm_transition(),
            canonical_frame("down", "short_press"),
        )
        self.assertIsNone(up.begin_transition(RELEASED, after_ms=800))
        self.assertEqual(
            up.confirm_transition(),
            canonical_frame("up", "long_press"),
        )


class StartupTests(unittest.TestCase):
    def test_startup_high_is_armed_immediately(self):
        button = ButtonDriver(initial_level=RELEASED)
        self.assertTrue(button.state.armed)
        self.assertEqual(button.state.stable_level, RELEASED)

    def test_press_immediately_after_startup_high_is_accepted(self):
        button = ButtonDriver(initial_level=RELEASED)
        self.assertIsNone(button.begin_transition(PRESSED, after_ms=1))
        self.assertIsNone(button.confirm_transition())
        self.assertFalse(button.state.armed)
        self.assertIsNone(button.begin_transition(RELEASED, after_ms=100))
        self.assertEqual(
            button.confirm_transition(), canonical_frame("up", "short_press")
        )
        self.assertTrue(button.state.armed)

    def test_startup_low_release_only_arms_and_next_press_emits(self):
        button = ButtonDriver(initial_level=PRESSED)
        self.assertIsNone(button.advance_after_startup())
        self.assertFalse(button.state.armed)

        self.assertIsNone(button.begin_transition(RELEASED, after_ms=500))
        self.assertIsNone(button.confirm_transition())
        self.assertTrue(button.state.armed)

        self.assertIsNone(button.begin_transition(PRESSED))
        self.assertIsNone(button.confirm_transition())
        self.assertIsNone(button.begin_transition(RELEASED, after_ms=100))
        self.assertEqual(
            button.confirm_transition(), canonical_frame("up", "short_press")
        )


class TickWrapTests(unittest.TestCase):
    def test_debounce_crosses_tick_wrap(self):
        period = 2_048
        button = ButtonDriver(
            started_at=2_040,
            ticks_diff=wrapping_ticks_diff(period),
            tick_period=period,
        )
        self.assertIsNone(button.begin_transition(PRESSED))
        self.assertIsNone(button.confirm_transition(19))
        self.assertEqual(button.state.stable_level, RELEASED)
        self.assertIsNone(button.confirm_transition(1))
        self.assertEqual(button.state.stable_level, PRESSED)
        self.assertFalse(button.state.armed)

    def test_long_press_threshold_crosses_tick_wrap(self):
        period = 2_048
        button = ButtonDriver(
            started_at=1_900,
            ticks_diff=wrapping_ticks_diff(period),
            tick_period=period,
        )
        self.assertIsNone(button.advance_after_startup())
        self.assertIsNone(button.begin_transition(PRESSED))
        self.assertIsNone(button.confirm_transition())
        self.assertIsNone(
            button.begin_transition(RELEASED, after_ms=LONG_PRESS_MS - DEBOUNCE_MS)
        )
        self.assertEqual(
            button.confirm_transition(), canonical_frame("up", "long_press")
        )


class InputValidationTests(unittest.TestCase):
    def test_invalid_initial_and_sample_levels_are_rejected(self):
        with self.assertRaises(ValueError):
            ButtonState("up", 2, 0, linear_ticks_diff)

        button = ButtonState("up", RELEASED, 0, linear_ticks_diff)
        with self.assertRaises(ValueError):
            button.poll(-1, 1)

    def test_unknown_control_is_rejected(self):
        with self.assertRaises(ValueError):
            ButtonState("unknown", RELEASED, 0, linear_ticks_diff)


if __name__ == "__main__":
    unittest.main()
