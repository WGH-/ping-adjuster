
### iptables rules

```
iptables -A OUTPUT --protocol icmp --icmp-type echo-reply -j NFQUEUE --queue-num 5256 --queue-bypass
ip6tables -A OUTPUT --protocol icmpv6 --icmpv6-type echo-reply -j NFQUEUE --queue-num 5256 --queue-bypass
```
