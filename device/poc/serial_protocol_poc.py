"""Emit debounced button gestures for the serial protocol PoC."""

from machine import Pin
import time


BUTTONS = (
    (2, "up"),
    (3, "down"),
    (4, "left"),
    (5, "right"),
    (6, "red"),
    (7, "yellow"),
    (8, "green"),
)

POLL_INTERVAL_MS = 1
DEBOUNCE_MS = 20
LONG_PRESS_MS = 700

PRESSED = 0
RELEASED = 1


class ButtonState:
    def __init__(self, gpio_number, control, now_ms):
        self.control = control
        self.pin = Pin(gpio_number, Pin.IN, Pin.PULL_UP)

        initial_level = self.pin.value()
        self.stable_level = initial_level
        self.candidate_level = initial_level
        self.candidate_since_ms = now_ms

        # A button held during startup must be released stably before it can
        # start a gesture.
        self.armed = initial_level == RELEASED
        self.press_started_ms = None

    def poll(self, now_ms):
        level = self.pin.value()

        if level != self.candidate_level:
            self.candidate_level = level
            self.candidate_since_ms = now_ms
            return

        if level == self.stable_level:
            return

        if time.ticks_diff(now_ms, self.candidate_since_ms) < DEBOUNCE_MS:
            return

        self.stable_level = level

        if level == PRESSED:
            if self.armed:
                self.press_started_ms = now_ms
            return

        if self.press_started_ms is not None:
            duration_ms = time.ticks_diff(now_ms, self.press_started_ms)
            gesture = "long_press" if duration_ms >= LONG_PRESS_MS else "short_press"
            print(
                '{{"v":1,"control":"{}","gesture":"{}"}}'.format(
                    self.control, gesture
                )
            )
            self.press_started_ms = None

        self.armed = True


def main():
    try:
        now_ms = time.ticks_ms()
        buttons = [ButtonState(gpio, control, now_ms) for gpio, control in BUTTONS]

        while True:
            now_ms = time.ticks_ms()
            for button in buttons:
                button.poll(now_ms)
            time.sleep_ms(POLL_INTERVAL_MS)
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
