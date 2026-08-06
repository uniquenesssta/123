$ErrorActionPreference = "Stop"

function Replace-ExactlyOnce([string]$Path, [string]$Old, [string]$New, [string]$Label) {
  $text = [System.IO.File]::ReadAllText($Path).Replace("`r`n", "`n")
  if (($text.Split($Old).Count - 1) -ne 1) {
    throw "$Label did not match exactly once"
  }
  $text = $text.Replace($Old, $New)
  [System.IO.File]::WriteAllText($Path, $text, [System.Text.UTF8Encoding]::new($false))
}

Replace-ExactlyOnce `
  "scripts/verify-force-team-delete.mjs" `
  'const domain = read("crates/domain/src/lib.rs");' `
  'const domain = read("crates/domain/src/team/deletion.rs");' `
  "force team delete verifier domain source"

$entityOld = 'const domain = text("crates/domain/src/lib.rs");'
$entityNew = @'
const domain = [
  "crates/domain/src/coach/catalog.rs",
  "crates/domain/src/coach/name.rs",
  "crates/domain/src/coach/membership.rs",
  "crates/domain/src/team/membership.rs",
  "crates/domain/src/shared/entity_reference.rs",
  "crates/domain/src/shared/entity_match.rs",
  "crates/domain/src/shared/bulk_archive.rs",
].map(text).join("\n");
'@.Trim()
Replace-ExactlyOnce `
  "scripts/verify-entity-relationships.mjs" `
  $entityOld `
  $entityNew `
  "entity relationships verifier domain sources"

Replace-ExactlyOnce `
  "scripts/verify-formation-usage.mjs" `
  'const domain = read("crates/domain/src/lib.rs");' `
  'const domain = read("crates/domain/src/formation/usage.rs") + read("crates/domain/src/formation/resolution.rs");' `
  "formation usage verifier domain sources"

Replace-ExactlyOnce `
  "scripts/verify-entity-resource-center.mjs" `
  'const domain = read("crates/domain/src/lib.rs");' `
  'const domain = read("crates/domain/src/team/detail.rs") + read("crates/domain/src/player/listing.rs");' `
  "entity resource center verifier domain sources"

Replace-ExactlyOnce `
  "scripts/verify-team-package-preview-recovery.mjs" `
  'const domain = read("crates/domain/src/lib.rs");' `
  'const domain = read("crates/domain/src/player/status.rs");' `
  "team package preview recovery verifier domain source"
