# Game Sales Scrapper
![GitHub Release](https://img.shields.io/github/v/release/KofiAnnan97/game_sales_scrapper?label=Latest%20Version) ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/KofiAnnan97/game_sales_scrapper/build.yml?label=Build) ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/KofiAnnan97/game_sales_scrapper/tests.yml?label=Tests) ![GitHub last commit](https://img.shields.io/github/last-commit/KofiAnnan97/game_sales_scrapper?label=Last%20Commit)

Roadmap: [[link](./Roadmap.md)]

A script that checks multiple storefront to determine if a game has reached a user-defined price. Automation can be set 
up to send an email if any game is at or falls below their respective price threshold.

### Supported Storefronts
- **Steam**
- **Good Old Games (GOG)**
- **Microsoft Store (PC)**

### Resources
- Requesting a STEAM API key := https://steamcommunity.com/dev

## Quick Start
1. Setup SMTP server/service (TLS required)
2. Nagivate to project folder and run `cargo build --release`
3. In the project folder, create `.env` with the following:
    ```
    STEAM_API_KEY={your_steam_api_key}
    RECIPIENT_EMAIL={destination_email_address}
    SMTP_HOST={smtp_host_domain}
    SMTP_PORT={port_number}
    SMTP_EMAIL={smtp_email_address}
    SMTP_USERNAME={smtp_username}
    SMTP_PWD={stmp_password}
    PROJECT_PATH={/path/to/game_sales_scrapper}
    TEST_PATH={/path/to/test_directory}
    ```
    - For Windows use `\\` when defining the path.
4. Initialize settings and properties (refer to [supported commands](#supported-commands))
5. Add games and their respective price threshold using the [support commands](#supported-commands) below (supports commands from cargo).
6. [Optional] Automate emails (in `setup/` folder)
    - **For Unix-based systems:** Update *SCHEDULE* variable to desired execution frequency and run `set_cron.sh -c "create"` with root privileges.
    - **For Windows systems:** Update *$trigger* variable to desired execution frequency and run `set_task_scheduler.ps1 -Cmd "create"`. 
    
        If Powershell scripts execution is not enabled run the following with administrative privileges: 
        ```
        Set-ExecutionPolicy RemoteSigned
        ```
7. [Optional] Run tests locally `cargo test -- --test-threads=1`

## Supported Commands
Use the`--help` flag in command line to get more information on the supported commands. Here's a brief description and example of each command.
- `config` := sets what storefronts are used to search for games and enable aliases for game titles (enabled by default). 
    - [`-a`,`-s`, `-g`, `-m`]  := determine which storefronts to search. This command will override previous choices.
    - `-e` := set whether aliases are enabled for games
    - `-r` := set whether an alias can be reused (good for dealing with stores that use different names for the same product)
    - `-p` := update properties using `.env` file
    - `-z` := toggles whether the testing mode is enabled (if enabled script uses `TEST_PATH` env)
  ```commandline
    game_sales_scrapper config -a -e 1 -r 0
    ```
- `add` := add a specified game (title must be exact to work).
    ```commandline
    game_sales_scrapper add --title <title> --price <price>
    ```
- `bulk-insert` := add multiple games with a price threshold using a CSV file.
    ```commandline
    game_sales_scrapper bulk-insert --file <file.csv>
    ```
    CSV Example:
    ```text
    games, price
    Hollow Knight, 9.99
    Cyberpunk 2077, 19.99
    Hades, 9.99
    Stardew Valley, 7.99
    ```
- `update` := update price threshold for a specified game.
    ```commandline
    game_sales_scrapper update --title <title> --price <price>
    ```
- `remove` := remove a specified game.
    ```commandline
    game_sales_scrapper remove --title <title>
    ```
- `list-selected-stores` := list whether a storefront is used to search for games.
    ```commandline 
    game_sales_scrapper --list-selected-stores
    ```
- `list-thresholds` := list all the stored price thresholds for selected games.
    ```commandline
    game_sales_scrapper --list-thresholds
    ```
- `update-cache` := update the locally stored cache of steam games (title and app ids).
    ```commandline
    game_sales_scrapper --update-cache
    ```
- `check-prices` := print out any games that are on sale that meet user respective price threshold.
    ```commandline
    game_sales_scrapper --check-prices
    ```
- `send-email` := sends an email (using SMTP) containing a list of games that are below user defined price threshold for each game. No email is sent if no game has reached their price threshold.
    ```commandline 
    game_sales_scrapper --send-email
    ```