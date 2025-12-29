# Roadmap

### Initial Release
- Completed
  - Features
    - [X] Add alias request when adding a game threshold (through cli prompts or as parameter)
    - [X] Give panic message if config command not used before adding game thresholds
    - [X] Add bulk insert option for game thresholds
    - [X] Set up Microsoft Store Storefront
    - [X] Show the game MSRP for Microsoft Store and GOG during add phase
    - [X] Added functionality to check and display desired sales without sending email
    - [X] Microsoft Store - Update get price call to only retrieve desired game (instead of reusing search)
  - Bugs/Fixes
    - [X] Change logic to only show "Update ID" if the id was actually updated not just called
  - Testing:
    - [X] API calls (search for game and check game price is all supported storefronts)
    - [X] Unit testing for thresholds and settings operations
    - [X] Functional testing for user commands
    - [X] Show status of build and tests done for Windows and Ubuntu on Readme

### Backlog
- Features
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
  - Set up Humble Bundle Storefront
  - Refactor project to use cargo workspace
- Bugs/Fixes
  - Fix alias not apply to multiple threshold entries (same product different name/edition)
  - Fix Steam game cache to check and update when any app info changes
  - Fix GOG discount percentage (manually calculate)
  - Fix Windows tests for GitHub actions
- Testing
  - Add the `add` and `bulk-insert` cmds for functional testing