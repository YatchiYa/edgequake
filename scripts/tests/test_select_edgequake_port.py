#!/usr/bin/env python3
"""Unit tests for dev port selection (no network required)."""

from __future__ import annotations

import pathlib
import sys
import unittest
from unittest.mock import patch

_SCRIPT_DIR = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_SCRIPT_DIR))

import select_edgequake_port as ports  # noqa: E402


class ChoosePortTests(unittest.TestCase):
    def test_skips_foreign_backend_on_preferred_port(self) -> None:
        with (
            patch.object(ports, "is_edgequake", return_value=False),
            patch.object(ports, "is_listening", side_effect=lambda p: p == 8090),
            patch.object(ports, "is_foreign_backend", side_effect=lambda p: p == 8090),
        ):
            chosen = ports.choose_port("backend", 8090, 5)
        self.assertEqual(chosen, 8091)

    def test_reuses_running_edgequake(self) -> None:
        with patch.object(ports, "is_edgequake", side_effect=lambda _k, p: p == 8092):
            chosen = ports.choose_port("backend", 8090, 5)
        self.assertEqual(chosen, 8092)

    def test_raises_when_range_saturated(self) -> None:
        with (
            patch.object(ports, "is_edgequake", return_value=False),
            patch.object(ports, "is_listening", return_value=True),
            patch.object(ports, "is_foreign_backend", return_value=True),
        ):
            with self.assertRaises(RuntimeError):
                ports.choose_port("backend", 8090, 2)


if __name__ == "__main__":
    unittest.main()
