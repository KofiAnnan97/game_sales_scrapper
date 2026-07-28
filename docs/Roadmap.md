# Roadmap

### Release 0.3.1
- Features/Updates
  - General
    - [ ] Configure Steam API call to not send steam key as plain text
    - [X] Modernize email html
    - [X] Only show sensitive passwords when requested by user.
    - [ ] Deteremine if defaulting the path should be in getters for project and test path if properties and dot env file have an invalid/empty path
  - Application
    - [ ] Add custom theme setup for user customization
    - [ ] Add interface from custom email cron jobs (Windows and Linux). Might need to rework the current setup.
    - [ ] Log filtering by all, lowest severity, and exact match when displayed (move to separate window)
      - [ ] include the ability to prune logs
    - [ ] Update check price table to show game image from url and hyperlink store page to title
    - [ ] Print out "Saved settings successfully"  or close Settings when Save Settings button is pressed
    - [ ] Add auto advance to the next store as a toggable option when a radial button is selected
- Bugs/Fixes
  - General
    - [X] Added custom error handling for api calls
    - [ ] Handle games thresholds with corrupted or incorrect data (try run search on fake query with incorret store ids)
  - Application
    - [ ] Fix logic to support updating log file when application is prompted to close
    - [X] Fix store search to filter out any game with no price
- Testing:
  - [ ] Add caching to Github actions

### Backlog
- Features/Updates
  - General
    - Set up Humble Bundle Storefront & test
    - Retrieve pricing data from Steam bundles 
    - Retrieve pricing data from game editions on GOG
    - Add the option to send emails through AWS SES
    - Remove/reduce duplicate code between the app and cli
- Bugs/Fixes
  - General
    - Update dependencies and resolve any potential issues
- Testing
  - To do
    - Mock api calls for user commands (check prices) -> may need to moved out to later
    - `add` and `bulk-insert` script cmds
    - Figure out if comprehensive testing is viable for application
  - Out of Scope 
    - `update-cache` and `send-email` script cmds