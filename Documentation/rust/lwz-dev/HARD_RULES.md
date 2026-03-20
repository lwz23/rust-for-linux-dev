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

## Callback-only runtime triggering

- If a phylib behavior is reachable only through a callback slot such as
  `link_change_notify`, benchmark runtime may trigger it only through the
  already-bound `phydev->drv->...` entry.
- Do not call driver-private static C symbols directly from KUnit or host-side
  scripts.
- Hold the same subsystem locking or serialization assumptions that phylib uses
  for the callback before treating the observation as trusted.

## Generated bitfield accessors

- When bindgen already emits `phy_device` bitfield accessors, use those
  accessors in safe Rust wrappers instead of copying numeric bit offsets into
  handwritten code.
- Treat handwritten bit offsets as ABI debt. They require explicit proof and
  should be removed before widening the safe API surface.
- If runtime diffing exposes stale field-layout assumptions, fix the
  abstraction first by deleting duplicated layout knowledge before changing
  driver logic.
