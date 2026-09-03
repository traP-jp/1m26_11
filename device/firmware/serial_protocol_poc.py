"""Raw-REPL-compatible entrypoint backed by the production firmware core."""

from button_firmware import run


def main():
    run()


if __name__ == "__main__":
    main()
