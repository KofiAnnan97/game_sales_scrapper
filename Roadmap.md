# Roadmap

### Release 0.2.0
- Incomplete
  - Features
    - [X] Refactor project to use cargo workspace
    - [ ] Set up Humble Bundle Storefront
  - Bugs/Fixes
    - [ ] Fix Windows tests for GitHub actions
    - [ ] Fix GOG discount percentage (manually calculate)
    - [ ] Fix alias not apply to multiple threshold entries (same product different name/edition)
  - Testing:
    - [ ] Add tests for Humble Bundle API calls

### Backlog
- Features
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
- Bugs/Fixes
  - Fix Steam game cache to check and update when any app info changes
- Testing
  - Add the `add` and `bulk-insert` cmds for functional testing