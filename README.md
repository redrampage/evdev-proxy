Evdev Proxy
-----------

Proxy multiple evdev input devices to single virtual uinput device.

Primary usecase are - passthrough input events to qemu-evdev device with 
support of input device hotplug/unplug. For example using some kvm-switches
will detach physical evdev devices from qemu on first switch to another 
machine and only way to attach it back is to restart the guest.

Note:

This is early version, WIP.
