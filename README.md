# recal
Rust Event Calendar

`Usage: ecal [OPTIONS]`

|Short|Long|Description|
|:--:|:--|:--|
| -m    |  --months <NUM>         | Number of months to display (1, 3, 6, or 12)
|  -cols|  --columns <NUM>        | Number of calendar columns per row (default: 3)
|  -s   |  --start <MONTH> <YEAR> | Start month and year
|  -f   |  --file <PATH>          | Path to events file (default: events.txt)
|  -mon |  --monday-first         | Week starts on Monday (default)
|  -sun |  --sunday-first         | Week starts on Sunday
|  -c   |  --calendar-only        | Show only calendar
|  -e   |  --events-only          | Show only events
|  -h   |  --help                 | Display this help message

## Event format


`Format: DateRule ;[type, [fg_color], [bg_color]]  Description`
`Colors: black, red, green, yellow, blue, magenta, cyan, white`
Foreground color (fg_color) and background color (bg_color) are optional

| DateRule | Description |
| :-- | :-- |
|   E            | (Easter Sunday) |
|   E+N / E-N    | (N days after/before Easter) |
|   MM/DOW#N     | (Nth Day of Week (DOW) of Month MM; |
|                |  DOW: 0=Sun, 1=Mon..6=Sat |
|                |  N:1-5. e.g. 5/1#1 is 1st Mon of May)|
|   MM/DD        | (Annual event on MM/DD of current year)|
|   MM/DD?       | (Same as MM/DD)|
|   MM/DD?YYYY   | (Event on MM/DD of specified YYYY)|
|   MM/DD?D[+-]N |(If MM/DD of year is DOW D (0=Sun..6=Sat), offset N days. e.g. 3/17?6+2) |
|   MM/DD/YYYY   | (Full US date)|
|   DD-MM-YYYY   | (Full date)|
 


## Screenshots

<img width="1086" height="579" alt="Screenshot_20251128-192110" src="https://github.com/user-attachments/assets/3f2c759a-ada0-4618-bedb-8f2fdbf0ad40" />
<img width="1086" height="579" alt="Screenshot_20251128-192108" src="https://github.com/user-attachments/assets/f9e907ed-6eba-423f-9942-1c3b5ea01d41" />
<img width="1903" height="818" alt="Screenshot_20251128-192243" src="https://github.com/user-attachments/assets/0ac42ef0-1016-4c2a-8f3c-febcf03b720f" />
