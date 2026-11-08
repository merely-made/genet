#!/bin/sh
# Controlled local fixture only. No user credentials. Run in WSL and pass the
# printed numeric endpoint as MURM_RTC_TEST_SERVER to the Windows test command.
set -eu
addr=$(hostname -I | cut -d ' ' -f 1)
printf 'MURM_RTC_TEST_SERVER=%s:34790\n' "$addr"
exec turnserver -n --verbose --listening-ip="$addr" --relay-ip="$addr" \
  --listening-port=34790 --min-port=50000 --max-port=50100 \
  --realm=murm-rtc.test --user=rtc-test:rtc-test-password --lt-cred-mech \
  --fingerprint --no-cli --no-tls --no-dtls --no-tcp --no-multicast-peers \
  --relay-threads=1 --no-rfc5780 --log-file=stdout --simple-log --pidfile= \
  --allow-loopback-peers
