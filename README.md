# castep-model-core
This crate serves for the fundamentals of parsing/edit/exporting a 3D atomistic model for performing tasks in `castep` and compatible with `Materials Studio.`
This is my project to study writing rust to handle practical problems and how the 3d atomistic data flows through `castep` and `Materials Studio`. The status of development is dependent on the needs and challenges I meet in my daily routine.

## Progress
The crate now supports reading from:
1. `.msi`: the Cerius 2 format

The crate now supports writing to:
1. `.cell`: the seed input file, containing the description of the lattice, atoms and necessary settings for running a `castep` task
2. `.msi`

Generic programming is leveraged to handle the interactions and conversions between different formats.
