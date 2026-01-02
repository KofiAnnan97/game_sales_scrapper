# Roadmap

### Release 0.2.0
- Features/Updates
  - [X] Refactor project to use cargo workspace
  - [X] Turn properties file into a crate
  - [ ] Update dependencies and resolve any potential issues
  - [ ] Set up Humble Bundle Storefront
- Bugs/Fixes
  - [X] Fix GOG discount percentage (manually calculate)
  - [X] Fixed thresholds with same alias to support update and remove command
  - [X] Allow user to determine if aliases can be reused after initial creation
  - [X] Fixed file pathing for tests using a test flag (stored within properties file)
  - [ ] Fix Steam game cache to check and update when any app info changes
  - [ ] Fix Windows tests for GitHub actions
- Testing:
  - [X] Add tests for multiple thresholds with the same alias
  - [ ] Add tests for Humble Bundle API calls

### Backlog
- Features/Updates
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
- Bugs/Fixes
  - Configure Steam API call to not send steam key as plain text
- Testing
  - Add the `add` and `bulk-insert` cmds for functional testing