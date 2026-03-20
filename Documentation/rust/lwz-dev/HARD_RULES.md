# Local Hard Rules Addendum

These benchmark-local rules were learned while blind-writing the `ax88796b`
Rust PHY driver in the isolated worktree.

## PHYLIB registration

- Do not add `unsafe impl Send/Sync` to raw PHY vtable containers just to make
  `kernel::Module` compile.
- First compile without the impls and identify the minimum object that truly
  must satisfy `Send + Sync`.
- Prefer concentrating the thread-safety proof on the registration wrapper when
  it is the only object crossing module boundaries.

## Safe PHY setters

- Do not expose safe setters that write arbitrary integers into `phy_device`
  fields when the current driver only needs a constrained value set.
- Use enums or similarly narrow types for `speed`, `duplex`, and comparable
  fields unless a broader safe contract is proven necessary.

## Registration lifetime pairing

- A PHY registration wrapper must own the exact pinned static driver table that
  was passed to `phy_drivers_register()`.
- The same wrapper must pair successful registration with
  `phy_drivers_unregister()` in `Drop`.
- The unsafe audit must state why moving the wrapper between threads does not
  move or invalidate the pinned driver table.
