#!/usr/bin/env python3

import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tarfile
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
INSTALLER = REPO_ROOT / "scripts" / "install" / "install-akram.sh"
VERSION = "0.145.0-ak.0.1"
ASSET = "codex-akram-package-aarch64-apple-darwin.tar.gz"


class InstallAkramTest(unittest.TestCase):
    def test_installs_fork_without_touching_official_codex(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            home = root / "home"
            bin_dir = home / ".local" / "bin"
            official_home = home / ".codex"
            official_home.mkdir(parents=True)
            bin_dir.mkdir(parents=True)
            (official_home / "sentinel").write_text("official", encoding="utf-8")
            official_bin = executable(bin_dir / "codex", "#!/bin/sh\necho official\n")

            package = root / "package"
            executable(
                package / "bin" / "codex-akram",
                f"#!/bin/sh\necho 'codex-akram {VERSION}'\n",
            )
            executable(package / "bin" / "codex-code-mode-host", "#!/bin/sh\nexit 0\n")
            executable(package / "codex-path" / "rg", "#!/bin/sh\nexit 0\n")
            (package / "codex-resources").mkdir()
            (package / "codex-package.json").write_text(
                json.dumps(
                    {
                        "layoutVersion": 1,
                        "version": VERSION,
                        "target": "aarch64-apple-darwin",
                        "variant": "codex-akram",
                        "entrypoint": "bin/codex-akram",
                    }
                ),
                encoding="utf-8",
            )
            archive = root / ASSET
            with tarfile.open(archive, "w:gz") as bundle:
                for path in package.rglob("*"):
                    bundle.add(path, arcname=path.relative_to(package), recursive=False)
            checksums = root / "codex-akram-package_SHA256SUMS"
            checksums.write_text(
                f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {ASSET}\n",
                encoding="utf-8",
            )

            tools = root / "tools"
            tools.mkdir()
            fake_uname = executable(
                tools / "uname",
                '#!/bin/sh\nif [ "${1:-}" = "-s" ]; then echo Darwin; else echo arm64; fi\n',
            )
            fake_curl = executable(
                tools / "curl",
                """#!/bin/sh
url="$2"
output="$4"
printf '%s\n' "$url" >> "$TEST_URL_LOG"
case "$url" in
  */releases/latest) cp "$TEST_RELEASE_JSON" "$output" ;;
  */codex-akram-package-aarch64-apple-darwin.tar.gz) cp "$TEST_ARCHIVE" "$output" ;;
  */codex-akram-package_SHA256SUMS) cp "$TEST_CHECKSUMS" "$output" ;;
  *) exit 1 ;;
esac
""",
            )
            self.assertTrue(fake_uname.exists() and fake_curl.exists())
            release_json = root / "release.json"
            release_json.write_text(
                json.dumps({"tag_name": f"v{VERSION}"}), encoding="utf-8"
            )
            url_log = root / "urls.log"

            env = {
                **os.environ,
                "HOME": str(home),
                "PATH": f"{tools}:{os.environ['PATH']}",
                "CODEX_AKRAM_HOME": str(home / ".codex-akram"),
                "CODEX_AKRAM_BIN_DIR": str(bin_dir),
                "TEST_RELEASE_JSON": str(release_json),
                "TEST_ARCHIVE": str(archive),
                "TEST_CHECKSUMS": str(checksums),
                "TEST_URL_LOG": str(url_log),
            }
            result = subprocess.run(
                ["sh", str(INSTALLER)],
                env=env,
                check=True,
                text=True,
                capture_output=True,
            )

            self.assertIn(f"codex-akram {VERSION} installed", result.stdout)
            self.assertEqual((official_home / "sentinel").read_text(), "official")
            self.assertEqual(
                subprocess.check_output([official_bin], text=True), "official\n"
            )
            self.assertEqual(
                subprocess.check_output(
                    [bin_dir / "codex-akram", "--version"], text=True
                ),
                f"codex-akram {VERSION}\n",
            )
            urls = url_log.read_text(encoding="utf-8").splitlines()
            self.assertTrue(urls)
            self.assertTrue(all("Akram012388/codex-akram" in url for url in urls))
            self.assertTrue(all("openai/codex" not in url for url in urls))

            current = home / ".codex-akram" / "packages" / "standalone" / "current"
            installed_release = current.resolve()
            bad_checksums = root / "bad-SHA256SUMS"
            bad_checksums.write_text(f"{'0' * 64}  {ASSET}\n", encoding="utf-8")
            failed_env = {**env, "TEST_CHECKSUMS": str(bad_checksums)}
            failed = subprocess.run(
                ["sh", str(INSTALLER)],
                env=failed_env,
                check=False,
                text=True,
                capture_output=True,
            )

            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("archive checksum mismatch", failed.stderr)
            self.assertEqual(current.resolve(), installed_release)
            self.assertEqual(
                subprocess.check_output(
                    [bin_dir / "codex-akram", "--version"], text=True
                ),
                f"codex-akram {VERSION}\n",
            )


def executable(path: Path, contents: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)
    return path


if __name__ == "__main__":
    unittest.main()
