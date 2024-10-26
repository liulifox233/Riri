# リリ/Riri

A simple menu to display lyrics for macOS and Apple Music.

## Usage

To run the project, use the following command:

```bash
$ cargo run
```

Then, modify the `config.yml` file located at:

```
/Users/username/Library/Application Support/Riri
```

### Example `config.yml`

```yaml
storefront: jp # Option
user_token: # Get from your Apple Music web cookie. (https://music.apple.com/)
authorization: # Option
expire: # Option
offset: 1.0 # Option, If the delay between your lyrics and the music is too large, then you can adjust this.
```