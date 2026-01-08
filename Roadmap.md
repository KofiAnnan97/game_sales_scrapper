# Roadmap

### Release 0.2.0
- Features/Updates
  - [X] Refactor project to use cargo workspace
  - [X] Turn properties file into a crate
    - [X] Add env vars to properties.json
    - [X] Write encrypt and decrypt function for secrets/passwords
- Bugs/Fixes
  - [X] Fix GOG discount percentage (manually calculate)
  - [X] Fixed thresholds with same alias to support update and remove command
  - [X] Allow user to determine if aliases can be reused after initial creation
  - [X] Fixed file pathing for tests using a test flag (stored within properties file)
  - [X] Fix Steam game cache to check and update game info (using sliding window approach)
  - [X] Made Steam games search case-insensitive
  - [X] Fix Windows tests for GitHub actions
    - [X] Optimized 'Run tests' step
- Testing:
  - [X] Add tests for multiple thresholds with the same alias
  - [X] Add tests for encrypting and decrypting secrets/passwords

### Backlog
- Features/Updates
  - Set up Humble Bundle Storefront & test
  - Implement option for Steam search without cache (cycle through games and display list of matching titles)
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
- Bugs/Fixes
  - Configure Steam API call to not send steam key as plain text
  - Update dependencies and resolve any potential issues
- Testing
  - Scope of untested code
    - `add` and `bulk-insert` script cmds
    - `update-cache` and `send-email` script cmds (not plans for implementation)
    - properties - creation, updating and retrieval
    - retrieving environment variables