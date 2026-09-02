"""Observe raw button transitions for the device hardware PoC.

The human-readable output is temporary diagnostic text for MicroPico vREPL.
It is NOT the production serial protocol. Field names, ordering, and delimiters
may change. No debounce, repeat, or long-press logic is implemented.
"""

from machine import Pin
import time


# Stage 2: switch 1 / GP2 succeeded on the real device on 2026-08-27.
# Monitor all seven switches while keeping their physical button numbers explicit.
BUTTON_GPIO_PAIRS = (
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 8),
)
POLL_INTERVAL_MS = 1


def state_name(level):
    return "PRESSED" if level == 0 else "RELEASED"


def main():
    inputs = []
    previous_levels = []

    for button_number, gpio_number in BUTTON_GPIO_PAIRS:
        pin = Pin(gpio_number, Pin.IN, Pin.PULL_UP)
        level = pin.value()
        inputs.append((button_number, gpio_number, pin))
        previous_levels.append(level)

    started_at = time.ticks_us()

    print("[button_test] temporary human-readable hardware diagnostic")
    print("[button_test] NOT the production serial protocol")
    print("[button_test] no debounce, repeat, or long-press logic")
    print("[button_test] polling interval: {} ms".format(POLL_INTERVAL_MS))

    for index, (button_number, gpio_number, _) in enumerate(inputs):
        level = previous_levels[index]
        print(
            "[button_test] initial button={} gpio=GP{} level={} state={}".format(
                button_number,
                gpio_number,
                level,
                state_name(level),
            )
        )

    try:
        while True:
            for index, (button_number, gpio_number, pin) in enumerate(inputs):
                level = pin.value()
                if level == previous_levels[index]:
                    continue

                previous_levels[index] = level
                elapsed_us = time.ticks_diff(time.ticks_us(), started_at)
                print(
                    (
                        "[button_test] elapsed_us={} button={} gpio=GP{} "
                        "level={} state={}"
                    ).format(
                        elapsed_us,
                        button_number,
                        gpio_number,
                        level,
                        state_name(level),
                    )
                )

            # This delay is the sampling interval, not debounce processing.
            time.sleep_ms(POLL_INTERVAL_MS)
    except KeyboardInterrupt:
        print("[button_test] stopped")


if __name__ == "__main__":
    main()
