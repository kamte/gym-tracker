# Gym Tracker — Product Requirements Specification

> **Version**: 1.0
> **Last Updated**: 2026-02-25
> **Status**: Draft — reference for all future work

---

## Table of Contents

1. [Product Overview & Design Principles](#1-product-overview--design-principles)
2. [User Persona](#2-user-persona)
3. [User Flows](#3-user-flows)
4. [Page-by-Page Requirements](#4-page-by-page-requirements)
5. [Exercise Library Requirements](#5-exercise-library-requirements)
6. [Mobile-First Design Requirements](#6-mobile-first-design-requirements)
7. [Data Model](#7-data-model)
8. [Known Bugs & Fixes](#8-known-bugs--fixes)
9. [Feature Prioritization](#9-feature-prioritization)
10. [Implementation Sequencing](#10-implementation-sequencing)

---

## 1. Product Overview & Design Principles

### What It Is

Gym Tracker is a web application for tracking weightlifting workouts. It runs on a Rust backend (Axum + SQLite + Askama templates) and serves server-rendered HTML pages. Users create exercises, plan workout sessions, and log their sets/reps/weight over time.

### Who It's For

Gym beginners (0–3 months of experience) who use their phone at the gym to follow a workout plan and log what they actually did.

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust |
| Web framework | Axum 0.8 |
| Database | SQLite (via sqlx 0.8) |
| Templates | Askama 0.15 |
| Auth | JWT (httponly cookie) + Argon2 password hashing |
| Static files | `tower-http` ServeDir |

### 4 Core Design Principles

1. **Beginner-first** — Assume the user doesn't know exercise names, rep ranges, or how to structure a program. Provide defaults, explanations, and guidance everywhere.

2. **Mobile-first** — The primary device is a phone held at the gym between sets. Base CSS targets phone screens; media queries add desktop enhancements. Every interaction must work with one thumb.

3. **Progressive disclosure** — Show what matters now, hide what doesn't. Dashboard shows today's workout, not all sessions. Exercise detail shows instructions only when the user asks. Advanced features (RPE, custom plans) stay out of the way until needed.

4. **Minimal taps** — Every extra tap at the gym is friction. Log a set in 2 taps (confirm weight + reps → save). Start today's workout in 1 tap from the dashboard. Navigate with a persistent bottom tab bar.

---

## 2. User Persona

### Profile

| Attribute | Value |
|-----------|-------|
| Name | Alex (representative) |
| Experience | 0–3 months lifting |
| Device | Phone (320–428px viewport), used at the gym |
| Knowledge | Knows they want to get stronger. Doesn't know the difference between a Romanian deadlift and a conventional deadlift. Needs form cues. |
| Motivation | Follow a structured plan, see progress over time |

### Usage Patterns

| When | What They Do | Key Needs |
|------|-------------|-----------|
| **Before gym** (at home, 2 min) | Open app → check what today's workout is → mentally prepare | See today's exercises, sets, reps, and weights at a glance |
| **During gym** (45–60 min) | Follow workout → log each set after completing it → check form cues if unsure | Fast set logging (2 taps), exercise instructions, auto-populated weight from last time |
| **After gym** (optional, 1 min) | Mark workout complete → glance at what they did | Summary of completed workout, personal records highlighted |
| **Weekly** (optional) | Browse exercise library → review progress | Exercise history per movement, weight progression over time |

---

## 3. User Flows

### 3.1 First-Time User Flow

```
Register (username, email, password)
    → 25 default exercises seeded automatically
    → Redirect to Login
    → Login
    → Dashboard (empty state)
    → "Browse Plans" CTA
    → Plans page (3 beginner-friendly plans)
    → "Use This Plan" on chosen plan
    → 3 workout sessions created (next Mon/Wed/Fri)
    → Redirect to Sessions list
    → Tap first session
    → Session detail (see planned exercises)
    → "Start Workout" button
    → Active Workout page (NEW — does not exist yet)
```

### 3.2 Daily Workout Flow (target state)

```
Open app → Dashboard
    → "Today's Workout" card (auto-detected from scheduled sessions)
    → Tap "Start Workout"
    → Active Workout page
        → See first exercise (name, target sets/reps/weight, instructions link)
        → Complete a set → tap "Log Set" (weight + reps pre-filled from plan)
        → Repeat for all sets of this exercise
        → Swipe/tap to next exercise
        → Repeat until all exercises done
    → Tap "Finish Workout"
    → Session marked as completed
    → Summary shown (total volume, any PRs)
    → Return to Dashboard
```

### 3.3 Exercise Exploration Flow

```
Dashboard → Bottom nav "Exercises"
    → Exercise Library (searchable, filterable by muscle group)
    → Tap an exercise
    → Exercise Detail (description, equipment needed, step-by-step instructions, tips)
    → "History" section (personal logs for this exercise, weight progression)
    → Back to library
```

### 3.4 Manual Logging Flow (fallback)

```
Dashboard → Bottom nav "Logs"
    → Tap "Log Workout"
    → Select exercise from dropdown
    → Enter set number, reps, weight, optional RPE
    → Save
    → Redirect to log list
```

---

## 4. Page-by-Page Requirements

### 4.1 Login Page

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /login` |
| **Template** | `templates/login.html` |
| **Handler** | `src/handlers/auth.rs` → `login_form` / `login` |
| **Purpose** | Authenticate returning users |

**What the user sees:**
- App title/logo
- Username field
- Password field
- "Login" button
- "Don't have an account? Register" link
- Optional success message (after registration redirect)
- Optional error message (wrong credentials)

**Actions:** Submit login form (POST `/login`), navigate to register.

**Mobile layout:** Centered card (max-width 400px), already works well on mobile.

**Changes needed:** None — this page works correctly.

---

### 4.2 Register Page

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /register` |
| **Template** | `templates/register.html` |
| **Handler** | `src/handlers/auth.rs` → `register_form` / `register` |
| **Purpose** | Create new account |

**What the user sees:**
- Username field (3–50 chars)
- Email field
- Password field (min 6 chars)
- Confirm password field
- "Register" button
- "Already have an account? Login" link
- Optional error message

**Actions:** Submit registration form (POST `/register`), navigate to login.

**What happens on submit:**
1. Validate inputs (username length, email format, password length, passwords match)
2. Check username/email uniqueness
3. Hash password with Argon2
4. Create user row
5. Seed 25 default exercises for this user
6. Redirect to `/login?message=Registration successful`

**Mobile layout:** Same centered card as login. Works correctly.

**Changes needed:** None — this page works correctly.

---

### 4.3 Dashboard

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /` |
| **Template** | `templates/dashboard.html` |
| **Handler** | `src/handlers/dashboard.rs` → `index` |
| **Purpose** | Home screen — show what matters right now |

**What the user currently sees:**
- Navbar (top)
- Stats grid: exercise count, session count, log count
- Two-column grid: upcoming sessions (left), recent logs (right)
- Quick actions: "New Exercise", "Plan Session", "Log Workout"

**What the user should see (target state):**
- **Today's Workout card** (top, prominent): Shows the session scheduled for today (if any), with exercise names and planned sets/reps. "Start Workout" button. If no session today, show next upcoming session with date.
- **Stats grid**: Same 3 cards (exercise count, session count, log count)
- **Recent activity**: Last 5 logged sets (exercise name, weight × reps)
- **Quick actions**: "Start Workout" (primary), "Log Set" (secondary), "Browse Plans" (if no sessions exist)

**Data queries:**
- Current: `ExerciseLog::find_recent_by_user(10)`, `WorkoutSession::find_upcoming_by_user(5)`, counts
- Needed: A query for today's session specifically (scheduled_at date = today AND status = 'planned')

**Mobile layout:**
- Single column, stacked vertically
- Today's workout card takes full width at the top
- Stats grid: 3 cards in a row (they already collapse nicely)
- Bottom tab navigation replaces top navbar

**Changes needed:**
- [ ] Add "today's workout" query and card
- [ ] Rearrange to prioritize today's workout
- [ ] Replace top navbar with bottom tab navigation
- [ ] Make quick actions contextual (show "Start Workout" if session exists today)

---

### 4.4 Exercise Library (List)

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /exercises` |
| **Template** | `templates/exercises/list.html` |
| **Handler** | `src/handlers/exercise.rs` → `list` |
| **Purpose** | Browse all exercises the user has |

**What the user currently sees:**
- Table with columns: Name, Muscle Group, Description, Actions (Edit button)
- "New Exercise" button in header

**What the user should see (target state):**
- **Search bar** at top (client-side filter by name)
- **Muscle group filter tabs** (All, Chest, Back, Shoulders, Legs, Arms, Core)
- **Card-based list** (not table): Each card shows exercise name, muscle group badge, difficulty badge, equipment icon/text, truncated description
- Tap card → exercise detail
- "New Exercise" floating action button or header button

**Data displayed per card:**
- Exercise name (linked to detail)
- Muscle group (colored badge)
- Difficulty level (new field — badge)
- Equipment needed (new field — icon or text)
- Description (first ~60 chars, truncated)

**Actions:** Tap card → detail, tap "New Exercise" → form, search/filter (client-side).

**Mobile layout:**
- Full-width cards, stacked vertically
- Each card is a single tappable block (44px+ height)
- Search bar is sticky at top below nav
- Filter tabs scroll horizontally if they overflow

**Changes needed:**
- [ ] Convert table to card-based layout
- [ ] Add search input (client-side JS filter)
- [ ] Add muscle group filter tabs
- [ ] Display new fields (difficulty, equipment) once added to schema
- [ ] Add delete button/swipe on list items (currently only on detail page)

---

### 4.5 Exercise Detail

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /exercises/{id}` |
| **Template** | `templates/exercises/detail.html` |
| **Handler** | `src/handlers/exercise.rs` → `detail` |
| **Purpose** | View full exercise info + personal history |

**What the user currently sees:**
- Exercise name, muscle group badge, description, created date
- Edit button, Delete button

**What the user should see (target state):**
- **Header**: Exercise name, muscle group badge, difficulty badge, equipment
- **Instructions section** (new): Step-by-step numbered instructions for performing the exercise
- **Tips section** (new): 2–3 form cues (e.g., "Keep your back straight", "Don't lock your knees")
- **Personal history section**: All logged sets for this exercise, ordered by date descending. Shows date, weight × reps for each set. Highlights personal records.
- **Actions**: Edit, Delete

**Data queries:**
- Current: `Exercise::find_by_id`
- Needed: `ExerciseLog::find_by_exercise_and_user` (new query for personal history)

**Mobile layout:**
- Single column, sections stacked
- Instructions collapsed by default (tap to expand) — progressive disclosure
- History shows last 10 entries by default, "Show more" to load rest

**Changes needed:**
- [ ] Add instructions and tips fields to exercise model/template
- [ ] Add personal history section (requires new query)
- [ ] Add difficulty and equipment display
- [ ] Collapsible sections for instructions on mobile

---

### 4.6 Exercise Form (New / Edit)

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /exercises/new`, `GET /exercises/{id}/edit` |
| **Template** | `templates/exercises/form.html` |
| **Handler** | `src/handlers/exercise.rs` → `new_form` / `edit_form` / `create` / `update` |
| **Purpose** | Create or edit a custom exercise |

**What the user currently sees:**
- Name input, muscle group dropdown (11 options), description textarea
- Create/Update + Cancel buttons

**What the user should see (target state):**
- Name input
- Muscle group dropdown (standardized list)
- Equipment dropdown (new): Barbell, Dumbbell, Cable, Machine, Bodyweight, Resistance Band, Other
- Difficulty dropdown (new): Beginner, Intermediate, Advanced
- Description textarea
- Instructions textarea (new) — multi-line, step-by-step
- Tips textarea (new) — form cues
- Create/Update + Cancel buttons

**Mobile layout:** Single column form, all fields stacked. Inputs are 16px font minimum (prevents iOS zoom).

**Changes needed:**
- [ ] Add equipment, difficulty, instructions, tips fields to form
- [ ] Update handler to accept and persist new fields
- [ ] 16px minimum font size on all inputs

---

### 4.7 Plans Page

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /plans` |
| **Template** | `templates/plans/list.html` |
| **Handler** | `src/handlers/plan.rs` → `list` |
| **Purpose** | Browse pre-built workout plans and adopt one |

**What the user currently sees:**
- Intro text explaining plans
- 3 plan cards (StrongLifts 5×5, Push/Pull/Legs, Full Body 3×), each with:
  - Plan name + difficulty badge
  - Description
  - Per-day tables (Exercise, Sets, Reps, Weight — shows "-" for 0 weight)
  - Tip paragraph
  - "Use This Plan" button

**What the user should see (target state):**
- Same 3 plans, better formatted for mobile
- Each plan card:
  - Plan name + difficulty badge + duration ("3 days/week")
  - Short description
  - Collapsible day sections (tap to expand exercise list)
  - Exercise list as simple rows (not table): "Exercise Name — 3×10 @ 20kg"
  - "Use This Plan" button (prominent, primary color)
- After using a plan: redirect to sessions with success message explaining what was created

**Data:** Hardcoded in `src/handlers/plan.rs` → `get_plans()`. No database table for plans.

**Mobile layout:**
- Plans stack vertically, each is a full-width card
- Day sections are collapsible accordions
- Exercise lists use simple text rows, not tables
- "Use This Plan" button is full-width at bottom of each card

**Changes needed:**
- [ ] Convert per-day tables to simple text rows on mobile
- [ ] Add collapsible day sections
- [ ] Improve plan card mobile layout
- [ ] Consider showing which plan the user already adopted (query for sessions created by plan)

---

### 4.8 Sessions List

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /sessions` |
| **Template** | `templates/sessions/list.html` |
| **Handler** | `src/handlers/session.rs` → `list` |
| **Purpose** | View all planned and past workout sessions |

**What the user currently sees:**
- Table with columns: Name, Scheduled, Status (badge), Actions (Edit button)
- "Plan Session" button in header

**What the user should see (target state):**
- **Card-based list** (not table), grouped or sorted by date
- Each card shows:
  - Session name
  - Scheduled date (human-readable: "Monday, Feb 25" not "2026-02-25T09:00:00")
  - Status badge (Planned / Completed / Cancelled)
  - **Exercise preview**: First 3 exercise names (e.g., "Squat, Bench Press, Row + 2 more") — this requires joining `session_exercises`
  - Tap → session detail
- "Plan Session" button
- Filter by status (All / Planned / Completed)

**Data queries:**
- Current: `WorkoutSession::find_all_by_user` — queries only the `workout_sessions` table
- Needed: Join with `session_exercises` + `exercises` to get exercise names for preview

**Mobile layout:**
- Full-width stacked cards
- Each card is tappable
- Date formatted for readability
- Delete button accessible (swipe or icon) — currently only on detail page

**Changes needed:**
- [ ] Convert table to card-based layout
- [ ] Add exercise preview to each session card (requires handler query change)
- [ ] Add delete button to list items
- [ ] Human-readable date formatting
- [ ] Status filter

---

### 4.9 Session Detail

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /sessions/{id}` |
| **Template** | `templates/sessions/detail.html` |
| **Handler** | `src/handlers/session.rs` → `detail` |
| **Purpose** | View session plan and start workout |

**What the user currently sees:**
- Session name, scheduled date, status badge, notes
- Planned exercises table: #, Exercise Name, Muscle Group, Sets, Reps, Weight (kg)
- Edit + Delete buttons
- Back link

**What the user should see (target state):**
- Session name, scheduled date (human-readable), status badge
- Notes (if any)
- **Planned exercises as cards** (not table):
  - Exercise name (linked to exercise detail)
  - Muscle group badge
  - "3 sets × 10 reps @ 40kg" format
- **"Start Workout" button** (prominent, primary) — navigates to Active Workout page
- Edit + Delete buttons (secondary)

**Known bug:** Weight column shows "0" instead of "-" or "bodyweight" when `planned_weight_kg` is 0.0. The plans template handles this correctly with a conditional, but the session detail template does not.

**Mobile layout:**
- Single column
- Exercise cards are full-width, stacked
- "Start Workout" button is full-width, fixed at bottom or prominently placed

**Changes needed:**
- [ ] Fix weight display: show "-" or "Bodyweight" when weight is 0
- [ ] Convert exercise table to cards
- [ ] Add "Start Workout" button (links to Active Workout page)
- [ ] Human-readable date formatting
- [ ] Link exercise names to exercise detail pages

---

### 4.10 Session Form (New / Edit)

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /sessions/new`, `GET /sessions/{id}/edit` |
| **Template** | `templates/sessions/form.html` |
| **Handler** | `src/handlers/session.rs` → `new_form` / `edit_form` / `create` / `update` |
| **Purpose** | Create or edit a workout session with exercises |

**What the user currently sees:**
- Session name input
- Scheduled date/time input (datetime-local)
- Status dropdown (Planned / Completed / Cancelled)
- Notes textarea
- Dynamic exercise rows: Exercise dropdown, Sets, Reps, Weight, Remove button
- "+ Add Exercise" button
- Submit + Cancel buttons

**What the user should see (target state):**
- Same fields, better mobile layout
- Exercise rows should stack fields vertically on mobile (they partially do this already)
- Exercise row header should also stack on mobile (currently stays horizontal — visual mismatch)
- Consider: exercise autocomplete/search instead of dropdown (for users with many exercises)

**Mobile layout:**
- All form fields full-width
- Exercise rows: stack vertically on mobile
- Fix header row responsive behavior to match data rows
- Large touch targets on all interactive elements

**Changes needed:**
- [ ] Fix `.exercise-row-header` to also stack vertically on mobile (CSS)
- [ ] Ensure 44px minimum touch targets on remove buttons
- [ ] 16px font size on all inputs

---

### 4.11 Active Workout Page (NEW — does not exist yet)

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /sessions/{id}/workout` (proposed) |
| **Template** | `templates/sessions/workout.html` (new) |
| **Handler** | `src/handlers/session.rs` → `active_workout` (new) |
| **Purpose** | Guide user through their workout, logging sets in real time |

**This is the most important new page.** It is the core gym experience.

**What the user sees:**
- Current exercise name (large text)
- Target: "3 sets × 10 reps @ 40kg" (from session plan)
- **Set logging cards**: For each set:
  - Weight input (pre-filled with planned weight or last session's weight)
  - Reps input (pre-filled with planned reps)
  - "Done" checkmark button to log the set
  - After logging: card shows completed state (green checkmark, weight × reps displayed)
- **Exercise progress indicator**: "Exercise 2 of 6" or progress dots
- **Navigation**: "Previous Exercise" / "Next Exercise" buttons or swipe
- **Exercise info link**: Small "How to do this?" link that shows instructions in a bottom sheet or expandable section
- **Finish Workout button**: Appears after all exercises are done (or always available to end early)

**Behavior:**
- Logging a set creates an `ExerciseLog` row linked to the session
- Weight pre-fills: first from session plan (`planned_weight_kg`), falling back to last logged weight for this exercise
- After logging all planned sets for an exercise, automatically advance to next exercise
- "Finish Workout" sets session status to `completed`

**Data queries needed:**
- Session with all session_exercises (already exists: `SessionExercise::find_by_session`)
- Last logged weight per exercise: `ExerciseLog::find_last_by_exercise_and_user` (new query)
- Create individual log entries: `ExerciseLog::create` (exists)

**Mobile layout:**
- Full-screen experience, no distractions
- Current exercise takes center stage
- Large weight/reps inputs (easy to tap and type)
- Bottom action buttons (Log Set, Next Exercise)
- Minimal chrome — hide normal navigation, show only "X" to exit workout

**Changes needed:**
- [ ] Create new route, handler, and template
- [ ] Add `ExerciseLog::find_last_by_exercise_and_user` query
- [ ] Add endpoint to mark session as completed
- [ ] Client-side JS for set logging without full page reload (or use htmx-style partial updates)
- [ ] Pre-fill weight from plan or last session

---

### 4.12 Logs List

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /logs` |
| **Template** | `templates/logs/list.html` |
| **Handler** | `src/handlers/log.rs` → `list` |
| **Purpose** | View all logged sets |

**What the user currently sees:**
- Table with 8 columns: Exercise, Set, Reps, Weight (kg), RPE, Session, Date, Actions (View button)
- "Log Workout" button in header

**What the user should see (target state):**
- **Card-based list**, grouped by date
- Each date group header: "Monday, Feb 25, 2026"
- Each card within a date group:
  - Exercise name (bold)
  - Set details: "Set 2: 40kg × 10 reps" (compact format)
  - Session name (if linked)
  - RPE (if present)
  - Delete button (icon)
- "Log Workout" button in header

**Mobile layout:**
- Full-width cards grouped by date
- Compact set display format
- Swipe to delete or trash icon on each card

**Changes needed:**
- [ ] Convert 8-column table to card-based layout
- [ ] Group logs by date
- [ ] Add delete button to list items (currently only on detail page)
- [ ] Human-readable date formatting

---

### 4.13 Log Detail

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /logs/{id}` |
| **Template** | `templates/logs/detail.html` |
| **Handler** | `src/handlers/log.rs` → `detail` |
| **Purpose** | View a single log entry |

**What the user currently sees:**
- Exercise name + muscle group badge
- Set number, reps completed, weight (kg), RPE (if present), session name (if linked), performed at date
- Delete button
- Back link

**What the user should see (target state):**
- Same information, cleaner layout
- Add context: was this a PR? How does it compare to the last time?
- Consider: is this page even necessary? Most users will just view logs in the list. Keep it but don't prioritize improvements.

**Changes needed:**
- [ ] Minor layout improvements (lower priority)
- [ ] Optional: show comparison to previous log for same exercise

---

### 4.14 Log Form

| Attribute | Value |
|-----------|-------|
| **Route** | `GET /logs/new` |
| **Template** | `templates/logs/form.html` |
| **Handler** | `src/handlers/log.rs` → `new_form` / `create` |
| **Purpose** | Manually log a single set |

**What the user currently sees:**
- Exercise dropdown (required)
- Session dropdown (optional)
- Set number + Reps inputs (row)
- Weight + RPE inputs (row)
- Submit + Cancel buttons
- Guard: if no exercises exist, shows error with link to create exercises

**What the user should see (target state):**
- Same fields, with smart defaults:
  - After selecting an exercise, auto-fill weight with last logged weight for that exercise
  - Default set number = (last set number for this exercise in this session) + 1
- Exercise search/filter instead of raw dropdown (if many exercises)

**Mobile layout:**
- Single column form
- Input pairs stack on mobile (already handled by `.form-row` CSS)
- 16px font size on all inputs

**Changes needed:**
- [ ] Auto-fill last weight on exercise select (requires JS + API endpoint)
- [ ] Auto-increment set number within a session
- [ ] 16px minimum font size on inputs

---

## 5. Exercise Library Requirements

### 5.1 New Exercise Fields

The current `exercises` table has: `id`, `user_id`, `name`, `description`, `muscle_group`, `created_at`.

**New fields to add:**

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `equipment` | TEXT NOT NULL | `''` | Equipment needed: Barbell, Dumbbell, Cable, Machine, Bodyweight, Resistance Band, Other |
| `difficulty` | TEXT NOT NULL | `'beginner'` | Difficulty level: Beginner, Intermediate, Advanced |
| `instructions` | TEXT NOT NULL | `''` | Step-by-step instructions (one step per line, numbered) |
| `tips` | TEXT NOT NULL | `''` | Form cues and tips (one tip per line) |

### 5.2 Standardized Categories

**Muscle Groups** (current, keep as-is):
Chest, Back, Shoulders, Biceps, Triceps, Legs, Glutes, Core, Full Body, Cardio, Other

**Equipment** (new):
Barbell, Dumbbell, Cable, Machine, Bodyweight, Resistance Band, Other

**Difficulty** (new):
Beginner, Intermediate, Advanced

### 5.3 Seed Data Specification

All 25 exercises should be updated with rich content. The `seed_defaults()` function in `src/models/exercise.rs` and a new `POST /exercises/seed` endpoint must provide this data.

Below is the full seed data for all 25 exercises:

---

#### 1. Barbell Squat
- **Muscle Group:** Legs
- **Equipment:** Barbell
- **Difficulty:** Beginner
- **Description:** The king of leg exercises. A compound movement that targets quads, glutes, and core.
- **Instructions:**
  1. Set the barbell on a squat rack at shoulder height
  2. Step under the bar and position it on your upper traps
  3. Grip the bar wide, squeeze your shoulder blades together
  4. Unrack the bar and take 2 steps back
  5. Stand with feet shoulder-width apart, toes slightly out
  6. Take a deep breath, brace your core
  7. Push your hips back and bend your knees to lower down
  8. Go until your thighs are at least parallel to the floor
  9. Drive through your whole foot to stand back up
  10. Exhale at the top
- **Tips:**
  - Keep your chest up and eyes forward throughout the movement
  - Your knees should track over your toes — it's okay for them to go past your toes
  - If you can't reach parallel, work on ankle and hip mobility
  - Start with just the bar (20kg) to learn the pattern

#### 2. Deadlift
- **Muscle Group:** Back
- **Equipment:** Barbell
- **Difficulty:** Intermediate
- **Description:** A full-body pull from the floor. Builds total-body strength, especially the posterior chain (back, glutes, hamstrings).
- **Instructions:**
  1. Stand with feet hip-width apart, bar over mid-foot
  2. Bend at the hips and grip the bar just outside your knees
  3. Drop your hips until your shins touch the bar
  4. Flatten your back, chest up, shoulders over or slightly in front of the bar
  5. Take a deep breath and brace your core hard
  6. Push the floor away with your legs while pulling the bar up your shins
  7. Once the bar passes your knees, thrust your hips forward to lock out
  8. Stand tall with shoulders back — don't lean back
  9. Lower by pushing hips back first, then bending knees once bar passes them
- **Tips:**
  - The bar should stay in contact with your legs the entire lift
  - Think "push the floor away" rather than "pull the bar up"
  - Don't round your lower back — if it rounds, the weight is too heavy
  - Use mixed grip or straps if grip fails before your back does

#### 3. Bench Press
- **Muscle Group:** Chest
- **Equipment:** Barbell
- **Difficulty:** Beginner
- **Description:** The primary chest builder. A horizontal press that also works shoulders and triceps.
- **Instructions:**
  1. Lie on the bench with eyes under the bar
  2. Grip the bar slightly wider than shoulder-width
  3. Plant your feet flat on the floor
  4. Squeeze your shoulder blades together and down (retract and depress)
  5. Unrack the bar with straight arms over your chest
  6. Lower the bar to your mid-chest / nipple line with control
  7. Touch your chest lightly — don't bounce
  8. Press the bar back up to the starting position
  9. Keep your butt on the bench throughout
- **Tips:**
  - Always use a spotter or safety bars when going heavy
  - Grip the bar hard — it activates more muscle fibers
  - A slight arch in your lower back is normal and safe
  - Lower the bar in a slight diagonal, not straight down to your neck

#### 4. Overhead Press
- **Muscle Group:** Shoulders
- **Equipment:** Barbell
- **Difficulty:** Intermediate
- **Description:** Standing barbell press overhead. Builds shoulder strength and stability. Also works core and triceps.
- **Instructions:**
  1. Set the bar at upper chest height in a rack
  2. Grip the bar just outside shoulder-width
  3. Unrack and hold the bar at your front shoulders (front rack position)
  4. Stand with feet shoulder-width apart
  5. Take a breath, brace your core and squeeze your glutes
  6. Press the bar straight up, moving your head back slightly to clear your chin
  7. Once the bar passes your forehead, push your head through (under the bar)
  8. Lock out with the bar directly over your mid-foot
  9. Lower back to front shoulders with control
- **Tips:**
  - Don't lean back excessively — that turns it into an incline press
  - Squeeze your glutes to protect your lower back
  - This lift progresses slower than bench or squat — that's normal
  - Start light — overhead pressing is harder than it looks

#### 5. Barbell Row
- **Muscle Group:** Back
- **Equipment:** Barbell
- **Difficulty:** Intermediate
- **Description:** Bent-over rowing builds a thick, strong back. Targets lats, rhomboids, and rear delts.
- **Instructions:**
  1. Stand with feet hip-width apart, bar over mid-foot
  2. Hinge at the hips until your torso is roughly 45 degrees to the floor
  3. Grip the bar just outside your knees (overhand grip)
  4. Let the bar hang at arm's length
  5. Brace your core and keep your back flat
  6. Pull the bar to your lower chest / upper belly
  7. Squeeze your shoulder blades together at the top
  8. Lower with control back to arm's length
- **Tips:**
  - Your torso angle matters — too upright makes it a shrug, too low strains your back
  - Pull with your elbows, not your hands — imagine your hands are hooks
  - A little body English is okay on the last rep or two, but don't make it a habit
  - If your lower back rounds, reduce the weight

#### 6. Incline Dumbbell Press
- **Muscle Group:** Chest
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Dumbbell press on an incline bench. Emphasizes the upper chest and allows a greater range of motion than barbell.
- **Instructions:**
  1. Set an adjustable bench to 30–45 degrees
  2. Sit down and place the dumbbells on your thighs
  3. Kick the dumbbells up one at a time as you lie back
  4. Hold the dumbbells at chest level with palms facing forward
  5. Press the dumbbells up and slightly together
  6. Lower with control until your upper arms are parallel to the floor or slightly below
  7. Press back up to the starting position
- **Tips:**
  - 30 degrees works upper chest effectively — you don't need a steep incline
  - Don't let your elbows flare out to 90 degrees; keep them at about 45 degrees
  - At the bottom, you should feel a stretch across your chest
  - Use the "kick-up" technique to get heavy dumbbells into position safely

#### 7. Dumbbell Row
- **Muscle Group:** Back
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Single-arm row using a bench for support. Great for beginners because the bench stabilizes your lower back.
- **Instructions:**
  1. Place one knee and one hand on a flat bench
  2. Your other foot is on the floor, slightly back for stability
  3. Hold a dumbbell in your free hand, arm hanging straight down
  4. Keep your back flat and parallel to the floor
  5. Pull the dumbbell to your hip, leading with your elbow
  6. Squeeze your shoulder blade at the top
  7. Lower with control
  8. Complete all reps on one side, then switch
- **Tips:**
  - Think about pulling your elbow to the ceiling, not curling the weight
  - Keep your shoulders square — don't rotate your torso to cheat the weight up
  - This is a great exercise to build up to barbell rows
  - Use a full range of motion — let the arm extend fully at the bottom

#### 8. Lat Pulldown
- **Muscle Group:** Back
- **Equipment:** Cable
- **Difficulty:** Beginner
- **Description:** Cable pulldown that mimics a pull-up. Builds lat width and is adjustable to any strength level.
- **Instructions:**
  1. Sit at the lat pulldown machine with thighs secured under the pads
  2. Grip the wide bar with an overhand grip, slightly wider than shoulder-width
  3. Lean back very slightly (about 10 degrees)
  4. Pull the bar down to your upper chest
  5. Focus on driving your elbows down and back
  6. Squeeze at the bottom for a moment
  7. Let the bar return up with control — fully extend your arms
- **Tips:**
  - Don't pull behind your neck — it's harder on your shoulders with no extra benefit
  - Lean back slightly but don't swing — your torso should stay mostly still
  - This is a great way to build up to doing pull-ups
  - Try different grip widths to target different parts of your back

#### 9. Leg Press
- **Muscle Group:** Legs
- **Equipment:** Machine
- **Difficulty:** Beginner
- **Description:** Machine-based leg press at 45 degrees. Easier to learn than squats and great for building leg volume safely.
- **Instructions:**
  1. Sit in the leg press machine with your back flat against the pad
  2. Place your feet shoulder-width apart on the platform, about halfway up
  3. Unlock the safety handles
  4. Lower the platform by bending your knees toward your chest
  5. Go as low as you can while keeping your lower back pressed into the pad
  6. Push through your whole foot to extend your legs
  7. Don't fully lock out your knees at the top
  8. Re-engage the safety handles when done
- **Tips:**
  - Higher foot placement = more glutes and hamstrings; lower = more quads
  - Never lock your knees fully — keep a slight bend at the top
  - If your lower back lifts off the pad, you're going too deep for your mobility
  - Start with moderate weight — the machine can handle a lot, but your joints need time to adapt

#### 10. Romanian Deadlift
- **Muscle Group:** Legs
- **Equipment:** Barbell
- **Difficulty:** Intermediate
- **Description:** A hip-hinge movement that targets hamstrings and glutes. Unlike conventional deadlifts, the bar doesn't touch the floor between reps.
- **Instructions:**
  1. Hold the barbell at hip height with an overhand grip (unrack from a rack or deadlift it up first)
  2. Stand with feet hip-width apart, slight bend in your knees
  3. Push your hips back while keeping the bar close to your legs
  4. Lower the bar along your thighs and shins
  5. Go until you feel a strong stretch in your hamstrings (usually mid-shin to just below the knee)
  6. Drive your hips forward to return to standing
  7. Squeeze your glutes at the top
- **Tips:**
  - This is NOT a squat — your knees should barely bend; the movement comes from your hips
  - Keep the bar in contact with your legs the entire time
  - Think about pushing your butt back toward the wall behind you
  - You should feel a strong stretch in the back of your thighs — that means you're doing it right

#### 11. Leg Curl
- **Muscle Group:** Legs
- **Equipment:** Machine
- **Difficulty:** Beginner
- **Description:** Machine exercise isolating the hamstrings. Simple and effective for building hamstring strength.
- **Instructions:**
  1. Adjust the machine so the pad sits on the back of your lower legs, just above your heels
  2. Lie face down (or sit, depending on machine type)
  3. Grip the handles for stability
  4. Curl your heels toward your glutes by bending your knees
  5. Squeeze your hamstrings at the top
  6. Lower with control — don't let the weight slam down
- **Tips:**
  - Control the lowering (eccentric) phase — that's where a lot of the muscle-building happens
  - Don't lift your hips off the pad during the curl
  - Point your toes slightly down to increase hamstring activation
  - If one leg is weaker, consider doing single-leg curls

#### 12. Leg Extension
- **Muscle Group:** Legs
- **Equipment:** Machine
- **Difficulty:** Beginner
- **Description:** Machine exercise isolating the quadriceps. Great for warming up before squats or finishing off your quads.
- **Instructions:**
  1. Sit in the machine with your back against the pad
  2. Adjust the pad so it sits on the front of your lower shins, just above your feet
  3. Grip the side handles
  4. Extend your legs by straightening your knees
  5. Squeeze your quads hard at the top (full extension)
  6. Lower back down with control
- **Tips:**
  - Don't use momentum — this is an isolation exercise, keep it strict
  - Pause briefly at the top for an extra quad squeeze
  - If you feel knee discomfort, try limiting the range of motion at the bottom
  - Great as a warmup with light weight before squats

#### 13. Lunges
- **Muscle Group:** Legs
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Walking or stationary lunges. A unilateral leg exercise that builds strength, balance, and addresses muscle imbalances.
- **Instructions:**
  1. Stand upright holding dumbbells at your sides (or bodyweight to start)
  2. Take a large step forward with one leg
  3. Lower your body until both knees are at about 90 degrees
  4. Your back knee should nearly touch the floor
  5. Push through your front heel to stand back up
  6. For walking lunges: step forward with the other leg. For stationary: step back to start.
  7. Alternate legs each rep
- **Tips:**
  - Keep your torso upright — don't lean forward
  - Your front knee should stay over your ankle, not past your toes
  - If balance is an issue, do stationary lunges (stepping back to center) before walking lunges
  - Start with bodyweight until the pattern feels natural

#### 14. Lateral Raise
- **Muscle Group:** Shoulders
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Dumbbell side raises targeting the medial (side) deltoid. Builds shoulder width and the "capped" look.
- **Instructions:**
  1. Stand with feet shoulder-width apart, a dumbbell in each hand at your sides
  2. Lean forward very slightly from the hips
  3. With a slight bend in your elbows, raise both arms out to the sides
  4. Lift until your arms are parallel to the floor (forming a T-shape)
  5. Pause briefly at the top
  6. Lower with control back to your sides
- **Tips:**
  - Use light weight — this is a small muscle and ego-lifting leads to bad form
  - Lead with your elbows, not your hands — imagine pouring water from a pitcher
  - Don't shrug your shoulders up; keep them down and relaxed
  - A slight lean forward helps target the side delt better

#### 15. Face Pull
- **Muscle Group:** Shoulders
- **Equipment:** Cable
- **Difficulty:** Beginner
- **Description:** Cable exercise for rear delts and rotator cuff. Essential for shoulder health and posture, especially for people who bench press.
- **Instructions:**
  1. Set a cable machine with a rope attachment at upper chest to face height
  2. Grip the rope with both hands, thumbs pointing toward you
  3. Step back to create tension
  4. Pull the rope toward your face, separating your hands as you pull
  5. At the end, your hands should be beside your ears, elbows high and back
  6. Squeeze your rear delts and hold for a moment
  7. Return with control
- **Tips:**
  - This is a health exercise, not an ego exercise — keep the weight moderate
  - Think about pulling the rope apart, not just toward you
  - Your elbows should end up high (above shoulder level)
  - Do these on every upper body day — your shoulders will thank you

#### 16. Barbell Curl
- **Muscle Group:** Biceps
- **Equipment:** Barbell
- **Difficulty:** Beginner
- **Description:** The classic bicep builder. A simple, effective isolation movement for the front of your arms.
- **Instructions:**
  1. Stand with feet shoulder-width apart
  2. Hold the barbell with an underhand (supinated) grip at shoulder width
  3. Let the bar hang at arm's length in front of you
  4. Keep your elbows pinned to your sides
  5. Curl the bar up toward your shoulders by bending your elbows
  6. Squeeze your biceps at the top
  7. Lower with control back to the starting position
- **Tips:**
  - Don't swing your body — if you need momentum, the weight is too heavy
  - Keep your elbows stationary at your sides throughout the movement
  - Use an EZ-curl bar if straight bar hurts your wrists
  - Full range of motion matters more than weight on this exercise

#### 17. Hammer Curl
- **Muscle Group:** Biceps
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Dumbbell curl with a neutral (hammer) grip. Targets the brachioradialis and long head of the bicep.
- **Instructions:**
  1. Stand with feet shoulder-width apart, dumbbells at your sides
  2. Hold the dumbbells with a neutral grip (palms facing each other / thumbs up)
  3. Keep your elbows pinned to your sides
  4. Curl both dumbbells up toward your shoulders
  5. The palms stay facing each other the entire time
  6. Squeeze at the top
  7. Lower with control
- **Tips:**
  - You can alternate arms or do both at the same time
  - Don't swing — keep your torso still
  - These build forearm strength in addition to biceps
  - You can usually go slightly heavier on hammer curls than regular curls

#### 18. Tricep Pushdown
- **Muscle Group:** Triceps
- **Equipment:** Cable
- **Difficulty:** Beginner
- **Description:** Cable pushdown targeting the triceps. The primary tricep isolation exercise in most programs.
- **Instructions:**
  1. Set a cable machine with a straight bar or rope attachment at the top
  2. Stand facing the machine, feet shoulder-width apart
  3. Grip the attachment with an overhand grip (or neutral for rope)
  4. Pin your elbows to your sides
  5. Push the weight down by extending your elbows until your arms are straight
  6. Squeeze your triceps at the bottom
  7. Let the weight come back up with control — stop when your forearms are parallel to the floor
- **Tips:**
  - Your elbows should not move forward or back — only your forearms move
  - Lean forward very slightly for better tricep activation
  - Rope attachment allows you to spread the ends at the bottom for an extra squeeze
  - If your elbows flare out, the weight is too heavy

#### 19. Overhead Tricep Extension
- **Muscle Group:** Triceps
- **Equipment:** Dumbbell
- **Difficulty:** Beginner
- **Description:** Overhead extension that targets the long head of the tricep. Can be done with a dumbbell or cable.
- **Instructions:**
  1. Hold a single dumbbell with both hands, gripping the inner plate
  2. Press it overhead with straight arms
  3. Your upper arms should point straight up, close to your ears
  4. Lower the dumbbell behind your head by bending your elbows
  5. Go until you feel a stretch in your triceps (forearms roughly parallel to the floor)
  6. Extend back up to straight arms
  7. Keep your upper arms stationary throughout
- **Tips:**
  - Keep your core tight — don't arch your back excessively
  - Your elbows will want to flare out; try to keep them pointing forward
  - Start light until you're comfortable with the movement overhead
  - This exercise gives a great stretch on the tricep, which helps growth

#### 20. Dip
- **Muscle Group:** Chest
- **Equipment:** Bodyweight
- **Difficulty:** Intermediate
- **Description:** Parallel bar dip. Compound pushing movement for chest, shoulders, and triceps. Lean forward for chest focus, stay upright for tricep focus.
- **Instructions:**
  1. Grip the parallel bars and jump or push yourself up to straight arms
  2. Cross your ankles behind you or keep legs straight
  3. Lean your torso forward slightly (about 15–30 degrees) for chest emphasis
  4. Lower yourself by bending your elbows until your upper arms are parallel to the floor
  5. Push back up to straight arms
  6. Don't lock out aggressively at the top
- **Tips:**
  - If you can't do bodyweight dips, use an assisted dip machine or resistance band
  - More forward lean = more chest; more upright = more triceps
  - Don't go too deep — upper arms parallel to the floor is deep enough
  - If you feel shoulder pain, this exercise may not be for you — substitute with decline push-ups

#### 21. Pull-up
- **Muscle Group:** Back
- **Equipment:** Bodyweight
- **Difficulty:** Intermediate
- **Description:** The ultimate back and bicep exercise. Builds a wide, strong back. Use assistance if you can't do a full pull-up yet.
- **Instructions:**
  1. Hang from a pull-up bar with an overhand grip, slightly wider than shoulder-width
  2. Let yourself hang fully (dead hang) to start
  3. Pull yourself up by driving your elbows down and back
  4. Continue until your chin is over the bar
  5. Lower yourself with control back to a full hang
  6. Don't swing or kip — strict form
- **Tips:**
  - If you can't do pull-ups yet: use an assisted pull-up machine, or do band-assisted pull-ups, or do lat pulldowns to build up
  - Think about pulling your elbows to your hips, not pulling your chin over the bar
  - Avoid swinging — dead stop at the bottom, controlled pull to the top
  - Even one good pull-up is an achievement — build from there

#### 22. Push-up
- **Muscle Group:** Chest
- **Equipment:** Bodyweight
- **Difficulty:** Beginner
- **Description:** The foundational pushing exercise. Works chest, shoulders, and triceps. Infinitely scalable from knees to feet to elevated.
- **Instructions:**
  1. Start in a plank position: hands slightly wider than shoulder-width, arms straight
  2. Your body should form a straight line from head to heels
  3. Lower your chest toward the floor by bending your elbows
  4. Go until your chest nearly touches the floor
  5. Push back up to the starting position
  6. Keep your core tight throughout — don't let your hips sag
- **Tips:**
  - Too hard? Do them on your knees, or with hands on a bench/wall
  - Too easy? Elevate your feet on a bench, wear a backpack, or slow down the lowering phase
  - Your elbows should be at about a 45-degree angle to your body, not flared at 90
  - A push-up done with good form is better than 10 sloppy ones

#### 23. Plank
- **Muscle Group:** Core
- **Equipment:** Bodyweight
- **Difficulty:** Beginner
- **Description:** Isometric core hold. Builds core stability and endurance. The foundation for all core training.
- **Instructions:**
  1. Get into a forearm plank position: elbows under shoulders, forearms on the floor
  2. Extend your legs behind you, up on your toes
  3. Your body should form a straight line from head to heels
  4. Brace your core as if someone is about to poke your stomach
  5. Hold this position for the target time
  6. Breathe normally — don't hold your breath
- **Tips:**
  - Squeeze your glutes to help keep your hips level
  - Look at the floor about a foot in front of your hands (don't crane your neck up)
  - If your hips sag, drop to your knees for a modified version
  - Start with 20-second holds and build up — form matters more than time

#### 24. Cable Crunch
- **Muscle Group:** Core
- **Equipment:** Cable
- **Difficulty:** Beginner
- **Description:** Kneeling crunch using a cable machine for resistance. Effective ab exercise that allows progressive overload (unlike bodyweight crunches).
- **Instructions:**
  1. Attach a rope to the high pulley of a cable machine
  2. Kneel facing the machine, about 2 feet away
  3. Hold the rope behind your head with both hands, at ear level
  4. Brace your core and crunch down, bringing your elbows toward your knees
  5. Focus on flexing your spine, not just bending at the hips
  6. Squeeze your abs at the bottom
  7. Return to the starting position with control
- **Tips:**
  - Don't sit back on your heels — keep your hips high
  - The movement should come from your abs curling your spine, not from your arms pulling
  - Don't go too heavy — you should feel your abs working, not your hip flexors
  - Exhale as you crunch down for a stronger contraction

#### 25. Seated Calf Raise
- **Muscle Group:** Legs
- **Equipment:** Machine
- **Difficulty:** Beginner
- **Description:** Seated machine exercise targeting the soleus (lower calf). Builds calf size and ankle stability.
- **Instructions:**
  1. Sit in the seated calf raise machine with the balls of your feet on the platform
  2. Adjust the pad so it sits snugly on your lower thighs
  3. Release the safety handle
  4. Lower your heels as far as they'll go (full stretch)
  5. Push up onto your toes as high as you can (full contraction)
  6. Hold the top position for a moment
  7. Lower back down with control
- **Tips:**
  - Calves respond well to high reps — try sets of 15–20
  - Full range of motion is critical — stretch all the way down, squeeze all the way up
  - Go slow and controlled — don't bounce
  - Seated calf raise targets a different muscle than standing calf raise — ideally do both

### 5.4 Seed Endpoint

**Problem**: `seed_defaults()` is only called during registration (`src/handlers/auth.rs`). Users who registered before the exercise library was complete have no exercises, or have exercises with empty descriptions/instructions.

**Solution**: Add a `POST /exercises/seed` endpoint.

**Behavior**:
1. For each exercise in the seed list:
   - If an exercise with that name already exists for the user: update its description, instructions, tips, equipment, and difficulty (don't duplicate)
   - If it doesn't exist: create it
2. Return redirect to `/exercises?message=Exercise library updated with N exercises`
3. This endpoint is idempotent — safe to call multiple times

**Where to add**: `src/handlers/exercise.rs` (new handler function), registered in router in `src/main.rs`.

---

## 6. Mobile-First Design Requirements

### 6.1 CSS Architecture Overhaul

**Current state**: `static/css/style.css` is desktop-first. Base styles assume a wide screen. A single `@media (max-width: 640px)` block overrides for mobile.

**Target state**: Invert to mobile-first.

- **Base styles** = phone (320–428px viewport)
- **`@media (min-width: 640px)`** = tablet/small desktop
- **`@media (min-width: 1024px)`** = large desktop

**File**: `static/css/style.css`

### 6.2 Bottom Tab Navigation

**Current state**: Horizontal navbar at top with 5 links + username + logout button. This wraps/overflows on phone screens.

**Target state**:
- **Mobile (< 640px)**: Fixed bottom tab bar with 4–5 icons + labels
  - Tabs: Home, Exercises, Plans, Sessions, Logs
  - Username and logout move to a settings/profile section or the dashboard
  - Active tab is highlighted
  - Tab bar is always visible (fixed bottom)
  - Safe area padding for phones with home indicators
- **Desktop (>= 640px)**: Keep the current horizontal top navbar

**Implementation**: The navbar HTML is currently **duplicated in every template** (12 files). The nav block in `base.html` is empty. A better approach:
1. Move the navbar HTML into `base.html` (requires passing `username` to all templates, which is already done)
2. Or: create a partial template and include it

**Additional note on deduplication**: Currently, each of the 12 authenticated page templates contains its own copy of the nav HTML. Any nav change must be replicated 12 times. Consolidating the nav into `base.html` is a prerequisite for the bottom tab bar.

### 6.3 Touch Targets

- All tappable elements (buttons, links, cards, nav tabs): minimum 44×44px
- Form inputs: minimum 48px height
- Spacing between tappable elements: minimum 8px

### 6.4 Tables → Cards

Every list page currently uses `<table>`. On mobile, tables with 4+ columns don't fit and require horizontal scrolling (poor UX).

**Pages to convert:**

| Page | Current Columns | Card Format |
|------|----------------|-------------|
| Exercise list | Name, Muscle Group, Description, Actions | Card: Name (bold), muscle group badge, description (truncated) |
| Session list | Name, Scheduled, Status, Actions | Card: Name, date (readable), status badge, exercise preview |
| Log list | Exercise, Set, Reps, Weight, RPE, Session, Date, Actions | Card: Exercise name, "Set N: Wkg × R reps", date |

**Desktop behavior**: Cards can display in a grid or retain table layout at wider breakpoints. Cards are acceptable on desktop too.

### 6.5 Font Sizes

- Body text: minimum 14px
- Form inputs: minimum 16px (**critical** — below 16px, iOS Safari auto-zooms on focus, which is disorienting)
- Headings: scale appropriately (h1: 24px, h2: 20px, h3: 18px on mobile)
- Buttons: minimum 16px

### 6.6 Spacing and Layout

- Container padding: 16px on mobile (not 0)
- Card padding: 16px internal
- Section spacing: 24px between major sections
- No horizontal scrolling on any page (except within code blocks or data tables if absolutely necessary)

---

## 7. Data Model

### 7.1 Current Schema

#### Table: `users`
**Migration**: `migrations/20240101000001_create_users.sql`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `username` | TEXT | NOT NULL UNIQUE |
| `email` | TEXT | NOT NULL UNIQUE |
| `password_hash` | TEXT | NOT NULL |
| `created_at` | TEXT | NOT NULL DEFAULT (datetime('now')) |

#### Table: `exercises`
**Migration**: `migrations/20240101000002_create_exercises.sql`
**Model**: `src/models/exercise.rs`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `user_id` | INTEGER | NOT NULL FK → users(id) |
| `name` | TEXT | NOT NULL |
| `description` | TEXT | NOT NULL DEFAULT '' |
| `muscle_group` | TEXT | NOT NULL DEFAULT '' |
| `created_at` | TEXT | NOT NULL DEFAULT (datetime('now')) |

Index: `idx_exercises_user_id` on `user_id`

#### Table: `workout_sessions`
**Migration**: `migrations/20240101000003_create_workout_sessions.sql`
**Model**: `src/models/workout_session.rs`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `user_id` | INTEGER | NOT NULL FK → users(id) |
| `name` | TEXT | NOT NULL |
| `scheduled_at` | TEXT | NOT NULL |
| `notes` | TEXT | NOT NULL DEFAULT '' |
| `status` | TEXT | NOT NULL DEFAULT 'planned', CHECK IN ('planned','completed','cancelled') |
| `created_at` | TEXT | NOT NULL DEFAULT (datetime('now')) |

Index: `idx_workout_sessions_user_id` on `user_id`

#### Table: `session_exercises`
**Migration**: `migrations/20240101000004_create_session_exercises.sql`
**Model**: `src/models/session_exercise.rs`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `workout_session_id` | INTEGER | NOT NULL FK → workout_sessions(id) ON DELETE CASCADE |
| `exercise_id` | INTEGER | NOT NULL FK → exercises(id) |
| `planned_sets` | INTEGER | NOT NULL DEFAULT 3 |
| `planned_reps` | INTEGER | NOT NULL DEFAULT 10 |
| `planned_weight_kg` | REAL | NOT NULL DEFAULT 0.0 |
| `sort_order` | INTEGER | NOT NULL DEFAULT 0 |

Index: `idx_session_exercises_session` on `workout_session_id`

#### Table: `exercise_logs`
**Migration**: `migrations/20240101000005_create_exercise_logs.sql`
**Model**: `src/models/exercise_log.rs`

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `user_id` | INTEGER | NOT NULL FK → users(id) |
| `exercise_id` | INTEGER | NOT NULL FK → exercises(id) |
| `workout_session_id` | INTEGER | NULLABLE FK → workout_sessions(id) |
| `set_number` | INTEGER | NOT NULL DEFAULT 1 |
| `reps_completed` | INTEGER | NOT NULL |
| `weight_kg` | REAL | NOT NULL DEFAULT 0.0 |
| `rpe` | REAL | NULLABLE |
| `performed_at` | TEXT | NOT NULL DEFAULT (datetime('now')) |

Indexes: `idx_exercise_logs_user_id`, `idx_exercise_logs_exercise_id`

### 7.2 Entity Relationships

```
users (1) ──< exercises (many)
users (1) ──< workout_sessions (many)
users (1) ──< exercise_logs (many)

workout_sessions (1) ──< session_exercises (many) >── exercises (many)
                                  [many-to-many join table with plan details]

workout_sessions (1) ──< exercise_logs (many)  [optional FK]
exercises (1) ──< exercise_logs (many)
```

### 7.3 New Fields (P1)

**Migration**: `migrations/2026XXXX_add_exercise_details.sql` (new)

Add to `exercises` table:

| Column | Type | Default | Purpose |
|--------|------|---------|---------|
| `equipment` | TEXT NOT NULL | `''` | Equipment needed |
| `difficulty` | TEXT NOT NULL | `'beginner'` | Difficulty level |
| `instructions` | TEXT NOT NULL | `''` | Step-by-step instructions (one per line) |
| `tips` | TEXT NOT NULL | `''` | Form cues (one per line) |

```sql
ALTER TABLE exercises ADD COLUMN equipment TEXT NOT NULL DEFAULT '';
ALTER TABLE exercises ADD COLUMN difficulty TEXT NOT NULL DEFAULT 'beginner';
ALTER TABLE exercises ADD COLUMN instructions TEXT NOT NULL DEFAULT '';
ALTER TABLE exercises ADD COLUMN tips TEXT NOT NULL DEFAULT '';
```

**Model changes**: `src/models/exercise.rs`
- Add fields to `Exercise` struct
- Update `create`, `update`, `seed_defaults` to include new fields
- Update `find_all_by_user`, `find_by_id` queries

### 7.4 P2 Additions (future)

#### Session timing fields

Add to `workout_sessions`:
| Column | Type | Default | Purpose |
|--------|------|---------|---------|
| `started_at` | TEXT | NULL | When user started the workout |
| `completed_at` | TEXT | NULL | When user finished the workout |

#### Personal records table (new)

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `user_id` | INTEGER | NOT NULL FK → users(id) |
| `exercise_id` | INTEGER | NOT NULL FK → exercises(id) |
| `record_type` | TEXT | NOT NULL ('max_weight', 'max_reps', 'max_volume') |
| `value` | REAL | NOT NULL |
| `exercise_log_id` | INTEGER | NOT NULL FK → exercise_logs(id) |
| `achieved_at` | TEXT | NOT NULL |

### 7.5 New API Endpoints

| Method | Route | Purpose | Priority |
|--------|-------|---------|----------|
| `POST` | `/exercises/seed` | Seed/update exercise library for existing users | P0 |
| `GET` | `/sessions/{id}/workout` | Active workout page | P1 |
| `POST` | `/sessions/{id}/workout/log` | Log a set during active workout (returns partial HTML or redirects) | P1 |
| `POST` | `/sessions/{id}/complete` | Mark session as completed | P1 |
| `GET` | `/api/exercises/{id}/last-weight` | Get last logged weight for an exercise (JSON for JS auto-fill) | P1 |
| `GET` | `/exercises/{id}/history` | Exercise log history (could be section on detail page) | P2 |

---

## 8. Known Bugs & Fixes

### Bug #1: Empty exercises for existing users (P0)

**Symptom**: Users who registered before seed data was added (or before it was comprehensive) have no exercises, or have exercises with missing descriptions/instructions.

**Root cause**: `Exercise::seed_defaults()` in `src/models/exercise.rs` is only called during registration in `src/handlers/auth.rs:87`. There is no way for existing users to get updated seed data.

**Fix**: Add `POST /exercises/seed` endpoint that upserts (creates or updates) all 25 default exercises for the current user. Add a button on the exercises list page: "Load Default Exercises" (shown if exercise count is low or always available).

**Files to modify**: `src/handlers/exercise.rs` (new handler), `src/models/exercise.rs` (upsert logic), `src/main.rs` (route), `templates/exercises/list.html` (button).

---

### Bug #2: Desktop-first CSS (P0)

**Symptom**: The app looks fine on desktop but has layout issues on mobile — content overflows, touch targets are too small, tables require horizontal scrolling.

**Root cause**: `static/css/style.css` uses desktop-first approach. Base styles assume wide screens. A single `@media (max-width: 640px)` block tries to fix mobile, but many components are missing mobile overrides.

**Fix**: Invert to mobile-first CSS. Base styles = phone. Add `@media (min-width: 640px)` and `@media (min-width: 1024px)` for progressive enhancement.

**Files to modify**: `static/css/style.css` (full rewrite of media query approach).

---

### Bug #3: Navbar wraps/overflows on mobile (P0)

**Symptom**: The navbar has 5 navigation links + username display + logout button. On phone screens (< 400px), this wraps to multiple lines or overflows horizontally.

**Root cause**: Navbar uses flexbox with `flex-wrap: wrap` and has too many items for small screens. The 640px media query reduces padding but doesn't reduce the number of items.

**Fix**: Replace with bottom tab navigation on mobile (4–5 icon+label tabs). Move username/logout to dashboard or profile area. See Section 6.2.

**Files to modify**: `templates/base.html` (or all 12 templates if nav stays per-template), `static/css/style.css`.

---

### Bug #4: Tables don't fit on mobile screens (P0)

**Symptom**: Exercise list (4 columns), session list (4 columns), and log list (8 columns) use HTML tables that require horizontal scrolling on phones.

**Root cause**: All list pages use `<table>` elements wrapped in `.table-responsive` (overflow-x: auto). This provides scrolling but poor mobile UX.

**Fix**: Convert tables to card-based layouts on mobile. See Section 6.4.

**Files to modify**: `templates/exercises/list.html`, `templates/sessions/list.html`, `templates/logs/list.html`, `static/css/style.css`.

---

### Bug #5: Missing delete buttons on list pages (P0)

**Symptom**: Users can only delete exercises, sessions, and logs from their detail pages. There is no delete option on list pages, forcing an extra navigation step.

**Root cause**: Delete buttons are only included in detail templates (`exercises/detail.html`, `sessions/detail.html`, `logs/detail.html`), not in list templates.

**Fix**: Add delete buttons (small trash icon or "X" button) to each item in list/card views. Include the `confirmDelete()` JavaScript confirmation.

**Files to modify**: `templates/exercises/list.html`, `templates/sessions/list.html`, `templates/logs/list.html`.

---

### Bug #6: Terse exercise descriptions (P1)

**Symptom**: The 25 seeded exercises have one-line descriptions that don't help a beginner understand how to perform the exercise.

**Root cause**: `seed_defaults()` in `src/models/exercise.rs` uses brief, technical descriptions. No step-by-step instructions. No form cues. No equipment info.

**Fix**: Add `equipment`, `difficulty`, `instructions`, and `tips` fields. Update seed data with the comprehensive content in Section 5.3.

**Files to modify**: `src/models/exercise.rs`, migration file, `templates/exercises/detail.html`, `templates/exercises/form.html`.

---

### Bug #7: No exercise preview on session list (P1)

**Symptom**: The session list shows session name, date, and status, but NOT which exercises are in the session. Users must tap into each session to see what exercises they'll do.

**Root cause**: The session list handler (`src/handlers/session.rs` → `list`) only queries `WorkoutSession::find_all_by_user`, which returns data from the `workout_sessions` table only. It does not join with `session_exercises` or `exercises`.

**Fix**: Modify the list handler to also query session exercises for each session (or use a single joined query). Pass exercise names to the template for preview display.

**Files to modify**: `src/handlers/session.rs` (list handler), `src/models/workout_session.rs` or `src/models/session_exercise.rs` (new query), `templates/sessions/list.html`.

---

### Bug #8: Weight shows "0" on session detail (P1)

**Symptom**: When a session exercise has `planned_weight_kg = 0.0` (bodyweight exercises), the session detail page displays "0" in the weight column instead of "-" or "Bodyweight".

**Root cause**: `templates/sessions/detail.html` renders `{{ exercise.planned_weight_kg }}` directly without a conditional. The plans template (`templates/plans/list.html`) correctly handles this: `{% if exercise.weight_kg > 0.0 %}{{ exercise.weight_kg }}{% else %}-{% endif %}`.

**Fix**: Add the same conditional to `templates/sessions/detail.html`:
```html
{% if exercise.planned_weight_kg > 0.0 %}{{ exercise.planned_weight_kg }}{% else %}-{% endif %}
```

**Files to modify**: `templates/sessions/detail.html`.

---

### Bug #9: No "today's workout" on dashboard (P1)

**Symptom**: The dashboard shows "upcoming sessions" but doesn't highlight today's workout. A gym user opening the app wants to see what they're doing TODAY at the top of the screen.

**Root cause**: The dashboard handler queries `find_upcoming_by_user(limit=5)`, which returns all future planned sessions. There is no specific query for today's session.

**Fix**: Add a query for today's session (`scheduled_at` date = today AND status = 'planned'). Display it prominently at the top of the dashboard with a "Start Workout" button.

**Files to modify**: `src/handlers/dashboard.rs`, `src/models/workout_session.rs` (new query), `templates/dashboard.html`.

---

### Bug #10: One-set-at-a-time logging (P1)

**Symptom**: The log form allows logging exactly one set at a time. For a typical workout with 5 exercises × 3 sets = 15 sets, the user must fill out and submit the form 15 times. This is extremely tedious at the gym.

**Root cause**: `templates/logs/form.html` is a single-set form. There is no multi-set logging interface or active workout mode.

**Fix**: The Active Workout page (Section 4.11) solves this by providing a streamlined set-logging interface within the context of a session. Each exercise shows all planned sets, and the user taps "Done" to log each one with pre-filled values.

**Files to modify**: New files for Active Workout page.

---

### Bug #11: No active workout flow (P1)

**Symptom**: There is no guided workout experience. The user must manually navigate between session detail, log form, and back — repeatedly — during their workout.

**Root cause**: The app has no Active Workout page. The session detail is view-only; logging is a separate, unconnected flow.

**Fix**: Build the Active Workout page (Section 4.11). This is the highest-impact new feature for the target user.

**Files to modify**: New handler, template, CSS, and JS. See Section 4.11 for full spec.

---

### Bug #12: No progress visibility (P1)

**Symptom**: Users have no way to see their progress over time for a specific exercise. "Am I getting stronger?" is unanswerable from the current UI.

**Root cause**: The exercise detail page shows only exercise metadata (name, description, muscle group). It does not display any log history for that exercise.

**Fix**: Add a "Personal History" section to the exercise detail page showing all logged sets for that exercise, ordered by date descending. Optionally highlight personal records (heaviest weight, most reps).

**Files to modify**: `src/handlers/exercise.rs` (detail handler), `src/models/exercise_log.rs` (new query: find by exercise and user), `templates/exercises/detail.html`.

---

### Bug #13: Session form exercise row header mismatch on mobile (P1)

**Symptom**: On mobile, the session form's exercise data rows stack vertically (flex-direction: column), but the header row stays horizontal. This creates a visual disconnect.

**Root cause**: The CSS has a responsive rule for `.exercise-row` at `max-width: 640px` that sets `flex-direction: column`, but `.exercise-row-header` does not have a corresponding rule.

**Fix**: Either hide the header row on mobile (since stacked rows don't need headers) or make it also stack.

**Files to modify**: `static/css/style.css`.

---

## 9. Feature Prioritization

### P0 — Broken / Unusable (fix first)

| # | Feature | Description | Bug Ref |
|---|---------|-------------|---------|
| 1 | Exercise seed endpoint | `POST /exercises/seed` for existing users to get/update exercises | Bug #1 |
| 2 | Mobile-first CSS | Invert CSS to mobile-first, fix base styles for phone screens | Bug #2 |
| 3 | Bottom tab navigation | Replace wrapping top navbar with mobile bottom tabs | Bug #3 |
| 4 | Tables → cards | Convert exercise, session, and log list tables to mobile-friendly cards | Bug #4 |
| 5 | Delete buttons on lists | Add delete capability to all list views | Bug #5 |

### P1 — Core UX for Beginners (build next)

| # | Feature | Description | Bug Ref |
|---|---------|-------------|---------|
| 6 | Rich exercise content | Add equipment, difficulty, instructions, tips fields and seed data | Bug #6 |
| 7 | Session exercise preview | Show exercise names on session list cards | Bug #7 |
| 8 | Fix weight display | Show "-" or "Bodyweight" for 0kg weights on session detail | Bug #8 |
| 9 | Today's workout on dashboard | Prominent card for today's scheduled session | Bug #9 |
| 10 | Active Workout page | Guided workout flow with set logging | Bug #10, #11 |
| 11 | Multi-set logging | Log sets within active workout (pre-filled weight/reps) | Bug #10 |
| 12 | Exercise history | Personal log history on exercise detail page | Bug #12 |
| 13 | Session form mobile fix | Fix exercise row header mismatch on mobile | Bug #13 |

### P2 — Nice-to-Have Enhancements (future)

| # | Feature | Description |
|---|---------|-------------|
| 14 | Session timing | Add `started_at` / `completed_at` to track workout duration |
| 15 | Personal records | Track and display PRs (max weight, max reps, max volume) |
| 16 | Workout summary | Show summary after completing a workout (total volume, PRs hit) |
| 17 | Exercise search | Client-side search/filter on exercise library |
| 18 | Muscle group filter | Filter exercises by muscle group on library page |
| 19 | Date formatting | Human-readable dates throughout ("Mon, Feb 25" instead of "2026-02-25T09:00:00") |
| 20 | Rest timer | Countdown timer between sets during active workout |
| 21 | Nav deduplication | Move navbar HTML from 12 templates into base.html |
| 22 | Log editing | Allow editing existing log entries (currently create/delete only) |
| 23 | Dark mode | Automatic dark mode based on system preference |

---

## 10. Implementation Sequencing

### Phase 1: Foundation Fixes

**Goal**: Make the app functional on phones and fix critical data issues.

**Tasks**:

1. **Seed endpoint** (`POST /exercises/seed`)
   - `src/models/exercise.rs`: Add upsert logic to `seed_defaults` (or new method)
   - `src/handlers/exercise.rs`: New `seed` handler function
   - `src/main.rs`: Register route `.route("/exercises/seed", post(exercise::seed))`
   - `templates/exercises/list.html`: Add "Load Default Exercises" button

2. **Mobile-first CSS rewrite**
   - `static/css/style.css`: Invert all media queries. Base = mobile. Add `min-width: 640px` and `min-width: 1024px` breakpoints.
   - Set 16px minimum on all form inputs
   - Ensure 44px touch targets on all interactive elements

3. **Bottom tab navigation**
   - `templates/base.html`: Add bottom tab bar HTML in the nav block (or in main layout)
   - `static/css/style.css`: Style bottom tabs (fixed bottom, icons + labels, safe area padding)
   - All 12 authenticated templates: Remove per-template navbar duplication (or phase this as part of nav dedup)

4. **Tables → cards**
   - `templates/exercises/list.html`: Replace `<table>` with card markup
   - `templates/sessions/list.html`: Replace `<table>` with card markup
   - `templates/logs/list.html`: Replace `<table>` with card markup
   - `static/css/style.css`: Add card styles for list items

5. **Delete buttons on lists**
   - `templates/exercises/list.html`: Add delete form/button per item
   - `templates/sessions/list.html`: Add delete form/button per item
   - `templates/logs/list.html`: Add delete form/button per item

**Estimated scope**: ~5 files modified significantly, ~3 new pieces of code.

---

### Phase 2: Exercise Content

**Goal**: Make exercises useful for beginners with instructions and form cues.

**Tasks**:

1. **Schema migration**
   - New migration file: `migrations/2026XXXX_add_exercise_details.sql`
   - Add `equipment`, `difficulty`, `instructions`, `tips` columns to `exercises`

2. **Model update**
   - `src/models/exercise.rs`: Add new fields to `Exercise` struct, update all queries (`create`, `update`, `find_all_by_user`, `find_by_id`, `seed_defaults`)

3. **Rich seed data**
   - `src/models/exercise.rs`: Update `seed_defaults()` with full content from Section 5.3
   - Or: load seed data from a TOML/JSON file (optional, could stay in code)

4. **Template updates**
   - `templates/exercises/detail.html`: Display instructions (collapsible), tips, equipment, difficulty
   - `templates/exercises/form.html`: Add equipment dropdown, difficulty dropdown, instructions textarea, tips textarea
   - `templates/exercises/list.html`: Show difficulty badge and equipment on cards

5. **Handler updates**
   - `src/handlers/exercise.rs`: Accept and persist new fields in `create` and `update` handlers

**Estimated scope**: 1 migration, ~4 files modified.

---

### Phase 3: Core Gym Experience

**Goal**: Build the features that matter most when the user is at the gym.

**Tasks**:

1. **Today's workout on dashboard**
   - `src/models/workout_session.rs`: Add `find_today_by_user` query (scheduled_at date = today AND status = 'planned')
   - `src/handlers/dashboard.rs`: Query today's session + its exercises, pass to template
   - `templates/dashboard.html`: Add prominent "Today's Workout" card at top with exercise list and "Start Workout" button

2. **Session exercise preview on list**
   - `src/models/session_exercise.rs`: Add query to get exercise names for multiple sessions (or batch query)
   - `src/handlers/session.rs` → `list`: Join session exercises, pass to template
   - `templates/sessions/list.html`: Display exercise preview on each session card

3. **Active Workout page**
   - `src/handlers/session.rs`: New `active_workout` handler (GET `/sessions/{id}/workout`)
   - `templates/sessions/workout.html`: New template per spec in Section 4.11
   - `src/models/exercise_log.rs`: Add `find_last_by_exercise_and_user` for weight pre-fill
   - `static/js/app.js`: Client-side JS for set logging (or full page reload approach)
   - `src/handlers/session.rs`: New `log_set` handler (POST `/sessions/{id}/workout/log`)
   - `src/handlers/session.rs`: New `complete_session` handler (POST `/sessions/{id}/complete`)

4. **Multi-set logging within active workout**
   - Already part of the Active Workout page — each exercise shows all planned sets, user logs each one

5. **Fix weight display on session detail**
   - `templates/sessions/detail.html`: Add conditional for 0 weight

**Estimated scope**: 2 new files, ~6 files modified, significant new JS.

---

### Phase 4: Polish

**Goal**: Round out the experience and address remaining P1/P2 items.

**Tasks**:

1. **Exercise history on detail page**
   - `src/models/exercise_log.rs`: Add `find_by_exercise_and_user` query
   - `src/handlers/exercise.rs` → `detail`: Query logs, pass to template
   - `templates/exercises/detail.html`: Add "History" section showing past sets

2. **Human-readable date formatting**
   - `src/templates.rs` or Askama custom filters: Add date formatting helper
   - Update all templates that display dates

3. **Nav deduplication**
   - Move navbar from 12 individual templates into `base.html`
   - Ensure `username` is available in base template context

4. **Plan flow improvements**
   - `templates/plans/list.html`: Collapsible day sections, mobile-friendly exercise display
   - Consider showing which plan user already adopted

5. **P2 features** (as time/priority allows):
   - Session timing (started_at / completed_at)
   - Personal records tracking
   - Workout summary after completion
   - Exercise search and muscle group filter
   - Rest timer
   - Log editing
   - Dark mode

**Estimated scope**: Varies per feature.

---

## Appendix: File Index

| File | Purpose |
|------|---------|
| `src/main.rs` | App entry point, router setup, server start |
| `src/config.rs` | Configuration struct |
| `src/db.rs` | SQLite connection pool |
| `src/error.rs` | AppError enum and IntoResponse |
| `src/templates.rs` | All Askama template structs |
| `src/models/mod.rs` | Model re-exports |
| `src/models/user.rs` | User model (find, create, exists) |
| `src/models/exercise.rs` | Exercise model (CRUD, seed_defaults, count) |
| `src/models/workout_session.rs` | Session model (CRUD, upcoming, count) |
| `src/models/session_exercise.rs` | Session-exercise join model (find_by_session, create, delete) |
| `src/models/exercise_log.rs` | Log model (CRUD, recent, count) |
| `src/handlers/mod.rs` | Handler re-exports, AppState struct |
| `src/handlers/auth.rs` | Register, login, logout handlers |
| `src/handlers/dashboard.rs` | Dashboard index handler |
| `src/handlers/exercise.rs` | Exercise CRUD handlers |
| `src/handlers/session.rs` | Session CRUD handlers |
| `src/handlers/log.rs` | Log CRUD handlers |
| `src/handlers/plan.rs` | Hardcoded plans, use-plan handler |
| `src/middleware/mod.rs` | Middleware re-exports |
| `src/middleware/auth.rs` | JWT auth, AuthUser extractor, token creation |
| `templates/base.html` | Base layout (head, empty nav block, container, app.js) |
| `templates/login.html` | Login page |
| `templates/register.html` | Registration page |
| `templates/dashboard.html` | Dashboard with stats, upcoming, recent |
| `templates/exercises/list.html` | Exercise list (table) |
| `templates/exercises/detail.html` | Exercise detail view |
| `templates/exercises/form.html` | Exercise new/edit form |
| `templates/sessions/list.html` | Session list (table) |
| `templates/sessions/detail.html` | Session detail with planned exercises |
| `templates/sessions/form.html` | Session new/edit form with dynamic exercise rows |
| `templates/logs/list.html` | Log list (table, 8 columns) |
| `templates/logs/detail.html` | Log detail view |
| `templates/logs/form.html` | Log new form |
| `templates/plans/list.html` | Hardcoded plans display |
| `static/css/style.css` | All styles (289 lines, single breakpoint) |
| `static/js/app.js` | Delete confirmation + dynamic exercise rows (22 lines) |
| `migrations/20240101000001_create_users.sql` | Users table |
| `migrations/20240101000002_create_exercises.sql` | Exercises table |
| `migrations/20240101000003_create_workout_sessions.sql` | Sessions table |
| `migrations/20240101000004_create_session_exercises.sql` | Session-exercises join table |
| `migrations/20240101000005_create_exercise_logs.sql` | Exercise logs table |
