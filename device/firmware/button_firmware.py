"""Production button firmware for Device Serial Protocol Wire v1.

The state machine in this module deliberately has no dependency on
MicroPython hardware modules.  ``run`` owns the RP2040 GPIO and clock adapter,
which keeps button behaviour deterministic and host-testable.
"""


BUTTONS = (
    (2, "up"),
    (3, "down"),
    (4, "left"),
    (5, "right"),
    (6, "red"),
    (7, "yellow"),
    (8, "green"),
)

CONTROLS = tuple(control for _, control in BUTTONS)
GESTURES = ("short_press", "long_press")

POLL_INTERVAL_MS = 1
DEBOUNCE_MS = 20
LONG_PRESS_MS = 700

PRESSED = 0
RELEASED = 1


def canonical_frame(control, gesture):
    """Return one canonical Wire v1 JSON frame, including its LF delimiter."""

    if control not in CONTROLS:
        raise ValueError("unknown control: {}".format(control))
    if gesture not in GESTURES:
        raise ValueError("unknown gesture: {}".format(gesture))

    return '{{"v":1,"control":"{}","gesture":"{}"}}\n'.format(
        control, gesture
    )


class ButtonState:
    """Debounce and classify one active-low button independently.

    ``ticks_diff`` must have the same contract as ``time.ticks_diff``.  It is
    injected so the exact RP2040 wrap-around behaviour can be exercised by
    host-side tests.
    """

    def __init__(self, control, initial_level, now_ms, ticks_diff):
        self._validate_level(initial_level)
        if control not in CONTROLS:
            raise ValueError("unknown control: {}".format(control))

        self.control = control
        # The first physical read is the initial debounced state.  A released
        # input is ready immediately; only a button already held at startup is
        # disarmed until its first confirmed release, as required by Wire v1.
        self.stable_level = initial_level
        self.candidate_level = initial_level
        self.candidate_since_ms = now_ms
        self.armed = initial_level == RELEASED
        self.press_started_ms = None
        self._ticks_diff = ticks_diff

    @staticmethod
    def _validate_level(level):
        if level != PRESSED and level != RELEASED:
            raise ValueError("button level must be 0 or 1")

    def poll(self, level, now_ms):
        """Consume one raw sample and return at most one canonical frame."""

        self._validate_level(level)

        if level != self.candidate_level:
            self.candidate_level = level
            self.candidate_since_ms = now_ms
            return None

        if self._ticks_diff(now_ms, self.candidate_since_ms) < DEBOUNCE_MS:
            return None

        if level == self.stable_level:
            return None

        self.stable_level = level

        if level == PRESSED:
            if self.armed:
                # One confirmed press consumes the arm until its matching
                # confirmed release.  Holding and bounce cannot start another
                # cycle while this gesture is in progress.
                self.armed = False
                self.press_started_ms = now_ms
            return None

        if self.press_started_ms is None:
            # A button held during startup is armed by its first stable release.
            self.armed = True
            return None

        duration_ms = self._ticks_diff(now_ms, self.press_started_ms)
        gesture = "long_press" if duration_ms >= LONG_PRESS_MS else "short_press"
        self.press_started_ms = None
        self.armed = True
        return canonical_frame(self.control, gesture)


def run():
    """Run the seven-button RP2040 polling loop until it is interrupted."""

    from machine import Pin
    import sys
    import time

    pins_and_states = []
    now_ms = time.ticks_ms()

    for gpio_number, control in BUTTONS:
        pin = Pin(gpio_number, Pin.IN, Pin.PULL_UP)
        state = ButtonState(control, pin.value(), now_ms, time.ticks_diff)
        pins_and_states.append((pin, state))

    while True:
        now_ms = time.ticks_ms()
        for pin, state in pins_and_states:
            frame = state.poll(pin.value(), now_ms)
            if frame is not None:
                sys.stdout.write(frame)
        time.sleep_ms(POLL_INTERVAL_MS)
