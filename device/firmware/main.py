"""Automatic production entrypoint for the Raspberry Pi Pico."""


RESET_DELAY_MS = 250


def reset_after_delay():
    """Reset after faults or an unexpected normal return without serial noise."""

    import machine
    import time

    time.sleep_ms(RESET_DELAY_MS)
    machine.reset()


def supervise(run_firmware, reset):
    """Reset after a stopped firmware loop, but preserve maintenance Ctrl-C."""

    try:
        run_firmware()
    except KeyboardInterrupt:
        # Ctrl-C intentionally reaches the REPL for maintenance.
        raise
    except Exception:
        reset()
    else:
        reset()


def load_and_run_firmware():
    # Keep the application import inside the supervised path so a corrupt or
    # incomplete upload also recovers by resetting instead of entering a REPL.
    from button_firmware import run

    run()


def main():
    supervise(load_and_run_firmware, reset_after_delay)


if __name__ == "__main__":
    main()
