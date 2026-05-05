# Roadmap

### Release 0.3.0
- Features/Updates
  - [ ] Implement script as a desktop app
  - [ ] Update alias from command line
  - [ ] Configure Steam API call to not send steam key as plain text
  - [ ] Add fuzzy search to update and remove command for potential suggestions if the title or alias is not found
  - [ ] Deteremine if defaulting the path should be in getters for project and test path if properties and dot env file have an invalid/empty path
- Bugs/Fixes
  - [ ] Configure Steam API call to not send steam key as plain text
  - [ ] Pull currency type for Microsoft store games
  - [ ] Update dependencies and resolve any potential issues
- Testing:
  - [X] Update tests to use stubbing
  - [X] Implement temp directories (for all file based tests)
  - [ ] Mock api calls for user commands (check prices)
  - [X] Write tests for
    - [X] properties (creation, updating and retrieval)
    - [X] retrieving environment variables

### Backlog
- Features/Updates
  - Set up Humble Bundle Storefront & test
  - Retrieve pricing data from Steam bundles 
  - Retrieve pricing data from game editions on GOG
  - Add the option to send emails through AWS SES
- Bugs/Fixes
- Testing
  - Scope of untested code
    - Needs implementation
      - `add` and `bulk-insert` script cmds
    - No plans for implementation
      - `update-cache` and `send-email` script cmds