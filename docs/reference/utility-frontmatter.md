# Utility Frontmatter

The frontmatter the `render()` will start with will always include:

- `{{today}}` - today's date in YYYY-MM-DD format
- `{{yesterday}}` - yesterday's date in YYYY-MM-DD format
- `{{tomorrow}}` - tomorrow's date in YYYY-MM-DD format
- `{{day_of_week}}` - the day of the week fully spelled out and capitalized (e.g., "Monday", "Tuesday", etc.)
- `{{day_of_week_abbr}}` - the day of the week in abbreviated form (e.g., "Mon", "Tue", etc.)
- `{{now}}` - the date and time in ISO DateTime format with local timezone information adjusted to UTC (e.g., `2025-12-12T06:45Z`)
- `{{now_local}}` - the date and time in ISO DateTime format (e.g., `2025-12-12T13:45-07:00`) with local timezone included
- `{{timezone}}` - will provide the user's timezone if it can be determined from the calling system; will return "unknown" if not
- `{{last_day_in_month}}` - provides a boolean value to indicate whether **today** is the last day of the month
- `{{month}}` - the current month in fully spelled out form (e.g., "January", "February", etc.)
- `{{month_abbr}}` - the current month represented in an abbreviated form (e.g., "Jan", "Feb", "Mar", etc.)
- `{{month_numeric}}` - the current month represented in numeric fashion
- `{{season}}` - the current season (e.g., "Spring", "Summer", "Fall", "Winter")
- `{{year}}`
