# SQL Judge design contract

`SQL Judge — UI redesign.html` defines the application shell, shared tokens and navigation. The sibling screen export directories define screen-level visual behavior.

These files are visual contracts, not production dependencies. Do not import their vendor runtime, copy generated markup or ship their scripts. Recreate the design through the existing React component system, preserve accessibility and safety invariants, and compare the rendered screen at the artboard viewport and responsive widths before claiming fidelity.
