#!/bin/sh

set -x

exec sudo -E capsh --caps='cap_net_admin+eip cap_setpcap,cap_setuid,cap_setgid+ep' --keep=1 --user=$USER --addamb=cap_net_admin -- -c "exec $SHELL"
