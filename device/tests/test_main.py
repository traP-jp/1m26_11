"""Host-side tests for the production firmware reset supervisor."""

import unittest

from firmware.main import supervise


class MainSupervisorTests(unittest.TestCase):
    def test_unexpected_normal_return_resets(self):
        calls = []

        supervise(lambda: calls.append("run"), lambda: calls.append("reset"))

        self.assertEqual(calls, ["run", "reset"])

    def test_runtime_error_resets_without_escaping(self):
        calls = []

        def fail():
            calls.append("run")
            raise RuntimeError("hardware failure")

        supervise(fail, lambda: calls.append("reset"))

        self.assertEqual(calls, ["run", "reset"])

    def test_keyboard_interrupt_escapes_without_reset(self):
        calls = []

        def interrupt():
            calls.append("run")
            raise KeyboardInterrupt

        with self.assertRaises(KeyboardInterrupt):
            supervise(interrupt, lambda: calls.append("reset"))

        self.assertEqual(calls, ["run"])


if __name__ == "__main__":
    unittest.main()
