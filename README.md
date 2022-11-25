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

## Features

1. I/O of existing `.msi` format model files. However the format for atom parsing is strict for the moment.
2. Edit atoms and lattice information in the model.
    1. Edit the element information, atom ID, xyz coordinates of target atom.
    2. Read/Write the lattice vectors.
3. Geometry transformation of the model:
    1. Translation to desired positions
    2. Rotate by axis-angle definition.
4. Export the seedfiles for `CASTEP` task, including:
    - `*.cell` - necessary file for `CASTEP` task. Can be visualized in many simple and lightweight model viewing software. E.g. `VESTA`.
    - `*.trjaux`, `*.kptaux`
    - `*.param`
    - `*.msi` - can be visualized in `Materials Studio`.
    - Copy potential files used for `CASTEP` standalone mode. (Potential files are not provided and included in this repository and library)
    - Miscellaneous files.
    - Auto-generation of a `perl` script to instruct the `Materials Studio` to generate `.xsd` from `.msi`.
