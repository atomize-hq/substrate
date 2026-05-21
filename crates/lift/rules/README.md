# Rules

This directory is reserved for detector and policy rule packs that compile into immutable runtime objects.

Phase C now compiles rule packs structurally inside the crate-private `pack` seam. That means schema validation, typed query refs, deterministic fingerprints, and bundle closure support exist, but rule execution still belongs to later engine and app seams.
