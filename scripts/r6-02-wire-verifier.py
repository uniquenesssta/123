from pathlib import Path

package = Path("package.json")
text = package.read_text(encoding="utf-8")
old_arch = 'node scripts/verify-team-directory-detail.mjs",'
new_arch = 'node scripts/verify-team-directory-detail.mjs && node scripts/verify-team-names-profiles.mjs",'
if old_arch not in text:
    raise SystemExit("verify:architecture R6-01 tail marker missing")
text = text.replace(old_arch, new_arch, 1)
old_script = '    "verify:team-directory-detail": "node scripts/verify-team-directory-detail.mjs"\n'
new_script = '    "verify:team-directory-detail": "node scripts/verify-team-directory-detail.mjs",\n    "verify:team-names-profiles": "node scripts/verify-team-names-profiles.mjs"\n'
if old_script not in text:
    raise SystemExit("verify:team-directory-detail script marker missing")
text = text.replace(old_script, new_script, 1)
package.write_text(text, encoding="utf-8", newline="\n")

Path("scripts/r6-02-wire-verifier.py").unlink(missing_ok=True)
Path(".github/workflows/r6-02-wire-verifier.yml").unlink(missing_ok=True)
