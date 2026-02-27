use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct Exercise {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    pub muscle_group: String,
    pub equipment: String,
    pub difficulty: String,
    pub instructions: String,
    pub tips: String,
    pub created_at: String,
}

impl Exercise {
    pub async fn find_all_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, description, muscle_group, equipment, difficulty, instructions, tips, created_at FROM exercises WHERE user_id = ? ORDER BY name"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, description, muscle_group, equipment, difficulty, instructions, tips, created_at FROM exercises WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool, user_id: i64, name: &str, description: &str, muscle_group: &str,
        equipment: &str, difficulty: &str, instructions: &str, tips: &str,
    ) -> sqlx::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO exercises (user_id, name, description, muscle_group, equipment, difficulty, instructions, tips) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(muscle_group)
        .bind(equipment)
        .bind(difficulty)
        .bind(instructions)
        .bind(tips)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update(
        pool: &SqlitePool, id: i64, user_id: i64, name: &str, description: &str, muscle_group: &str,
        equipment: &str, difficulty: &str, instructions: &str, tips: &str,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query(
            "UPDATE exercises SET name = ?, description = ?, muscle_group = ?, equipment = ?, difficulty = ?, instructions = ?, tips = ? WHERE id = ? AND user_id = ?"
        )
        .bind(name)
        .bind(description)
        .bind(muscle_group)
        .bind(equipment)
        .bind(difficulty)
        .bind(instructions)
        .bind(tips)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM exercises WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM exercises WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_by_name_and_user(pool: &SqlitePool, name: &str, user_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, description, muscle_group, equipment, difficulty, instructions, tips, created_at FROM exercises WHERE name = ? AND user_id = ?"
        )
        .bind(name)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn seed_defaults(pool: &SqlitePool, user_id: i64) -> sqlx::Result<()> {
        for (name, muscle_group, description, equipment, difficulty, instructions, tips) in get_default_exercises() {
            Self::create(pool, user_id, name, description, muscle_group, equipment, difficulty, instructions, tips).await?;
        }
        Ok(())
    }

    pub async fn seed_or_update_defaults(pool: &SqlitePool, user_id: i64) -> sqlx::Result<u32> {
        let mut count = 0u32;
        for (name, muscle_group, description, equipment, difficulty, instructions, tips) in get_default_exercises() {
            let existing = Self::find_by_name_and_user(pool, name, user_id).await?;
            if let Some(ex) = existing {
                sqlx::query(
                    "UPDATE exercises SET description = ?, muscle_group = ?, equipment = ?, difficulty = ?, instructions = ?, tips = ? WHERE id = ? AND user_id = ?"
                )
                .bind(description)
                .bind(muscle_group)
                .bind(equipment)
                .bind(difficulty)
                .bind(instructions)
                .bind(tips)
                .bind(ex.id)
                .bind(user_id)
                .execute(pool)
                .await?;
            } else {
                Self::create(pool, user_id, name, description, muscle_group, equipment, difficulty, instructions, tips).await?;
            }
            count += 1;
        }
        Ok(count)
    }
}

fn get_default_exercises() -> Vec<(&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str)> {
    vec![
        (
            "Barbell Squat", "Legs",
            "Barbell on upper back, squat to parallel or below",
            "Barbell", "beginner",
            "Set up the barbell at shoulder height in the rack\nStep under the bar, positioning it on your upper traps\nUnrack and take 2-3 steps back\nStand with feet shoulder-width apart, toes slightly out\nBrace your core, then push hips back and bend knees\nDescend until thighs are at least parallel to the floor\nDrive through your whole foot to stand back up",
            "Keep your chest up throughout the movement\nDon't let your knees cave inward\nStart with just the bar (20kg) to learn the pattern",
        ),
        (
            "Deadlift", "Back",
            "Barbell from floor, hip hinge movement",
            "Barbell", "beginner",
            "Stand with feet hip-width apart, bar over mid-foot\nBend at hips and knees to grip the bar just outside your legs\nFlatten your back, brace your core\nPush through your feet while keeping the bar close to your body\nStand up fully, squeezing glutes at the top\nReverse the motion to lower the bar to the floor",
            "The bar should travel in a straight vertical line\nDon't round your lower back\nThink of pushing the floor away rather than pulling the bar up",
        ),
        (
            "Bench Press", "Chest",
            "Flat barbell bench press",
            "Barbell", "beginner",
            "Lie on the bench with eyes under the bar\nGrip the bar slightly wider than shoulder width\nPlant feet firmly on the floor\nUnrack and hold the bar over your chest with arms locked\nLower the bar to your mid-chest in a controlled arc\nPress back up to the starting position",
            "Keep your shoulder blades pinched together\nMaintain a slight arch in your lower back\nAlways use a spotter or safety bars when going heavy",
        ),
        (
            "Overhead Press", "Shoulders",
            "Standing barbell press overhead",
            "Barbell", "beginner",
            "Start with the bar at shoulder height in the front rack position\nGrip slightly wider than shoulder width\nBrace your core and squeeze your glutes\nPress the bar straight up, moving your head out of the way\nLock out arms overhead with the bar over mid-foot\nLower back to shoulders under control",
            "Don't lean back excessively - keep your core tight\nBreathe in before pressing, exhale at the top\nThis is the hardest lift to progress - small jumps are fine",
        ),
        (
            "Barbell Row", "Back",
            "Bent-over barbell row to lower chest",
            "Barbell", "beginner",
            "Stand with feet shoulder-width apart, holding the barbell\nHinge at the hips until your torso is roughly 45 degrees\nLet the bar hang at arm's length\nPull the bar to your lower chest/upper abdomen\nSqueeze your shoulder blades together at the top\nLower the bar under control",
            "Keep your core tight to protect your lower back\nDon't use momentum to swing the weight up\nA slight bit of body English is okay on heavier sets",
        ),
        (
            "Incline Dumbbell Press", "Chest",
            "Dumbbell press on 30-45 degree incline bench",
            "Dumbbell", "beginner",
            "Set the bench to a 30-45 degree incline\nSit back with a dumbbell in each hand at shoulder level\nPress the dumbbells up and slightly inward\nLock out at the top without clanking the dumbbells\nLower under control until your upper arms are parallel to the floor",
            "The incline targets your upper chest\nDon't set the bench too steep - 30 degrees is enough\nUse a controlled tempo on the way down",
        ),
        (
            "Dumbbell Row", "Back",
            "Single-arm dumbbell row on bench",
            "Dumbbell", "beginner",
            "Place one knee and hand on a flat bench\nHold a dumbbell in the other hand, arm hanging straight\nPull the dumbbell up toward your hip\nSqueeze your back at the top, then lower under control\nComplete all reps on one side, then switch",
            "Keep your back flat and parallel to the floor\nDon't twist your torso to lift the weight\nThink about driving your elbow toward the ceiling",
        ),
        (
            "Lat Pulldown", "Back",
            "Cable pulldown to upper chest",
            "Cable", "beginner",
            "Sit at the lat pulldown machine and secure your thighs under the pad\nGrip the bar wider than shoulder width with palms facing away\nLean back slightly and pull the bar to your upper chest\nSqueeze your lats at the bottom\nSlowly return the bar to the starting position",
            "Don't lean way back or use momentum\nFocus on pulling with your elbows, not your hands\nGreat for building up to bodyweight pull-ups",
        ),
        (
            "Leg Press", "Legs",
            "Machine leg press at 45 degrees",
            "Machine", "beginner",
            "Sit in the leg press machine with your back flat against the pad\nPlace feet shoulder-width apart on the platform\nRelease the safety handles\nLower the platform by bending your knees toward your chest\nPress back up without locking your knees completely",
            "Don't let your lower back round off the pad\nKeep the full foot on the platform\nHigher foot placement emphasizes glutes and hamstrings",
        ),
        (
            "Romanian Deadlift", "Legs",
            "Barbell hip hinge, slight knee bend, hamstring stretch",
            "Barbell", "intermediate",
            "Hold the barbell at hip level with an overhand grip\nKeep a slight bend in your knees (don't lock them)\nPush your hips back, lowering the bar along your thighs\nDescend until you feel a strong stretch in your hamstrings\nDrive your hips forward to return to standing",
            "Keep the bar close to your legs throughout\nYour back should stay flat - stop before it rounds\nThis is NOT a stiff-leg deadlift - maintain the knee bend",
        ),
        (
            "Leg Curl", "Legs",
            "Machine lying or seated leg curl",
            "Machine", "beginner",
            "Adjust the machine so the pad sits on the back of your ankles\nLie face down (or sit if using seated version)\nCurl your legs by bending your knees\nSqueeze your hamstrings at the top\nLower back under control",
            "Don't let the weight stack slam between reps\nControl the eccentric (lowering) phase\nPair with leg extensions for balanced leg development",
        ),
        (
            "Leg Extension", "Legs",
            "Machine leg extension",
            "Machine", "beginner",
            "Sit in the machine with the pad on your lower shins\nGrip the handles at the sides\nExtend your legs by straightening your knees\nSqueeze your quads at the top\nLower back under control",
            "Don't use momentum - slow and controlled\nPause briefly at the top for maximum quad activation\nStart light to protect your knees",
        ),
        (
            "Lunges", "Legs",
            "Walking or stationary lunges with dumbbells",
            "Dumbbell", "beginner",
            "Hold a dumbbell in each hand at your sides\nStep forward with one leg\nLower your body until both knees are at about 90 degrees\nPush through your front foot to return to standing\nAlternate legs each rep",
            "Keep your torso upright - don't lean forward\nYour front knee should track over your toes\nStart with bodyweight to master balance first",
        ),
        (
            "Lateral Raise", "Shoulders",
            "Dumbbell lateral raises",
            "Dumbbell", "beginner",
            "Stand with a dumbbell in each hand at your sides\nWith a slight bend in your elbows, raise your arms out to the sides\nLift until your arms are parallel to the floor\nPause briefly at the top\nLower under control",
            "Use lighter weight than you think - form matters here\nDon't shrug your shoulders up\nLead with your elbows, not your hands",
        ),
        (
            "Face Pull", "Shoulders",
            "Cable face pull for rear delts and posture",
            "Cable", "beginner",
            "Set a cable machine to upper chest height with a rope attachment\nGrab the rope with both hands, palms facing each other\nStep back to create tension\nPull the rope toward your face, separating your hands\nSqueeze your rear delts and upper back\nReturn under control",
            "This is a posture and shoulder health exercise - keep it light\nPull to your forehead/eye level, not your chest\nGreat as a warmup or finisher",
        ),
        (
            "Barbell Curl", "Biceps",
            "Standing barbell curl",
            "Barbell", "beginner",
            "Stand with feet shoulder-width apart, holding a barbell with palms facing up\nKeep your elbows pinned at your sides\nCurl the bar up by bending your elbows\nSqueeze your biceps at the top\nLower the bar under control",
            "Don't swing your body to lift the weight\nKeep your upper arms stationary\nA slight lean forward at the start is fine",
        ),
        (
            "Hammer Curl", "Biceps",
            "Dumbbell hammer curl (neutral grip)",
            "Dumbbell", "beginner",
            "Stand with a dumbbell in each hand, palms facing your body\nKeep your elbows at your sides\nCurl the dumbbells up while keeping your palms facing inward\nSqueeze at the top, then lower under control",
            "The neutral grip also works the brachioradialis (forearm)\nCan be done alternating or simultaneously\nGreat complement to regular curls",
        ),
        (
            "Tricep Pushdown", "Triceps",
            "Cable tricep pushdown with rope or bar",
            "Cable", "beginner",
            "Stand facing a cable machine with a rope or bar attachment at the top\nGrab the attachment with palms facing down\nKeep your elbows pinned at your sides\nExtend your arms downward until fully straight\nSqueeze your triceps at the bottom\nReturn to the start under control",
            "Don't let your elbows flare out\nKeep your torso still - no leaning into it\nWith rope, spread the ends apart at the bottom for extra squeeze",
        ),
        (
            "Overhead Tricep Extension", "Triceps",
            "Dumbbell or cable overhead extension",
            "Dumbbell", "beginner",
            "Hold a dumbbell with both hands behind your head\nKeep your upper arms close to your ears\nExtend the dumbbell upward by straightening your arms\nSqueeze your triceps at the top\nLower back behind your head under control",
            "Keep your elbows pointing forward, not flaring out\nCan be done seated or standing\nUse a cable for constant tension throughout the range",
        ),
        (
            "Dip", "Chest",
            "Parallel bar dip (chest or tricep focus)",
            "Bodyweight", "intermediate",
            "Grip parallel bars and lift yourself up with arms straight\nLean forward slightly for chest emphasis (stay upright for triceps)\nLower your body by bending your elbows until upper arms are parallel to floor\nPush back up to the starting position",
            "If too difficult, use an assisted dip machine or resistance band\nDon't go too deep if you have shoulder issues\nLeaning forward targets chest; staying upright targets triceps",
        ),
        (
            "Pull-up", "Back",
            "Bodyweight or assisted pull-up",
            "Bodyweight", "intermediate",
            "Grip a pull-up bar with palms facing away, hands shoulder-width apart\nHang with arms fully extended\nPull yourself up until your chin is over the bar\nLower yourself under control back to full hang",
            "If you can't do a pull-up yet, use an assisted machine or resistance band\nAvoid kipping or swinging\nFocus on pulling your elbows down to your sides",
        ),
        (
            "Push-up", "Chest",
            "Bodyweight push-up (scale with knees or elevate feet)",
            "Bodyweight", "beginner",
            "Start in a high plank position, hands slightly wider than shoulders\nKeep your body in a straight line from head to heels\nLower your chest to the floor by bending your elbows\nPush back up to the starting position",
            "Scale easier by doing push-ups on your knees or against a wall\nScale harder by elevating your feet\nDon't let your hips sag or pike up",
        ),
        (
            "Plank", "Core",
            "Isometric core hold (time-based)",
            "Bodyweight", "beginner",
            "Start face down, then prop yourself up on your forearms and toes\nKeep your body in a perfectly straight line\nBrace your core as if someone is about to punch your stomach\nHold for the prescribed time\nBreathe normally throughout",
            "Don't let your hips sag or pike up\nSqueeze your glutes to keep your body straight\nStart with 20-30 seconds and build up",
        ),
        (
            "Cable Crunch", "Core",
            "Kneeling cable crunch",
            "Cable", "beginner",
            "Kneel in front of a cable machine with rope attachment at the top\nHold the rope behind your head\nCrunch your torso downward, bringing your elbows toward your knees\nSqueeze your abs at the bottom\nReturn to the starting position under control",
            "Don't pull with your arms - the motion comes from your abs\nKeep your hips stationary\nExhale as you crunch down",
        ),
        (
            "Seated Calf Raise", "Legs",
            "Machine seated calf raise",
            "Machine", "beginner",
            "Sit in the calf raise machine with the pad on your lower thighs\nPlace the balls of your feet on the platform\nRelease the safety and lower your heels as far as possible\nPush up onto your toes, squeezing your calves at the top\nLower back down under control",
            "Use a full range of motion - stretch at the bottom, squeeze at the top\nPause for 1-2 seconds at the top of each rep\nCalves respond well to higher reps (15-20)",
        ),
    ]
}
