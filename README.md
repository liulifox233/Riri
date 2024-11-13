<p align="center">
    <img src="./i_ll.png" alt="Riri" width="25%">
</p>

<div align="center">
<h1> リリ/Riri </h1>
</div>

A simple menubar tool to display lyrics for Apple Music in MacOS.

## Usage

First, get your user token from `https://music.apple.com/`, you can find it in cookie under `media-user-token`.

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
user_token: # Get from your Apple Music web cookie.
authorization: # Option
expire: # Option
offset: 1.0 # Option, If the delay between your lyrics and the music is too large, then you can adjust this.
length: 24 #Option, Lyrics length
```

Lyrics data was locate at `/Users/username/Library/Application Support/Riri/Data`
