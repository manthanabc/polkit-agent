# Wayland Polkit Agent

A Polkit authentication agent for Wayland desktops, built with `iced_layershell` and `polkit-agent-rs`. This aims to be a Simple and Customizable GUI for handling Polkit requests.

![screenshot](https://github.com/user-attachments/assets/ea527ea8-d499-468c-9e0a-9bfd45929c72)

## Features

*   Lightweight and fast
*   Built with Rust and the Iced toolkit
*   Wayland native
*   Retry mechanism (up to 3 attempts) if authentication fails

### Planned Features

*   [ ] Use system theme for a more integrated look and feel
*   [ ] CSS-based styling for customization

## Building from Source

To build the Wayland Polkit Agent from source, you'll need to have Rust and Cargo installed.

1.  Clone the repository:
    ```sh
    git clone https://github.com/your-username/polkit-agent.git
    ```
2.  Build the project:
    ```sh
    cd polkit-agent
    cargo build --release
    ```
3.  The binary will be located in `target/release/polkit-agent`.

## Contributing

Contributions are welcome! If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull request.

## Credits

This project is heavily based on the `polkit-min` example from the [decodetalks/polkit-agent-rs](https://github.com/decodetalks/polkit-agent-rs).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
