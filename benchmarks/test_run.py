"""Non-performance tests: fake child processes and clocks, independent expectations."""

import contextlib
import csv
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import run


class HarnessTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.cases = run.load_cases(Path(__file__).with_name("cases.json"))
        self.expected = {case["id"]: case["expected"] for case in self.cases}
        self.argv = []
        for flag in ("source-cli", "cue", "time"):
            self.argv += ["--" + flag, str(Path(sys.executable).resolve())]
        self.argv += ["--cases", str(Path(__file__).with_name("cases.json").resolve()),
                      "--output", str(self.root / "output"), "--revision", "a" * 40,
                      "--harness-revision", "b" * 40, "--samples", "2", "--warmups", "1",
                      "--rss-samples", "1"]
        self.args = run.arguments(self.argv)
        self.calls = []

    def fake_invoke(self, argv, env, cwd, timeout, timed=False):
        self.calls.append((list(argv), dict(env), timed))
        if argv[-1] == "version":
            return "cue test version", None
        if argv[-1] == "--version":
            return "time (GNU Time) 1.10", None
        if "-f" in argv:
            Path(argv[argv.index("-o") + 1]).write_text("123\n", encoding="ascii")
        value = self.expected[Path(argv[-1]).stem]
        return json.dumps(value), 100 if timed else None

    def test_strict_json_and_exact_types(self):
        for invalid in ('1.0', 'NaN', 'Infinity', '{"a":1,"a":2}', '1 trailing',
                        str(2**63), str(-(2**63) - 1)):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                run.read_json(invalid)
        self.assertEqual(run.read_json(str(2**63 - 1)), 2**63 - 1)
        self.assertEqual(run.read_json('-0'), 0)
        self.assertFalse(run.same_json({"x": [True]}, {"x": [1]}))
        self.assertFalse(run.same_json(1, 1.0))
        self.assertFalse(run.same_json([1, 2], [2, 1]))
        self.assertTrue(run.same_json({"a": 1, "b": None}, {"b": None, "a": 1}))

    def test_manifest_schema(self):
        for mutation in (lambda doc: doc.update(version=True),
                         lambda doc: doc.update(extra=1),
                         lambda doc: doc["cases"].pop(),
                         lambda doc: doc["cases"][0].update(id="../escape"),
                         lambda doc: doc["cases"][1].update(id=doc["cases"][0]["id"])):
            doc = {"version": 1, "cases": json.loads(json.dumps(self.cases))}
            mutation(doc)
            path = self.root / "bad.json"
            path.write_text(json.dumps(doc), encoding="utf-8")
            with self.assertRaises(ValueError):
                run.load_cases(path)

    def test_argument_bounds_paths_and_help(self):
        for flag, value in (("samples", "1"), ("warmups", "0"), ("rss-samples", "1001"),
                            ("seed", "4294967296"), ("timeout", "nan"), ("timeout", "0"),
                            ("timeout", "3601"), ("revision", "bad"), ("cue", "relative")):
            with self.subTest(flag=flag), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    run.arguments([*self.argv, "--" + flag, value])
                self.assertEqual(error.exception.code, 2)
        with contextlib.redirect_stdout(io.StringIO()) as output:
            with self.assertRaises(SystemExit) as error:
                run.arguments(["--help"])
        self.assertEqual(error.exception.code, 0)
        self.assertIn("--harness-revision", output.getvalue())
        self.assertEqual(run.revision("A" * 40), "a" * 40)

    def test_direct_child_clock_stops_before_checks(self):
        result = subprocess.CompletedProcess(["/tool"], 0, "42", "")
        events = []
        with patch.object(run.time, "perf_counter_ns", side_effect=lambda: events.append("clock") or len(events) * 10), \
                patch.object(run.subprocess, "run", side_effect=lambda *a, **k: events.append("child") or result) as child:
            text, elapsed = run.invoke(["/tool", "/input"], {}, self.root, 30, True)
        self.assertEqual(events, ["clock", "child", "clock"])
        self.assertEqual((text, elapsed), ("42", 20))
        self.assertNotIn("shell", child.call_args.kwargs)
        self.assertTrue(child.call_args.kwargs["capture_output"])
        for bad in (subprocess.CompletedProcess([], 2, "", "bad"),
                    subprocess.CompletedProcess([], 0, "42", "warning")):
            with patch.object(run.subprocess, "run", return_value=bad), self.assertRaises(ValueError):
                run.invoke(["/tool"], {}, self.root, 30)
        with patch.object(run.subprocess, "run", side_effect=subprocess.TimeoutExpired("/tool", 30)):
            with self.assertRaises(subprocess.TimeoutExpired):
                run.invoke(["/tool"], {}, self.root, 30)

    def test_complete_fake_run_and_lossless_tsv(self):
        with patch.object(run, "invoke", side_effect=self.fake_invoke), patch.dict("os.environ", {"SECRET": "hidden"}):
            run.run(self.args)
        metadata = json.loads((self.args.output / "metadata.json").read_text())
        self.assertTrue(metadata["complete"])
        self.assertEqual(set(path.name for path in self.args.output.iterdir()), {"raw.tsv", "metadata.json"})
        with (self.args.output / "raw.tsv").open(newline="") as stream:
            rows = list(csv.DictReader(stream, delimiter="\t"))
        self.assertEqual(tuple(rows[0]), run.COLUMNS)
        self.assertEqual(len(rows), 48)
        for row in rows:
            self.assertEqual(row["iterations"], "1")
            self.assertEqual(row["track"], "runtime")
            values = [int(item) for item in row["samples"].split(",")]
            self.assertEqual(values, [100, 100] if row["metric"] == "latency" else [123])
            self.assertEqual(row["unit"], "ns" if row["metric"] == "latency" else "KiB")
        self.assertEqual(metadata["raw_tsv_sha256"], run.sha256(self.args.output / "raw.tsv"))
        self.assertEqual(len(metadata["schedule"]), 4)
        self.assertEqual(sum(timed for _, _, timed in self.calls), 48)
        first_timed = next(index for index, call in enumerate(self.calls) if call[2])
        self.assertEqual(first_timed, 50)  # Two version calls +24 preflight +24 warmup.
        for argv, env, timed in self.calls:
            self.assertNotIn("SECRET", env)
            self.assertEqual(env["PATH"], "/nonexistent")
            self.assertEqual(env["TZ"], "UTC")
            self.assertFalse(timed and "-f" in argv)
        original = metadata["schedule"]
        self.args.output = self.root / "second"
        with patch.object(run, "invoke", side_effect=self.fake_invoke):
            run.run(self.args)
        self.assertEqual(json.loads((self.args.output / "metadata.json").read_text())["schedule"], original)

    def test_last_preflight_failure_never_starts_clock_or_publishes(self):
        def fail_last(argv, *args, **kwargs):
            text, elapsed = self.fake_invoke(argv, *args, **kwargs)
            return ("true", elapsed) if len(self.calls) == 26 else (text, elapsed)
        with patch.object(run, "invoke", side_effect=fail_last), self.assertRaisesRegex(ValueError, "preflight"):
            run.run(self.args)
        self.assertFalse(any(call[2] for call in self.calls))
        self.assertFalse(self.args.output.exists())

    def test_bad_rss_failure_and_existing_output_refusal(self):
        def bad_rss(argv, *args, **kwargs):
            result = self.fake_invoke(argv, *args, **kwargs)
            if "-o" in argv:
                Path(argv[argv.index("-o") + 1]).write_text("1\n2\n")
            return result
        with patch.object(run, "invoke", side_effect=bad_rss), self.assertRaisesRegex(ValueError, "peak-rss"):
            run.run(self.args)
        self.assertFalse(self.args.output.exists())
        self.args.output.mkdir()
        with patch.object(run, "invoke") as child, self.assertRaises(ValueError):
            run.run(self.args)
        child.assert_not_called()
        self.assertTrue(self.args.output.is_dir())

    def test_raw_sample_order_and_defaults(self):
        path = self.root / "ordered.tsv"
        run.write_tsv(path, {("literal-zero", "source-cli", "latency"): [17, 1, 9]})
        with path.open(newline="") as stream:
            row = next(csv.DictReader(stream, delimiter="\t"))
        self.assertEqual(row["samples"], "17,1,9")
        defaults = run.arguments(self.argv[:-6])
        self.assertEqual((defaults.samples, defaults.warmups, defaults.rss_samples,
                          defaults.seed, defaults.timeout), (31, 5, 5, 1729, 30))

    def test_invalid_version_and_missing_rss_abort(self):
        def broken(argv, *args, **kwargs):
            text, elapsed = self.fake_invoke(argv, *args, **kwargs)
            return ("not GNU", elapsed) if argv[-1] == "--version" else (text, elapsed)
        with patch.object(run, "invoke", side_effect=broken), self.assertRaisesRegex(ValueError, "not GNU"):
            run.run(self.args)
        self.assertFalse(self.args.output.exists())
        def missing(argv, *args, **kwargs):
            result = self.fake_invoke(argv, *args, **kwargs)
            if "-o" in argv:
                Path(argv[argv.index("-o") + 1]).unlink()
            return result
        with patch.object(run, "invoke", side_effect=missing), self.assertRaisesRegex(ValueError, "peak-rss"):
            run.run(self.args)
        self.assertFalse(self.args.output.exists())

    def test_stats_independent_arrays(self):
        self.assertEqual(run.summary([40, 10, 30, 20]),
                         {"count": 4, "median": 25, "q1": 17.5, "q3": 32.5,
                          "iqr": 15, "min": 10, "max": 40})
        self.assertEqual(run.summary([0])["iqr"], 0)

    def test_provider_and_publication_failure(self):
        with self.assertRaises(ValueError):
            run.provider(self.root)
        executable = Path(sys.executable).resolve()
        alias = self.root / "python"
        alias.symlink_to(executable)
        self.assertEqual(run.provider(alias)["resolved"], str(executable))
        with patch.object(run, "write_tsv", side_effect=OSError("disk full")), self.assertRaises(OSError):
            run.publish(self.args.output, {}, {})
        self.assertFalse(self.args.output.exists())
        with patch.object(run, "run", side_effect=ValueError("failed")), \
                contextlib.redirect_stderr(io.StringIO()) as error, contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(run.main(self.argv), 2)
        self.assertEqual(out.getvalue(), "")
        self.assertIn("error: failed", error.getvalue())


if __name__ == "__main__":
    unittest.main()
