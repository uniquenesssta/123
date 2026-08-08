from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = [
    "crates/application/src/use_cases/prediction/execute_prediction/mod.rs",
    "crates/application/src/use_cases/prediction/inspect_match_prediction_readiness/mod.rs",
    "crates/application/src/use_cases/prediction/preview_route/mod.rs",
]
old = "ensure_model_selection_registered(&registry, &model_selection)?;"
new = "ensure_model_selection_registered(registry, &model_selection)?;"

for relative in FILES:
    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"expected exactly one needless borrow in {relative}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

print("R3-06 Prediction registry needless borrows removed")
