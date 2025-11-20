-- Seed the four core streetlifting movements with proper display order
-- These movements are fundamental to streetlifting competitions and should
-- always exist in the database

INSERT INTO movements (name, display_order)
VALUES
    ('Muscle-up', 1),
    ('Pull-up', 2),
    ('Dips', 3),
    ('Squat', 4)
ON CONFLICT (name) DO UPDATE
SET display_order = EXCLUDED.display_order;
