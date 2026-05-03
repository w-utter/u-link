Enumerates and monitors linkstates for network controllers on Linux.

Currently the only fields supported are the name of the controller (ex: eth0, lo, enx, etc) and its operstate (up, down, dormant, etc). Although other fields can be accessed through the attributes of `NetlinkItfPacket`.

Its like udev, but for monitoring the state of network devices
