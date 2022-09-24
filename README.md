This is demo program accompanying [a blog post of mine](https://wgh.torlan.ru/2022/09/23/ping-abuse.html) describing fun things that can be done by messing with timestamps hidden inside ICMP ping packets.

[![asciicast](https://asciinema.org/a/CPjWSNnwBQSD3q9C9iOOo29c8.svg)](https://asciinema.org/a/CPjWSNnwBQSD3q9C9iOOo29c8)

### Building

`cargo build`, `cargo install`, etc., build the program normally.

`cargo deb` builds a Debian package. Included `./build_in_docker.sh` script can be used to do so inside a Docker container. The package was tested on Debian Bullseye, and should hopefully work on Ubuntu 20.04 and later.

### Running

Unless you install it from the Debian package (which handles this itself), in order this to work, certain `iptables`/`nftables` rules are required to redirect ICMP traffic into the program:
```
iptables -A OUTPUT --protocol icmp --icmp-type echo-reply -j NFQUEUE --queue-num 5256 --queue-bypass
ip6tables -A OUTPUT --protocol icmpv6 --icmpv6-type echo-reply -j NFQUEUE --queue-num 5256 --queue-bypass
```

systemd service included in the Debian package includes settings from `/etc/default/ping-adjuster`.
