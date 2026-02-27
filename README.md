# Gym Tracker — Home Assistant Add-on

Personal gym workout tracker with exercise logging, session management, and progress tracking. Runs as a Home Assistant add-on or standalone Docker container.

## Features

- Exercise library with muscle group categorization
- Workout session logging with sets, reps, and weight
- Progress tracking over time
- JWT-based authentication (multi-user support)
- SQLite database — no external DB needed
- Responsive web UI

## Installation (Home Assistant Add-on)

1. In Home Assistant, go to **Settings → Add-ons → Add-on Store**
2. Click the **⋮** menu (top right) → **Repositories**
3. Add this repository URL: `https://github.com/kamte/gym-tracker`
4. Find **Gym Tracker** in the store and click **Install**
5. Start the add-on
6. Open the Web UI at `http://homeassistant.local:3000`

### First-Time Setup

1. Open the web UI after starting the add-on
2. Register a new account
3. Log in and start tracking your workouts

## Configuration

| Option | Type | Default | Description |
|-----------|--------|---------|--------------------------------------|
| port | int | 3000 | Port the web UI listens on |
| log_level | string | info | Log verbosity (trace/debug/info/warn/error) |

Configuration is set through the Home Assistant add-on UI under the **Configuration** tab.

## Data Persistence

All data is stored in Home Assistant's `/data` volume:

- **Database**: `/data/gym.db` — all workout data
- **JWT secret**: `/data/.jwt_secret` — auto-generated on first run

Data survives add-on restarts and updates. Include the add-on in your Home Assistant backups to preserve your workout history.

## Standalone Docker

```bash
docker build -t gym-tracker .
docker run -d \
  -p 3000:3000 \
  -v gym-data:/data \
  --name gym-tracker \
  gym-tracker
```

Then open `http://localhost:3000`.

## Development

```bash
# Create .env from example
cp .env.example .env
# Edit .env and set JWT_SECRET (generate with: openssl rand -base64 32)

# Run with cargo
cargo run
```

## Architecture

- **Backend**: Rust / Axum
- **Database**: SQLite via SQLx (migrations compiled into binary)
- **Templates**: Askama (compiled into binary)
- **Auth**: JWT with Argon2 password hashing
