<div align="center">
  <img src="images/social_preview.jpg" alt="ttypr - terminal typing practice" width="500" />
</div>

<div align="center">

**t**erminal **ty**ping **pr**actice

_ttypr_ is a simple, lightweight typing practice application that runs in your terminal, built with [Rust](https://www.rust-lang.org/) and [Ratatui](https://ratatui.rs).

</div>

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/ttypr?style=for-the-badge)](https://crates.io/crates/ttypr)
[![GitHub repo](https://img.shields.io/badge/github-repo-blue?style=for-the-badge)](https://github.com/hotellogical05/ttypr)

</div>

## Features

- **Multiple Typing Modes:** Practice with ASCII characters, random words, or your own text.
- **Real-time Feedback:** Get immediate feedback on your accuracy and typing speed.
- **Mistake Analysis:** Track your most commonly mistyped characters.
- **Customizable:** Toggle notifications, character counting, and more.

## Preview

![](images/preview.gif)

## Installation

```shell
cargo install ttypr
```

## Usage

> **Notes:**
>
> - The application starts in the **Menu mode**.
>
> - For larger font - increase the terminal font size.

### Menu mode:

- **h** - display the help page
- **q** - exit the application
- **i** - switch to Typing mode
- **o** - switch Typing option (ASCII, Words, Text)
- **n** - toggle notifications
- **c** - toggle counting mistyped characters
- **w** - display top mistyped characters
- **r** - clear mistyped characters count
- **a** - toggle displaying WPM

### Typing mode:

- **ESC** - switch to Menu mode
- **Character keys** - Type the corresponding characters
- **Backspace** - Remove characters

## Acknowledgements

- [filipriec][FilipsGitLab] - creating a vector of styled Spans idea, if needs_redraw rendering concept
- Concept taken from: [Monkeytype][MonkeytypeLink]

## License

This project is licensed under the [MIT License][MITLicense].

[FilipsGitLab]: https://gitlab.com/filipriec
[MonkeytypeLink]: https://monkeytype.com
[MITLicense]: https://github.com/hotellogical05/ttypr/blob/main/LICENSE
