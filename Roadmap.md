# Roadmap

### Release 0.2.0
- Features/Updates
  - [X] Refactor project to use cargo workspace
  - [X] Turn properties file into a crate
    - [X] Add env vars to properties.json
    - [X] Write encrypt and decrypt function for secrets/passwords
  - [X] Turn properties and settings in subcommands for config cmd
    - [X] Setup separate config folder for settings and properties data
  - [X] Add visual reminder if test mode is enabled
  - [X] Set properties without .env file
    - [X] Add check to confirm properties are set before use
    - ~~[X] Check project path before allowing user to run commands (excluding properties subcmd)~~
    - [X] Default project path and test path if not set by user 
- Bugs/Fixes
  - [X] Fix GOG discount percentage (manually calculate)
  - [X] Fixed thresholds with same alias to support update and remove command
  - [X] Allow user to determine if aliases can be reused after initial creation
  - [X] Fixed file pathing for tests using a test flag (stored within properties file)
  - [X] Fix Steam game cache to check and update game info (using sliding window approach)
  - [X] Made Steam games search case-insensitive
  - [X] Fix Windows tests for GitHub actions
    - [X] Optimized 'Run tests' step
  - [ ] Don't prevent previous file path no longer exists from updating to a new valid one (properties)
- Testing:
  - [X] Add tests for multiple thresholds with the same alias
  - [X] Add tests for encrypting and decrypting secrets/passwords
  - [X] Move tests to a separate crate

### Backlog
- Features/Updates
  - Set up Humble Bundle Storefront & test
  - Implement option for Steam search without cache (cycle through games and display list of matching titles)
  - Implement script as a desktop app
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
  - Add the option to send emails through AWS SES
- Bugs/Fixes
  - Configure Steam API call to not send steam key as plain text
  - Update dependencies and resolve any potential issues
- Testing
  - Scope of untested code
    - Needs implementation
      - `add` and `bulk-insert` script cmds
      - properties (creation, updating and retrieval)
      - retrieving environment variables
    - No plans for implementation
      - `update-cache` and `send-email` script cmds