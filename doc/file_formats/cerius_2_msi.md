# Cerius II (`.msi`) format

## Structure
- msi
    - misc params
    - lattice vector
    - Atom or Bond
        - Atom
            - Starts with a left-parenthesis: `(4 Atom`, the number is the id of components in the file. The following fields have additional two spaces indent.
            - element symbol with atomic number: `(A C ACL "6 C")`
            - (optional) label, shown as "Name" in `MS`: `(A C Label "C")`
            - XYZ, the absolute cartesian coordinate: `(A D XYZ (2.0000 0 0.2222))` (Though it specifies the type is `double`, it accepts writing zero as `0` instead of with digits: `0.0`)
            - Id, the index of the atom in the model: `(A I Id 1)`
            - Ends with a right-parenthesis: `)`
