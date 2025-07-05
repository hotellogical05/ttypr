![](preview.gif)

# ttypr

**t**erminal **ty**ping **pr**actice

Current typing options: **ASCII** and **Words**

Based on [Ratatui][RatatuiRepo]

# Installation

```shell
cargo install ttypr
```

# Usage

> **Note:** the application starts in the **Menu mode**.

**Menu mode:**

- **q** - exit the application
- **i** - switch to Typing mode
- **m** - switch Typing option (ASCII, Words)

**Typing mode:**

- **ESC** - switch to Menu mode

# Acknowledgements

- [filipriec][FilipsGitLab] - creating a vector of styled Spans idea, if needs_redraw rendering concept

- Concept taken from: [Monkeytype][MonkeytypeLink]

# License

This project is licensed under the [MIT License][MITLicense].

[RatatuiRepo]: https://github.com/ratatui/ratatui
[FilipsGitLab]: https://gitlab.com/filipriec
[MonkeytypeLink]: https://monkeytype.com
[MITLicense]: https://github.com/hotellogical05/ttypr/blob/main/LICENSE
