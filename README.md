![](app-preview.gif)

# ttypr

**t**erminal **ty**ping **pr**actice

Small application to practice ASCII (for now) typing

Based on [Ratatui][Repo]

# Installation

```shell
cargo install ttypr
```

# Usage

**Menu mode:**
q - exit the application
i - switch to typing mode
m - switch typing option (ASCII, Words)

**Typing mode:**
ESC - switch to menu mode

# Credits

- filipriec [ gitlab.com/filipriec ] - creating a vector of styled Spans idea, if needs_redraw rendering concept

- Concept taken from: monkeytype.com

[Repo]: https://github.com/ratatui/ratatui
