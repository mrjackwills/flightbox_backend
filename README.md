<p align="center">
	<img src='./.github/logo.svg' width='125px'/>
</p>

<p align="center">
	<h1 align="center">flightbox client</h1>
</p>

<p align="center">
 The pi client for flightbox, powered by <a href='https://www.staticpi.com' target='_blank' rel='noopener noreferrer'>staticPi.com</a>
</p>

<p align="center">
	See the frontend website source <a href='https://github.com/mrjackwills/flightbox_vue' target='_blank' rel='noopener noreferrer'>here</a>
</p>

<p align="center">
	Using the api & data from <a href='https://adsbdb.com' target='_blank' rel='noopener noreferrer'>adsbdb.com</a>, source-code for that <a href='https://www.github.com/mrjackwills/adsbdb' target='_blank' rel='noopener noreferrer'>seen here</a>
</p>


### Requirements
Built specifically to work in conjunction with [this](https://mikenye.gitbook.io/ads-b/intro/overview)

### Build
pi 4 64bit

```bash
cross build --target aarch64-unknown-linux-musl --release
```
### Tests

<!-- aarch64-unknown-linux-musl -->
```bash
# Watch
cargo watch -q -c -w src/ -x 'test  -- --test-threads=1 --nocapture'

# Run all 
cargo test -- --test-threads=1 --nocapture
```

### Ignore this

dbus in docker?

https://georgik.rocks/how-to-start-d-bus-in-docker-container/

https://gist.github.com/eoli3n/93111f23dbb1233f2f00f460663f99e2

sudo nano /etc/gdm3/custom.conf

In wayland - although so far untested
screen toggling requires these envs:
```bash 
export XDG_RUNTIME_DIR="/run/user/$UID"
export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
```