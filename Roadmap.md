# Roadmap

### Release 0.2.0
- Features
  - [X] Refactor project to use cargo workspace
  - [ ] Set up Humble Bundle Storefront
- Bugs/Fixes
  - [ ] Fix Windows tests for GitHub actions
  - [X] Fix GOG discount percentage (manually calculate)
  - [X] Fixed thresholds with same alias to support update and remove command
  - [X] Allow user to determine if aliases can be reused after initial creation
  - [ ] Fix Steam game cache to check and update when any app info changes
  - [X] Fixed file pathing for tests using a test flag (stored within properties file)
- Testing:
  - [ ] Add tests for Humble Bundle API calls
  - [X] Add tests for multiple thresholds with the same alias

### Backlog
- Features
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
- Bugs/Fixes
- Testing
  - Add the `add` and `bulk-insert` cmds for functional testing