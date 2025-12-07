# Canonical Format

## What is it?

The canonical format is a JSON file that all competition data sources must produce before an importer translate the format into database.

# Convention

- **No computed fields** - Rankings, best lifts, total scores, RIS scores are calculated by the application, not stored in the canonical format
- **Common denominator** - Only information that can reasonably be extracted from any source type (PDF, CSV, API)

## Why use it?

1. Human review: plain text file can be reviewed and corrected before importing into database
2. Version control: imported data are available, in their canonical format inside git.

## File location

Files are saved in: `./imports/{competition-slug}/{timestamp}_{source}.json`

## Structure

### Top level

```json
{
  "format_version": "1.0.0",
  "source": {...},
  "competition": {...},
  "movements": [...],
  "categories": [...]
}
```

### Source metadata

Where the data came from and when.

```json
{
  "type": "liftcontrol",
  "url": "https://api.liftcontrol.com/contest/123",
  "extracted_at": "2025-01-30T10:00:00Z",
  "extractor": "liftcontrol-api-v1"
}
```

### Competition

Basic competition info.

```json
{
  "name": "Annecy 4 Lift 2025",
  "slug": "annecy-4-lift-2025",
  "federation": {
    "name": "4 Lift",
    "abbreviation": "4L"
  },
  "start_date": "2025-01-15",
  "end_date": "2025-01-15",
  "country": "France",
  "number_of_judges": 3
}
```

All dates are ISO 8601 format (`YYYY-MM-DD`).

### Movements

List of exercises in the competition, in display order.

```json
[
  { "name": "Pull-up", "order": 1 },
  { "name": "Dips", "order": 2 },
  { "name": "Muscle-up", "order": 3 },
  { "name": "Squat", "order": 4 }
]
```

**Movement names must use canonical names:**

### Categories

Categories directly contain athletes.

```json
[
  {
    "name": "Men -73kg",
    "gender": "M",
    "weight_class_max": 73.0,
    "athletes": [...]
  }
]
```

- `gender`: `"M"` or `"F"`
- `weight_class_min`: Lower bound in kg (null for open lower bound like "-73kg")
- `weight_class_max`: Upper bound in kg (null for open upper bound like "120kg+")

### Athletes

Athlete performance data.

```json
{
  "first_name": "Jean",
  "last_name": "Dupont",
  "country": "France",
  "bodyweight": 72.5,
  "lifts": [...]
}
```

Required fields: `first_name`, `last_name`, `country`, `lifts`

Optional fields: `bodyweight`, `nationality`, `gender`, `is_disqualified`, `disqualified_reason`

**Note:** Rankings are computed by the application and should not be included in the canonical format.

### Lifts

One entry per movement.

```json
{
  "movement": "Pull-up",
  "attempts": [...]
}
```

The `movement` field must match a movement name from the `movements` array.

**Note:** Best lift is computed from the attempts by finding the highest successful weight. Do not include it in the canonical format.

### Attempts

Individual attempt data (typically 3 attempts per movement).

```json
{
  "attempt_number": 1,
  "weight": 100.0,
  "is_successful": true
}
```

Required: `attempt_number`, `weight`, `is_successful`

Optional: `no_rep_reason`

**Important:** Only store the boolean result (`is_successful`), not the individual judge decisions. The canonical format represents the final outcome, not the voting breakdown.

## Complete Example

```json
{
  "format_version": "1.0.0",
  "source": {
    "type": "liftcontrol",
    "extracted_at": "2025-01-30T10:30:00Z",
    "extractor": "liftcontrol-api-v1"
  },
  "competition": {
    "name": "Annecy 4 Lift 2025",
    "slug": "annecy-4-lift-2025",
    "federation": {
      "name": "4 Lift",
      "abbreviation": "4L"
    },
    "start_date": "2025-01-15",
    "end_date": "2025-01-15",
    "country": "France",
    "number_of_judges": 3
  },
  "movements": [
    { "name": "Pull-up", "order": 1 },
    { "name": "Dips", "order": 2 }
  ],
  "categories": [
    {
      "name": "Men -73kg",
      "gender": "M",
      "weight_class_max": 73.0,
      "athletes": [
        {
          "first_name": "Jean",
          "last_name": "Dupont",
          "country": "France",
          "bodyweight": 72.5,
          "lifts": [
            {
              "movement": "Pull-up",
              "attempts": [
                {
                  "attempt_number": 1,
                  "weight": 100.0,
                  "is_successful": true
                },
                {
                  "attempt_number": 2,
                  "weight": 110.0,
                  "is_successful": true
                },
                {
                  "attempt_number": 3,
                  "weight": 115.0,
                  "is_successful": false,
                  "no_rep_reason": "Elbows not locked"
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## Validation

The importer validates the JSON before inserting to database:

**Errors** (must fix):

- Missing required fields
- Invalid gender (must be M or F)
- Invalid dates
- Movement name not in canonical list
- Negative weights

**Warnings** (should review):

- Missing optional fields like bodyweight
- Unusual weight values
- Duplicate athletes in same competition
