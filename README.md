<p align="center">
	<img src='./.github/logo.svg' width='125px'/>
</p>

<p align="center">
	<h1 align="center">flightbox client</h1>
</p>


dbus in docker?

https://georgik.rocks/how-to-start-d-bus-in-docker-container/

https://gist.github.com/eoli3n/93111f23dbb1233f2f00f460663f99e2

## Run

sudo nano /etc/gdm3/custom.conf

In wayland - although so far untested
screen toggling requires these envs:
```bash 
export XDG_RUNTIME_DIR="/run/user/$UID"
export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
```
### Build
pi 4 64bit

```bash
cross build --target aarch64-unknown-linux-musl --release
```


## Tests

Requires postgres & redis to both be operational and seeded with data

<!-- aarch64-unknown-linux-musl -->
```bash
# Watch
cargo watch -q -c -w src/ -x 'test  -- --test-threads=1 --nocapture'

# Run all 
cargo test -- --test-threads=1 --nocapture
```