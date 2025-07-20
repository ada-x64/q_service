## 0.1 -> 0.2

**Major** update. This reworks the entire crate to allow asynchronous init/deinit,
asset and resource dependencies, and more predictable state management.

Number of tests bumped from 11 to 19, not including doc tests.

### Service declarations

- Remove all macros in favor of Plugin-style implementation.
- Add `ServiceScope` to create the underlying `ServiceSpec`.
- Remove `ServiceError` and `ServiceData` traits in favor of new `ServiceError`
  struct and underlying service implementor's struct.
- Lots of trial and error getting to this state.

### Dependencies

- Add Resource and Asset dependencies.
- Unify all service dependencies under the `GraphData` abstraction.
- Thorough testing.
- Utilize existing Bevy structs like `ResourceId` and `UntypedAssetId` to
  underlie the `NodeId` struct.
- Unify all errors under the `ServiceError` struct.

### Lifecycle - Commands, Events, Hooks

- Simplify service state model.
  - Replace (Init, Enable, Disable, Update, Fail) with (Init, Up, Deinit,
    Down(reason?)).
- Simplify service commands.
- All commands now send events which update service lifecycle on `PreUpdate` and
  `PostStartup` in order to avoid chaotic Observer-based management.
- Commands are prioritized to avoid any extraneous work.
- `Init` and `Deinit` hooks can now be asynchronous thanks to built-in task
  management.
- All state changes happen in the `PreUpdate` schedule.
- Service systems now run in top-sorted order, as they are placed in
  `SystemSets` which ensure this.

### Misc

- Cleaner logging practices.
- Updated documentation.
- Added bacon configs.
- Code coverage dropped due to increasing complexity of the crate. Will need to
  update.
