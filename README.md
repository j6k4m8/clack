# clack

Clack is a code editor for your ears.

While clack does have a terminal-based user interface like vim or nano, it is designed to be used with keyboard inputs only, and audio outputs only. That makes clack a perfect editor low low-vision or vision-impaired users, or for users who are sitting in a park with a keyboard-only device like a Raspberry Pi 400 or a PC-K2.

# Usage

Run clack from the command line like any other terminal-based editor. You can pass a filename to clack to open or create it.

## Keyboard Shortcuts

Clack's keyboard shortcuts are still a work-in-progress and support will improve as time goes on.

For now, the following shortcuts are supported:

| Key      | Action |
| -------- | ------ |
| `Ctrl+S` | Save   |
| `Ctrl+Q` | Quit   |

## Speech Commands

| Key     | Action                               |
| ------- | ------------------------------------ |
| `Alt+L` | Speak the current line               |
| `Alt+/` | Speak the current location (Row/Col) |
| `Alt+.` | Speak the current word               |

## Navigation

Arrow keys move the cursor around. Page-up and page-down scroll the text by a page, and Home/End jump to the start/end of the line.

# Hardware

One fun aspect of using Clack is that it can be run entirely by ear, and does not require an integrated terminal. This means that you can use Clack on a keyboard-all-in-one. Here are some (untested and un-verified) examples of such hardware:

-   Raspberry Pi 400
-   PC-K2
-   U310
