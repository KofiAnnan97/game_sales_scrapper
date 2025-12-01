# Roadmap

### Initial Release
- Completed
  - [X] Add alias request when adding a game threshold (through cli prompts or as parameter)
  - [X] Give panic message if config command not used before adding game thresholds
  - [X] Add bulk insert option for game thresholds
  - [X] Change logic to only show "Update ID" if the id was actually updated not just called
  - [X] Set up Microsoft Store Storefront
- Incomplete
  - [ ] Add tests for the following:
    - [ ] API calls (search for game and check game price is all supported storefronts)
    - [ ] Creating and manipulating price thresholds

### Backlog
- Retrieve pricing data from Steam bundles 
- Retrieve pricing data from game editions on GOG
- Set up Humble Bundle Storefront
- Fix alias not apply to multiple threshold entries (same product different name/edition)