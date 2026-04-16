# Recipes

This directory is reserved for rewrite recipes consumed by the pack compiler and later patch-planning seams.

Phase C now compiles recipe packs structurally inside the crate-private `pack` seam. Recipes remain declarative inputs only; transform execution and patch generation are still deferred to later seams.
